//! What models wrote about the documents, searched only on request, and pieces not read.

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn analyses_are_searched_only_on_request_with_their_field() {
    let f = Fixture::new();
    let doc = f.document("Lettre", InputKind::Pdf);
    f.extraction(&doc, &["Une lettre de Bamako."]);
    let schema = AnalysisSchema {
        fields: vec![Field {
            key: "people".into(),
            label: "People".into(),
            instructions: String::new(),
            kind: FieldKind::List { max: None },
            reason: true,
        }],
    };
    let analysis = f.job(
        Task::Analysis,
        &doc,
        &[(
            "Claude",
            vec![Read {
                piece: Some(PieceRef::Whole),
                fields: vec![FieldAnswer {
                    key: "people".into(),
                    value: FieldValue::List(vec!["Moussa Traoré".into(), "Oumar Tall".into()]),
                    reason: Some("Traoré signs the letter.".into()),
                }],
                ..Read::default()
            }],
        )],
        Some(schema),
    );
    assert_eq!(f.find(&request("traore")).matches, 0);
    let mut generated = request("traore");
    generated.generated = true;
    let view = f.find(&generated);
    let result = &view.documents[0].results[0];
    assert!(result.generated);
    assert_eq!(result.task, Task::Analysis);
    let fields: Vec<_> = result
        .hits
        .iter()
        .map(|h| (h.field.as_ref().unwrap().label.as_str(), h.reason))
        .collect();
    assert_eq!(fields, vec![("People", false), ("People", true)]);

    // A corrected answer replaces the model's.
    f.store
        .put_correction(&Correction {
            job_id: analysis.id,
            setup_id: analysis.setups[0].id,
            source_id: doc.1.id,
            index: 0,
            text: None,
            fields: Some(vec![FieldAnswer {
                key: "people".into(),
                value: FieldValue::List(vec!["Moussa Diallo".into()]),
                reason: None,
            }]),
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap();
    assert_eq!(f.find(&generated).matches, 0);
    // The task filter keeps to one task.
    let mut text_only = request("bamako");
    text_only.task = Some(Task::Analysis);
    text_only.generated = true;
    assert_eq!(f.find(&text_only).matches, 0);
}

#[test]
fn failed_pieces_count_as_unread_and_tables_of_contents_search_their_entries() {
    let f = Fixture::new();
    let doc = f.document("Issue", InputKind::Pdf);
    f.job(
        Task::TextExtraction,
        &doc,
        &[(
            "Gemini",
            vec![
                page("La zakat"),
                Read {
                    failed: true,
                    ..Read::default()
                },
            ],
        )],
        None,
    );
    let view = f.find(&request("zakat"));
    assert_eq!(view.coverage.pieces, 1);
    assert_eq!(view.coverage.pieces_unread, 1);

    let entry_schema = AnalysisSchema {
        fields: vec![Field {
            key: "summary".into(),
            label: "Summary".into(),
            instructions: String::new(),
            kind: FieldKind::Text,
            reason: false,
        }],
    };
    let entry = |title: &str, summary: &str| {
        vec![
            FieldAnswer {
                key: "title".into(),
                value: FieldValue::Text(title.into()),
                reason: None,
            },
            FieldAnswer {
                key: "summary".into(),
                value: FieldValue::Text(summary.into()),
                reason: None,
            },
        ]
    };
    f.job(
        Task::Contents,
        &doc,
        &[(
            "Gemini",
            vec![
                // A page's reading is not searched; the combined entries are.
                Read {
                    fields: kalamu_core::contents::answers(vec![entry("Brouillon", "La zakat")]),
                    ..Read::default()
                },
                Read {
                    piece: Some(PieceRef::Whole),
                    fields: kalamu_core::contents::answers(vec![
                        entry("La zakat au village", "Collecte de la zakat."),
                        entry("Sports", "Football."),
                    ]),
                    ..Read::default()
                },
            ],
        )],
        Some(entry_schema),
    );
    let mut generated = request("zakat");
    generated.generated = true;
    generated.task = Some(Task::Contents);
    let view = f.find(&generated);
    let hits = &view.documents[0].results[0].hits;
    let found: Vec<_> = hits
        .iter()
        .map(|h| {
            (
                h.entry.as_deref().unwrap(),
                h.field.as_ref().unwrap().label.as_str(),
            )
        })
        .collect();
    assert_eq!(
        found,
        vec![
            ("La zakat au village", "Title"),
            ("La zakat au village", "Summary")
        ]
    );
    // Each match names its entry's place in the table, which the review opens at.
    let mut sports = generated.clone();
    sports.query = "football".into();
    let view = f.find(&sports);
    let hit = &view.documents[0].results[0].hits[0];
    assert_eq!(hit.entry_number, Some(2));
    assert_eq!(hit.entry.as_deref(), Some("Sports"));
}
