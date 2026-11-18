//! Projects in the job database: what each deletion removes, and what it leaves.

use kalamu_core::{
    Collection, CollectionId, Document, DocumentId, DocumentLink, Project, ProjectId, ProjectStore,
};

use super::*;

fn project(name: &str) -> Project {
    Project {
        id: ProjectId::new(),
        name: name.into(),
        description: String::new(),
        archived: false,
        source_defaults: Default::default(),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    }
}

fn collection(project: &Project, name: &str) -> Collection {
    Collection {
        id: CollectionId::new(),
        project_id: project.id,
        name: name.into(),
        created_at: Timestamp(2),
    }
}

fn document(project: &Project, job: &Job) -> Document {
    Document {
        id: DocumentId::new(),
        project_id: project.id,
        title: "Letter of 1915".into(),
        source: job.sources[0].clone(),
        created_at: Timestamp(3),
        tags: vec!["Letters".into()],
        source_description: Default::default(),
    }
}

#[test]
fn a_project_keeps_its_documents_collections_links_and_choices() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut p = project("Mission archives");
    store.put_project(&p)?;
    let (letters, dossier) = (collection(&p, "Letters"), collection(&p, "Dossier 3"));
    store.put_collection(&letters)?;
    store.put_collection(&dossier)?;
    let doc = document(&p, &job);
    store.put_document(&doc)?;
    // One document in two collections, without a copy.
    store.add_member(letters.id, doc.id)?;
    store.add_member(dossier.id, doc.id)?;
    store.add_member(dossier.id, doc.id)?;
    assert_eq!(store.members(p.id)?.len(), 2);
    assert_eq!(store.documents(p.id)?, vec![doc.clone()]);
    store.put_link(&DocumentLink {
        document_id: doc.id,
        job_id: job.id,
        source_id: job.sources[0].id,
        linked_at: Timestamp(4),
    })?;
    store.set_preferred(
        doc.id,
        Task::TextExtraction,
        Some((job.id, job.setups[0].id)),
    )?;
    let preferred = store.preferred(p.id)?;
    assert_eq!(preferred.len(), 1);
    assert_eq!(preferred[0].task, Task::TextExtraction);

    // Renamed and archived in place.
    p.name = "Missions".into();
    p.archived = true;
    p.updated_at = Timestamp(9);
    store.put_project(&p)?;
    assert_eq!(store.project(p.id)?, p);
    assert_eq!(
        store.collections(p.id)?[0].name,
        "Dossier 3",
        "sorted by name"
    );

    // A collection removed leaves its documents.
    store.delete_collection(dossier.id)?;
    assert_eq!(store.members(p.id)?, vec![(letters.id, doc.id)]);
    assert_eq!(store.documents(p.id)?.len(), 1);

    // A choice cleared.
    store.set_preferred(doc.id, Task::TextExtraction, None)?;
    assert!(store.preferred(p.id)?.is_empty());
    Ok(())
}

#[test]
fn what_attaching_makes_is_written_whole_or_not_at_all() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let p = project("Mission archives");
    store.put_project(&p)?;
    let letters = collection(&p, "Letters");
    store.put_collection(&letters)?;
    let doc = document(&p, &job);
    let link = |document_id, job_id| DocumentLink {
        document_id,
        job_id,
        source_id: job.sources[0].id,
        linked_at: Timestamp(4),
    };

    // The last link names a job that does not exist: the document and its place go too.
    let refused = store.attach_documents(
        std::slice::from_ref(&doc),
        &[(letters.id, doc.id)],
        &[link(doc.id, job.id), link(doc.id, JobId::new())],
    );
    assert!(refused.is_err());
    assert!(store.documents(p.id)?.is_empty());
    assert!(store.members(p.id)?.is_empty());
    assert!(store.links(p.id)?.is_empty());

    store.attach_documents(
        std::slice::from_ref(&doc),
        &[(letters.id, doc.id)],
        &[link(doc.id, job.id)],
    )?;
    assert_eq!(store.documents(p.id)?, vec![doc.clone()]);
    assert_eq!(store.members(p.id)?, vec![(letters.id, doc.id)]);
    assert_eq!(store.links(p.id)?.len(), 1);
    Ok(())
}

#[test]
fn deleting_organisation_never_deletes_jobs_and_deleting_a_job_unlinks_it() -> Result<(), StoreError>
{
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let p = project("Interviews");
    store.put_project(&p)?;
    let doc = document(&p, &job);
    store.put_document(&doc)?;
    let link = DocumentLink {
        document_id: doc.id,
        job_id: job.id,
        source_id: job.sources[0].id,
        linked_at: Timestamp(4),
    };
    store.put_link(&link)?;
    store.set_preferred(
        doc.id,
        Task::TextExtraction,
        Some((job.id, job.setups[0].id)),
    )?;

    // The job goes: the document stays, without its link or its choice.
    store.delete_job(job.id)?;
    assert_eq!(store.documents(p.id)?.len(), 1);
    assert!(store.links(p.id)?.is_empty());
    assert!(store.preferred(p.id)?.is_empty());

    // A document or a project deleted leaves the jobs that read it.
    let job = super::job();
    store.create_job(&job)?;
    let doc = document(&p, &job);
    store.put_document(&doc)?;
    store.put_link(&DocumentLink {
        job_id: job.id,
        document_id: doc.id,
        source_id: job.sources[0].id,
        ..link
    })?;
    store.delete_document(doc.id)?;
    assert!(store.links(p.id)?.is_empty());
    assert!(store.job(job.id).is_ok());
    store.delete_project(p.id)?;
    assert!(matches!(store.project(p.id), Err(StoreError::NotFound)));
    assert!(store.job(job.id).is_ok());
    assert!(matches!(
        store.delete_project(p.id),
        Err(StoreError::NotFound)
    ));
    Ok(())
}

/// Each export names the documents its source is: their links are found by job and source,
/// in every project, oldest first, and no other link comes with them.
#[test]
fn a_source_s_links_are_found_in_every_project() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let mut job = job();
    let mut other = job.sources[0].clone();
    other.id = SourceId::new();
    job.sources.push(other);
    store.create_job(&job)?;
    let (a, b) = (project("Mission archives"), project("Press"));
    store.put_project(&a)?;
    store.put_project(&b)?;
    let mut links = Vec::new();
    for (at, (p, source)) in [(&b, 0), (&a, 0), (&a, 1)].into_iter().enumerate() {
        let doc = document(p, &job);
        store.put_document(&doc)?;
        let link = DocumentLink {
            document_id: doc.id,
            job_id: job.id,
            source_id: job.sources[source].id,
            linked_at: Timestamp(10 + at as i64),
        };
        store.put_link(&link)?;
        links.push(link);
    }
    assert_eq!(
        store.links_of(job.id, job.sources[0].id)?,
        links[..2].to_vec()
    );
    assert_eq!(
        store.links_of(job.id, job.sources[1].id)?,
        links[2..].to_vec()
    );
    assert!(store.links_of(JobId::new(), job.sources[0].id)?.is_empty());
    Ok(())
}

#[test]
fn the_review_queue_s_sample_and_items_kept_aside_survive_a_restart() -> Result<(), StoreError> {
    use kalamu_core::{KeptAside, SampleSpec};

    let dir = tempfile::tempdir().map_err(corrupt)?;
    let path = dir.path().join("kalamu.db");
    let job = job();
    let p = project("Mission archives");
    let sample = SampleSpec {
        seed: 271_828,
        size: 20,
        drawn_at: Timestamp(5),
    };
    let page = KeptAside {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index: Some(1),
        revision: Some("3f9a1c07be52".into()),
        note: "Ask the archivist about the stamp".into(),
        kept_at: Timestamp(6),
    };
    let whole = KeptAside {
        index: None,
        revision: None,
        note: "Start it again next week".into(),
        ..page.clone()
    };
    {
        let store = SqliteStore::open(&path)?;
        store.create_job(&job)?;
        store.put_project(&p)?;
        store.set_review_sample(p.id, "project", Some(&sample))?;
        store.put_kept_aside(p.id, &page)?;
        store.put_kept_aside(p.id, &whole)?;
        // Kept aside again, the item has one note: the last.
        store.put_kept_aside(
            p.id,
            &KeptAside {
                note: "Ask the archivist about the seal".into(),
                ..page.clone()
            },
        )?;
    }

    let store = SqliteStore::open(&path)?;
    assert_eq!(store.review_sample(p.id, "project")?, Some(sample));
    // Another scope has its own sample, none yet.
    assert_eq!(store.review_sample(p.id, "collection:x")?, None);
    let kept = store.kept_aside(p.id)?;
    assert_eq!(kept.len(), 2);
    assert!(kept.contains(&whole));
    assert!(
        kept.iter()
            .any(|k| k.index == Some(1) && k.note == "Ask the archivist about the seal")
    );

    store.remove_kept_aside(p.id, job.id, job.setups[0].id, job.sources[0].id, None)?;
    assert_eq!(store.kept_aside(p.id)?.len(), 1);
    store.set_review_sample(p.id, "project", None)?;
    assert_eq!(store.review_sample(p.id, "project")?, None);

    // Deleting the job deletes what was kept aside of its results.
    store.delete_job(job.id)?;
    assert!(store.kept_aside(p.id)?.is_empty());
    Ok(())
}

#[test]
fn results_set_aside_survive_a_restart_per_scope_and_leave_with_their_job_or_project()
-> Result<(), StoreError> {
    let dir = tempfile::tempdir().map_err(corrupt)?;
    let path = dir.path().join("kalamu.db");
    let (job, other) = (job(), job());
    let p = project("Mission archives");
    let flash = (job.id, job.setups[0].id);
    let theirs = (other.id, other.setups[0].id);
    {
        let store = SqliteStore::open(&path)?;
        store.create_job(&job)?;
        store.create_job(&other)?;
        store.put_project(&p)?;
        store.set_result_aside(p.id, "project", theirs, Some(Timestamp(8)))?;
        store.set_result_aside(p.id, "project", flash, Some(Timestamp(7)))?;
        // Set aside again, it is still one.
        store.set_result_aside(p.id, "project", flash, Some(Timestamp(9)))?;
        store.set_result_aside(p.id, "collection:x", theirs, Some(Timestamp(7)))?;
    }

    let store = SqliteStore::open(&path)?;
    // In the order set aside, each scope its own.
    assert_eq!(
        store.results_set_aside(p.id, "project")?,
        vec![flash, theirs]
    );
    assert_eq!(store.results_set_aside(p.id, "collection:x")?, vec![theirs]);
    assert!(store.results_set_aside(p.id, "collection:y")?.is_empty());

    // Brought back in one scope, it stays aside in the other.
    store.set_result_aside(p.id, "project", theirs, None)?;
    assert_eq!(store.results_set_aside(p.id, "project")?, vec![flash]);
    assert_eq!(store.results_set_aside(p.id, "collection:x")?, vec![theirs]);

    // Deleting a job deletes its results set aside; deleting the project, all of them.
    store.delete_job(job.id)?;
    assert!(store.results_set_aside(p.id, "project")?.is_empty());
    store.delete_project(p.id)?;
    assert!(store.results_set_aside(p.id, "collection:x")?.is_empty());
    let left: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM review_set_aside", [], |row| {
            row.get(0)
        })
        .map_err(db)?;
    assert_eq!(left, 0);
    Ok(())
}
