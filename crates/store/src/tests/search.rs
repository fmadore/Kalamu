//! The search index: passages found by their folded text, replaced, removed and forgotten, and
//! a damaged file made again.

#![allow(clippy::unwrap_used)]

use kalamu_core::{
    JobId, Passage, PassageKey, PassagePart, PieceRef, SearchIndex, SetupId, SourceId,
};

use crate::SqliteIndex;

fn passage(job: JobId, index: u32, text: &str) -> (Passage, String) {
    let passage = Passage {
        key: PassageKey {
            job_id: job,
            setup_id: SetupId::new(),
            source_id: SourceId::new(),
            index,
        },
        stamp: format!("v{index}"),
        piece: PieceRef::page(index + 1),
        reviewed: false,
        corrected: false,
        parts: vec![PassagePart {
            field: None,
            entry: None,
            entry_number: None,
            reason: false,
            text: text.into(),
            lines: Vec::new(),
            by_hand: Vec::new(),
        }],
    };
    // The search crate folds; here the text is already plain.
    (passage, text.to_lowercase())
}

/// Each word required.
fn terms(words: &[&str]) -> Vec<Vec<String>> {
    words.iter().map(|w| vec![w.to_string()]).collect()
}

/// Groups of alternatives, a term of each required.
fn groups(groups: &[&[&str]]) -> Vec<Vec<String>> {
    groups
        .iter()
        .map(|g| g.iter().map(|w| w.to_string()).collect())
        .collect()
}

#[test]
fn a_term_of_each_group_is_required() {
    let index = SqliteIndex::in_memory().unwrap();
    let job = JobId::new();
    let (a, b, c) = (
        passage(job, 0, "La mission catholique de Ouagadougou"),
        passage(job, 1, "La mission protestante de Bobo"),
        passage(job, 2, "Le marché de Bobo"),
    );
    index.put_passages(&[a.clone(), b.clone(), c]).unwrap();
    let mut found = index
        .find(&groups(&[&["catholique", "protestante"]]))
        .unwrap();
    found.sort_by_key(|p| p.key.index);
    assert_eq!(found, vec![a.0.clone(), b.0.clone()]);
    assert_eq!(
        index
            .find(&groups(&[&["catholique", "protestante"], &["bobo"]]))
            .unwrap(),
        vec![b.0]
    );
    // A group with nothing in it, or a term too short, finds nothing.
    assert!(
        index
            .find(&groups(&[&["mission"], &[]]))
            .unwrap()
            .is_empty()
    );
    assert!(
        index
            .find(&groups(&[&["mission", "de"]]))
            .unwrap()
            .is_empty()
    );
    // Quotes and the words of FTS5's syntax are text, not operators.
    assert!(index.find(&groups(&[&["\" or \""]])).unwrap().is_empty());
    assert!(index.find(&groups(&[&["NOT", "AND"]])).unwrap().is_empty());
}

#[test]
fn a_phrase_is_found_anywhere_in_the_text() {
    let index = SqliteIndex::in_memory().unwrap();
    let job = JobId::new();
    let (a, b) = (
        passage(job, 0, "La mission catholique de Ouagadougou"),
        passage(job, 1, "Rien ici"),
    );
    index.put_passages(&[a.clone(), b]).unwrap();
    assert_eq!(
        index.find(&terms(&["mission cath"])).unwrap(),
        vec![a.0.clone()]
    );
    // Inside a word, as the review's search finds it.
    assert_eq!(index.find(&terms(&["tholiq"])).unwrap(), vec![a.0]);
    assert!(
        index
            .find(&terms(&["catholique mission"]))
            .unwrap()
            .is_empty()
    );
    // Two letters are too few for the index.
    assert!(index.find(&terms(&["la"])).unwrap().is_empty());
    // Quotes in the words are words.
    assert!(index.find(&terms(&["\"mission"])).unwrap().is_empty());
}

#[test]
fn several_terms_are_all_required_anywhere_in_the_passage() {
    let index = SqliteIndex::in_memory().unwrap();
    let job = JobId::new();
    let (a, b) = (
        passage(job, 0, "La mission catholique de Ouagadougou"),
        passage(job, 1, "Ouagadougou, la mosquée"),
    );
    index.put_passages(&[a.clone(), b]).unwrap();
    assert_eq!(
        index.find(&terms(&["ouagadougou", "mission"])).unwrap(),
        vec![a.0.clone()]
    );
    // In any order, a phrase among them.
    assert_eq!(
        index.find(&terms(&["catholique", "la mission"])).unwrap(),
        vec![a.0]
    );
    assert_eq!(index.find(&terms(&["ouagadougou"])).unwrap().len(), 2);
    assert!(
        index
            .find(&terms(&["mission", "mosquée"]))
            .unwrap()
            .is_empty()
    );
    // A term too short finds nothing rather than everything.
    assert!(index.find(&terms(&["mission", "la"])).unwrap().is_empty());
    assert!(index.find(&[]).unwrap().is_empty());
}

#[test]
fn passages_are_replaced_removed_and_forgotten() {
    let index = SqliteIndex::in_memory().unwrap();
    let job = JobId::new();
    let (mut first, _) = passage(job, 0, "Union culturelle musulmane");
    index
        .put_passages(&[(first.clone(), "union culturelle musulmane".into())])
        .unwrap();
    // A correction replaces the passage: the old words are no longer found.
    first.stamp = "v2".into();
    index
        .put_passages(&[(first.clone(), "union des musulmans".into())])
        .unwrap();
    assert!(index.find(&terms(&["culturelle"])).unwrap().is_empty());
    assert_eq!(
        index.find(&terms(&["des musulmans"])).unwrap(),
        vec![first.clone()]
    );
    assert_eq!(
        index.stamps(job).unwrap(),
        vec![(first.key, "v2".to_string())]
    );

    let other = JobId::new();
    let (second, folded) = passage(other, 0, "union des musulmans");
    index.put_passages(&[(second.clone(), folded)]).unwrap();
    let mut jobs = index.indexed_jobs().unwrap();
    jobs.sort();
    let mut expected = vec![job, other];
    expected.sort();
    assert_eq!(jobs, expected);

    index.remove_passages(&[first.key]).unwrap();
    assert_eq!(
        index.find(&terms(&["musulmans"])).unwrap(),
        vec![second.clone()]
    );
    index.forget_job(other).unwrap();
    assert!(index.find(&terms(&["musulmans"])).unwrap().is_empty());
    assert_eq!(index.size().unwrap().passages, 0);
}

#[test]
fn clearing_empties_the_index() {
    let index = SqliteIndex::in_memory().unwrap();
    index
        .put_passages(&[passage(JobId::new(), 0, "abc def")])
        .unwrap();
    index.clear().unwrap();
    assert!(index.find(&terms(&["abc"])).unwrap().is_empty());
    assert_eq!(index.size().unwrap().passages, 0);
}

#[test]
fn a_damaged_file_is_made_again() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("search.sqlite");
    std::fs::write(
        &path,
        b"not a database at all, only bytes that look like nothing",
    )
    .unwrap();
    let index = SqliteIndex::open(&path).unwrap();
    index
        .put_passages(&[passage(JobId::new(), 0, "kept again")])
        .unwrap();
    assert_eq!(index.find(&terms(&["kept"])).unwrap().len(), 1);
    drop(index);
    // Opened again, it keeps what it holds.
    let index = SqliteIndex::open(&path).unwrap();
    assert_eq!(index.size().unwrap().passages, 1);
}
