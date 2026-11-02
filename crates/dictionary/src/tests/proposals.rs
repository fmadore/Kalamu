//! Merges proposed by a model: the rows sent in groups, the answers read back against them,
//! references to no row dropped, pairs kept apart never offered, and nothing applied until
//! the researcher merges.

use kalamu_core::Entry;
use kalamu_core::dictionary::proposal;

use super::*;
use crate::proposals::windows;

/// The people of three documents, with variants of two names.
fn people(f: &Fixture) {
    let letter = f.document("letter-1902");
    let report = f.document("report-1905");
    let diary = f.document("diary");
    f.analysis(
        &[letter, report, diary],
        &[&[
            (&["Moussa Traoré", "El Hadj Oumar"], &[]),
            (&["Moussa Traore", "Oumar Tall"], &[]),
            (&["Samory", "Almamy Samory Touré"], &[]),
        ]],
    );
}

/// A finished proposal job whose one source answered `groups`, each its references, label and
/// certainty, as the engine would have stored it.
fn answered(f: &Fixture, prepared: &Prepared, groups: &[(&[&str], &str, &str)]) -> JobId {
    let source = SourceFile {
        kind: InputKind::Text,
        ..source("people")
    };
    let setup = Setup {
        id: SetupId::new(),
        label: "model".into(),
        provider: ProviderId::new("gemini"),
        model: "model".into(),
        prompt: PromptSnapshot {
            preset_id: Some("dictionary.merges".into()),
            preset_name: "Dictionary: proposed merges".into(),
            template: String::new(),
            text: String::new(),
            user_prompt: String::new(),
            schema: Some(proposal::schema()),
        },
        options: SetupOptions::default(),
        merge: None,
        pages_cap: None,
    };
    let job = Job {
        id: JobId::new(),
        name: "Dictionary: merges in People".into(),
        task: Task::Analysis,
        created_at: Timestamp::now(),
        sources: vec![source.clone()],
        setups: vec![setup.clone()],
        settings: JobSettings::default(),
        output: OutputSettings::default(),
        status: JobStatus::Completed,
    };
    f.store.create_job(&job).unwrap();
    let entries = groups
        .iter()
        .map(|(rows, label, certainty)| {
            vec![
                answer(
                    proposal::ROWS,
                    FieldValue::List(rows.iter().map(|r| r.to_string()).collect()),
                ),
                answer(proposal::LABEL, FieldValue::Text(label.to_string())),
                answer(proposal::REASON, FieldValue::Text("Same name.".into())),
                answer(proposal::CERTAINTY, FieldValue::Text(certainty.to_string())),
            ]
        })
        .collect();
    let key = format!("{}-proposals", job.id);
    f.store
        .put_result(&StoredResult {
            cache_key: key.clone(),
            provider: ProviderId::new("gemini"),
            model: "model".into(),
            output: ProcessOutput {
                fields: vec![answer(proposal::GROUPS, FieldValue::Entries(entries))],
                ..ProcessOutput::default()
            },
            request: Vec::new(),
            cost_usd: Some(0.01),
            duration_ms: 1,
            created_at: Timestamp::now(),
        })
        .unwrap();
    f.store
        .upsert_piece(&PieceState {
            job_id: job.id,
            setup_id: setup.id,
            source_id: source.id,
            index: 0,
            piece: PieceRef::Whole,
            status: PieceStatus::Done,
            cache_key: Some(key),
            error: None,
            attempts: 1,
            updated_at: Timestamp::now(),
        })
        .unwrap();
    f.dictionary
        .record_request(f.project, None, prepared, job.id, &[source.sha256])
        .unwrap();
    job.id
}

/// The entry a form becomes when the researcher reviews it alone.
fn entry_of(f: &Fixture, form_: &str) -> Entry {
    f.dictionary
        .set_reviewed(f.project, "people", &[form(form_)], true)
        .unwrap();
    f.store
        .entries(f.project)
        .unwrap()
        .into_iter()
        .find(|e| e.forms.iter().any(|x| x == form_))
        .unwrap()
}

/// The reference a group gives a form's row.
fn reference(prepared: &Prepared, form: &str) -> String {
    let n = prepared.groups[0]
        .rows
        .iter()
        .position(|r| r.iter().any(|f| f == form))
        .unwrap_or_else(|| panic!("{form} not sent"));
    format!("R{}", n + 1)
}

#[test]
fn every_row_is_in_a_group_and_neighbours_share_one() {
    // Thousands of rows fit in one request.
    assert_eq!(windows(&[20; 3_000], INPUT_TOKENS), vec![0..3_000]);
    // Beyond it, groups as large as the budget allows, a tenth of each shared with the next.
    let ranges = windows(&[10; 1_000], 3_000);
    assert_eq!(ranges, vec![0..300, 270..570, 540..840, 810..1_000]);
    // A row longer than the budget still goes, alone.
    assert_eq!(windows(&[50, 5, 5], 10), vec![0..1, 1..3]);
}

#[test]
fn the_rows_go_with_their_spellings_documents_and_the_pairs_kept_apart() {
    let f = fixture();
    people(&f);
    let view = f.view();
    let moussa = f
        .dictionary
        .merge(
            f.project,
            "people",
            &[form("Moussa Traoré"), form("Moussa Traore")],
            "Moussa Traoré",
        )
        .unwrap();
    let samory = entry_of(&f, "Samory");
    f.dictionary
        .keep_apart(
            f.project,
            "people",
            &Item::Entry { id: moussa.id },
            &Item::Entry { id: samory.id },
        )
        .unwrap();
    assert!(view.rows.len() > 2);

    let prepared = f
        .dictionary
        .prepare_proposals(f.project, None, "people")
        .unwrap();
    assert_eq!((prepared.label.as_str(), prepared.rows), ("People", 5));
    assert_eq!(prepared.groups.len(), 1);
    let text = &prepared.groups[0].text;
    assert!(text.starts_with("Field: People\n"), "{text}");
    // Sorted with accents set aside, so `Almamy` comes first; the entry's other spelling and
    // both documents travel with it.
    assert!(text.contains("R1. Almamy Samory Touré | named in 1 document: \"diary\""));
    assert!(text.contains(
        "Moussa Traoré | also written: Moussa Traore | named in 2 documents: \"letter-1902\"; \
         \"report-1905\""
    ));
    let (m, s) = (
        reference(&prepared, "Moussa Traoré"),
        reference(&prepared, "Samory"),
    );
    let pair = if m < s {
        format!("{m} and {s}")
    } else {
        format!("{s} and {m}")
    };
    assert!(text.contains(&format!("Kept apart by the researcher: {pair}.")));

    // Nothing to compare in a field with one row, or none.
    assert!(matches!(
        f.dictionary.prepare_proposals(f.project, None, "places"),
        Err(DictionaryError::NothingToCompare)
    ));
}

#[test]
fn answers_are_checked_against_the_rows_and_nothing_is_applied_unasked() {
    let f = fixture();
    people(&f);
    let samory = entry_of(&f, "Samory");
    let oumar = entry_of(&f, "Oumar Tall");
    let hadj = entry_of(&f, "El Hadj Oumar");
    f.dictionary
        .keep_apart(
            f.project,
            "people",
            &Item::Entry { id: oumar.id },
            &Item::Entry { id: hadj.id },
        )
        .unwrap();
    let prepared = f
        .dictionary
        .prepare_proposals(f.project, None, "people")
        .unwrap();
    let r = |form: &str| reference(&prepared, form);
    let job = answered(
        &f,
        &prepared,
        &[
            (
                &[&r("Moussa Traoré"), &r("Moussa Traore")],
                "Moussa Traoré",
                "certain",
            ),
            // A reference to no row is dropped, the rest kept.
            (
                &[&r("Samory"), &r("Almamy Samory Touré"), "R99"],
                "Almamy Samory Touré",
                "likely",
            ),
            // Kept apart by the researcher: never offered.
            (
                &[&r("Oumar Tall"), &r("El Hadj Oumar")],
                "El Hadj Oumar Tall",
                "possible",
            ),
            // One row is no merge.
            (&[&r("Samory")], "Samory", "certain"),
        ],
    );
    let before = f.view();

    let view = f.dictionary.proposals(f.project).unwrap();
    assert_eq!(view.requests.len(), 1);
    let request = &view.requests[0];
    assert_eq!(
        (request.job_id, request.groups, request.answered),
        (job, 1, 1)
    );
    assert_eq!((request.proposals, request.unknown), (2, 1));
    let labels: Vec<(&str, &str)> = view
        .proposals
        .iter()
        .map(|p| (p.label.as_str(), p.certainty.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            ("Moussa Traoré", "certain"),
            ("Almamy Samory Touré", "likely")
        ]
    );
    let samory_proposal = &view.proposals[1];
    assert!(
        samory_proposal
            .members
            .iter()
            .any(|m| m.item == Item::Entry { id: samory.id })
    );
    // Proposing changed nothing.
    assert_eq!(f.view(), before);

    // Turned down, it stays listed as such, and can be taken back.
    let moussa = view.proposals[0].clone();
    f.dictionary
        .dismiss(f.project, "people", &moussa.key, true)
        .unwrap();
    let view = f.dictionary.proposals(f.project).unwrap();
    assert!(view.proposals[0].dismissed);
    assert_eq!(view.requests[0].proposals, 1);
    f.dictionary
        .dismiss(f.project, "people", &moussa.key, false)
        .unwrap();

    // Accepted through the ordinary merge, the proposal is done: its rows are one entry.
    let items: Vec<Item> = samory_proposal
        .members
        .iter()
        .map(|m| m.item.clone())
        .collect();
    f.dictionary
        .merge(f.project, "people", &items, &samory_proposal.label)
        .unwrap();
    let view = f.dictionary.proposals(f.project).unwrap();
    assert_eq!(view.proposals.len(), 1);
    assert_eq!(view.proposals[0].label, "Moussa Traoré");
    // The merged entry kept the oldest id, which the proposal named.
    assert!(
        f.view()
            .rows
            .iter()
            .any(|r| r.entry == Some(samory.id) && r.label == "Almamy Samory Touré")
    );

    // A forgotten request takes its proposals with it; its job stays.
    f.dictionary.forget_request(f.project, &request.id).unwrap();
    assert!(
        f.dictionary
            .proposals(f.project)
            .unwrap()
            .proposals
            .is_empty()
    );
    assert!(f.store.job(job).is_ok());
}

#[test]
fn proposals_and_turned_down_ones_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kalamu.db");
    let f = fixture_on(Arc::new(SqliteStore::open(&path).unwrap()));
    people(&f);
    let prepared = f
        .dictionary
        .prepare_proposals(f.project, None, "people")
        .unwrap();
    let r = |form: &str| reference(&prepared, form);
    answered(
        &f,
        &prepared,
        &[(
            &[&r("Oumar Tall"), &r("El Hadj Oumar")],
            "El Hadj Oumar Tall",
            "possible",
        )],
    );
    let key = f.dictionary.proposals(f.project).unwrap().proposals[0]
        .key
        .clone();
    f.dictionary
        .dismiss(f.project, "people", &key, true)
        .unwrap();
    let project = f.project;
    drop(f);

    let store = Arc::new(SqliteStore::open(&path).unwrap());
    let dictionary = Dictionary::new(store.clone(), store.clone(), store);
    let view = dictionary.proposals(project).unwrap();
    assert_eq!(view.proposals.len(), 1);
    assert!(view.proposals[0].dismissed);
}
