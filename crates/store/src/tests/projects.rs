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
