//! Starting, pausing and cancelling a job: its one run at a time, the requests checked
//! between pieces, and the status it ends with.

use std::sync::{Arc, Mutex};

use kalamu_core::{JobId, JobStatus, PieceStatus, ProgressEvent, StopSignal, Timestamp};

use crate::{Engine, EngineError};

/// Pause and cancel requests for a running job, checked between pieces. `stop` reaches the
/// pieces in flight, so long offline work can stop part-way.
#[derive(Default)]
pub(crate) struct Control {
    pub pause: bool,
    pub cancel: bool,
    pub stop: StopSignal,
}

impl Engine {
    pub fn is_running(&self, id: JobId) -> bool {
        self.running
            .lock()
            .map(|r| r.contains_key(&id))
            .unwrap_or(false)
    }

    /// Stops after the pieces in flight; everything finished is kept and the job can resume.
    pub fn pause(&self, id: JobId) {
        if let Some(control) = self.control(id) {
            let mut control = control.lock().unwrap_or_else(|p| p.into_inner());
            control.pause = true;
            control.stop.stop();
        }
    }

    pub fn cancel(&self, id: JobId) {
        if let Some(control) = self.control(id) {
            let mut control = control.lock().unwrap_or_else(|p| p.into_inner());
            control.cancel = true;
            control.stop.stop();
        }
    }

    fn control(&self, id: JobId) -> Option<Arc<Mutex<Control>>> {
        self.running.lock().ok()?.get(&id).cloned()
    }

    /// Starts or resumes a job in the background. Progress arrives through [`Engine::subscribe`].
    /// Call it from within the Tokio runtime, which runs the job.
    pub fn start(self: &Arc<Self>, id: JobId) -> Result<(), EngineError> {
        self.launch(id, true, |engine, control| async move {
            engine.run(id, control).await
        })
    }

    /// Collects the answers of a job's batches that the provider still holds, and writes the
    /// documents they complete, without sending anything: at launch, a job paused or cut short
    /// by the last close waits for the researcher for the rest.
    pub fn collect_waiting(self: &Arc<Self>, id: JobId) -> Result<(), EngineError> {
        self.launch(id, false, |engine, control| async move {
            engine.collect_batches(id, control).await
        })
    }

    /// Runs `work` for a job in the background, as the job's one run at a time, and records
    /// the status it ends with. A workflow the job belongs to then goes on, starting its next
    /// steps when `then_start` (the researcher started this work), or only making their jobs.
    fn launch<F, Fut>(
        self: &Arc<Self>,
        id: JobId,
        then_start: bool,
        work: F,
    ) -> Result<(), EngineError>
    where
        F: FnOnce(Arc<Self>, Arc<Mutex<Control>>) -> Fut,
        Fut: std::future::Future<Output = Result<JobStatus, EngineError>> + Send + 'static,
    {
        let control = Arc::new(Mutex::new(Control::default()));
        {
            let mut running = self.running.lock().unwrap_or_else(|p| p.into_inner());
            if running.contains_key(&id) {
                return Err(EngineError::AlreadyRunning);
            }
            running.insert(id, control.clone());
        }
        let engine = self.clone();
        let work = work(self.clone(), control);
        tokio::spawn(async move {
            let status = work.await;
            let status = status.unwrap_or_else(|e| {
                tracing::error!(job = %id, error = %e, "job stopped");
                JobStatus::Interrupted
            });
            let _ = engine.store.set_job_status(id, status);
            engine
                .running
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
            engine.emit(ProgressEvent::JobFinished { job_id: id, status });
            engine.job_ended(id, then_start);
        });
        Ok(())
    }

    /// Puts failed pieces back in the queue and starts the job.
    pub fn retry_failed(self: &Arc<Self>, id: JobId) -> Result<(), EngineError> {
        for mut piece in self.store.pieces(id)? {
            if piece.status == PieceStatus::Failed {
                piece.status = PieceStatus::Pending;
                piece.error = None;
                piece.attempts = 0;
                piece.updated_at = Timestamp::now();
                self.store.upsert_piece(&piece)?;
            }
        }
        self.start(id)
    }
}
