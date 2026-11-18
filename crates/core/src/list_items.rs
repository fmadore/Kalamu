//! The names an analysis lists in a field, one item at a time: whether an answer holds one, the
//! answers with one renamed, taken out or added by the researcher, and which items are the
//! researcher's rather than the model's. Items are matched as the analysis review collates
//! them, ignoring case and the spaces around them. [`folded`] sets aside accents and punctuation
//! too, for the comparisons that only ask whether two forms are probably the same name.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::{FieldAnswer, FieldValue};

/// A form folded for comparison only: capitals, accents and punctuation set aside, as
/// iwac-ai-pipelines folds names before matching them, so `Traoré, Moussa` and `traore moussa`
/// fold alike. The dictionary offers two forms with one key as possibly the same, and agreement
/// between set-ups counts them as the same item; neither ever changes a result on that account.
pub fn folded(form: &str) -> String {
    let plain: String = form
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    plain.split_whitespace().collect::<Vec<_>>().join(" ")
}

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

/// An item as the researcher typed it, its runs of spaces closed up and its ends trimmed.
fn tidy(item: &str) -> String {
    item.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The items in what the researcher typed or pasted to add to a list: one per line or between
/// semicolons, as [`FieldValue::joined`] writes a list on one line, each tidied, the blank ones
/// left out and one typed twice kept once.
pub fn split_items(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in text.split([';', '\n', '\r']).map(tidy) {
        if !item.is_empty() && !out.iter().any(|o| same(o, &item)) {
            out.push(item);
        }
    }
    out
}

/// The answers with `items` added at the end of list field `key`, and which of them were added:
/// one the list already holds (matched as [`holds_item`] matches) is not added twice. A field
/// the answers lack is added to them, without a reason; a text or entries answer is left as it
/// is, since only a list takes items. Other fields are left as they are.
pub fn add_items(
    answers: &[FieldAnswer],
    key: &str,
    items: &[String],
) -> (Vec<FieldAnswer>, Vec<String>) {
    let mut added: Vec<String> = Vec::new();
    let mut grow = |list: &mut Vec<String>| {
        for item in items.iter().map(|i| tidy(i)) {
            if !item.is_empty() && !list.iter().any(|v| same(v, &item)) {
                list.push(item.clone());
                added.push(item);
            }
        }
    };
    let mut out: Vec<FieldAnswer> = answers.to_vec();
    match out.iter_mut().find(|a| a.key == key) {
        Some(answer) => match &mut answer.value {
            FieldValue::List(list) => grow(list),
            FieldValue::Text(text) if text.trim().is_empty() => {
                let mut list = Vec::new();
                grow(&mut list);
                answer.value = FieldValue::List(list);
            }
            FieldValue::Text(_) | FieldValue::Entries(_) => {}
        },
        None => {
            let mut list = Vec::new();
            grow(&mut list);
            if !list.is_empty() {
                out.push(FieldAnswer {
                    key: key.to_string(),
                    value: FieldValue::List(list),
                    reason: None,
                });
            }
        }
    }
    (out, added)
}

/// The items of each list answer in `corrected` that the model's own `answers` to the same field
/// did not hold: the researcher's, added or renamed from the model's form, field by field in
/// the order of `corrected`. Fields with none are left out.
pub fn by_hand(answers: &[FieldAnswer], corrected: &[FieldAnswer]) -> Vec<(String, Vec<String>)> {
    corrected
        .iter()
        .filter_map(|c| {
            let FieldValue::List(items) = &c.value else {
                return None;
            };
            let own: Vec<String> = items
                .iter()
                .filter(|i| !holds_item(answers, &c.key, i))
                .cloned()
                .collect();
            (!own.is_empty()).then(|| (c.key.clone(), own))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_pasted_are_split_at_semicolons_and_lines_and_tidied() {
        assert_eq!(
            split_items("  Kankan ; Siguiri\r\n\nkankan;  Kouroussa   Nord ;"),
            vec!["Kankan", "Siguiri", "Kouroussa Nord"]
        );
        assert!(split_items(" ; \n ").is_empty());
    }

    #[test]
    fn an_item_is_added_once_at_the_end_of_its_list() {
        let (added, which) = add_items(
            &answers(),
            "people",
            &["Almamy  Samory ".into(), "SAMORY TOURÉ".into(), "".into()],
        );
        assert_eq!(which, vec!["Almamy Samory"]);
        assert_eq!(
            people(&added),
            &FieldValue::List(vec![
                "Samory Touré".into(),
                "El Hadj Omar".into(),
                "samory touré ".into(),
                "Almamy Samory".into(),
            ])
        );
        assert_eq!(added[1], answers()[1], "another field is left as it was");
        assert_eq!(
            added[0].reason.as_deref(),
            Some("Named in the second paragraph.")
        );
        // Accents are not set aside: another spelling is another item.
        let (_, which) = add_items(&answers(), "people", &["Samory Toure".into()]);
        assert_eq!(which, vec!["Samory Toure"]);
    }

    #[test]
    fn an_item_added_to_a_field_not_answered_makes_its_answer() {
        let (added, which) = add_items(&answers(), "places", &["Bissandugu".into()]);
        assert_eq!(which, vec!["Bissandugu"]);
        assert_eq!(added.len(), 3);
        assert_eq!(added[2].value, FieldValue::List(vec!["Bissandugu".into()]));
        assert_eq!(added[2].reason, None);
        // Nothing to add, nothing made.
        let (same_answers, which) = add_items(&answers(), "places", &[" ".into()]);
        assert!(which.is_empty());
        assert_eq!(same_answers, answers());
        // A text answer takes no items.
        let (text, which) = add_items(&answers(), "summary", &["Samory".into()]);
        assert!(which.is_empty());
        assert_eq!(text, answers());
    }

    #[test]
    fn the_researchers_items_are_those_the_model_did_not_list() {
        let (corrected, _) = add_items(&answers(), "people", &["Bissandugu".into()]);
        let corrected = rename_item(&corrected, "people", "El Hadj Omar", Some("Omar Tall"));
        assert_eq!(
            by_hand(&answers(), &corrected),
            vec![(
                "people".to_string(),
                vec!["Omar Tall".to_string(), "Bissandugu".to_string()]
            )]
        );
        assert!(by_hand(&answers(), &answers()).is_empty());
    }

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
