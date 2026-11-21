//! A question asked of a project's documents, kept with the project and asked again of the
//! documents added since (#51): the analyses that asked it alike are one group, a document's
//! newest answer counting; the documents with a text it has not been asked of are listed, and
//! those with none are not; another question or another model stays apart; and the table is
//! written with every document the question was asked of.

use kalamu_core::DocumentId;

use super::*;

#[test]
fn a_question_asked_again_of_new_documents_adds_to_the_same_group() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    let b = f.file("b.txt", "b");
    let added = f.file("c.txt", "c");
    let mut later = f.file("d.txt", "d");
    later.kind = InputKind::Text;
    let scan = f.file("e.pdf", "e");
    let docs = f
        .projects
        .add_documents(
            p.id,
            vec![a.clone(), b.clone(), added.clone(), later, scan],
            None,
        )
        .unwrap();
    let first = f.screening(vec![a.clone(), b.clone()], &["Relevant", "Unsure"]);
    f.projects.link_job(p.id, first.id, None).unwrap();

    // Before it is asked again: the documents with a text not yet read against it. The text
    // file is its own text; the scan has none, and `c.txt` was given as a file but is no text.
    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.choices.len(), 1);
    assert_eq!(view.choices[0].unread, vec![docs[3]]);
    assert_eq!(view.choices[0].provider, ProviderId::new("gemini"));
    assert_eq!(view.choices[0].model, "gemini-3.8-flash");

    // Asked again of a new document, and of one already read, answered otherwise now.
    std::thread::sleep(std::time::Duration::from_millis(5));
    let again = f.screening(
        vec![added.clone(), b.clone()],
        &["Partly relevant", "Relevant"],
    );
    f.projects.link_job(p.id, again.id, None).unwrap();
    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.choices.len(), 1, "one question, one group");
    let group = &view.choices[0];
    assert_eq!(group.job_id, again.id, "named by the newest");
    let mut answers: Vec<(DocumentId, &str, JobId)> = group
        .answers
        .iter()
        .map(|x| (x.document, x.value.as_str(), x.job_id))
        .collect();
    answers.sort();
    let mut expected = vec![
        (docs[0], "Relevant", first.id),
        (docs[1], "Relevant", again.id),
        (docs[2], "Partly relevant", again.id),
    ];
    expected.sort();
    assert_eq!(answers, expected);

    // The table holds every document asked, whichever analysis is named.
    for job in [&first, &again] {
        let table = f
            .projects
            .answers_table(p.id, job.id, job.setups[0].id)
            .unwrap();
        let rows: Vec<(&str, &str)> = table
            .rows
            .iter()
            .map(|r| (r[0].as_str(), r[2].as_str()))
            .collect();
        assert_eq!(
            rows,
            [
                ("a", "Relevant"),
                ("b", "Relevant"),
                ("c", "Partly relevant")
            ]
        );
    }
}

#[test]
fn another_question_or_another_model_stays_apart() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    f.projects
        .add_documents(p.id, vec![a.clone()], None)
        .unwrap();
    let first = f.screening(vec![a.clone()], &["Relevant"]);
    let other = f.screening_with(
        vec![a.clone()],
        &["Unsure"],
        super::screened::screening(),
        "Something else",
    );
    f.projects
        .link_jobs(p.id, &[first.id, other.id], None)
        .unwrap();
    assert_eq!(f.projects.view(p.id).unwrap().choices.len(), 2);
}
