//! The screening table reads each job a project's links name once, whatever the number of
//! documents linked to it (#244): a job database that counts the jobs it is asked for.

use std::sync::atomic::{AtomicUsize, Ordering};

use kalamu_core::{
    BatchRecord, CacheKey, Charge, JobId, JobProgress, JobSources, JobSummary, OutdatedFiles,
    SpeakerNames, StoreError,
};

use super::*;

/// The fixture's database, counting every job read.
struct Counting {
    store: Arc<SqliteStore>,
    reads: AtomicUsize,
}

impl JobStore for Counting {
    fn create_job(&self, job: &Job) -> Result<(), StoreError> {
        self.store.create_job(job)
    }

    fn job(&self, id: JobId) -> Result<Job, StoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.store.job(id)
    }

    fn list_jobs(&self) -> Result<Vec<JobSummary>, StoreError> {
        self.store.list_jobs()
    }

    fn job_sources(&self) -> Result<Vec<JobSources>, StoreError> {
        self.store.job_sources()
    }

    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError> {
        self.store.set_job_status(id, status)
    }

    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError> {
        self.store.rename_job(id, name)
    }

    fn delete_job(&self, id: JobId) -> Result<(), StoreError> {
        self.store.delete_job(id)
    }

    fn delete_jobs(&self, ids: &[JobId]) -> Result<(), StoreError> {
        self.store.delete_jobs(ids)
    }

    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError> {
        self.store.mark_running_as_interrupted()
    }

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError> {
        self.store.upsert_piece(piece)
    }

    fn upsert_pieces(&self, pieces: &[PieceState]) -> Result<(), StoreError> {
        self.store.upsert_pieces(pieces)
    }

    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError> {
        self.store.pieces(job)
    }

    fn job_progress(&self, job: JobId, sources: &[SourceId]) -> Result<JobProgress, StoreError> {
        self.store.job_progress(job, sources)
    }

    fn source_pieces(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<PieceState>, StoreError> {
        self.store.source_pieces(job, setup, source)
    }

    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError> {
        self.store.cached_result(key)
    }

    fn spent(&self, job: JobId) -> Result<f64, StoreError> {
        self.store.spent(job)
    }

    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError> {
        self.store.add_charge(charge)
    }

    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError> {
        self.store.charges(job)
    }

    fn source_charges(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Charge>, StoreError> {
        self.store.source_charges(job, setup, source)
    }

    fn prune_results(&self) -> Result<kalamu_core::PrunedResults, StoreError> {
        self.store.prune_results()
    }

    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError> {
        self.store.put_result(result)
    }

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError> {
        self.store.put_correction(correction)
    }

    fn put_corrections(&self, corrections: &[Correction]) -> Result<(), StoreError> {
        self.store.put_corrections(corrections)
    }

    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError> {
        self.store.corrections(job)
    }

    fn source_corrections(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Correction>, StoreError> {
        self.store.source_corrections(job, setup, source)
    }

    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError> {
        self.store.put_speaker_names(job, names)
    }

    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError> {
        self.store.speaker_names(job)
    }

    fn source_speaker_names(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<SpeakerNames>, StoreError> {
        self.store.source_speaker_names(job, setup, source)
    }

    fn outdated_files(&self, job: Option<JobId>) -> Result<Vec<OutdatedFiles>, StoreError> {
        self.store.outdated_files(job)
    }

    fn clear_outdated(&self, files: &OutdatedFiles) -> Result<(), StoreError> {
        self.store.clear_outdated(files)
    }

    fn put_batch(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        self.store.put_batch(batch)
    }

    fn batches(&self, job: JobId) -> Result<Vec<BatchRecord>, StoreError> {
        self.store.batches(job)
    }

    fn batch_leftovers(&self) -> Result<Vec<BatchRecord>, StoreError> {
        self.store.batch_leftovers()
    }

    fn put_batch_leftover(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        self.store.put_batch_leftover(batch)
    }

    fn put_snapshot(&self, snapshot: &kalamu_core::Snapshot) -> Result<(), StoreError> {
        self.store.put_snapshot(snapshot)
    }

    fn snapshot(&self, id: &str) -> Result<kalamu_core::Snapshot, StoreError> {
        self.store.snapshot(id)
    }

    fn put_unit(&self, unit: &kalamu_core::DocumentUnit) -> Result<(), StoreError> {
        self.store.put_unit(unit)
    }

    fn unit(&self, id: kalamu_core::UnitId) -> Result<kalamu_core::DocumentUnit, StoreError> {
        self.store.unit(id)
    }

    fn units(&self, job: JobId) -> Result<Vec<kalamu_core::DocumentUnit>, StoreError> {
        self.store.units(job)
    }

    fn delete_unit(&self, id: kalamu_core::UnitId) -> Result<(), StoreError> {
        self.store.delete_unit(id)
    }

    fn replace_units(
        &self,
        put: &[kalamu_core::DocumentUnit],
        delete: &[kalamu_core::UnitId],
    ) -> Result<(), StoreError> {
        self.store.replace_units(put, delete)
    }

    fn create_run(&self, run: &kalamu_core::WorkflowRun) -> Result<(), StoreError> {
        self.store.create_run(run)
    }

    fn run(&self, id: kalamu_core::RunId) -> Result<kalamu_core::WorkflowRun, StoreError> {
        self.store.run(id)
    }

    fn runs(&self) -> Result<Vec<kalamu_core::WorkflowRun>, StoreError> {
        self.store.runs()
    }

    fn set_step_progress(
        &self,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        self.store.set_step_progress(run, index, progress)
    }

    fn create_step_job(
        &self,
        job: &Job,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        self.store.create_step_job(job, run, index, progress)
    }

    fn cancel_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        self.store.cancel_run(id)
    }

    fn delete_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        self.store.delete_run(id)
    }

    fn run_of_job(&self, job: JobId) -> Result<Option<(kalamu_core::RunId, u32)>, StoreError> {
        self.store.run_of_job(job)
    }
}

#[test]
fn the_table_reads_each_linked_job_once_whatever_the_links() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let files: Vec<SourceFile> = (0..20)
        .map(|n| f.file(&format!("{n}.pdf"), &n.to_string()))
        .collect();
    f.projects.add_documents(p.id, files.clone(), None).unwrap();
    // Twenty documents linked to one extraction and to one screening: two jobs.
    let extraction = f.job(Task::TextExtraction, files.clone());
    let screening = f.screening(files, &["Relevant"; 20]);
    f.projects
        .link_jobs(p.id, &[extraction.id, screening.id], None)
        .unwrap();

    let counting = Arc::new(Counting {
        store: f.store.clone(),
        reads: AtomicUsize::new(0),
    });
    let projects = Projects::new(f.store.clone(), counting.clone());
    let table = projects
        .answers_table(p.id, screening.id, screening.setups[0].id)
        .unwrap();
    assert_eq!(table.rows.len(), 20);
    assert_eq!(
        counting.reads.load(Ordering::SeqCst),
        2,
        "one read per distinct job, not one per link"
    );
}
