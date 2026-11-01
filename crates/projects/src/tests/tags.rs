//! Tags: labels beside the collections, one spelling per project, kept through a merge and a
//! restart.

use super::*;

#[test]
fn tags_keep_one_spelling_and_are_counted() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(
            p.id,
            vec![
                f.file("a.pdf", "a"),
                f.file("b.pdf", "b"),
                f.file("c.pdf", "c"),
            ],
            None,
        )
        .unwrap();

    f.projects
        .tag_documents(p.id, &docs[..2], "  Pilgrimage   1957 ")
        .unwrap();
    // The same tag in other capitals takes the project's spelling.
    f.projects
        .tag_documents(p.id, &docs[1..], "pilgrimage 1957")
        .unwrap();
    f.projects.tag_documents(p.id, &docs[..1], "Dakar").unwrap();
    assert!(matches!(
        f.projects.tag_documents(p.id, &docs, "   "),
        Err(ProjectError::NoTag)
    ));

    let view = f.projects.view(p.id).unwrap();
    let names: Vec<(&str, u32)> = view
        .tags
        .iter()
        .map(|t| (t.name.as_str(), t.documents))
        .collect();
    assert_eq!(names, vec![("Dakar", 1), ("Pilgrimage 1957", 3)]);
    assert_eq!(
        view.documents[0].document.tags,
        vec!["Dakar", "Pilgrimage 1957"]
    );

    f.projects
        .untag_documents(p.id, &docs[1..], "PILGRIMAGE 1957")
        .unwrap();
    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.tags[1].documents, 1);
    assert!(view.documents[2].document.tags.is_empty());
}

#[test]
fn a_document_of_another_project_is_not_tagged() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let q = f.projects.create("Elsewhere", "").unwrap();
    let mine = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    let theirs = f
        .projects
        .add_documents(q.id, vec![f.file("b.pdf", "b")], None)
        .unwrap();
    let both = [mine[0], theirs[0]];
    assert!(matches!(
        f.projects.tag_documents(p.id, &both, "Letters"),
        Err(ProjectError::Elsewhere(_))
    ));
    // Nothing is tagged when one of them is refused.
    assert!(f.projects.view(p.id).unwrap().tags.is_empty());
}

#[test]
fn merged_documents_keep_both_tags_and_tags_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.sqlite");
    let f = fixture();
    let (project, kept) = {
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let projects = Projects::new(store.clone(), store);
        let p = projects.create("Archives", "").unwrap();
        let docs = projects
            .add_documents(
                p.id,
                vec![f.file("scan.pdf", "same"), f.file("scan-copy.pdf", "same")],
                None,
            )
            .unwrap();
        projects.tag_documents(p.id, &docs[..1], "Letters").unwrap();
        projects.tag_documents(p.id, &docs, "Box 4").unwrap();
        projects.tag_documents(p.id, &docs[1..], "Damaged").unwrap();
        projects.merge_documents(p.id, docs[1], docs[0]).unwrap();
        (p.id, docs[0])
    };
    let store = Arc::new(SqliteStore::open(&path).unwrap());
    let projects = Projects::new(store.clone(), store);
    let view = projects.view(project).unwrap();
    assert_eq!(view.documents.len(), 1);
    assert_eq!(view.documents[0].document.id, kept);
    assert_eq!(
        view.documents[0].document.tags,
        vec!["Box 4", "Damaged", "Letters"]
    );
}
