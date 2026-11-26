//! What a run does, step by step, and the weekly set: per provider, one page read, one short
//! recording, one analysis, one analysis sent as a batch and collected, and one embedding,
//! each on the cheapest model the catalogue offers for it, chosen by its fields and its
//! estimate, never by its name.

use std::path::Path;

use kalamu_catalog::ModelStatus;
use kalamu_core::{ExportFormat, JobSettings, Pace, ProviderId, SetupOptions, SetupRequest, Task};
use kalamu_engine::{Engine, EstimateRequest};

/// The weekly set's files, all in the repository's `samples/`.
pub const PAGE: &str = "bulletin-page.jpg";
pub const RECORDING: &str = "interview.wav";
pub const TEXT: &str = "bulletin.txt";
/// The analysis the weekly set asks for: the people, places and organisations named.
pub const ANALYSIS_PRESET: &str = "analysis.entities";
/// What of [`TEXT`] the embedding step sends, in characters.
pub const EMBEDDING_CHARS: usize = 1_000;

/// One job, as the engine runs it.
pub struct Plan {
    pub name: String,
    pub task: Task,
    pub files: Vec<String>,
    pub setups: Vec<SetupRequest>,
    pub settings: JobSettings,
    pub formats: Vec<ExportFormat>,
}

pub enum Work {
    Job(Plan),
    /// One text embedded, with the model's price per million tokens.
    Embedding {
        provider: ProviderId,
        model: String,
        price: Option<f64>,
        file: String,
    },
    /// Nothing to run for this provider, and why: it offers no model for the step.
    Absent {
        provider: ProviderId,
        why: String,
    },
}

pub struct Step {
    pub name: String,
    pub work: Work,
}

impl Step {
    pub fn job(plan: Plan) -> Self {
        Self {
            name: plan.name.clone(),
            work: Work::Job(plan),
        }
    }

    /// Whether it goes through a provider's batch service.
    pub fn is_batch(&self) -> bool {
        matches!(&self.work, Work::Job(p) if p.settings.pace == Pace::Economy)
    }
}

pub fn setup(
    provider: &str,
    model: &str,
    preset: Option<&str>,
    options: SetupOptions,
) -> SetupRequest {
    SetupRequest {
        provider: ProviderId::new(provider),
        model: model.into(),
        preset_id: preset.map(Into::into),
        label: None,
        options,
        merge: None,
    }
}

/// The type a sample is sent as, from its extension, to match a model's inputs.
pub fn mime(file: &str) -> &'static str {
    let lower = file.to_ascii_lowercase();
    match lower.rsplit('.').next().unwrap_or_default() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

/// The weekly set for each of `providers`, in the order they are given: the batches first, so
/// that their wait overlaps the other steps.
pub fn weekly(engine: &Engine, samples: &Path, providers: &[ProviderId]) -> Vec<Step> {
    let mut batches = Vec::new();
    let mut others = Vec::new();
    for provider in providers {
        let id = provider.as_str();
        let step = |kind: &str| format!("{kind}-{id}");
        batches.push(cheapest(
            engine,
            samples,
            provider,
            &step("batch"),
            Task::Analysis,
            TEXT,
            Some(ANALYSIS_PRESET),
            Pace::Economy,
        ));
        others.push(cheapest(
            engine,
            samples,
            provider,
            &step("page"),
            Task::TextExtraction,
            PAGE,
            None,
            Pace::Fast,
        ));
        others.push(cheapest(
            engine,
            samples,
            provider,
            &step("recording"),
            Task::AudioTranscription,
            RECORDING,
            None,
            Pace::Fast,
        ));
        others.push(cheapest(
            engine,
            samples,
            provider,
            &step("analysis"),
            Task::Analysis,
            TEXT,
            Some(ANALYSIS_PRESET),
            Pace::Fast,
        ));
        others.push(embedding(engine, provider, &step("embedding")));
    }
    batches.into_iter().chain(others).collect()
}

/// The model a provider offers for new work (neither replaced, retired nor being retired) that
/// does `task` on `file` at the lowest estimate, at `pace`; at economy pace, only a model whose
/// work the provider sends as a batch. A model without an estimate is never chosen: the ceiling
/// holds a step to its estimate, so one it cannot price is not run.
#[allow(clippy::too_many_arguments)]
pub fn cheapest(
    engine: &Engine,
    samples: &Path,
    provider: &ProviderId,
    name: &str,
    task: Task,
    file: &str,
    preset: Option<&str>,
    pace: Pace,
) -> Step {
    let absent = |why: String| Step {
        name: name.to_string(),
        work: Work::Absent {
            provider: provider.clone(),
            why,
        },
    };
    let path = samples.join(file);
    let source = match engine.inspect(&[&path]).into_iter().next() {
        Some(Ok((source, _))) => source,
        _ => return absent(format!("the sample {file} could not be read")),
    };
    let economy = pace == Pace::Economy;
    let batches = |model: &str| {
        engine
            .provider(provider)
            .is_ok_and(|p| p.batches(task, model))
    };
    let catalog = engine.catalog();
    let mut best: Option<(f64, SetupRequest)> = None;
    let mut unpriced = 0;
    for model in catalog.models.iter().filter(|m| {
        &m.provider == provider
            && m.is_offered()
            && m.status != ModelStatus::Deprecated
            && m.does(task)
            && m.inputs.iter().any(|i| i == mime(file))
            && (!economy || batches(&m.id))
    }) {
        let candidate = setup(
            provider.as_str(),
            &model.id,
            preset,
            SetupOptions::default(),
        );
        let estimate = engine.estimate(&EstimateRequest {
            task,
            sources: vec![source.clone()],
            setups: vec![candidate.clone()],
            sample_pages: None,
        });
        let usd = if economy {
            estimate.total_economy_usd
        } else {
            estimate.total_usd
        };
        match usd {
            Some(usd) if best.as_ref().is_none_or(|(b, _)| usd < *b) => {
                best = Some((usd, candidate));
            }
            Some(_) => {}
            // No price, or a sample whose length could not be read: nothing says it is cheap,
            // and nothing would hold it to the ceiling.
            None => unpriced += 1,
        }
    }
    let Some((_, chosen)) = best else {
        return absent(if unpriced > 0 {
            format!("no estimate for its {unpriced} model(s): no price, or {file}'s length unknown")
        } else if economy {
            "no batch service for this step".into()
        } else {
            "no model for this step".into()
        });
    };
    let formats = match task {
        Task::Analysis => vec![ExportFormat::Json],
        _ => vec![ExportFormat::Txt],
    };
    Step::job(Plan {
        name: name.to_string(),
        task,
        files: vec![file.to_string()],
        setups: vec![chosen],
        settings: JobSettings {
            pace,
            ..JobSettings::default()
        },
        formats,
    })
}

/// The provider's cheapest embedding model run in the cloud, by its price.
fn embedding(engine: &Engine, provider: &ProviderId, name: &str) -> Step {
    let catalog = engine.catalog();
    let model = catalog
        .embedding_models()
        .iter()
        .filter(|m| &m.provider == provider && !m.local && m.input_price.is_some())
        .min_by(|a, b| {
            a.input_price
                .partial_cmp(&b.input_price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    Step {
        name: name.to_string(),
        work: match model {
            Some(m) => Work::Embedding {
                provider: provider.clone(),
                model: m.id.clone(),
                price: m.input_price,
                file: TEXT.into(),
            },
            None => Work::Absent {
                provider: provider.clone(),
                why: "no embeddings".into(),
            },
        },
    }
}
