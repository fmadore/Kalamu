//! Running a job: each source is opened once and planned ([`crate::plan`]), and every piece ×
//! set-up is either taken from the cache or sent to its provider ([`crate::piece`]), a few at a
//! time. In economy mode, the pieces of set-ups whose provider has a batch service are gathered
//! into batches instead ([`crate::batch`]) and collected once every source has been read.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use kalamu_core::{JobId, JobStatus, Pace, ProgressEvent, Setup, SetupId, Task};
use kalamu_media::OpenSource;

use crate::piece::{Input, Outcome};
use crate::plan::Existing;
use crate::{Control, Engine, EngineError};

impl Engine {
    pub(crate) async fn run(
        self: &Arc<Self>,
        job_id: JobId,
        control: Arc<Mutex<Control>>,
    ) -> Result<JobStatus, EngineError> {
        let job = self.store.job(job_id)?;
        // Check keys and offline models before touching anything, so a missing key or an
        // undownloaded model is reported at once.
        for step in job
            .setups
            .iter()
            .flat_map(|s| [Some(s.step()), s.merge_step()])
        {
            let Some(step) = step else { continue };
            let provider = self.provider(step.provider)?;
            if provider.needs_key() && self.keys.key(step.provider).is_none() {
                return Err(EngineError::MissingKey(step.provider.to_string()));
            }
            provider
                .check_ready(step.model)
                .map_err(EngineError::NotReady)?;
        }
        let stop = control
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stop
            .clone();
        self.store.set_job_status(job_id, JobStatus::Running)?;
        self.reopen_unsent(job_id)?;
        // Set-ups sent as batches; the others, and every set-up of a fast job, go one by one.
        let batched: HashSet<SetupId> = if job.settings.pace == Pace::Economy {
            let mut batched = HashSet::new();
            for setup in &job.setups {
                if self
                    .provider(&setup.provider)?
                    .batches(job.task, &setup.model)
                {
                    batched.insert(setup.id);
                }
            }
            batched
        } else {
            HashSet::new()
        };
        let mut gathered = HashMap::new();
        let mut batch_outcome = crate::batch::BatchOutcome::default();
        let existing: Existing = self
            .store
            .pieces(job_id)?
            .into_iter()
            .map(|p| ((p.setup_id, p.source_id, p.index), p))
            .collect();
        let finished_before = existing.values().filter(|p| p.status.is_finished()).count();
        self.emit(ProgressEvent::JobStarted {
            job_id,
            pieces_total: existing.len() as u32,
            pieces_finished: finished_before as u32,
        });

        let mut failed = false;
        // A piece stopped part-way is pending again; the job must not read as finished.
        let mut interrupted = false;
        // The cost shown is the job's, over all its runs, not only this one's.
        let earlier = self.store.spent(job_id)?;
        if earlier > 0.0 {
            self.emit(ProgressEvent::CostUpdated {
                job_id,
                spent_usd: earlier,
            });
        }
        let spent = Arc::new(Mutex::new(earlier));
        for source in &job.sources {
            if stop_requested(&control).is_some() {
                break;
            }
            // A file changed since it was added is refused: its pieces would be cached under the
            // fingerprint of what it held then.
            let opened = {
                let source = source.clone();
                let ffmpeg = self.config.ffmpeg.clone();
                tokio::task::spawn_blocking(move || {
                    kalamu_media::check_unchanged(&source)?;
                    OpenSource::open(&source, ffmpeg.as_ref())
                })
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?
            };
            let opened = match opened {
                Ok(o) => Arc::new(o),
                Err(e) => {
                    failed = true;
                    self.emit(ProgressEvent::SourceFailed {
                        job_id,
                        source_id: source.id,
                        error: e.to_string(),
                    });
                    continue;
                }
            };
            let pieces = match self.plan(&job, source, &opened, &existing).await? {
                Ok(pieces) => pieces,
                Err(error) => {
                    failed = true;
                    self.emit(ProgressEvent::SourceFailed {
                        job_id,
                        source_id: source.id,
                        error,
                    });
                    continue;
                }
            };
            self.emit(ProgressEvent::SourcePrepared {
                job_id,
                source_id: source.id,
                pieces: pieces.len() as u32,
            });

            let work = self.record_pieces(&job, source, &pieces, &existing)?;

            let (to_batch, work): (Vec<_>, Vec<_>) = work
                .into_iter()
                .partition(|(setup, _, _)| batched.contains(&setup.id));
            for (setup, state, key) in to_batch {
                if stop_requested(&control).is_some() {
                    break;
                }
                let outcome = self
                    .gather(
                        &job,
                        &source.file_name,
                        &setup,
                        state,
                        &key,
                        &opened,
                        &mut gathered,
                    )
                    .await?;
                if let Some(message) = &outcome.fatal {
                    stop_for(&control, job_id, message);
                }
                batch_outcome.merge(outcome);
            }

            let mut in_flight = FuturesUnordered::new();
            let mut queue = work.into_iter();
            let mut stopped = false;
            let mut engine_error: Option<EngineError> = None;
            loop {
                while !stopped && in_flight.len() < self.config.concurrency.max(1) {
                    if stop_requested(&control).is_some() {
                        stopped = true;
                        break;
                    }
                    let Some((setup, state, key)) = queue.next() else {
                        break;
                    };
                    let engine = self.clone();
                    let (job, source, opened, spent, stop) = (
                        job.clone(),
                        source.clone(),
                        opened.clone(),
                        spent.clone(),
                        stop.clone(),
                    );
                    in_flight.push(tokio::spawn(async move {
                        engine
                            .process_piece(
                                &job,
                                &source,
                                setup.step(),
                                state,
                                key,
                                Input::Open(opened),
                                spent,
                                &stop,
                            )
                            .await
                    }));
                }
                let Some(done) = in_flight.next().await else {
                    break;
                };
                match done {
                    Ok(Ok(Outcome::Finished)) => {}
                    Ok(Ok(Outcome::Interrupted)) => interrupted = true,
                    Ok(Ok(Outcome::Failed)) | Err(_) => failed = true,
                    Ok(Ok(Outcome::Fatal(message))) => {
                        stop_for(&control, job_id, &message);
                        stopped = true;
                        failed = true;
                    }
                    // The calls already started finish and are stored before the error is
                    // reported: returning at once would leave them running unwatched, and an
                    // immediate resume would send them again.
                    Ok(Err(e)) => {
                        stop.stop();
                        stopped = true;
                        engine_error.get_or_insert(e);
                    }
                }
            }
            if let Some(e) = engine_error {
                return Err(e);
            }

            // A table of contents: combine the pages of each set-up that read them all.
            if job.task == Task::Contents && !stopped {
                for setup in job.setups.iter().filter(|s| !batched.contains(&s.id)) {
                    match self.merge_pages(&job, source, setup, &spent, &stop).await? {
                        Outcome::Finished => {}
                        Outcome::Interrupted => interrupted = true,
                        Outcome::Failed => failed = true,
                        Outcome::Fatal(message) => {
                            stop_for(&control, job_id, &message);
                            failed = true;
                        }
                    }
                }
            }

            // Write outputs for every set-up whose pieces of this source are all done.
            let setups: Vec<&Setup> = job.setups.iter().collect();
            failed |= self.export_finished(&job, source.id, &setups)?;
        }

        // Economy mode: send what is gathered, then wait for every batch of the job, including
        // those sent before the app was last closed.
        if !batched.is_empty() {
            if stop_requested(&control).is_none() {
                let outcome = self.send_gathered(&job, &mut gathered).await?;
                batch_outcome.merge(outcome);
            }
            if batch_outcome.fatal.is_none() {
                let outcome = self.wait_for_batches(&job, &control, &spent).await?;
                batch_outcome.merge(outcome);
            }
            let setups: Vec<&Setup> = job
                .setups
                .iter()
                .filter(|s| batched.contains(&s.id))
                .collect();
            for source in &job.sources {
                // The pages came back from the batch: combine them, at once.
                if job.task == Task::Contents && stop_requested(&control).is_none() {
                    for setup in &setups {
                        match self.merge_pages(&job, source, setup, &spent, &stop).await? {
                            Outcome::Finished => {}
                            Outcome::Interrupted => interrupted = true,
                            Outcome::Failed | Outcome::Fatal(_) => failed = true,
                        }
                    }
                }
                failed |= self.export_finished(&job, source.id, &setups)?;
            }
            failed |= batch_outcome.failed;
            interrupted |= batch_outcome.waiting;
        }

        Ok(match stop_requested(&control) {
            Some(StopKind::Cancel) => JobStatus::Cancelled,
            Some(StopKind::Pause) => JobStatus::Paused,
            None if interrupted => JobStatus::Paused,
            None if failed => JobStatus::CompletedWithErrors,
            None => JobStatus::Completed,
        })
    }
}

pub(crate) enum StopKind {
    Pause,
    Cancel,
}

pub(crate) fn stop_requested(control: &Mutex<Control>) -> Option<StopKind> {
    let c = control.lock().unwrap_or_else(|p| p.into_inner());
    if c.cancel {
        Some(StopKind::Cancel)
    } else if c.pause {
        Some(StopKind::Pause)
    } else {
        None
    }
}

/// Pauses the job: its key or account stopped working, and every further call would fail.
fn stop_for(control: &Mutex<Control>, job_id: JobId, message: &str) {
    tracing::warn!(job = %job_id, %message, "stopping job");
    control.lock().unwrap_or_else(|p| p.into_inner()).pause = true;
}
