//! The analysis schema: keys, checks and answers read back.

#![allow(clippy::unwrap_used)]

use serde_json::json;

use super::*;

fn field(label: &str, kind: FieldKind, reason: bool) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind,
        reason,
    }
}

fn schema() -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![
            field("Résumé", FieldKind::Text, false),
            field("Lieux cités", FieldKind::List { max: Some(2) }, false),
            field(
                "Tone",
                FieldKind::Choice {
                    values: vec!["Positive".into(), "Negative".into()],
                },
                true,
            ),
            field(
                "Themes",
                FieldKind::Choices {
                    values: vec!["Education".into(), "Pilgrimage".into()],
                },
                false,
            ),
        ],
    };
    schema.assign_keys();
    schema
}

#[test]
fn keys_come_from_labels_and_stay_unique() {
    let keys: Vec<_> = schema().fields.into_iter().map(|f| f.key).collect();
    assert_eq!(keys, ["resume", "lieux_cites", "tone", "themes"]);
    let mut twice = AnalysisSchema {
        fields: vec![
            field("Places", FieldKind::Text, false),
            field("places!", FieldKind::Text, false),
            field("2024", FieldKind::Text, false),
        ],
    };
    twice.assign_keys();
    let keys: Vec<_> = twice.fields.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["places", "places_2", "field_3"]);
    assert_eq!(twice.validate(), Ok(()));
    twice.fields[1].label = "places".into();
    assert_eq!(
        twice.validate(),
        Err(SchemaError::Duplicate("places".into()))
    );
}

#[test]
fn choices_need_two_values() {
    let mut one = AnalysisSchema {
        fields: vec![field(
            "Tone",
            FieldKind::Choice {
                values: vec!["Positive".into(), " positive ".into()],
            },
            false,
        )],
    };
    one.assign_keys();
    assert_eq!(
        one.validate(),
        Err(SchemaError::TooFewValues("Tone".into()))
    );
    assert_eq!(schema().validate(), Ok(()));
    assert_eq!(
        AnalysisSchema::default().validate(),
        Err(SchemaError::Empty)
    );
}

#[test]
fn answers_are_tidied_and_checked() {
    let schema = schema();
    let answers = schema
        .read_answer(&json!({
            "resume": "  Une réunion à Bamako. ",
            "lieux_cites": ["Bamako", "bamako", " Dakar ", "Ségou"],
            "tone": "negative",
            "tone_reason": "Critical of the decision.",
            "themes": ["pilgrimage", "Education", "Pilgrimage"]
        }))
        .unwrap();
    assert_eq!(
        answers[0].value,
        FieldValue::Text("Une réunion à Bamako.".into())
    );
    assert_eq!(
        answers[1].value,
        FieldValue::List(vec!["Bamako".into(), "Dakar".into()])
    );
    assert_eq!(answers[2].value, FieldValue::Text("Negative".into()));
    assert_eq!(
        answers[2].reason.as_deref(),
        Some("Critical of the decision.")
    );
    assert_eq!(
        answers[3].value,
        FieldValue::List(vec!["Pilgrimage".into(), "Education".into()])
    );
    assert_eq!(
        schema.render(&answers),
        "Résumé\nUne réunion à Bamako.\n\nLieux cités: Bamako; Dakar\n\nTone: Negative\n\
         Reason: Critical of the decision.\n\nThemes: Pilgrimage; Education"
    );
}

#[test]
fn broken_answers_are_errors_not_empty_results() {
    let schema = schema();
    let base = json!({ "resume": "x", "lieux_cites": [], "tone": "Positive", "themes": [] });
    assert!(schema.read_answer(&base).is_ok());
    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("lieux_cites");
    assert!(
        schema
            .read_answer(&missing)
            .unwrap_err()
            .contains("Lieux cités")
    );
    let mut outside = base.clone();
    outside["tone"] = json!("Mixed");
    assert!(schema.read_answer(&outside).unwrap_err().contains("Mixed"));
    assert!(schema.read_answer(&json!(["x"])).is_err());
}
