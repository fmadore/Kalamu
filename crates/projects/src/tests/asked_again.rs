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

/// A preset edited between two runs keeps its id: the question, preset and model are the same,
/// but the choices offered are not. The two runs are two groups, told apart by when each was
/// first asked, and never share a table (#241).
#[test]
fn a_question_asked_under_an_edited_prompt_is_a_group_of_its_own() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    let b = f.file("b.txt", "b");
    f.projects
        .add_documents(p.id, vec![a.clone(), b.clone()], None)
        .unwrap();
    let before = f.screening(vec![a.clone()], &["Relevant"]);
    std::thread::sleep(std::time::Duration::from_millis(5));
    // The preset edited: a choice taken out and another added.
    let mut edited = super::screened::screening();
    if let kalamu_core::FieldKind::Choice { values } = &mut edited.fields[0].kind {
        values.retain(|v| v != "Unsure");
        values.push("Off topic".into());
    }
    let after = f.screening_with(
        vec![b.clone()],
        &["Off topic"],
        edited,
        "  Women traders at the market  ",
    );
    assert_eq!(
        before.setups[0].prompt.preset_id,
        after.setups[0].prompt.preset_id
    );
    f.projects
        .link_jobs(p.id, &[before.id, after.id], None)
        .unwrap();

    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.choices.len(), 2, "two wordings, two groups");
    let (newer, older) = (&view.choices[0], &view.choices[1]);
    assert_eq!(newer.question, older.question);
    assert_eq!(newer.job_id, after.id);
    assert_eq!(older.job_id, before.id);
    assert_ne!(newer.prompt, older.prompt);
    assert_eq!(newer.prompt, after.setups[0].prompt.fingerprint());
    assert!(older.first_asked < newer.first_asked, "told apart by when");
    assert!(newer.answers.iter().all(|x| x.job_id == after.id));
    assert!(older.answers.iter().all(|x| x.job_id == before.id));

    // Each table holds its own run's documents only, under its own prompt.
    let table = |job: &Job| {
        f.projects
            .answers_table(p.id, job.id, job.setups[0].id)
            .unwrap()
    };
    let rows = |t: &kalamu_core::Table| -> Vec<(String, String, String)> {
        t.rows
            .iter()
            .map(|r| (r[0].clone(), r[2].clone(), r[4].clone()))
            .collect()
    };
    let old = table(&before);
    let new = table(&after);
    assert_eq!(
        rows(&old),
        [(
            "a".to_string(),
            "Relevant".to_string(),
            before.id.to_string()
        )]
    );
    assert_eq!(
        rows(&new),
        [(
            "b".to_string(),
            "Off topic".to_string(),
            after.id.to_string()
        )]
    );
    assert_ne!(old.rows[0][5], new.rows[0][5], "each row says which prompt");
}

/// Two runs that froze the same prompt are still one group: what tells them apart is not in
/// what was asked (the set-up's id, its label, the preset's name).
#[test]
fn runs_with_identical_frozen_prompts_stay_one_group() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    let b = f.file("b.txt", "b");
    f.projects
        .add_documents(p.id, vec![a.clone(), b.clone()], None)
        .unwrap();
    let first = f.screening(vec![a.clone()], &["Relevant"]);
    std::thread::sleep(std::time::Duration::from_millis(5));
    let again = f.screening(vec![b.clone()], &["Unsure"]);
    assert_ne!(first.setups[0].id, again.setups[0].id);
    f.projects
        .link_jobs(p.id, &[first.id, again.id], None)
        .unwrap();

    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.choices.len(), 1);
    assert_eq!(view.choices[0].first_asked, first.created_at);
    let table = f
        .projects
        .answers_table(p.id, again.id, again.setups[0].id)
        .unwrap();
    let rows: Vec<(&str, &str)> = table
        .rows
        .iter()
        .map(|r| (r[0].as_str(), r[4].as_str()))
        .collect();
    let (first_id, again_id) = (first.id.to_string(), again.id.to_string());
    assert_eq!(rows, [("a", first_id.as_str()), ("b", again_id.as_str())]);
    assert_eq!(table.rows[0][5], table.rows[1][5], "one prompt");
}
