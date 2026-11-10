//! The names an analysis lists in a field, one item at a time: whether an answer holds one, and
//! the answers with one renamed or taken out. Items are matched as the analysis review collates
//! them, ignoring case and the spaces around them.

use crate::{FieldAnswer, FieldValue};

fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Whether the answer to field `key` holds `item`: one of a list's items, or a whole text.
pub fn holds_item(answers: &[FieldAnswer], key: &str, item: &str) -> bool {
    answers
        .iter()
        .find(|a| a.key == key)
        .is_some_and(|a| match &a.value {
            FieldValue::List(items) => items.iter().any(|v| same(v, item)),
            FieldValue::Text(text) => !text.trim().is_empty() && same(text, item),
            FieldValue::Entries(_) => false,
        })
}

/// The answers with one item of list field `key` renamed to `to`, or taken out when `to` is
/// none or blank. An item renamed to one the list already holds is kept once. Other fields,
/// and each field's reason, are left as they are.
pub fn rename_item(
    answers: &[FieldAnswer],
    key: &str,
    from: &str,
    to: Option<&str>,
) -> Vec<FieldAnswer> {
    let next = to.map(str::trim).unwrap_or_default();
    answers
        .iter()
        .map(|a| match &a.value {
            FieldValue::List(items) if a.key == key => {
                let mut out: Vec<String> = Vec::with_capacity(items.len());
                for item in items {
                    let value = if same(item, from) { next } else { item };
                    if !value.is_empty() && !out.iter().any(|o| same(o, value)) {
                        out.push(value.to_string());
                    }
                }
                FieldAnswer {
                    value: FieldValue::List(out),
                    ..a.clone()
                }
            }
            _ => a.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answers() -> Vec<FieldAnswer> {
        vec![
            FieldAnswer {
                key: "people".into(),
                value: FieldValue::List(vec![
                    "Samory Touré".into(),
                    "El Hadj Omar".into(),
                    "samory touré ".into(),
                ]),
                reason: Some("Named in the second paragraph.".into()),
            },
            FieldAnswer {
                key: "summary".into(),
                value: FieldValue::Text("Samory Touré".into()),
                reason: None,
            },
        ]
    }

    fn people(answers: &[FieldAnswer]) -> &FieldValue {
        &answers[0].value
    }

    #[test]
    fn an_item_is_renamed_in_its_field_only_ignoring_case() {
        let renamed = rename_item(
            &answers(),
            "people",
            "samory touré ",
            Some(" Almamy Samory"),
        );
        assert_eq!(
            people(&renamed),
            &FieldValue::List(vec!["Almamy Samory".into(), "El Hadj Omar".into()])
        );
        assert_eq!(
            renamed[0].reason.as_deref(),
            Some("Named in the second paragraph.")
        );
        assert_eq!(renamed[1], answers()[1], "another field is left as it was");
        assert!(holds_item(&renamed, "people", "almamy samory"));
        assert!(!holds_item(&renamed, "people", "Samory Touré"));
    }

    #[test]
    fn an_item_renamed_to_one_already_listed_is_kept_once() {
        let renamed = rename_item(&answers(), "people", "El Hadj Omar", Some("SAMORY TOURÉ"));
        assert_eq!(
            people(&renamed),
            &FieldValue::List(vec!["Samory Touré".into()])
        );
    }

    #[test]
    fn an_item_renamed_to_nothing_is_taken_out() {
        for to in [None, Some("  ")] {
            let renamed = rename_item(&answers(), "people", "El Hadj Omar", to);
            assert_eq!(
                people(&renamed),
                &FieldValue::List(vec!["Samory Touré".into()])
            );
        }
    }

    #[test]
    fn a_text_answer_holds_its_whole_text_but_is_never_renamed() {
        assert!(holds_item(&answers(), "summary", "samory touré"));
        assert!(!holds_item(&answers(), "summary", "Samory"));
        assert!(!holds_item(&answers(), "places", "Bamako"));
        let renamed = rename_item(&answers(), "summary", "Samory Touré", Some("Samory"));
        assert_eq!(renamed, answers());
    }
}
