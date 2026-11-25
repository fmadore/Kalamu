//! Running a job: each source is opened once and planned ([`crate::plan`]), and every piece ×
//! set-up is either taken from the cache or sent to its provider ([`crate::piece`]), a few at a
//! time across the whole job ([`opened`]). In economy mode, the pieces of set-ups whose provider has a batch service are gathered
//! into batches instead ([`crate::batch`]) and collected once every source has been read.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use kalamu_core::{JobId, JobStatus, Pace, ProgressEvent, Setup, SetupId, Task};

use crate::control::lock;
use crate::output::OutputNames;
use crate::piece::Outcome;
use crate::plan::Existing;
use crate::{Control, Engine, EngineError};

mod opened;

use opened::{Opened, Prepared, Shared, Tally};

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
                .check_ready(step.model, &job.settings.languages)
                .map_err(EngineError::NotReady)?;
        }
        let stop = lock(&control).stop.clone();
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

        // The cost shown is the job's, over all its runs, not only this one's.
        let earlier = self.store.spent(job_id)?;
        if earlier > 0.0 {
            self.emit(ProgressEvent::CostUpdated {
                job_id,
                spent_usd: earlier,
            });
        }
        let spent = Arc::new(Mutex::new(earlier));
        // The names of the job's files, worked out once for every source the run writes.
        let names = OutputNames::of(&job);
        let run = Shared {
            job: &job,
            control: &control,
            stop: &stop,
            existing: &existing,
            batched: &batched,
            spent: &spent,
            names: &names,
        };
        let mut tally = Tally::default();

        // One pool of calls for the whole job, fed source after source: the limit on calls at
        // the same time holds across the job's files, so a folder of one-page photographs is
        // read as many at once as a long PDF. A source is opened only when those open have
        // nothing left to send, and never more of them than calls, so a few are open at once.
        let limit = self.config.concurrency.max(1);
        let mut sources = job.sources.iter().enumerate();
        let mut all_opened = false;
        let mut open: Vec<Opened> = Vec::new();
        let mut in_flight = FuturesUnordered::new();
        let mut stopped = false;
        let mut engine_error: Option<EngineError> = None;
        loop {
            while !stopped && in_flight.len() < limit {
                if stop_requested(&control).is_some() {
                    stopped = true;
                    break;
                }
                if let Some(from) = open.iter_mut().find(|o| o.has_next()) {
                    let index = from.index;
                    if let Some(call) = self.send_unit(&run, from) {
                        let handle = tokio::spawn(call);
                        in_flight.push(async move { (index, handle.await) });
                    }
                    continue;
                }
                if open.len() >= limit {
                    break;
                }
                let Some((index, source)) = sources.next() else {
                    all_opened = true;
                    break;
                };
                match self.prepare(&run, &mut tally, index, source).await {
                    Ok(Prepared::Ready(opened)) => open.push(opened),
                    Ok(Prepared::Unread) => {}
                    Ok(Prepared::Stopped) => stopped = true,
                    Err(e) => halt(&stop, &mut stopped, &mut engine_error, e),
                }
            }

            // Each source none of whose units is running, with nothing left to send or nothing
            // more to be sent, is finished: its files are written once its pieces are stored.
            while engine_error.is_none() {
                stopped |= stop_requested(&control).is_some();
                let Some(i) = open.iter().position(|o| o.settled(stopped)) else {
                    break;
                };
                let done = open.remove(i);
                if let Err(e) = self.finish_source(&run, &mut tally, &done, stopped).await {
                    halt(&stop, &mut stopped, &mut engine_error, e);
                }
            }

            let Some((index, done)) = in_flight.next().await else {
                if stopped || engine_error.is_some() || (all_opened && open.is_empty()) {
                    break;
                }
                continue;
            };
            if let Some(from) = open.iter_mut().find(|o| o.index == index) {
                from.running -= 1;
            }
            match done {
                Ok(Ok(Outcome::Finished)) => {}
                Ok(Ok(Outcome::Interrupted)) => tally.interrupted = true,
                Ok(Ok(Outcome::Failed)) | Err(_) => tally.failed = true,
                Ok(Ok(Outcome::Fatal(message))) => {
                    stop_for(&control, job_id, &message);
                    stopped = true;
                    tally.failed = true;
                }
                Ok(Err(e)) => halt(&stop, &mut stopped, &mut engine_error, e),
            }
        }
        if let Some(e) = engine_error {
            return Err(e);
        }
        let Tally {
            mut failed,
            mut interrupted,
            unread,
            mut gathered,
            batch: mut batch_outcome,
        } = tally;

        // Economy mode: send what is gathered, then wait for every batch of the job, including
        // those sent before the app was last closed.
        if !batched.is_empty() {
            if stop_requested(&control).is_none() {
                let outcome = self.send_gathered(&job, &mut gathered, &stop).await?;
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
                if job.task == Task::Contents
                    && stop_requested(&control).is_none()
                    && !unread.contains(&source.id)
                {
                    for setup in &setups {
                        match self.merge_pages(&job, source, setup, &spent, &stop).await? {
                            Outcome::Finished => {}
                            Outcome::Interrupted => interrupted = true,
                            Outcome::Failed | Outcome::Fatal(_) => failed = true,
                        }
                    }
                }
                failed |= self.export_finished(&job, &names, source.id, &setups)?;
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

/// Stops the job's calls on an engine error, kept to be reported once they are stored: the
/// calls already started finish and are stored first, as returning at once would leave them
/// running unwatched, and an immediate resume would send them again.
fn halt(
    stop: &kalamu_core::StopSignal,
    stopped: &mut bool,
    error: &mut Option<EngineError>,
    e: EngineError,
) {
    stop.stop();
    *stopped = true;
    error.get_or_insert(e);
}

pub(crate) enum StopKind {
    Pause,
    Cancel,
}

pub(crate) fn stop_requested(control: &Mutex<Control>) -> Option<StopKind> {
    let c = lock(control);
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
    lock(control).pause = true;
}
