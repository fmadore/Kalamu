#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::json;

use super::*;
use crate::analysis::cited::tests::interview;
use crate::snapshot::Anchor;

fn evidence(start: u32, end: u32, quote: &str, turn: u32) -> Evidence {
    Evidence {
        anchor: Anchor {
            part: 0,
            start,
            end,
        },
        quote: quote.into(),
        turn,
        speaker: Some("Awa".into()),
        start_ms: Some(u64::from(turn) * 10_000),
        end_ms: None,
        added: false,
    }
}

fn theme(name: &str, quotes: Vec<Evidence>) -> CitedEntry {
    CitedEntry {
        answers: vec![
            FieldAnswer {
                key: "name".into(),
                value: FieldValue::Text(name.into()),
                reason: None,
            },
            FieldAnswer {
                key: "description".into(),
                value: FieldValue::Text(format!("About {name}.")),
                reason: None,
            },
        ],
        evidence: quotes,
        claims: Vec::new(),
    }
}

fn part_answers(themes: Vec<CitedEntry>, left_out: u32) -> Vec<FieldAnswer> {
    vec![
        FieldAnswer {
            key: "summary".into(),
            value: FieldValue::Text("A part.".into()),
            reason: None,
        },
        FieldAnswer {
            key: "themes".into(),
            value: FieldValue::Cited(CitedValue {
                entries: themes,
                left_out,
                dropped: 0,
            }),
            reason: None,
        },
        FieldAnswer {
            key: "keywords".into(),
            value: FieldValue::List(vec!["port".into()]),
            reason: None,
        },
        FieldAnswer {
            key: "quotations".into(),
            value: FieldValue::Cited(CitedValue::default()),
            reason: None,
        },
    ]
}

/// Two parts that overlap: the quotation of turn 5 found by both.
fn two_parts() -> (Vec<FieldAnswer>, Vec<FieldAnswer>) {
    let shared = evidence(500, 520, "le port de Dakar", 5);
    (
        part_answers(
            vec![theme(
                "Work",
                vec![
                    evidence(100, 130, "j'ai commencé à seize ans", 1),
                    shared.clone(),
                ],
            )],
            1,
        ),
        part_answers(
            vec![theme(
                "Port",
                vec![shared, evidence(900, 940, "nous partions avant l'aube", 9)],
            )],
            2,
        ),
    )
}

fn pieces() -> (PieceRef, PieceRef) {
    (
        PieceRef::Turns {
            number: 1,
            first: 1,
            last: 6,
        },
        PieceRef::Turns {
            number: 2,
            first: 5,
            last: 10,
        },
    )
}

#[test]
fn the_step_answers_each_cited_field_with_entries_and_the_numbers_of_their_quotations() {
    let step = schema(&interview());
    let themes = step.fields.iter().find(|f| f.key == "themes").unwrap();
    let FieldKind::Entries { fields } = &themes.kind else {
        panic!("{themes:?}")
    };
    let keys: Vec<&str> = fields.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["name", "description", QUOTES]);
    assert!(themes.instructions.contains("At most 2"));
    // The other fields are asked as they were.
    assert_eq!(step.fields[0], interview().fields[0]);
    assert_eq!(step.fields[2], interview().fields[2]);
}

#[test]
fn a_quotation_two_parts_found_is_numbered_once() {
    let (a, b) = two_parts();
    let (pa, pb) = pieces();
    let found = findings(
        &interview().fields,
        &[
            PartRead {
                piece: &pa,
                answers: &a,
            },
            PartRead {
                piece: &pb,
                answers: &b,
            },
        ],
        &[(11, 14)],
    );
    assert_eq!(found.evidence.len(), 3);
    let input: Value = serde_json::from_str(&found.input).unwrap();
    assert_eq!(input["parts"][0]["turns"], "1-6");
    assert_eq!(
        input["parts"][0]["answers"]["themes"][0]["quotes"],
        json!(["Q1", "Q2"])
    );
    assert_eq!(
        input["parts"][1]["answers"]["themes"][0]["quotes"],
        json!(["Q2", "Q3"])
    );
    assert_eq!(input["quotations"]["Q2"]["words"], "le port de Dakar");
    assert_eq!(input["quotations"]["Q2"]["speaker"], "Awa");
    assert_eq!(input["turns_not_read"], json!(["11-14"]));
    // What the parts left out is carried into the whole answer.
    assert_eq!(
        found.carried,
        vec![
            ("themes".to_string(), 3, 0),
            ("quotations".to_string(), 0, 0)
        ]
    );
}

/// The step's answer as the provider reads it, in the step's own fields.
fn answer(themes: Value) -> Vec<FieldAnswer> {
    schema(&interview())
        .read_answer(&json!({
            "summary": "The whole interview.",
            "themes": themes,
            "keywords": ["port", "Dakar"],
            "quotations": [],
        }))
        .unwrap()
}

#[test]
fn the_quotations_are_put_back_from_their_numbers_never_from_the_steps_words() {
    let (a, b) = two_parts();
    let (pa, pb) = pieces();
    let found = findings(
        &interview().fields,
        &[
            PartRead {
                piece: &pa,
                answers: &a,
            },
            PartRead {
                piece: &pb,
                answers: &b,
            },
        ],
        &[],
    );
    let mut answers = answer(json!([
        {
            "name": "Work at the port",
            "description": "From the first day to the dawn departures.",
            // A number, the same in brackets, words written in place of a number (altered), a
            // number past the last, and a number with words after it.
            "quotes": ["Q1", "[Q3]", "j'ai commencé à dix-sept ans", "Q9", "Q2: le port"]
        },
        { "name": "Nothing", "description": "A theme with no quotation it can keep.", "quotes": ["Q44"] }
    ]));
    let counted = resolve(&interview().fields, &mut answers, &found);
    assert_eq!(counted, Resolved { given: 6, found: 2 });
    let themes = answers.iter().find(|a| a.key == "themes").unwrap();
    let value = themes.value.cited().unwrap();
    assert_eq!(value.entries.len(), 1);
    let entry = &value.entries[0];
    assert_eq!(
        entry.evidence,
        vec![found.evidence[0].clone(), found.evidence[2].clone()]
    );
    assert_eq!(entry.evidence[0].quote, "j'ai commencé à seize ans");
    assert!(entry.answers.iter().all(|a| a.key != QUOTES));
    // The parts' three left out, and the step's four numbers not known; its empty entry.
    assert_eq!((value.left_out, value.dropped), (3 + 4, 1));
    // The other fields are the step's.
    assert_eq!(
        answers[0].value,
        FieldValue::Text("The whole interview.".into())
    );
}

#[test]
fn a_quotation_inside_another_of_the_same_entry_is_kept_once_the_longer() {
    let long = evidence(100, 160, "j'ai commencé à seize ans, au port", 1);
    let short = evidence(100, 130, "j'ai commencé à seize ans", 1);
    let parts = part_answers(vec![theme("Work", vec![short, long.clone()])], 0);
    let (pa, _) = pieces();
    let found = findings(
        &interview().fields,
        &[PartRead {
            piece: &pa,
            answers: &parts,
        }],
        &[],
    );
    let mut answers = answer(json!([
        { "name": "Work", "description": "d", "quotes": ["Q1", "Q2", "Q1"] }
    ]));
    resolve(&interview().fields, &mut answers, &found);
    let value = answers[1].value.cited().unwrap();
    assert_eq!(value.entries[0].evidence, vec![long]);
}
