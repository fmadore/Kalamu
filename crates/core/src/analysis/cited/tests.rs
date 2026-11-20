use serde_json::json;

use super::*;
use crate::analysis::{AnalysisSchema, SchemaError};
use crate::snapshot::{Built, PartText, TimedPart, build_with_turns};
use crate::{PieceRef, Segment, Task};

fn field(label: &str, kind: FieldKind) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

/// An interview analysis: a summary, themes with a name and a description, keywords, and
/// quotations with why they matter.
pub(crate) fn interview() -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![
            field("Summary", FieldKind::Text),
            field(
                "Themes",
                FieldKind::Cited {
                    fields: vec![
                        field("Name", FieldKind::Text),
                        field("Description", FieldKind::Text),
                    ],
                    max: Some(2),
                },
            ),
            field("Keywords", FieldKind::List { max: None }),
            field(
                "Quotations",
                FieldKind::Cited {
                    fields: vec![field("Why it matters", FieldKind::Text)],
                    max: None,
                },
            ),
        ],
    };
    schema.assign_keys();
    schema
}

/// Two speakers, with times, a phrase said twice, French typography.
fn transcript() -> Built {
    let piece = PieceRef::Segment {
        index: 0,
        start_ms: 0,
        end_ms: 120_000,
    };
    let segments = [
        Segment {
            start_ms: 1_000,
            end_ms: 9_000,
            text: "Comment avez-vous commencé au port ?".into(),
            speaker: Some("Interviewer".into()),
        },
        Segment {
            start_ms: 9_000,
            end_ms: 40_000,
            text: "J’ai commencé en 1998. Au début, on ne savait pas.".into(),
            speaker: Some("Awa".into()),
        },
        Segment {
            start_ms: 40_000,
            end_ms: 52_000,
            text: "Et ensuite ?".into(),
            speaker: Some("Interviewer".into()),
        },
        Segment {
            start_ms: 52_000,
            end_ms: 90_000,
            text: "Ensuite, «\u{a0}on ne savait pas\u{a0}» est devenu notre devise.".into(),
            speaker: Some("Awa".into()),
        },
    ];
    let text = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    build_with_turns(
        Task::AudioTranscription,
        &[TimedPart {
            text: PartText {
                piece: &piece,
                text: &text,
                reviewed: true,
                corrected: true,
                failed: false,
            },
            segments: &segments,
        }],
    )
}

fn doc(built: &Built) -> CitedText<'_> {
    CitedText {
        text: &built.text,
        parts: &built.parts,
        turns: &built.turns,
    }
}

#[test]
fn a_cited_field_sits_beside_others_and_is_not_records() {
    let schema = interview();
    assert_eq!(schema.validate_analysis(), Ok(()));
    assert!(!crate::records::is_records(&schema));
    assert!(has_cited(&schema.fields));
    let keys: Vec<&str> = schema.fields.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["summary", "themes", "keywords", "quotations"]);
    // Quotations alone, with no field of their own.
    let alone = AnalysisSchema {
        fields: vec![Field {
            key: "passages".into(),
            ..field(
                "Passages",
                FieldKind::Cited {
                    fields: vec![],
                    max: None,
                },
            )
        }],
    };
    assert_eq!(alone.validate_analysis(), Ok(()));
}

#[test]
fn a_cited_field_refuses_a_group_inside_or_the_quotations_name() {
    let mut nested = AnalysisSchema {
        fields: vec![field(
            "Themes",
            FieldKind::Cited {
                fields: vec![field(
                    "Inner",
                    FieldKind::Entries {
                        fields: vec![field("A", FieldKind::Text)],
                    },
                )],
                max: None,
            },
        )],
    };
    nested.assign_keys();
    assert_eq!(
        nested.validate(),
        Err(SchemaError::NestedGroup("Inner".into()))
    );
    let mut reserved = AnalysisSchema {
        fields: vec![field(
            "Themes",
            FieldKind::Cited {
                fields: vec![field("Quotes", FieldKind::Text)],
                max: None,
            },
        )],
    };
    reserved.assign_keys();
    assert_eq!(
        reserved.validate(),
        Err(SchemaError::ReservedName("Quotes".into()))
    );
    // A cited field inside a repeating group, and a group beside a cited field.
    let mut inside = AnalysisSchema {
        fields: vec![field(
            "Rows",
            FieldKind::Entries {
                fields: vec![field(
                    "Themes",
                    FieldKind::Cited {
                        fields: vec![],
                        max: None,
                    },
                )],
            },
        )],
    };
    inside.assign_keys();
    assert_eq!(
        inside.validate(),
        Err(SchemaError::NestedGroup("Themes".into()))
    );
}

#[test]
fn the_answer_is_read_with_its_claims_which_are_never_stored() {
    let schema = interview();
    let answer = json!({
        "summary": " Un entretien. ",
        "themes": [
            {"name": "Débuts", "description": "Les premières années.",
             "quotes": [{"turn": 2, "words": "J'ai commencé en 1998."}]},
            {"name": "Devise", "description": "Une phrase reprise.",
             "quotes": [{"turn": "[4]", "words": "on ne savait pas"}, {"turn": 4, "words": "  "}]},
            {"name": "De trop", "description": "Au-delà du maximum.", "quotes": []},
        ],
        "keywords": ["port", "1998"],
        "quotations": [],
    });
    let answers = schema
        .read_answer(&answer)
        .unwrap_or_else(|e| panic!("{e}"));
    let themes = answers[1].value.cited().cloned().unwrap_or_default();
    assert_eq!(themes.entries.len(), 2, "cut to its maximum");
    assert_eq!(
        themes.entries[1].claims,
        [Claim {
            turn: Some(4),
            words: "on ne savait pas".into()
        }]
    );
    assert!(themes.entries.iter().all(|e| e.evidence.is_empty()));
    // Written and read back, an unchecked answer has no quotation at all.
    let stored: FieldAnswer =
        serde_json::from_str(&serde_json::to_string(&answers[1]).unwrap_or_default())
            .unwrap_or_else(|e| panic!("{e}"));
    let stored = stored.value.cited().cloned().unwrap_or_default();
    assert!(stored.entries.iter().all(|e| e.claims.is_empty()));
    // An entry with no list of quotations breaks the answer.
    let broken = json!({"summary": "x", "themes": [{"name": "a", "description": "b"}],
        "keywords": [], "quotations": []});
    assert!(schema.read_answer(&broken).is_err());
}

#[test]
fn quotations_are_placed_in_the_turn_named_with_its_speaker_and_times() {
    let built = transcript();
    assert_eq!(built.turns.len(), 4);
    let schema = interview();
    let mut answers = schema
        .read_answer(&json!({
            "summary": "Un entretien.",
            "themes": [
                // The model names turn 4 for a phrase said in turns 2 and 4: turn 4's is kept.
                {"name": "Devise", "description": "d",
                 "quotes": [{"turn": 4, "words": "on ne savait pas"}]},
                // Turn 1 named, the words are in turn 2: the nearest turn holding them.
                {"name": "Débuts", "description": "d",
                 "quotes": [{"turn": 1, "words": "« J'ai commencé en 1998 »"},
                            {"turn": 2, "words": "Elle n'a jamais dit cela."}]},
            ],
            "keywords": [],
            "quotations": [
                {"why_it_matters": "x", "quotes": [{"turn": 3, "words": "Introuvable ici."}]},
                {"why_it_matters": "y", "quotes": [{"turn": 9, "words": "Au début, on ne savait pas."}]},
            ],
        }))
        .unwrap_or_else(|e| panic!("{e}"));
    let counted = cite(&schema.fields, &mut answers, doc(&built));
    assert_eq!(counted, Cited { given: 5, found: 3 });

    let themes = answers[1].value.cited().cloned().unwrap_or_default();
    let devise = &themes.entries[0].evidence[0];
    assert_eq!(devise.turn, 4);
    assert_eq!(devise.quote, "on ne savait pas");
    assert_eq!(devise.speaker.as_deref(), Some("Awa"));
    assert_eq!(
        (devise.start_ms, devise.end_ms),
        (Some(52_000), Some(90_000))
    );
    assert_eq!(
        crate::snapshot::check(&built.text, &built.parts, &devise.anchor, &devise.quote),
        Ok(())
    );
    let debut = &themes.entries[1].evidence;
    assert_eq!(debut.len(), 1, "the words not in the text are left out");
    assert_eq!(debut[0].turn, 2);
    // The text's own apostrophe, not the model's.
    assert_eq!(debut[0].quote, "J’ai commencé en 1998");
    assert_eq!(themes.left_out, 1);
    assert_eq!(themes.dropped, 0);

    let quotations = answers[3].value.cited().cloned().unwrap_or_default();
    assert_eq!(
        quotations.entries.len(),
        1,
        "an entry with no quotation found is left out"
    );
    assert_eq!((quotations.left_out, quotations.dropped), (1, 1));
    // A turn number past the end: looked for everywhere, in order.
    assert_eq!(quotations.entries[0].evidence[0].turn, 2);
    assert_eq!(
        quotations.entries[0].evidence[0].quote,
        "Au début, on ne savait pas."
    );

    assert_eq!(
        counts(&schema.fields, &answers),
        [
            ("themes".to_string(), 3, 1, 0),
            ("quotations".to_string(), 2, 1, 1)
        ]
    );
}

#[test]
fn a_quotation_running_across_two_turns_is_found_in_neither() {
    let built = transcript();
    let claim = Claim {
        turn: Some(1),
        words: "port ? J’ai commencé".into(),
    };
    assert_eq!(place(doc(&built), &claim), None);
}

#[test]
fn the_text_is_sent_with_its_turns_numbered() {
    let built = transcript();
    let sent = numbered(&built.text, &built.turns);
    let lines: Vec<&str> = sent.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(
        lines[0],
        "[1] [00:00:01] Interviewer: Comment avez-vous commencé au port ?"
    );
    assert!(lines[3].starts_with("[4] [00:00:52] Awa: Ensuite, «"));
    // Paragraphs of a plain text: numbers alone, each paragraph on one line.
    let (text, parts) = crate::snapshot::plain("Premier\nparagraphe.\n\nSecond.\n");
    let turns = crate::snapshot::turns_of(Task::Analysis, &text, &parts);
    assert_eq!(
        numbered(&text, &turns),
        "[1] Premier paragraphe.\n[2] Second."
    );
}

#[test]
fn rendered_as_text_each_entry_has_its_quotations_and_what_was_left_out() {
    let built = transcript();
    let schema = interview();
    let mut answers = schema
        .read_answer(&json!({
            "summary": "Un entretien.",
            "themes": [{"name": "Devise", "description": "Une phrase reprise.",
                        "quotes": [{"turn": 4, "words": "on ne savait pas"},
                                   {"turn": 4, "words": "absent"},
                                   {"turn": 4, "words": "absent aussi"}]}],
            "keywords": ["port"],
            "quotations": [],
        }))
        .unwrap_or_else(|e| panic!("{e}"));
    cite(&schema.fields, &mut answers, doc(&built));
    assert_eq!(
        schema.render(&answers),
        "Summary\nUn entretien.\n\nThemes\n\nName\nDevise\n\nDescription\nUne phrase reprise.\n\
         \"on ne savait pas\" (Awa, 00:00:52)\n\n2 quotations could not be found in the text \
         and were left out.\n\nKeywords: port\n\nQuotations"
    );
    assert_eq!(answers[1].value.joined(), "Devise");
}

fn cited_answers() -> (AnalysisSchema, Vec<FieldAnswer>) {
    let built = transcript();
    let schema = interview();
    let mut answers = schema
        .read_answer(&json!({
            "summary": "s",
            "themes": [
                {"name": "Devise", "description": "d",
                 "quotes": [{"turn": 4, "words": "on ne savait pas"},
                            {"turn": 2, "words": "Au début"}]},
                {"name": "Débuts", "description": "d",
                 "quotes": [{"turn": 2, "words": "J’ai commencé en 1998"}]},
            ],
            "keywords": [],
            "quotations": [],
        }))
        .unwrap_or_else(|e| panic!("{e}"));
    cite(&schema.fields, &mut answers, doc(&built));
    (schema, answers)
}

#[test]
fn the_researcher_edits_an_entry_removes_one_or_one_of_its_quotations() {
    let (schema, model) = cited_answers();
    let renamed = edit(
        &schema.fields,
        &model,
        "themes",
        &CitedEdit::Answer {
            entry: 1,
            key: "name".into(),
            value: "  Les débuts ".into(),
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let themes = renamed[1].value.cited().cloned().unwrap_or_default();
    assert_eq!(
        themes.entries[1].answers[0].value,
        FieldValue::Text("Les débuts".into())
    );
    assert_eq!(renamed[0], model[0], "other fields are kept");

    let one_less = edit(
        &schema.fields,
        &renamed,
        "themes",
        &CitedEdit::RemoveQuotation {
            entry: 0,
            quotation: 1,
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let themes = one_less[1].value.cited().cloned().unwrap_or_default();
    assert_eq!(themes.entries[0].evidence.len(), 1);
    let removed = edit(
        &schema.fields,
        &one_less,
        "themes",
        &CitedEdit::RemoveEntry { entry: 0 },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(removed[1].value.joined(), "Les débuts");

    for (key, change, code) in [
        (
            "summary",
            CitedEdit::RemoveEntry { entry: 0 },
            "cited.not_cited",
        ),
        (
            "themes",
            CitedEdit::RemoveEntry { entry: 5 },
            "cited.no_entry",
        ),
        (
            "themes",
            CitedEdit::RemoveQuotation {
                entry: 0,
                quotation: 9,
            },
            "cited.no_quotation",
        ),
        (
            "themes",
            CitedEdit::Answer {
                entry: 0,
                key: "colour".into(),
                value: "x".into(),
            },
            "cited.no_field",
        ),
    ] {
        let refused = edit(&schema.fields, &model, key, &change);
        assert_eq!(refused.err().map(|e| e.code()), Some(code), "{change:?}");
    }
}

#[test]
fn a_correction_keeps_only_quotations_the_model_found() {
    let (schema, model) = cited_answers();
    // Removing is fine, and the counts stay the model's.
    let mut fewer = edit(
        &schema.fields,
        &model,
        "themes",
        &CitedEdit::RemoveEntry { entry: 1 },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    if let FieldValue::Cited(value) = &mut fewer[1].value {
        value.left_out = 7;
    }
    assert_eq!(keeps_evidence(&schema.fields, &model, &mut fewer), Ok(()));
    assert_eq!(fewer[1].value.cited().map(|v| v.left_out), Some(0));
    // An altered quotation, or one moved elsewhere in the text, is refused.
    let alterations: [fn(&mut Evidence); 4] = [
        |e: &mut Evidence| e.quote = "on savait".into(),
        |e: &mut Evidence| e.anchor.start += 1,
        |e: &mut Evidence| e.speaker = Some("Interviewer".into()),
        |e: &mut Evidence| e.start_ms = Some(0),
    ];
    for alter in alterations {
        let mut changed = model.clone();
        if let FieldValue::Cited(value) = &mut changed[1].value {
            alter(&mut value.entries[0].evidence[0]);
        }
        assert_eq!(
            keeps_evidence(&schema.fields, &model, &mut changed),
            Err(CitedError::Invented("Themes".into()))
        );
    }
    // A cited field given as something else is refused.
    let mut listed = model.clone();
    listed[1].value = FieldValue::List(vec!["Devise".into()]);
    assert_eq!(
        keeps_evidence(&schema.fields, &model, &mut listed),
        Err(CitedError::NotCited("Themes".into()))
    );
}

#[test]
fn a_value_written_by_this_version_reads_back_as_cited() {
    let (_, model) = cited_answers();
    let written = serde_json::to_string(&model[1].value).unwrap_or_default();
    assert!(written.starts_with("{\"entries\":["), "{written}");
    let read: FieldValue = serde_json::from_str(&written).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read, model[1].value);
    // The kind is written with its tag, as the others are.
    let kind = serde_json::to_value(FieldKind::Cited {
        fields: vec![],
        max: Some(3),
    })
    .unwrap_or_default();
    assert_eq!(kind, json!({"type": "cited", "fields": [], "max": 3}));
}

/// Two pages read from a scan, in paragraphs, the second page's words also on the first.
fn pages() -> Built {
    let one = PieceRef::page(1);
    let two = PieceRef::page(2);
    let part = |piece, text| TimedPart {
        text: PartText {
            piece,
            text,
            reviewed: false,
            corrected: false,
            failed: false,
        },
        segments: &[],
    };
    build_with_turns(
        Task::TextExtraction,
        &[
            part(
                &one,
                "Le port de Dakar s’agrandit.\n\nLes dockers parlent de grève.",
            ),
            part(&two, "La grève des dockers dure trois jours."),
        ],
    )
}

#[test]
fn a_passage_of_a_text_read_from_pages_keeps_its_page_and_no_speaker() {
    let built = pages();
    assert_eq!(built.turns.len(), 3, "each paragraph is a passage");
    let schema = AnalysisSchema {
        fields: vec![Field {
            key: "passages".into(),
            ..field(
                "Passages",
                FieldKind::Cited {
                    fields: vec![],
                    max: None,
                },
            )
        }],
    };
    let mut answers = schema
        .read_answer(&json!({"passages": [
            {"quotes": [{"turn": 3, "words": "La grève des dockers dure trois jours."}]},
            {"quotes": [{"turn": 1, "words": "Le port de Dakar s'agrandit."}]},
            {"quotes": [{"turn": 2, "words": "Les dockers sont en grève."}]},
        ]}))
        .unwrap_or_else(|e| panic!("{e}"));
    let counted = cite(&schema.fields, &mut answers, doc(&built));
    assert_eq!(counted, Cited { given: 3, found: 2 });
    let value = answers[0].value.cited().cloned().unwrap_or_default();
    assert_eq!(
        (value.left_out, value.dropped),
        (1, 1),
        "a passage not in the text"
    );
    let first = &value.entries[0].evidence[0];
    assert_eq!(first.page, Some(PieceRef::page(2)));
    assert_eq!(
        (first.turn, &first.speaker, first.start_ms),
        (3, &None, None)
    );
    assert_eq!(first.anchor.part, 1);
    // The text's own apostrophe is kept, not the model's.
    let second = &value.entries[1].evidence[0];
    assert_eq!(second.quote, "Le port de Dakar s’agrandit.");
    assert_eq!(second.page, Some(PieceRef::page(1)));
    assert_eq!(said_by(second), "page 1");
}
