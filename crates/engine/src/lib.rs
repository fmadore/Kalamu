//! Job orchestration: the one place that calls providers, so every call is cached, resumable,
//! retried and reported.
//!
//! A job runs one or more set-ups over its sources. Each piece (page or segment) is recorded in
//! the store before and after its call; a result is keyed by everything that shapes the answer,
//! so re-running a folder, adding a set-up or resuming after a crash only pays for new work.

mod attention;
mod batch;
mod compare;
mod contents;
mod control;
mod document;
mod error;
mod estimate;
mod group;
mod later;
mod listen;
mod output;
mod piece;
mod plan;
mod realign;
mod record;
mod review;
mod rules;
mod run;
mod setup;
mod snapshot;
mod units;
mod workflow;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};
use std::time::Duration;

use crate::control::lock;
use kalamu_catalog::Catalog;
use kalamu_core::{
    ApiKey, Correction, InputKind, InputWarning, Job, JobId, JobStore, JobSummary, ProgressEvent,
    Provider, ProviderId, SetupId, SourceDocuments, SourceFile, SourceId, SpeakerNames, Timestamp,
};
use kalamu_media::{Ffmpeg, OpenSource, Playback};
use kalamu_prompts::PromptLibrary;
use tokio::sync::broadcast;

pub use compare::{
    ComparisonReport, DiffKind, DiffSpan, JobScores, PageComparison, SetupScore, SourceScores,
    diff, error_rate,
};
pub(crate) use control::Control;
pub use error::EngineError;
pub use estimate::{Estimate, EstimateRequest, EstimateRow};
pub use kalamu_core::SetupRequest;
pub use listen::{LISTENING_CACHE_BYTES, PieceAudio};
pub use review::PieceView;
pub use rules::{ModelRef, TaskRules, task_rules};
pub use setup::NewJob;
pub use snapshot::{DerivedState, Freshness};
pub use units::{SeedReport, UnitEdit, UnitView, UnitsView};
pub use workflow::{JobStep, RunView, StartRun, StepView};

/// Where the engine gets API keys; the app reads them from the system keychain.
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
    /// How often a waiting batch is asked about, in economy mode.
    pub batch_poll: Duration,
    /// Where pieces converted for listening in the review view are kept.
    pub listening_cache: PathBuf,
    /// What the review view's player plays, given by the app for the webview it runs in.
    pub playback: Playback,
    /// Where frozen copies of corrected text are kept for later jobs to read.
    pub snapshots: PathBuf,
    /// How long after a job's last correction or speakers' names its files are written again.
    pub write_delay: Duration,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            concurrency: 2,
            max_attempts: 4,
            base_delay: Duration::from_secs(5),
            app_version: "0.0.0".into(),
            ffmpeg: None,
            batch_poll: Duration::from_secs(60),
            listening_cache: std::env::temp_dir().join("kalamu-listening"),
            playback: Playback::default(),
            snapshots: std::env::temp_dir().join("kalamu-snapshots"),
            write_delay: Duration::from_secs(3),
        }
    }
}

pub struct Engine {
    pub(crate) store: Arc<dyn JobStore>,
    pub(crate) providers: Vec<Arc<dyn Provider>>,
    /// Shared, so a call reads it without copying it; replaced whole when refreshed.
    pub(crate) catalog: RwLock<Arc<Catalog>>,
    pub(crate) prompts: Mutex<PromptLibrary>,
    pub(crate) keys: Arc<dyn KeySource>,
    pub(crate) config: EngineConfig,
    pub(crate) events: broadcast::Sender<ProgressEvent>,
    pub(crate) running: Mutex<HashMap<JobId, Arc<Mutex<Control>>>>,
    /// Held while a workflow run decides which steps to start.
    pub(crate) workflows: Mutex<()>,
    /// Held while a result's files are built and written, so a rewrite of its provenance never
    /// lands after a fuller write from older state, nor the other way round.
    pub(crate) writing: Mutex<()>,
    /// The project documents a source is, named in its results' provenance; none until the app
    /// gives them.
    pub(crate) documents: OnceLock<Arc<dyn SourceDocuments>>,
    /// Files out of date, written once the researcher stops saving.
    pub(crate) later: Arc<later::Later>,
    /// The engine itself, for the task that writes them.
    pub(crate) me: Weak<Engine>,
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
        Arc::new_cyclic(|me| Self {
            store,
            providers,
            catalog: RwLock::new(Arc::new(catalog)),
            prompts: Mutex::new(prompts),
            keys,
            config,
            events,
            running: Mutex::new(HashMap::new()),
            workflows: Mutex::new(()),
            writing: Mutex::new(()),
            documents: OnceLock::new(),
            later: Arc::default(),
            me: me.clone(),
        })
    }

    /// Where the documents a source is are read from, for its results' provenance. Set once,
    /// at startup; a second call changes nothing.
    pub fn set_source_documents(&self, documents: Arc<dyn SourceDocuments>) {
        let _ = self.documents.set(documents);
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

    /// A copy of the catalogue in use.
    pub fn catalog(&self) -> Catalog {
        Catalog::clone(&self.current_catalog())
    }

    /// The catalogue in use, shared: what the engine reads several times per piece.
    pub(crate) fn current_catalog(&self) -> Arc<Catalog> {
        self.catalog
            .read()
            .map(|c| Arc::clone(&c))
            .unwrap_or_else(|p| Arc::clone(&p.into_inner()))
    }

    pub fn set_catalog(&self, catalog: Catalog) {
        let mut guard = self.catalog.write().unwrap_or_else(|p| p.into_inner());
        *guard = Arc::new(catalog);
    }

    pub fn with_prompts<T>(&self, f: impl FnOnce(&mut PromptLibrary) -> T) -> T {
        let mut guard = lock(&self.prompts);
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

    /// The names of every job's sources, for finding a job by a file it read.
    pub fn job_sources(&self) -> Result<Vec<kalamu_core::JobSources>, EngineError> {
        Ok(self.store.job_sources()?)
    }

    pub fn job(&self, id: JobId) -> Result<Job, EngineError> {
        Ok(self.store.job(id)?)
    }

    pub fn pieces(&self, id: JobId) -> Result<Vec<kalamu_core::PieceState>, EngineError> {
        Ok(self.store.pieces(id)?)
    }

    /// Renames a job, running or not: the name is only its label, and the files it writes are
    /// named after their sources. A blank name leaves the old one.
    pub fn rename_job(&self, id: JobId, name: &str) -> Result<(), EngineError> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(());
        }
        Ok(self.store.rename_job(id, name)?)
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
                let source = kalamu_media::inspect(p.as_ref(), ffmpeg)?;
                let warnings = match source.kind {
                    InputKind::Audio | InputKind::Video => {
                        kalamu_media::recording_warnings(&source, ffmpeg)
                    }
                    _ => OpenSource::open(&source, ffmpeg)
                        .map(|open| open.warnings(source.id))
                        .unwrap_or_default(),
                };
                Ok((source, warnings))
            })
            .collect()
    }

    /// YouTube links pasted for a new job, each checked and turned into a source. The video is
    /// not fetched: its length and whether it is public are known when the provider reads it.
    pub fn inspect_links(&self, links: &[impl AsRef<str>]) -> Vec<Result<SourceFile, EngineError>> {
        links
            .iter()
            .map(|link| {
                let link = link.as_ref();
                SourceFile::youtube(link).ok_or_else(|| EngineError::NotYouTube(link.to_string()))
            })
            .collect()
    }

    /// The bundled FFmpeg, for the app's previews.
    pub fn ffmpeg(&self) -> Option<&Ffmpeg> {
        self.config.ffmpeg.as_ref()
    }

    /// How long a recording's pieces may be for every set-up of a job: twenty minutes, or less
    /// when a model accepts less (with room for the last piece, which may run a quarter over).
    /// How many pieces a recording is cut into at the usual length, for New job's diagram. The
    /// cuts move to pauses when the job runs, so this is the count before they are known.
    pub fn recording_pieces(source: &SourceFile) -> Option<u32> {
        match source.kind {
            InputKind::Audio | InputKind::Video => source
                .duration_ms
                .map(|ms| kalamu_media::plan(ms, &[], kalamu_media::DEFAULT_PIECE_MS).len() as u32),
            _ => None,
        }
    }

    pub(crate) fn piece_length(&self, job: &Job) -> u64 {
        let catalog = self.current_catalog();
        job.setups
            .iter()
            .filter_map(|s| catalog.model(&s.provider, &s.model))
            .filter_map(|m| m.limits.max_audio_minutes)
            .map(|minutes| u64::from(minutes) * 60_000 * 4 / 5)
            .fold(kalamu_media::DEFAULT_PIECE_MS, u64::min)
    }

    /// Saves the user's correction or review mark for one piece. Its source's files are
    /// marked out of date with it and written once the researcher stops saving
    /// ([`Engine::write_outdated`]), or sooner when they are listed or exported.
    pub fn save_correction(&self, correction: Correction) -> Result<(), EngineError> {
        let mut correction = correction;
        correction.updated_at = Timestamp::now();
        self.store.put_correction(&correction)?;
        self.write_later(correction.job_id);
        Ok(())
    }

    /// Saves the names given to one set-up's speakers in one recording; its files are written
    /// again with them as after a correction.
    pub fn save_speaker_names(
        &self,
        job_id: JobId,
        names: SpeakerNames,
    ) -> Result<(), EngineError> {
        self.store.put_speaker_names(job_id, &names)?;
        self.write_later(job_id);
        Ok(())
    }

    /// Deletes the stored answers no job's pieces refer to any more, those of deleted jobs
    /// above all, and gives the space back to the disk: how many, and how many bytes. Safe
    /// while a job runs, since a piece records its key before its answer is stored; the
    /// database is held while it is compacted, which on a large one takes a while.
    pub fn prune_results(&self) -> Result<kalamu_core::PrunedResults, EngineError> {
        Ok(self.store.prune_results()?)
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

/// A source's file name without its extension, for the names of what is made from it (a page
/// sent to a provider, an output file); `fallback` for a name that has none (`.pdf`, empty).
pub(crate) fn source_stem(file_name: &str, fallback: &str) -> String {
    Path::new(file_name).file_stem().map_or_else(
        || fallback.to_string(),
        |s| s.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests;
