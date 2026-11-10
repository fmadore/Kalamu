//! Projects over a real job database: documents added without copies, jobs attached after the
//! matches are checked, results gathered by document, and deletions that never reach a job.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use kalamu_core::{
    Correction, InputKind, Job, JobSettings, JobStatus, JobStore, OutputSettings, PieceRef,
    PieceState, PieceStatus, ProcessOutput, PromptSnapshot, ProviderId, Setup, SetupId,
    SetupOptions, SourceFile, SourceId, StoredResult, Task, Timestamp,
};
use kalamu_store::SqliteStore;

use super::*;

mod described;
mod profiled;
mod queue;
mod tags;

struct Fixture {
    projects: Projects,
    store: Arc<SqliteStore>,
    dir: tempfile::TempDir,
}

/// A CSV export's text after the byte-order mark it must start with.
/// A table as the app writes it, without the byte-order mark it must start with.
fn csv_body(table: kalamu_core::Table) -> String {
    let csv = kalamu_export::table_csv(&table).unwrap();
    csv.strip_prefix('\u{feff}')
        .unwrap_or_else(|| panic!("a CSV starts with a byte-order mark, for Excel"))
        .to_string()
}

fn fixture() -> Fixture {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    Fixture {
        projects: Projects::new(store.clone(), store.clone()),
        store,
        dir: tempfile::tempdir().unwrap(),
    }
}

impl Fixture {
    /// A file on disk, as New job would have inspected it.
    fn file(&self, name: &str, content: &str) -> SourceFile {
        let path = self.dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        SourceFile {
            id: SourceId::new(),
            path: path.display().to_string(),
            file_name: name.into(),
            sha256: format!("sha-of-{content}"),
            size_bytes: content.len() as u64,
            modified_ms: None,
            kind: InputKind::Pdf,
            page_count: Some(2),
            duration_ms: None,
            split_spreads: false,
            even_out_volume: false,
            title: None,
            derived_from: None,
        }
    }

    /// A job of `task` over `sources` with one set-up, both pages of each read, the first
    /// reviewed, at $0.01 a page.
    fn job(&self, task: Task, sources: Vec<SourceFile>) -> Job {
        let job = Job {
            id: kalamu_core::JobId::new(),
            name: format!("{} job", task.as_str()),
            task,
            created_at: Timestamp::now(),
            sources,
            setups: vec![Setup {
                id: SetupId::new(),
                label: "gemini-3.8-flash".into(),
                provider: ProviderId::new("gemini"),
                model: "gemini-3.8-flash".into(),
                prompt: PromptSnapshot {
                    preset_id: None,
                    preset_name: "Printed".into(),
                    template: String::new(),
                    text: String::new(),
                    user_prompt: String::new(),
                    schema: None,
                },
                options: SetupOptions::default(),
                merge: None,
                pages_cap: None,
            }],
            settings: JobSettings::default(),
            output: OutputSettings::default(),
            status: JobStatus::Completed,
        };
        self.store.create_job(&job).unwrap();
        for source in &job.sources {
            for index in 0..2 {
                let key = format!("{}-{}-{index}", job.id, source.id);
                self.store
                    .put_result(&StoredResult {
                        cache_key: key.clone(),
                        provider: ProviderId::new("gemini"),
                        model: "gemini-3.8-flash".into(),
                        output: ProcessOutput::default(),
                        request: Vec::new(),
                        cost_usd: Some(0.01),
                        duration_ms: 1,
                        created_at: Timestamp::now(),
                    })
                    .unwrap();
                self.store
                    .upsert_piece(&PieceState {
                        job_id: job.id,
                        setup_id: job.setups[0].id,
                        source_id: source.id,
                        index,
                        piece: PieceRef::page(index + 1),
                        status: PieceStatus::Done,
                        cache_key: Some(key),
                        error: None,
                        attempts: 1,
                        updated_at: Timestamp::now(),
                    })
                    .unwrap();
            }
            self.store
                .put_correction(&Correction {
                    job_id: job.id,
                    setup_id: job.setups[0].id,
                    source_id: source.id,
                    index: 0,
                    text: None,
                    fields: None,
                    reviewed: true,
                    updated_at: Timestamp::now(),
                })
                .unwrap();
        }
        job
    }
}

#[test]
fn documents_are_added_once_and_grouped_without_copies() {
    let f = fixture();
    let p = f
        .projects
        .create("  Mission archives ", "Letters, 1890-1920")
        .unwrap();
    assert_eq!(p.name, "Mission archives");
    assert!(matches!(
        f.projects.create("  ", ""),
        Err(ProjectError::NoName)
    ));
    let letters = f.projects.create_collection(p.id, "Letters").unwrap();
    let dossier = f.projects.create_collection(p.id, "Dossier 3").unwrap();

    let a = f.file("letter-1915.pdf", "a");
    let copy = f.file("copy-of-letter.pdf", "a");
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone(), copy, a.clone()], Some(letters.id))
        .unwrap();
    // The same file twice is one document; the same bytes elsewhere is another, pointed out.
    assert_eq!(docs[0], docs[2]);
    assert_ne!(docs[0], docs[1]);
    f.projects
        .add_to_collection(p.id, dossier.id, &[docs[0]])
        .unwrap();

    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.documents.len(), 2);
    let first = &view.documents[0];
    assert_eq!(first.document.title, "letter-1915");
    assert_eq!(first.collections.len(), 2, "in two collections, held once");
    assert_eq!(first.same_content, 1);
    assert!(!first.missing);
    let counts: Vec<(String, u32)> = view
        .collections
        .iter()
        .map(|c| (c.collection.name.clone(), c.documents))
        .collect();
    assert_eq!(
        counts,
        vec![("Dossier 3".to_string(), 1), ("Letters".to_string(), 2)]
    );

    // Out of a collection, still in the project; a collection deleted, its documents stay.
    f.projects
        .remove_from_collection(p.id, letters.id, &[docs[1]])
        .unwrap();
    f.projects.delete_collection(p.id, letters.id).unwrap();
    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.documents.len(), 2);
    assert_eq!(view.collections.len(), 1);

    // A file moved or deleted since is said to be missing; the document stays.
    std::fs::remove_file(&a.path).unwrap();
    assert!(f.projects.view(p.id).unwrap().documents[0].missing);
}

#[test]
fn an_existing_job_is_attached_as_the_researcher_chooses() {
    let f = fixture();
    let p = f.projects.create("Newspapers", "").unwrap();
    let issue = f.file("issue-12.pdf", "issue");
    let copy = f.file("issue-12 (copy).pdf", "issue");
    let other = f.file("issue-13.pdf", "other");
    let docs = f
        .projects
        .add_documents(p.id, vec![issue.clone(), copy], None)
        .unwrap();

    // A job run earlier, outside the project, on the same file re-inspected (a new source id)
    // and on a file the project does not hold.
    let mut again = issue.clone();
    again.id = SourceId::new();
    let job = f.job(Task::TextExtraction, vec![again.clone(), other.clone()]);
    let plan = f.projects.attach_plan(p.id, job.id).unwrap();
    assert_eq!(plan[0].proposed, Some(docs[0]), "same file, same place");
    assert_eq!(
        plan[0].same_content.len(),
        1,
        "the copy is offered, not chosen"
    );
    assert_eq!(plan[0].same_content[0].id, docs[1]);
    assert_eq!(plan[1].proposed, None, "a new document");
    assert!(plan.iter().all(|m| m.linked.is_none()));

    let dossier = f.projects.create_collection(p.id, "1958").unwrap();
    let choices: Vec<AttachChoice> = plan
        .iter()
        .map(|m| AttachChoice {
            source_id: m.source_id,
            document: m.proposed,
        })
        .collect();
    let attached = f
        .projects
        .attach(p.id, job.id, &choices, Some(dossier.id))
        .unwrap();
    assert_eq!(attached[0], docs[0]);
    let plan = f.projects.attach_plan(p.id, job.id).unwrap();
    assert_eq!(plan[0].linked, Some(docs[0]));

    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.jobs, vec![job.id]);
    assert_eq!(view.documents.len(), 3, "the new file became a document");
    let new = view
        .documents
        .iter()
        .find(|d| d.document.source.path == other.path)
        .unwrap();
    assert_eq!(
        new.collections,
        vec![dossier.id],
        "and went to the chosen collection"
    );
    let row = &view.documents[0];
    assert_eq!(row.tasks.len(), 1);
    assert_eq!(
        (
            row.tasks[0].results,
            row.tasks[0].finished,
            row.tasks[0].reviewed
        ),
        (1, 1, 0),
        "read in full, one page of two reviewed"
    );

    // The document's page: the result with its counts, its cost, and how to open it.
    let detail = f.projects.document(p.id, docs[0]).unwrap();
    assert_eq!(detail.results.len(), 1);
    let result = &detail.results[0];
    assert_eq!(
        result.source_id, again.id,
        "the job's own source, for its review"
    );
    assert_eq!((result.pieces, result.finished, result.reviewed), (2, 2, 1));
    assert!(result.cost_usd.is_some_and(|c| (c - 0.02).abs() < 1e-12));
    assert!(!result.preferred);

    // A source not in the job, or a document of another project, is refused, with nothing
    // written.
    let elsewhere = f.projects.create("Other", "").unwrap();
    let theirs = f
        .projects
        .add_documents(elsewhere.id, vec![f.file("x.pdf", "x")], None)
        .unwrap();
    let wrong = [AttachChoice {
        source_id: plan[1].source_id,
        document: Some(theirs[0]),
    }];
    assert!(matches!(
        f.projects.attach(p.id, job.id, &wrong, None),
        Err(ProjectError::Elsewhere(_))
    ));
}

#[test]
fn a_job_started_in_the_project_is_linked_to_its_documents() {
    let f = fixture();
    let p = f.projects.create("Interviews", "").unwrap();
    let a = f.file("a.pdf", "a");
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone()], None)
        .unwrap();
    let documents = f.store.documents(p.id).unwrap();
    // New job was given the documents' sources, ids and all, and one more file.
    let b = f.file("b.pdf", "b");
    let job = f.job(Task::TextExtraction, vec![documents[0].source.clone(), b]);
    let linked = f.projects.link_job(p.id, job.id, None).unwrap();
    assert_eq!(linked[0], docs[0]);
    assert_eq!(f.store.documents(p.id).unwrap().len(), 2);
    // Linking again changes nothing.
    f.projects.link_job(p.id, job.id, None).unwrap();
    assert_eq!(f.store.links(p.id).unwrap().len(), 2);
    assert_eq!(f.store.documents(p.id).unwrap().len(), 2);
}

#[test]
fn several_jobs_are_linked_together_or_not_at_all() {
    let f = fixture();
    let p = f.projects.create("Interviews", "").unwrap();
    let c = f.projects.create_collection(p.id, "Added").unwrap();
    let (a, b) = (f.file("a.pdf", "a"), f.file("b.pdf", "b"));
    let first = f.job(Task::TextExtraction, vec![a.clone()]);
    // The same file read again by a second job, and another file.
    let second = f.job(Task::Analysis, vec![a, b]);

    // A job that does not exist stops the whole selection before anything is written.
    let missing = kalamu_core::JobId::new();
    assert!(
        f.projects
            .link_jobs(p.id, &[first.id, missing], Some(c.id))
            .is_err()
    );
    assert!(f.store.documents(p.id).unwrap().is_empty());
    assert!(f.store.links(p.id).unwrap().is_empty());

    let linked = f
        .projects
        .link_jobs(p.id, &[first.id, second.id], Some(c.id))
        .unwrap();
    assert_eq!(linked.len(), 3);
    assert_eq!(linked[0], linked[1], "the same file is one document");
    assert_eq!(f.store.documents(p.id).unwrap().len(), 2);
    assert_eq!(f.store.links(p.id).unwrap().len(), 3);
    assert_eq!(f.store.members(p.id).unwrap().len(), 2);
}

#[test]
fn an_analysis_of_frozen_text_joins_the_document_it_was_read_from() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    let source = f.store.documents(p.id).unwrap()[0].source.clone();
    let extraction = f.job(Task::TextExtraction, vec![source.clone()]);
    f.projects.link_job(p.id, extraction.id, None).unwrap();
    // The analysis read a text frozen in the app's data folder, not the scan.
    let mut frozen = f.file("snapshot.txt", "frozen");
    frozen.kind = InputKind::Text;
    frozen.derived_from = Some(kalamu_core::DerivedFrom {
        snapshot_id: "s1".into(),
        job_id: extraction.id,
        setup_id: extraction.setups[0].id,
        source_id: source.id,
        original_path: source.path.clone(),
        unit: None,
    });
    let analysis = f.job(Task::Analysis, vec![frozen]);
    let linked = f.projects.link_job(p.id, analysis.id, None).unwrap();
    assert_eq!(linked, docs, "the same document, not a new one");
    let detail = f.projects.document(p.id, docs[0]).unwrap();
    let tasks: Vec<Task> = detail.results.iter().map(|r| r.task).collect();
    assert!(tasks.contains(&Task::Analysis) && tasks.contains(&Task::TextExtraction));
}

#[test]
fn a_preferred_result_is_the_researchers_and_survives_a_newer_run() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let a = f.file("a.pdf", "a");
    let docs = f.projects.add_documents(p.id, vec![a], None).unwrap();
    let source = f.store.documents(p.id).unwrap()[0].source.clone();
    let first = f.job(Task::TextExtraction, vec![source.clone()]);
    f.projects.link_job(p.id, first.id, None).unwrap();
    let choice = Some((first.id, first.setups[0].id));
    // Not for another task, nor from a job the document is not tied to.
    assert!(
        f.projects
            .set_preferred(p.id, docs[0], Task::Analysis, choice)
            .is_err()
    );
    let unrelated = f.job(Task::TextExtraction, vec![f.file("z.pdf", "z")]);
    assert!(
        f.projects
            .set_preferred(
                p.id,
                docs[0],
                Task::TextExtraction,
                Some((unrelated.id, unrelated.setups[0].id))
            )
            .is_err()
    );
    f.projects
        .set_preferred(p.id, docs[0], Task::TextExtraction, choice)
        .unwrap();

    // A newer run does not take its place.
    let second = f.job(Task::TextExtraction, vec![source]);
    f.projects.link_job(p.id, second.id, None).unwrap();
    let detail = f.projects.document(p.id, docs[0]).unwrap();
    assert_eq!(detail.results.len(), 2);
    let preferred: Vec<JobId> = detail
        .results
        .iter()
        .filter(|r| r.preferred)
        .map(|r| r.job_id)
        .collect();
    assert_eq!(preferred, vec![first.id]);
    assert!(f.projects.view(p.id).unwrap().documents[0].tasks[0].preferred);

    // Untied from the job, the choice goes with it.
    f.projects
        .detach(p.id, docs[0], first.id, detail.results[1].source_id)
        .unwrap();
    let detail = f.projects.document(p.id, docs[0]).unwrap();
    assert_eq!(detail.results.len(), 1);
    assert!(detail.results.iter().all(|r| !r.preferred));
}

#[test]
fn two_documents_found_to_be_one_are_merged_without_reading_again() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let c = f.projects.create_collection(p.id, "Box 4").unwrap();
    let docs = f
        .projects
        .add_documents(
            p.id,
            vec![f.file("scan.pdf", "same"), f.file("scan-copy.pdf", "same")],
            None,
        )
        .unwrap();
    f.projects
        .add_to_collection(p.id, c.id, &[docs[1]])
        .unwrap();
    let sources: Vec<SourceFile> = f
        .store
        .documents(p.id)
        .unwrap()
        .into_iter()
        .map(|d| d.source)
        .collect();
    let job = f.job(Task::TextExtraction, vec![sources[1].clone()]);
    f.projects.link_job(p.id, job.id, None).unwrap();

    f.projects.merge_documents(p.id, docs[1], docs[0]).unwrap();
    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.documents.len(), 1);
    assert_eq!(view.documents[0].document.id, docs[0]);
    assert_eq!(view.documents[0].collections, vec![c.id]);
    assert_eq!(view.documents[0].tasks[0].results, 1);
}

#[test]
fn deleting_a_document_or_a_project_leaves_the_jobs_and_their_results() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    let source = f.store.documents(p.id).unwrap()[0].source.clone();
    let job = f.job(Task::TextExtraction, vec![source.clone()]);
    f.projects.link_job(p.id, job.id, None).unwrap();

    f.projects.delete_documents(p.id, &docs).unwrap();
    assert!(f.store.job(job.id).is_ok());
    assert_eq!(f.store.pieces(job.id).unwrap().len(), 2);
    assert!(f.projects.view(p.id).unwrap().jobs.is_empty());

    // A deleted job leaves its document without that result.
    f.projects.add_documents(p.id, vec![source], None).unwrap();
    f.projects.link_job(p.id, job.id, None).unwrap();
    f.store.delete_job(job.id).unwrap();
    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.documents.len(), 1);
    assert!(view.documents[0].tasks.is_empty());

    let other = f.job(Task::TextExtraction, vec![f.file("b.pdf", "b")]);
    f.projects.link_job(p.id, other.id, None).unwrap();
    f.projects.delete(p.id).unwrap();
    assert!(f.store.job(other.id).is_ok());
    assert!(matches!(f.projects.view(p.id), Err(ProjectError::NotFound)));
}

#[test]
fn a_project_and_its_organisation_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.sqlite");
    let file = dir.path().join("a.pdf");
    std::fs::write(&file, "a").unwrap();
    let source = SourceFile {
        id: SourceId::new(),
        path: file.display().to_string(),
        file_name: "a.pdf".into(),
        sha256: "a".into(),
        size_bytes: 1,
        modified_ms: None,
        kind: InputKind::Pdf,
        page_count: Some(1),
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        title: None,
        derived_from: None,
    };
    let (project, collection) = {
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let projects = Projects::new(store.clone(), store);
        let p = projects.create("Kept", "").unwrap();
        let c = projects.create_collection(p.id, "Box 1").unwrap();
        projects
            .add_documents(p.id, vec![source], Some(c.id))
            .unwrap();
        projects.set_archived(p.id, true).unwrap();
        (p.id, c.id)
    };
    let store = Arc::new(SqliteStore::open(&path).unwrap());
    let projects = Projects::new(store.clone(), store);
    let listed = projects.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].project.archived);
    assert_eq!((listed[0].documents, listed[0].collections), (1, 1));
    let view = projects.view(project).unwrap();
    assert_eq!(view.documents[0].collections, vec![collection]);
}
