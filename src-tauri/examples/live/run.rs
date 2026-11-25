//! Running the steps: each job through the engine, as the app runs it, under the ledger's
//! ceiling; each embedding through its provider.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use kalamu_core::{
    EmbedRequest, EmbedRole, Embedder, JobId, JobStatus, JobStore, OutputDestination,
    OutputSettings, Pace, PieceStatus, ProgressEvent, StopSignal,
};
use kalamu_engine::{Engine, EstimateRequest, KeySource, NewJob};
use kalamu_store::SqliteStore;
use tokio::sync::broadcast::error::RecvError;

use crate::ceiling::{Admission, Ledger};
use crate::keys::{Keys, Redactor};
use crate::report::{Outcome, Row};
use crate::steps::{EMBEDDING_CHARS, Plan, Step, Work};

/// How long an error message may be in the summary.
const MESSAGE_CHARS: usize = 200;

pub struct Runner {
    pub engine: Arc<Engine>,
    pub store: Arc<SqliteStore>,
    pub embedders: Vec<Arc<dyn Embedder>>,
    pub keys: Arc<Keys>,
    pub redactor: Redactor,
    pub samples: PathBuf,
    pub out: PathBuf,
    /// The longest a batch is waited for before it is cancelled at the provider.
    pub batch_wait: Duration,
    pub ledger: Mutex<Ledger>,
    /// The jobs running, cancelled together once the ceiling is passed.
    running: Mutex<Vec<JobId>>,
    next: AtomicU64,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Runner {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: Arc<Engine>,
        store: Arc<SqliteStore>,
        embedders: Vec<Arc<dyn Embedder>>,
        keys: Arc<Keys>,
        samples: PathBuf,
        out: PathBuf,
        batch_wait: Duration,
        ceiling: Option<f64>,
    ) -> Arc<Self> {
        let redactor = keys.redactor();
        Arc::new(Self {
            engine,
            store,
            embedders,
            keys,
            redactor,
            samples,
            out,
            batch_wait,
            ledger: Mutex::new(Ledger::new(ceiling)),
            running: Mutex::default(),
            next: AtomicU64::new(1),
        })
    }

    pub fn spent(&self) -> f64 {
        lock(&self.ledger).spent()
    }

    /// Whether what was billed passed the ceiling.
    pub fn ledger_passed(&self) -> bool {
        lock(&self.ledger).passed()
    }

    /// Runs one step to its end and says how it went, a row per set-up.
    pub async fn step(self: &Arc<Self>, step: Step) -> Vec<Row> {
        match step.work {
            Work::Job(plan) => self.job(plan).await,
            Work::Embedding {
                provider,
                model,
                price,
                file,
            } => vec![
                self.embedding(&step.name, &provider, &model, price, &file)
                    .await,
            ],
            Work::Absent { provider, why } => {
                vec![Row::new(
                    &step.name,
                    provider.as_str(),
                    "–",
                    Outcome::Skipped(why),
                )]
            }
        }
    }

    /// The ledger's answer for a step about to start, as an outcome when it may not.
    fn admit(&self, id: u64, estimate: Option<f64>) -> Option<Outcome> {
        match lock(&self.ledger).admit(id, estimate) {
            Admission::Go => None,
            Admission::TooDear { estimate, left } => Some(Outcome::Ceiling(format!(
                "not started: its estimate ${estimate:.4} passes the ${left:.4} left"
            ))),
            Admission::Unpriced => Some(Outcome::Ceiling(
                "not started: no price in the catalogue to keep it under the ceiling".into(),
            )),
            Admission::Stopped => Some(Outcome::Ceiling(
                "not started: the ceiling was passed earlier in the run".into(),
            )),
        }
    }

    /// Records what a step has been billed in all; once that passes the ceiling, every job
    /// running is cancelled. True when the ceiling is passed.
    fn bill(&self, id: u64, total: f64) -> bool {
        let passed = lock(&self.ledger).bill(id, total);
        if passed {
            for job in lock(&self.running).iter() {
                self.engine.cancel(*job);
            }
        }
        passed
    }

    async fn job(self: &Arc<Self>, mut plan: Plan) -> Vec<Row> {
        let name = plan.name.clone();
        let rows_for = |plan: &Plan, outcome: Outcome| -> Vec<Row> {
            plan.setups
                .iter()
                .map(|s| Row::new(&plan.name, s.provider.as_str(), &s.model, outcome.clone()))
                .collect()
        };
        let mut rows = Vec::new();
        // Set-ups whose provider has no key, or whose model the catalogue no longer lists, are
        // left out and said so.
        let catalog = self.engine.catalog();
        plan.setups.retain(|s| {
            let listed = catalog
                .models
                .iter()
                .any(|m| m.provider == s.provider && m.id == s.model && !m.retired);
            let ready = self
                .engine
                .provider(&s.provider)
                .is_ok_and(|p| !p.needs_key() || self.keys.key(&s.provider).is_some());
            let why = if !listed {
                Some("the catalogue no longer lists this model")
            } else if !ready {
                Some("no key for this provider")
            } else {
                None
            };
            if let Some(why) = why {
                rows.push(Row::new(
                    &name,
                    s.provider.as_str(),
                    &s.model,
                    Outcome::Skipped(why.into()),
                ));
            }
            why.is_none()
        });
        if plan.setups.is_empty() {
            return rows;
        }
        println!("\n== {} ({:?}) ==", plan.name, plan.settings.pace);
        let paths: Vec<PathBuf> = plan.files.iter().map(|f| self.samples.join(f)).collect();
        if let Some(missing) = paths.iter().find(|p| !p.is_file()) {
            let file = missing
                .file_name()
                .map(|f| f.to_string_lossy().into_owned());
            let why = format!("sample {} missing", file.unwrap_or_default());
            println!("  {why}");
            rows.extend(rows_for(&plan, Outcome::Skipped(why)));
            return rows;
        }
        let mut sources = Vec::new();
        for inspected in self.engine.inspect(&paths) {
            match inspected {
                Ok((source, warnings)) => {
                    for w in warnings {
                        println!("  warning: {w:?}");
                    }
                    sources.push(source);
                }
                Err(e) => {
                    let why = self.redactor.line(&e.to_string(), MESSAGE_CHARS);
                    rows.extend(rows_for(&plan, Outcome::Failed(why)));
                    return rows;
                }
            }
        }
        // A plan named `document-<name>` reads its images as the pages of one document.
        if let Some(document) = plan.name.strip_prefix("document-") {
            match self.engine.image_document(document, &sources) {
                Ok(document) => sources = vec![document.source],
                Err(e) => {
                    let why = self.redactor.line(&e.to_string(), MESSAGE_CHARS);
                    rows.extend(rows_for(&plan, Outcome::Failed(why)));
                    return rows;
                }
            }
        }
        let economy = plan.settings.pace == Pace::Economy;
        // The estimate New job would show, to set against what the providers billed.
        let estimate = self.engine.estimate(&EstimateRequest {
            task: plan.task,
            sources: sources.clone(),
            setups: plan.setups.clone(),
            sample_pages: plan.settings.sample_pages,
        });
        let row_estimate = |s: &kalamu_core::SetupRequest| -> Option<f64> {
            let mine: Vec<_> = estimate
                .rows
                .iter()
                .filter(|r| r.provider == s.provider && r.model == s.model)
                .collect();
            let price = |r: &&kalamu_engine::EstimateRow| {
                if economy {
                    r.estimate.economy_usd.or(r.estimate.usd)
                } else {
                    r.estimate.usd
                }
            };
            // Two set-ups of one model (two efforts) share the rows: each takes its share.
            let same = plan
                .setups
                .iter()
                .filter(|o| o.provider == s.provider && o.model == s.model)
                .count()
                .max(1) as f64;
            mine.iter()
                .map(price)
                .sum::<Option<f64>>()
                .map(|u| u / same)
        };
        for row in &estimate.rows {
            println!(
                "  estimate {:<28} {} pages: ${:.4} (economy {})",
                row.model,
                row.pages,
                row.estimate.usd.unwrap_or(0.0),
                row.estimate
                    .economy_usd
                    .map_or("-".into(), |u| format!("${u:.4}"))
            );
        }
        let total = if economy {
            estimate.total_economy_usd
        } else {
            estimate.total_usd
        };
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        if let Some(outcome) = self.admit(id, total) {
            println!("  {}", outcome_words(&outcome));
            rows.extend(rows_for(&plan, outcome));
            return rows;
        }
        let estimates: Vec<Option<f64>> = plan.setups.iter().map(row_estimate).collect();
        let folder = self.out.join(&plan.name);
        let created = self.engine.create_job(NewJob {
            name: plan.name.clone(),
            task: plan.task,
            sources,
            setups: plan.setups.clone(),
            settings: plan.settings.clone(),
            output: OutputSettings {
                formats: plan.formats.clone(),
                destination: OutputDestination::Folder(folder.to_string_lossy().into_owned()),
                ..Default::default()
            },
        });
        let job = match created {
            Ok(job) => job,
            Err(e) => {
                lock(&self.ledger).finish(id);
                let why = self.redactor.line(&e.to_string(), MESSAGE_CHARS);
                rows.extend(rows_for(&plan, Outcome::Failed(why)));
                return rows;
            }
        };
        let mut events = self.engine.subscribe();
        if let Err(e) = self.engine.start(job.id) {
            lock(&self.ledger).finish(id);
            let why = self.redactor.line(&e.to_string(), MESSAGE_CHARS);
            rows.extend(rows_for(&plan, Outcome::Failed(why)));
            return rows;
        }
        lock(&self.running).push(job.id);
        let deadline = tokio::time::Instant::now() + self.batch_wait;
        let (mut late, mut ceiling, mut source_failed) = (false, false, None);
        let status = loop {
            let event = if economy && !late {
                match tokio::time::timeout_at(deadline, events.recv()).await {
                    Ok(event) => event,
                    Err(_) => {
                        // Not back in time: cancelled at the provider, which bills nothing for
                        // what it has not answered.
                        late = true;
                        self.engine.cancel(job.id);
                        continue;
                    }
                }
            } else {
                events.recv().await
            };
            match event {
                Ok(ProgressEvent::CostUpdated { job_id, spent_usd }) if job_id == job.id => {
                    if self.bill(id, spent_usd) {
                        ceiling = true;
                    }
                }
                Ok(ProgressEvent::SourceFailed { job_id, error, .. }) if job_id == job.id => {
                    let why = self.redactor.line(&error, MESSAGE_CHARS);
                    println!("  source failed: {why}");
                    source_failed = Some(why);
                }
                Ok(ProgressEvent::JobFinished { job_id, status }) if job_id == job.id => {
                    break status;
                }
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break JobStatus::Interrupted,
            }
        };
        lock(&self.running).retain(|j| *j != job.id);
        // What the store holds is the job's last word on its cost.
        if let Ok(spent) = self.store.spent(job.id)
            && self.bill(id, spent)
        {
            ceiling = true;
        }
        lock(&self.ledger).finish(id);
        let pieces = self.store.pieces(job.id).unwrap_or_default();
        for (setup, estimate) in job.setups.iter().zip(estimates) {
            let mine: Vec<_> = pieces.iter().filter(|p| p.setup_id == setup.id).collect();
            let mut billed = 0.0;
            let mut first_error = None;
            for piece in &mine {
                let source = job.sources.iter().find(|s| s.id == piece.source_id);
                let result = self.store.finished_result(piece).ok().flatten();
                let cost = result.as_ref().and_then(|r| r.cost_usd).unwrap_or(0.0);
                billed += cost;
                let error = piece
                    .error
                    .as_deref()
                    .filter(|_| piece.status == PieceStatus::Failed)
                    .map(|e| self.redactor.line(e, MESSAGE_CHARS));
                let usage = result.map(|r| {
                    let u = &r.output.usage;
                    format!(
                        " in {} (cached {}, written {}) out {} audio {:.0}s pages {} ${cost:.5}",
                        u.input_tokens,
                        u.cached_input_tokens,
                        u.cache_write_tokens,
                        u.output_tokens,
                        u.audio_seconds,
                        u.pages,
                    )
                });
                println!(
                    "  {:<28} {:<16} #{} {:?}{}{}",
                    setup.label,
                    source.map_or("?", |s| s.file_name.as_str()),
                    piece.index,
                    piece.status,
                    usage.unwrap_or_default(),
                    error.as_ref().map(|e| format!(": {e}")).unwrap_or_default()
                );
                if first_error.is_none() {
                    first_error = error;
                }
            }
            let all =
                |status: PieceStatus| !mine.is_empty() && mine.iter().all(|p| p.status == status);
            let done = !mine.is_empty()
                && mine
                    .iter()
                    .all(|p| matches!(p.status, PieceStatus::Done | PieceStatus::Cached));
            let outcome = if all(PieceStatus::Cached) {
                Outcome::Skipped("answered from the cache of an earlier run in this folder".into())
            } else if done && source_failed.is_none() {
                Outcome::Passed
            } else if ceiling {
                Outcome::Ceiling("stopped: what was billed passed the ceiling".into())
            } else if late {
                Outcome::Late(format!(
                    "not back after {} min; cancelled at the provider",
                    self.batch_wait.as_secs() / 60
                ))
            } else if let Some(error) = first_error.or(source_failed.clone()) {
                Outcome::Failed(error)
            } else {
                Outcome::Failed(format!("the job ended {status:?}"))
            };
            rows.push(Row {
                step: plan.name.clone(),
                provider: setup.provider.as_str().into(),
                model: setup.model.clone(),
                economy,
                estimate,
                billed: (billed > 0.0).then_some(billed),
                outcome,
            });
        }
        println!(
            "  {status:?}, ${:.4} in all",
            self.store.spent(job.id).unwrap_or(0.0)
        );
        rows
    }

    /// One text embedded by a provider's cheapest model, under the ceiling.
    async fn embedding(
        &self,
        name: &str,
        provider: &kalamu_core::ProviderId,
        model: &str,
        price: Option<f64>,
        file: &str,
    ) -> Row {
        let row = |outcome| Row::new(name, provider.as_str(), model, outcome);
        let Some(embedder) = self.embedders.iter().find(|e| &e.id() == provider) else {
            return row(Outcome::Skipped("no embedder for this provider".into()));
        };
        let Some(key) = self.keys.key(provider) else {
            return row(Outcome::Skipped("no key for this provider".into()));
        };
        let Ok(text) = std::fs::read_to_string(self.samples.join(file)) else {
            return row(Outcome::Skipped(format!("sample {file} missing")));
        };
        let limit = embedder.limits(model).chars.min(EMBEDDING_CHARS);
        let texts = vec![text.chars().take(limit).collect::<String>()];
        let tokens = texts[0].chars().count() as f64 / kalamu_catalog::CHARS_PER_TOKEN as f64;
        let estimate = price.map(|p| tokens * p / 1_000_000.0);
        println!("\n== {name} ==");
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        if let Some(outcome) = self.admit(id, estimate) {
            println!("  {}", outcome_words(&outcome));
            return Row {
                estimate,
                ..row(outcome)
            };
        }
        let answer = embedder
            .embed(
                Some(&key),
                EmbedRequest {
                    model,
                    role: EmbedRole::Passage,
                    texts: &texts,
                    dimensions: None,
                    stop: &StopSignal::default(),
                },
            )
            .await;
        let (outcome, billed) = match answer {
            Ok(embedded) => {
                let billed = price.map(|p| embedded.input_tokens as f64 * p / 1_000_000.0);
                let ceiling = self.bill(id, billed.unwrap_or(0.0));
                let well_formed = embedded.vectors.len() == 1
                    && embedded.vectors[0].iter().all(|v| v.is_finite())
                    && !embedded.vectors[0].is_empty();
                println!(
                    "  {} tokens, a vector of {}",
                    embedded.input_tokens,
                    embedded.vectors.first().map_or(0, Vec::len)
                );
                let outcome = if !well_formed {
                    Outcome::Failed("the answer held no usable vector".into())
                } else if ceiling {
                    Outcome::Ceiling("what was billed passed the ceiling".into())
                } else {
                    Outcome::Passed
                };
                (outcome, billed)
            }
            Err(e) => {
                let why = self.redactor.line(&e.to_string(), MESSAGE_CHARS);
                println!("  failed: {why}");
                (Outcome::Failed(why), None)
            }
        };
        lock(&self.ledger).finish(id);
        Row {
            estimate,
            billed: billed.filter(|b| *b > 0.0),
            ..row(outcome)
        }
    }
}

fn outcome_words(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Ceiling(why) | Outcome::Skipped(why) | Outcome::Failed(why) => why.clone(),
        Outcome::Late(why) => why.clone(),
        Outcome::Passed => "passed".into(),
    }
}
