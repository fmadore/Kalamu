//! A project saved to a file and opened in another job database reads as it did: the same
//! documents in the same collections, with the same results, reviews, choices and tags (#213).

use kalamu_core::{ArchivePlaces, ProjectArchive};

use super::*;

/// The project's view as the researcher reads it, without the identifiers and dates that a
/// project opened from a file has anew.
/// A document as seen: its title, its collections, its results by task and its tags.
type Seen = (String, Vec<String>, Vec<TaskBrief>, Vec<String>);

fn seen(view: &ProjectView) -> Vec<Seen> {
    view.documents
        .iter()
        .map(|row| {
            let collections = row
                .collections
                .iter()
                .filter_map(|id| view.collections.iter().find(|c| c.collection.id == *id))
                .map(|c| c.collection.name.clone())
                .collect();
            (
                row.document.title.clone(),
                collections,
                row.tasks.clone(),
                row.document.tags.clone(),
            )
        })
        .collect()
}

#[test]
fn a_project_opened_from_its_file_reads_as_it_did() {
    let f = fixture();
    let p = f.projects.create("Mission archives", "Letters").unwrap();
    let letters = f.projects.create_collection(p.id, "Letters").unwrap();
    let (a, b) = (
        f.file("letter-1915.pdf", "a"),
        f.file("letter-1916.pdf", "b"),
    );
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone(), b.clone()], Some(letters.id))
        .unwrap();
    let job = f.job(Task::TextExtraction, vec![a, b]);
    f.projects.link_job(p.id, job.id, None).unwrap();
    f.projects
        .set_preferred(
            p.id,
            docs[0],
            Task::TextExtraction,
            Some((job.id, job.setups[0].id)),
        )
        .unwrap();
    f.projects.tag_documents(p.id, &[docs[1]], "Kept").unwrap();

    let places = ArchivePlaces {
        snapshots: f.dir.path().join("snapshots"),
        originals: f.dir.path().join("originals"),
        working: f.dir.path().join("working"),
        app_version: "1.9.0".into(),
    };
    let path = f.dir.path().join("Mission archives.kalamu");
    f.store
        .save_project_file(p.id, &path, false, &places, &|_| {})
        .unwrap();
    let there = Arc::new(SqliteStore::in_memory().unwrap());
    let opened = there.open_project_file(&path, &places, &|_| {}).unwrap();
    let projects = Projects::new(there.clone(), there.clone());

    let before = f.projects.view(p.id).unwrap();
    let after = projects.view(opened.project_id).unwrap();
    assert_eq!(seen(&after), seen(&before));
    assert_eq!(after.project.name, "Mission archives");
    assert_eq!(after.jobs.len(), 1);
    assert_eq!(projects.list().unwrap()[0].jobs, 1);
}
