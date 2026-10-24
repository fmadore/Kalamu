//! Job orchestration: the one place that calls providers, so every call is cached, resumable,
//! retried and reported.
//!
//! A job runs one or more set-ups over its sources. Each piece (page or segment) is recorded in
//! the store before and after its call; a result is keyed by everything that shapes the answer,
//! so re-running a folder, adding a set-up or resuming after a crash only pays for new work.

mod compare;
mod output;
mod run;
mod setup;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use scriptorium_catalog::Catalog;
use scriptorium_core::{
    ApiKey, Correction, InputKind, InputWarning, Job, JobId, JobStatus, JobStore, JobSummary,
    PieceStatus, ProgressEvent, Provider, ProviderError, ProviderId, SetupId, SourceFile, SourceId,
    StopSignal, StoreError, Timestamp,
};
use scriptorium_media::{Ffmpeg, MediaError, OpenSource};
use scriptorium_prompts::{PromptError, PromptLibrary};
use thiserror::Error;
use tokio::sync::broadcast;

pub use compare::{
    ComparisonReport, DiffKind, DiffSpan, PageComparison, SetupScore, diff, error_rate,
};
pub use run::PieceView;
pub use setup::{NewJob, SetupRequest};

/// Where the engine gets API keys; the app reads them from Windows Credential Manager.
pub trait KeySource: Send + Sync {
    fn key(&self, provider: &ProviderId) -> Option<ApiKey>;
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Pieces sent at the same time, across set-ups. zmo-ai-pipelines defaults to 2.
    pub concurrency: usize,
    /// Calls per piece before it is marked as failed.
    pub max_attempts: u32,
    /// First retry delay; doubled on each attempt.
    pub base_delay: Duration,
    /// Version written into provenance.
    pub app_version: String,
    /// The bundled FFmpeg, for recordings and videos; `None` when it was not found.
    pub ffmpeg: Option<Ffmpeg>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            concurrency: 2,
            max_attempts: 4,
            base_delay: Duration::from_secs(5),
            app_version: "0.0.0".into(),
            ffmpeg: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error(transparent)]
    Prompt(#[from] PromptError),
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),
    #[error("No API key saved for {0}")]
    MissingKey(String),
    /// An offline model not downloaded, or a processor that cannot run it.
    #[error("{0}")]
    NotReady(ProviderError),
    #[error("{0} cannot do this task")]
    UnsupportedModel(String),
    #[error("The job is already running")]
    AlreadyRunning,
    #[error("The job needs at least one file and one set-up")]
    EmptyJob,
    #[error("Could not write the results: {0}")]
    Output(String),
}

impl EngineError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Store(StoreError::NotFound) => "job.not_found",
            Self::Store(_) => "job.database",
            Self::Media(MediaError::Encrypted) => "media.encrypted",
            Self::Media(MediaError::UnsupportedKind) => "media.unsupported",
            Self::Media(_) => "media.read",
            Self::Prompt(_) => "prompt.invalid",
            Self::UnknownProvider(_) => "provider.unknown",
            Self::MissingKey(_) => "provider.missing_key",
            Self::NotReady(e) => e.code(),
            Self::UnsupportedModel(_) => "provider.unsupported",
            Self::AlreadyRunning => "job.running",
            Self::EmptyJob => "job.empty",
            Self::Output(_) => "output.write",
        }
    }
}

/// Pause and cancel requests for a running job, checked between pieces. `stop` reaches the
/// pieces in flight, so long offline work can stop part-way.
#[derive(Default)]
pub(crate) struct Control {
    pub pause: bool,
    pub cancel: bool,
    pub stop: StopSignal,
}

pub struct Engine {
    pub(crate) store: Arc<dyn JobStore>,
    pub(crate) providers: Vec<Arc<dyn Provider>>,
    pub(crate) catalog: RwLock<Catalog>,
    pub(crate) prompts: Mutex<PromptLibrary>,
    pub(crate) keys: Arc<dyn KeySource>,
    pub(crate) config: EngineConfig,
    pub(crate) events: broadcast::Sender<ProgressEvent>,
    pub(crate) running: Mutex<HashMap<JobId, Arc<Mutex<Control>>>>,
}

impl Engine {
    pub fn new(
        store: Arc<dyn JobStore>,
        providers: Vec<Arc<dyn Provider>>,
        catalog: Catalog,
        prompts: PromptLibrary,
        keys: Arc<dyn KeySource>,
        config: EngineConfig,
    ) -> Arc<Self> {
        let (events, _) = broadcast::channel(1024);
        Arc::new(Self {
            store,
            providers,
            catalog: RwLock::new(catalog),
            prompts: Mutex::new(prompts),
            keys,
            config,
            events,
            running: Mutex::new(HashMap::new()),
        })
    }

    /// Call once at startup: jobs that were running when the app closed become resumable.
    pub fn recover(&self) -> Result<Vec<JobId>, EngineError> {
        Ok(self.store.mark_running_as_interrupted()?)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.events.subscribe()
    }

    pub(crate) fn emit(&self, event: ProgressEvent) {
        // No subscriber (no window open) is fine: progress is also in the store.
        let _ = self.events.send(event);
    }

    pub fn catalog(&self) -> Catalog {
        self.catalog
            .read()
            .map(|c| c.clone())
            .unwrap_or_else(|p| p.into_inner().clone())
    }

    pub fn set_catalog(&self, catalog: Catalog) {
        let mut guard = self.catalog.write().unwrap_or_else(|p| p.into_inner());
        *guard = catalog;
    }

    pub fn with_prompts<T>(&self, f: impl FnOnce(&mut PromptLibrary) -> T) -> T {
        let mut guard = self.prompts.lock().unwrap_or_else(|p| p.into_inner());
        f(&mut guard)
    }

    pub fn provider(&self, id: &ProviderId) -> Result<Arc<dyn Provider>, EngineError> {
        self.providers
            .iter()
            .find(|p| &p.id() == id)
            .cloned()
            .ok_or_else(|| EngineError::UnknownProvider(id.to_string()))
    }

    pub fn jobs(&self) -> Result<Vec<JobSummary>, EngineError> {
        Ok(self.store.list_jobs()?)
    }

    pub fn job(&self, id: JobId) -> Result<Job, EngineError> {
        Ok(self.store.job(id)?)
    }

    pub fn pieces(&self, id: JobId) -> Result<Vec<scriptorium_core::PieceState>, EngineError> {
        Ok(self.store.pieces(id)?)
    }

    pub fn delete_job(&self, id: JobId) -> Result<(), EngineError> {
        self.cancel(id);
        Ok(self.store.delete_job(id)?)
    }

    /// Inspects files for a new job: hash, kind, page count or length, and the input checks.
    pub fn inspect(
        &self,
        paths: &[impl AsRef<Path>],
    ) -> Vec<Result<(SourceFile, Vec<InputWarning>), EngineError>> {
        let ffmpeg = self.config.ffmpeg.as_ref();
        paths
            .iter()
            .map(|p| {
                let source = scriptorium_media::inspect(p.as_ref(), ffmpeg)?;
                let warnings = match source.kind {
                    InputKind::Audio | InputKind::Video => {
                        scriptorium_media::recording_warnings(&source, ffmpeg)
                    }
                    _ => OpenSource::open(&source, ffmpeg)
                        .map(|open| open.warnings(source.id))
                        .unwrap_or_default(),
                };
                Ok((source, warnings))
            })
            .collect()
    }

    /// The bundled FFmpeg, for the app's previews.
    pub fn ffmpeg(&self) -> Option<&Ffmpeg> {
        self.config.ffmpeg.as_ref()
    }

    /// How long a recording's pieces may be for every set-up of a job: twenty minutes, or less
    /// when a model accepts less (with room for the last piece, which may run a quarter over).
    pub(crate) fn piece_length(&self, job: &Job) -> u64 {
        let catalog = self.catalog();
        job.setups
            .iter()
            .filter_map(|s| catalog.model(&s.provider, &s.model))
            .filter_map(|m| m.limits.max_audio_minutes)
            .map(|minutes| u64::from(minutes) * 60_000 * 4 / 5)
            .fold(scriptorium_media::DEFAULT_PIECE_MS, u64::min)
    }

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
    pub fn start(self: &Arc<Self>, id: JobId) -> Result<(), EngineError> {
        let control = Arc::new(Mutex::new(Control::default()));
        {
            let mut running = self.running.lock().unwrap_or_else(|p| p.into_inner());
            if running.contains_key(&id) {
                return Err(EngineError::AlreadyRunning);
            }
            running.insert(id, control.clone());
        }
        let engine = self.clone();
        tokio::spawn(async move {
            let status = engine.run(id, control).await;
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

    /// Saves the user's correction or review mark for one piece and rewrites that source's
    /// outputs so files on disk always match what the review view shows.
    pub fn save_correction(&self, correction: Correction) -> Result<Vec<String>, EngineError> {
        let mut correction = correction;
        correction.updated_at = Timestamp::now();
        self.store.put_correction(&correction)?;
        let job = self.store.job(correction.job_id)?;
        self.write_outputs(&job, correction.setup_id, correction.source_id)
    }

    /// The outputs for one source and set-up, written again from stored results.
    pub fn export(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Vec<String>, EngineError> {
        let job = self.store.job(job_id)?;
        self.write_outputs(&job, setup_id, source_id)
    }
}

/// Pieces of a job by status, for summaries.
pub fn count(pieces: &[scriptorium_core::PieceState], status: PieceStatus) -> usize {
    pieces.iter().filter(|p| p.status == status).count()
}

pub(crate) fn is_retryable(err: &ProviderError) -> bool {
    err.is_retryable()
}

#[cfg(test)]
mod tests;
