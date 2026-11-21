//! What a job reached comes back with it: a finished job opened from a file is finished, never
//! offered to resume; and the jobs a file is said to hold are those the project's card counts.

use kalamu_core::ProposalRequest;

use super::*;

/// The status each of the opened project's jobs has, the analysis's first.
fn statuses(there: &SqliteStore, p: ProjectId) -> Result<Vec<JobStatus>, StoreError> {
    let mut ids: Vec<JobId> = there.links(p)?.iter().map(|l| l.job_id).collect();
    ids.dedup();
    let mut out = Vec::new();
    for id in ids {
        out.push(there.job(id)?.status);
    }
    Ok(out)
}

#[test]
fn a_finished_job_comes_back_finished() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    // As the engine leaves it: the status in its column, the job's data as it was created.
    f.store.set_job_status(f.derived.id, JobStatus::Completed)?;
    let path = f.saved(false)?;
    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    let p = opened.project_id;
    let links = there.links(p)?;
    let analysis = there.job(links[1].job_id)?;
    assert_eq!(analysis.status, JobStatus::Completed);
    assert!(!analysis.status.is_unfinished(), "never offered to resume");
    let listed = there.list_jobs()?;
    let summary = listed
        .iter()
        .find(|j| j.id == analysis.id)
        .ok_or(StoreError::NotFound)?;
    assert_eq!(summary.status, JobStatus::Completed);
    // The job caught running still goes as interrupted.
    assert_eq!(there.job(links[0].job_id)?.status, JobStatus::Interrupted);

    // A status its pieces could not tell is the one the job reached, too.
    f.store.set_job_status(f.derived.id, JobStatus::Cancelled)?;
    let path = f.saved(false)?;
    let again = super::opened(&f, &there, &path, "there")?;
    assert_eq!(
        statuses(&there, again.project_id)?,
        [JobStatus::Interrupted, JobStatus::Cancelled]
    );
    Ok(())
}

#[test]
fn a_file_that_saved_its_jobs_as_waiting_brings_them_back_as_they_were()
-> Result<(), ProjectFileError> {
    let f = fixture()?;
    f.store.set_job_status(f.derived.id, JobStatus::Completed)?;
    let path = f.saved(false)?;
    // As 1.10.0-rc1 wrote it: every job as it was created, waiting to start.
    {
        let file = rusqlite::Connection::open(&path).map_err(db)?;
        file.execute("UPDATE jobs SET status = 'pending'", [])
            .map_err(db)?;
    }
    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    let found = statuses(&there, opened.project_id)?;
    // The extraction has a piece that was held in a batch, now to read again: interrupted;
    // the analysis read its one piece: finished.
    assert_eq!(found, [JobStatus::Interrupted, JobStatus::Completed]);
    Ok(())
}

#[test]
fn a_job_never_started_stays_waiting() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let mut waiting = job();
    waiting.sources[0] = f.job.sources[0].clone();
    f.store.create_job(&waiting)?;
    let doc = f.store.documents(f.project)?[0].id;
    f.store.put_link(&DocumentLink {
        document_id: doc,
        job_id: waiting.id,
        source_id: waiting.sources[0].id,
        linked_at: Timestamp(20),
    })?;
    let path = f.saved(false)?;
    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    let p = opened.project_id;
    let pending = there
        .links(p)?
        .iter()
        .map(|l| there.job(l.job_id))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|j| j.status == JobStatus::Pending)
        .count();
    assert_eq!(pending, 1, "a job with no pieces had not started");
    Ok(())
}

#[test]
fn the_jobs_counted_are_those_the_project_s_card_counts() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    // The dictionary's proposals, asked of a model: a job that reads no document.
    let mut proposals = job();
    proposals.task = Task::Analysis;
    f.store.create_job(&proposals)?;
    f.store.put_request(&ProposalRequest {
        id: "request-1".into(),
        project_id: f.project,
        namespace: "people".into(),
        collection: None,
        job_id: proposals.id,
        groups: Vec::new(),
        rows: 2,
        created_at: Timestamp(15),
    })?;
    // The card's count: the jobs that read at least one of the project's documents.
    let card = |store: &SqliteStore, p: ProjectId| -> Result<u32, StoreError> {
        let mut jobs: Vec<JobId> = store.links(p)?.iter().map(|l| l.job_id).collect();
        jobs.sort();
        jobs.dedup();
        Ok(jobs.len() as u32)
    };
    let plan = f.store.plan_project_file(f.project, &f.places("here"))?;
    assert_eq!(plan.jobs, card(&f.store, f.project)?);
    assert_eq!(plan.jobs, 2);

    let path = f.saved(false)?;
    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    assert_eq!(opened.jobs, card(&there, opened.project_id)?);
    assert_eq!(opened.jobs, 2);
    // The proposal's job is carried all the same, so its answers come with the request.
    assert_eq!(there.list_jobs()?.len(), 3);
    let requests = there.requests(opened.project_id)?;
    assert_eq!(requests.len(), 1);
    assert!(there.job(requests[0].job_id).is_ok());
    Ok(())
}
