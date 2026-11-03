//! The review queue's result policy: the preferred result where there is one, else every result
//! the researcher has not set aside, and documents counted once whatever reads them.

use kalamu_core::DocumentId;

use super::*;

/// A comparison job of `task` over `sources` with two set-ups, `flash` and `pro`; nothing read.
fn comparison(f: &Fixture, task: Task, sources: Vec<SourceFile>) -> Job {
    let mut job = f.job(task, Vec::new());
    f.store.delete_job(job.id).unwrap();
    let mut pro = job.setups[0].clone();
    pro.id = SetupId::new();
    pro.label = "pro".into();
    pro.model = "gemini-pro".into();
    job.setups[0].label = "flash".into();
    job.setups.push(pro);
    job.id = kalamu_core::JobId::new();
    job.sources = sources;
    f.store.create_job(&job).unwrap();
    job
}

fn keys(plan: &QueuePlan) -> Vec<(DocumentId, String)> {
    plan.scope
        .targets
        .iter()
        .map(|t| {
            let label = if t.preferred {
                "preferred".to_string()
            } else {
                plan.choices
                    .iter()
                    .find(|c| c.key.setup_id == t.setup_id)
                    .map(|c| c.setup_label.clone())
                    .unwrap_or_default()
            };
            (t.document_id, label)
        })
        .collect()
}

#[test]
fn a_preferred_result_stands_alone_and_alternatives_are_listed_otherwise() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let (a, b) = (f.file("a.pdf", "a"), f.file("b.pdf", "b"));
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone(), b.clone()], None)
        .unwrap();
    let job = comparison(&f, Task::TextExtraction, vec![a, b]);
    f.projects.link_job(p.id, job.id, None).unwrap();

    // No preference: both set-ups read both documents, which stay two documents.
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project, &[])
        .unwrap();
    assert_eq!(plan.scope.documents, 2);
    assert_eq!(plan.scope.targets.len(), 4);
    assert_eq!(plan.preferred, 0);
    assert_eq!(plan.choices.len(), 2);
    assert!(plan.choices.iter().all(|c| c.documents == 2 && c.included));

    // The researcher prefers `pro` for the first document: it alone is read there.
    f.projects
        .set_preferred(
            p.id,
            docs[0],
            Task::TextExtraction,
            Some((job.id, job.setups[1].id)),
        )
        .unwrap();
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project, &[])
        .unwrap();
    assert_eq!(plan.preferred, 1);
    let first: Vec<_> = plan
        .scope
        .targets
        .iter()
        .filter(|t| t.document_id == docs[0])
        .collect();
    assert_eq!(first.len(), 1);
    assert!(first[0].preferred);
    assert_eq!(first[0].setup_id, job.setups[1].id);
    // The alternatives now stand for the second document only.
    assert!(plan.choices.iter().all(|c| c.documents == 1));

    // Setting `flash` aside leaves the second document with `pro` alone; a preferred result is
    // never set aside.
    let flash = ResultKey {
        job_id: job.id,
        setup_id: job.setups[0].id,
    };
    let pro = ResultKey {
        job_id: job.id,
        setup_id: job.setups[1].id,
    };
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project, &[flash, pro])
        .unwrap();
    assert_eq!(
        keys(&plan),
        vec![(docs[0], "preferred".to_string())],
        "the preferred result stays; the second document has nothing left to read"
    );
    assert!(plan.choices.iter().all(|c| !c.included));
    assert_eq!(plan.scope.documents, 2, "coverage still counts it");
}

#[test]
fn a_queue_covers_a_collection_or_a_selection() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let letters = f.projects.create_collection(p.id, "Letters").unwrap();
    let files = vec![
        f.file("a.pdf", "a"),
        f.file("b.pdf", "b"),
        f.file("c.pdf", "c"),
    ];
    let docs = f.projects.add_documents(p.id, files.clone(), None).unwrap();
    f.projects
        .add_to_collection(p.id, letters.id, &docs[..2])
        .unwrap();
    let job = f.job(Task::TextExtraction, files);
    f.projects.link_job(p.id, job.id, None).unwrap();

    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Collection { id: letters.id }, &[])
        .unwrap();
    assert_eq!(plan.scope.documents, 2);
    assert_eq!(plan.scope.targets.len(), 2);

    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Documents { ids: vec![docs[2]] }, &[])
        .unwrap();
    assert_eq!(plan.scope.documents, 1);
    assert_eq!(plan.scope.targets.len(), 1);
    assert_eq!(plan.scope.targets[0].document_id, docs[2]);
    assert_eq!(plan.scope.targets[0].document, "c");

    // A document of another project is refused, not quietly left out.
    let other = f.projects.create("Other", "").unwrap();
    let stray = f
        .projects
        .add_documents(other.id, vec![f.file("d.pdf", "d")], None)
        .unwrap();
    assert!(matches!(
        f.projects
            .queue_plan(p.id, &QueueSelection::Documents { ids: stray }, &[]),
        Err(ProjectError::Elsewhere(_))
    ));
}

#[test]
fn a_document_not_yet_read_is_counted_but_reads_nothing() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    f.projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project, &[])
        .unwrap();
    assert_eq!(plan.scope.documents, 1);
    assert!(plan.scope.targets.is_empty());
    assert!(plan.choices.is_empty());
}
