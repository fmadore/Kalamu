//! A job database that fails to read any job, as a locked or damaged file would: search must
//! say so rather than count the documents as having nothing read.

use kalamu_core::{
    BatchRecord, CacheKey, Charge, JobProgress, JobSources, JobSummary, OutdatedFiles, SetupId,
    SourceId, StoreError,
};

use super::*;

struct Failing(Arc<SqliteStore>);

impl JobStore for Failing {
    fn create_job(&self, job: &Job) -> Result<(), StoreError> {
        self.0.create_job(job)
    }

    fn job(&self, _id: JobId) -> Result<Job, StoreError> {
        Err(StoreError::Database("disk I/O error".into()))
    }

    fn list_jobs(&self) -> Result<Vec<JobSummary>, StoreError> {
        self.0.list_jobs()
    }

    fn job_sources(&self) -> Result<Vec<JobSources>, StoreError> {
        self.0.job_sources()
    }

    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError> {
        self.0.set_job_status(id, status)
    }

    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError> {
        self.0.rename_job(id, name)
    }

    fn delete_job(&self, id: JobId) -> Result<(), StoreError> {
        self.0.delete_job(id)
    }

    fn delete_jobs(&self, ids: &[JobId]) -> Result<(), StoreError> {
        self.0.delete_jobs(ids)
    }

    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError> {
        self.0.mark_running_as_interrupted()
    }

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError> {
        self.0.upsert_piece(piece)
    }

    fn upsert_pieces(&self, pieces: &[PieceState]) -> Result<(), StoreError> {
        self.0.upsert_pieces(pieces)
    }

    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError> {
        self.0.pieces(job)
    }

    fn job_progress(&self, job: JobId, sources: &[SourceId]) -> Result<JobProgress, StoreError> {
        self.0.job_progress(job, sources)
    }

    fn source_pieces(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<PieceState>, StoreError> {
        self.0.source_pieces(job, setup, source)
    }

    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError> {
        self.0.cached_result(key)
    }

    fn spent(&self, job: JobId) -> Result<f64, StoreError> {
        self.0.spent(job)
    }

    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError> {
        self.0.add_charge(charge)
    }

    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError> {
        self.0.charges(job)
    }

    fn source_charges(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Charge>, StoreError> {
        self.0.source_charges(job, setup, source)
    }

    fn prune_results(&self) -> Result<kalamu_core::PrunedResults, StoreError> {
        self.0.prune_results()
    }

    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError> {
        self.0.put_result(result)
    }

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError> {
        self.0.put_correction(correction)
    }

    fn put_corrections(&self, corrections: &[Correction]) -> Result<(), StoreError> {
        self.0.put_corrections(corrections)
    }

    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError> {
        self.0.corrections(job)
    }

    fn source_corrections(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Correction>, StoreError> {
        self.0.source_corrections(job, setup, source)
    }

    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError> {
        self.0.put_speaker_names(job, names)
    }

    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError> {
        self.0.speaker_names(job)
    }

    fn source_speaker_names(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<SpeakerNames>, StoreError> {
        self.0.source_speaker_names(job, setup, source)
    }

    fn outdated_files(&self, job: Option<JobId>) -> Result<Vec<OutdatedFiles>, StoreError> {
        self.0.outdated_files(job)
    }

    fn clear_outdated(&self, files: &OutdatedFiles) -> Result<(), StoreError> {
        self.0.clear_outdated(files)
    }

    fn put_batch(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        self.0.put_batch(batch)
    }

    fn batches(&self, job: JobId) -> Result<Vec<BatchRecord>, StoreError> {
        self.0.batches(job)
    }

    fn put_snapshot(&self, snapshot: &kalamu_core::Snapshot) -> Result<(), StoreError> {
        self.0.put_snapshot(snapshot)
    }

    fn snapshot(&self, id: &str) -> Result<kalamu_core::Snapshot, StoreError> {
        self.0.snapshot(id)
    }

    fn put_unit(&self, unit: &kalamu_core::DocumentUnit) -> Result<(), StoreError> {
        self.0.put_unit(unit)
    }

    fn unit(&self, id: kalamu_core::UnitId) -> Result<kalamu_core::DocumentUnit, StoreError> {
        self.0.unit(id)
    }

    fn units(&self, job: JobId) -> Result<Vec<kalamu_core::DocumentUnit>, StoreError> {
        self.0.units(job)
    }

    fn delete_unit(&self, id: kalamu_core::UnitId) -> Result<(), StoreError> {
        self.0.delete_unit(id)
    }

    fn replace_units(
        &self,
        put: &[kalamu_core::DocumentUnit],
        delete: &[kalamu_core::UnitId],
    ) -> Result<(), StoreError> {
        self.0.replace_units(put, delete)
    }

    fn create_run(&self, run: &kalamu_core::WorkflowRun) -> Result<(), StoreError> {
        self.0.create_run(run)
    }

    fn run(&self, id: kalamu_core::RunId) -> Result<kalamu_core::WorkflowRun, StoreError> {
        self.0.run(id)
    }

    fn runs(&self) -> Result<Vec<kalamu_core::WorkflowRun>, StoreError> {
        self.0.runs()
    }

    fn set_step_progress(
        &self,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        self.0.set_step_progress(run, index, progress)
    }

    fn create_step_job(
        &self,
        job: &Job,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        self.0.create_step_job(job, run, index, progress)
    }

    fn cancel_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        self.0.cancel_run(id)
    }

    fn delete_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        self.0.delete_run(id)
    }

    fn run_of_job(&self, job: JobId) -> Result<Option<(kalamu_core::RunId, u32)>, StoreError> {
        self.0.run_of_job(job)
    }
}

#[test]
fn a_job_database_failure_is_said_not_counted_as_nothing_read() {
    let f = Fixture::new();
    let document = f.document("lettre.pdf", InputKind::Pdf);
    f.extraction(&document, &["Le cercle de Dakar."]);
    let search = Search::new(
        f.store.clone(),
        Arc::new(Failing(f.store.clone())),
        f.index.clone(),
    );
    let failed = search.search(f.project, &request("cercle"), &never, &quiet);
    assert!(
        matches!(failed, Err(SearchError::Jobs(StoreError::Database(_)))),
        "{failed:?}"
    );
    assert!(matches!(
        search.prepare(f.project, &never, &quiet),
        Err(SearchError::Jobs(_))
    ));
}
