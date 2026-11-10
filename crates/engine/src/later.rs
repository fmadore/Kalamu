//! Files written later. Saving a correction or speakers' names marks the source's files out of
//! date in the store, in the save's own transaction, instead of writing them at once: a long
//! PDF's searchable copy is drawn again from the whole original, too slow for every page a
//! researcher marks. The files are written once the researcher has stopped saving for a few
//! seconds, before anything lists or exports them, when the app quits and at the next launch,
//! so neither a crash nor a write that failed leaves them behind for good.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use kalamu_core::{Job, JobId, ProgressEvent, StoreError};
use tokio::sync::Notify;
use tokio::time::Instant;

use crate::control::lock;
use crate::{Engine, EngineError};

/// The jobs whose files are due, and the one task that writes them.
#[derive(Default)]
pub(crate) struct Later {
    /// When each job's files are next due: the quiet period after its last save.
    due: Mutex<HashMap<JobId, Instant>>,
    wake: Notify,
    /// Started by the first save made where a Tokio runtime runs; without one the marks wait
    /// for the next write that asks for them.
    started: AtomicBool,
    /// Set when the app quits or the engine is dropped: nothing more is started.
    stopped: AtomicBool,
    /// Rounds of writing the task has done, for the tests.
    #[cfg(test)]
    pub(crate) rounds: std::sync::atomic::AtomicUsize,
}

impl Later {
    pub(crate) fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.wake.notify_one();
    }

    fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    fn next_due(&self) -> Option<Instant> {
        lock(&self.due).values().min().copied()
    }

    /// The jobs due by `now`, taken out of the list.
    fn take_due(&self, now: Instant) -> Vec<JobId> {
        let mut due = lock(&self.due);
        let jobs: Vec<JobId> = due
            .iter()
            .filter(|(_, at)| **at <= now)
            .map(|(job, _)| *job)
            .collect();
        for job in &jobs {
            due.remove(job);
        }
        jobs
    }
}

/// The task: sleeps until the earliest job is due, or a save moves it, then writes each job due,
/// one after another, off the runtime's threads. It holds the engine only while it writes, and
/// ends once the engine is stopped or gone; a write it started runs to its end.
async fn run(later: Arc<Later>, engine: Weak<Engine>) {
    loop {
        if later.is_stopped() {
            return;
        }
        match later.next_due() {
            None => later.wake.notified().await,
            Some(at) => {
                let _ = tokio::time::timeout_at(at, later.wake.notified()).await;
            }
        }
        if later.is_stopped() {
            return;
        }
        for job in later.take_due(Instant::now()) {
            let Some(engine) = engine.upgrade() else {
                return;
            };
            #[cfg(test)]
            later.rounds.fetch_add(1, Ordering::SeqCst);
            let written = tokio::task::spawn_blocking(move || engine.write_outdated_of(Some(job)));
            match written.await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => tracing::warn!(%job, error = %e, "files out of date not written"),
                Err(e) => tracing::warn!(%job, error = %e, "writing files out of date stopped"),
            }
        }
    }
}

impl Engine {
    /// Writes a job's files once no save has come for the quiet period. Without a Tokio runtime
    /// (some tests) the mark simply waits for the next write.
    pub(crate) fn write_later(&self, job: JobId) {
        let later = &self.later;
        if later.is_stopped() {
            return;
        }
        lock(&later.due).insert(job, Instant::now() + self.config.write_delay);
        if !later.started.load(Ordering::SeqCst)
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
            && !later.started.swap(true, Ordering::SeqCst)
        {
            runtime.spawn(run(later.clone(), self.me.clone()));
        }
        later.wake.notify_one();
    }

    /// Writes every file marked out of date, of every job: at launch, for what a crash or a
    /// failed write left. Returns how many sources' files could not be written; each is logged
    /// and reported as the job's `SourceFailed`, keeps its mark, and is tried again at the next
    /// write that asks for it.
    pub fn write_outdated(&self) -> Result<usize, EngineError> {
        self.write_outdated_of(None)
    }

    /// Before the app exits: nothing more is left for later, and everything out of date is
    /// written, waiting at most `limit` (a folder on a share that stopped answering must not
    /// hold the exit; what is not written keeps its mark for the next launch). Returns how many
    /// sources' files could not be written.
    pub fn finish_writing(self: &Arc<Self>, limit: Duration) -> Result<usize, EngineError> {
        self.later.stop();
        let (done, finished) = std::sync::mpsc::channel();
        let engine = Arc::clone(self);
        std::thread::spawn(move || {
            let _ = done.send(engine.write_outdated());
        });
        finished.recv_timeout(limit).unwrap_or_else(|_| {
            Err(EngineError::Output(format!(
                "files out of date still being written after {} s",
                limit.as_secs()
            )))
        })
    }

    /// Writes the files marked out of date, of one job or of every job; how many sources'
    /// files could not be written.
    pub(crate) fn write_outdated_of(&self, job: Option<JobId>) -> Result<usize, EngineError> {
        let marks = self.store.outdated_files(job)?;
        let mut jobs: HashMap<JobId, Job> = HashMap::new();
        let mut failed = 0;
        for mark in marks {
            let job = match jobs.get(&mark.job_id) {
                Some(job) => Ok(job),
                None => match self.store.job(mark.job_id) {
                    Ok(job) => Ok(&*jobs.entry(mark.job_id).or_insert(job)),
                    // Deleted meanwhile: its marks went with it.
                    Err(StoreError::NotFound) => continue,
                    Err(e) => Err(e),
                },
            };
            let written = job
                .map_err(EngineError::from)
                .and_then(|job| self.write_each_output(job, mark.setup_id, mark.source_id));
            let problems = match written {
                Ok((_, problems)) => problems,
                Err(e) => vec![e.to_string()],
            };
            if !problems.is_empty() {
                failed += 1;
            }
            for error in problems {
                tracing::warn!(job = %mark.job_id, source = %mark.source_id, %error,
                    "files out of date not written; kept for the next try");
                self.emit(ProgressEvent::SourceFailed {
                    job_id: mark.job_id,
                    source_id: mark.source_id,
                    error,
                });
            }
        }
        Ok(failed)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.later.stop();
    }
}
