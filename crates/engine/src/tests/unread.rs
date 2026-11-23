//! A job database that fails to read the passages corrected by a model, or the regions read
//! again, as a locked or damaged file would (#243): the export says so and writes no
//! `provenance.json` that leaves them out, rather than take the failure for none asked.

use std::sync::atomic::AtomicBool;

use kalamu_core::{
    BatchRecord, Charge, Correction, Job, JobId, JobProgress, JobSources, JobSummary,
    OutdatedFiles, PieceState, SetupId, SourceId, SpeakerNames, StoredResult,
};

use super::*;

/// What the job database fails to read once broken.
#[derive(Clone, Copy, PartialEq)]
enum Unreadable {
    Passages,
    RegionReadings,
}

/// The job database, failing to read one kind of request once `broken` is set.
struct Failing {
    inner: Arc<SqliteStore>,
    unreadable: Unreadable,
    broken: AtomicBool,
}

impl Failing {
    fn fails(&self, what: Unreadable) -> bool {
        self.broken.load(Ordering::SeqCst) && self.unreadable == what
    }
}

fn disk() -> StoreError {
    StoreError::Database("disk I/O error".into())
}

impl JobStore for Failing {
    fn create_job(&self, job: &Job) -> Result<(), StoreError> {
        self.inner.create_job(job)
    }
    fn job(&self, id: JobId) -> Result<Job, StoreError> {
        self.inner.job(id)
    }
    fn list_jobs(&self) -> Result<Vec<JobSummary>, StoreError> {
        self.inner.list_jobs()
    }
    fn job_sources(&self) -> Result<Vec<JobSources>, StoreError> {
        self.inner.job_sources()
    }
    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError> {
        self.inner.set_job_status(id, status)
    }
    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError> {
        self.inner.rename_job(id, name)
    }
    fn delete_job(&self, id: JobId) -> Result<(), StoreError> {
        self.inner.delete_job(id)
    }
    fn delete_jobs(&self, ids: &[JobId]) -> Result<(), StoreError> {
        self.inner.delete_jobs(ids)
    }
    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError> {
        self.inner.mark_running_as_interrupted()
    }
    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError> {
        self.inner.upsert_piece(piece)
    }
    fn upsert_pieces(&self, pieces: &[PieceState]) -> Result<(), StoreError> {
        self.inner.upsert_pieces(pieces)
    }
    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError> {
        self.inner.pieces(job)
    }
    fn job_progress(&self, job: JobId, sources: &[SourceId]) -> Result<JobProgress, StoreError> {
        self.inner.job_progress(job, sources)
    }
    fn source_pieces(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<PieceState>, StoreError> {
        self.inner.source_pieces(job, setup, source)
    }
    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError> {
        self.inner.cached_result(key)
    }
    fn spent(&self, job: JobId) -> Result<f64, StoreError> {
        self.inner.spent(job)
    }
    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError> {
        self.inner.add_charge(charge)
    }
    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError> {
        self.inner.charges(job)
    }
    fn source_charges(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Charge>, StoreError> {
        self.inner.source_charges(job, setup, source)
    }
    fn prune_results(&self) -> Result<kalamu_core::PrunedResults, StoreError> {
        self.inner.prune_results()
    }
    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError> {
        self.inner.put_result(result)
    }
    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError> {
        self.inner.put_correction(correction)
    }
    fn put_corrections(&self, corrections: &[Correction]) -> Result<(), StoreError> {
        self.inner.put_corrections(corrections)
    }
    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError> {
        self.inner.corrections(job)
    }
    fn source_corrections(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Correction>, StoreError> {
        self.inner.source_corrections(job, setup, source)
    }
    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError> {
        self.inner.put_speaker_names(job, names)
    }
    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError> {
        self.inner.speaker_names(job)
    }
    fn source_speaker_names(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<SpeakerNames>, StoreError> {
        self.inner.source_speaker_names(job, setup, source)
    }
    fn outdated_files(&self, job: Option<JobId>) -> Result<Vec<OutdatedFiles>, StoreError> {
        self.inner.outdated_files(job)
    }
    fn clear_outdated(&self, files: &OutdatedFiles) -> Result<(), StoreError> {
        self.inner.clear_outdated(files)
    }
    fn source_passages(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<kalamu_core::PassageProposal>, StoreError> {
        if self.fails(Unreadable::Passages) {
            return Err(disk());
        }
        self.inner.source_passages(job, setup, source)
    }
    fn source_region_readings(
        &self,
        job: JobId,
        source: SourceId,
    ) -> Result<Vec<kalamu_core::RegionReading>, StoreError> {
        if self.fails(Unreadable::RegionReadings) {
            return Err(disk());
        }
        self.inner.source_region_readings(job, source)
    }
    fn source_regions(
        &self,
        job: JobId,
        source: SourceId,
    ) -> Result<Vec<kalamu_core::DrawnRegion>, StoreError> {
        self.inner.source_regions(job, source)
    }
    fn put_batch(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        self.inner.put_batch(batch)
    }
    fn batches(&self, job: JobId) -> Result<Vec<BatchRecord>, StoreError> {
        self.inner.batches(job)
    }
    fn batch_leftovers(&self) -> Result<Vec<BatchRecord>, StoreError> {
        self.inner.batch_leftovers()
    }
    fn put_batch_leftover(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        self.inner.put_batch_leftover(batch)
    }
    fn put_snapshot(&self, snapshot: &kalamu_core::Snapshot) -> Result<(), StoreError> {
        self.inner.put_snapshot(snapshot)
    }
    fn snapshot(&self, id: &str) -> Result<kalamu_core::Snapshot, StoreError> {
        self.inner.snapshot(id)
    }
    fn put_unit(&self, unit: &kalamu_core::DocumentUnit) -> Result<(), StoreError> {
        self.inner.put_unit(unit)
    }
    fn unit(&self, id: kalamu_core::UnitId) -> Result<kalamu_core::DocumentUnit, StoreError> {
        self.inner.unit(id)
    }
    fn units(&self, job: JobId) -> Result<Vec<kalamu_core::DocumentUnit>, StoreError> {
        self.inner.units(job)
    }
    fn delete_unit(&self, id: kalamu_core::UnitId) -> Result<(), StoreError> {
        self.inner.delete_unit(id)
    }
    fn replace_units(
        &self,
        put: &[kalamu_core::DocumentUnit],
        delete: &[kalamu_core::UnitId],
    ) -> Result<(), StoreError> {
        self.inner.replace_units(put, delete)
    }
    fn create_run(&self, run: &kalamu_core::WorkflowRun) -> Result<(), StoreError> {
        self.inner.create_run(run)
    }
    fn run(&self, id: kalamu_core::RunId) -> Result<kalamu_core::WorkflowRun, StoreError> {
        self.inner.run(id)
    }
    fn runs(&self) -> Result<Vec<kalamu_core::WorkflowRun>, StoreError> {
        self.inner.runs()
    }
    fn set_step_progress(
        &self,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        self.inner.set_step_progress(run, index, progress)
    }
    fn create_step_job(
        &self,
        job: &Job,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        self.inner.create_step_job(job, run, index, progress)
    }
    fn cancel_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        self.inner.cancel_run(id)
    }
    fn delete_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        self.inner.delete_run(id)
    }
    fn run_of_job(&self, job: JobId) -> Result<Option<(kalamu_core::RunId, u32)>, StoreError> {
        self.inner.run_of_job(job)
    }
}

/// The `provenance.json` files in `dir`.
fn provenance_files(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with("provenance.json"))
        .collect()
}

/// A page read and its files written; then their `provenance.json` deleted, the database
/// broken, and the source exported again: the error, and what the folder holds after it.
async fn exported_while_unreadable(unreadable: Unreadable) -> (EngineError, Vec<String>) {
    let inner = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let store = Arc::new(Failing {
        inner: inner.clone(),
        unreadable,
        broken: AtomicBool::new(false),
    });
    let fake = Fake::new();
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![fake.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let f = Fixture {
        engine: engine.clone(),
        fake,
        store: inner,
        dir,
    };
    let job = f.new_job(vec![f.pdf("bulletin.pdf", 1)], &["m"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let written = f.dir.path().join("bulletin.provenance.json");
    assert_eq!(
        provenance_files(f.dir.path()),
        vec!["bulletin.provenance.json"]
    );
    std::fs::remove_file(&written).unwrap_or_else(|e| panic!("{e}"));

    store.broken.store(true, Ordering::SeqCst);
    let failed = engine.export(job.id, job.setups[0].id, job.sources[0].id);
    let error = match failed {
        Err(e) => e,
        Ok(files) => panic!("exported as if nothing had been asked: {files:?}"),
    };
    (error, provenance_files(f.dir.path()))
}

#[tokio::test]
async fn passages_that_cannot_be_read_fail_the_export_and_no_provenance_is_written() {
    let (error, provenance) = exported_while_unreadable(Unreadable::Passages).await;
    assert_eq!(error.code(), "job.database");
    let message = error.to_string();
    assert!(
        message.contains("the passages corrected by a model could not be read"),
        "{message}"
    );
    assert!(message.contains("disk I/O error"), "{message}");
    assert!(provenance.is_empty(), "{provenance:?}");
}

#[tokio::test]
async fn region_readings_that_cannot_be_read_fail_the_export_and_no_provenance_is_written() {
    let (error, provenance) = exported_while_unreadable(Unreadable::RegionReadings).await;
    assert_eq!(error.code(), "job.database");
    let message = error.to_string();
    assert!(
        message.contains("the regions read again by a model could not be read"),
        "{message}"
    );
    assert!(message.contains("disk I/O error"), "{message}");
    assert!(provenance.is_empty(), "{provenance:?}");
}
