//! A detailed table of contents, in two steps, as iwac-ai-pipelines' `AI_summary_issue` makes
//! one: each page is read for the articles or items on it (title, a note when it continues on
//! another page, the researcher's fields for the part on that page), then a second model
//! combines the pages' readings into one entry per article or item, with all its pages.
//!
//! The researcher chooses the entry's fields (a summary, the authors, a date, the kind of
//! document); the title and the pages are always there, so every entry can be found in the
//! source. Both steps reach providers as analyses with an entries field.

use serde_json::{Value, json};

use crate::analysis::reason_key;
use crate::{AnalysisSchema, Field, FieldAnswer, FieldKind, FieldValue, SchemaError};

/// The field that holds the entries in both steps' answers.
pub const ENTRIES: &str = "entries";
/// What else a page holds: a cover, advertisements, a blank page.
pub const PAGE_NOTE: &str = "page_note";
pub const TITLE: &str = "title";
pub const PAGES: &str = "pages";
pub const CONTINUES: &str = "continues";

/// Keys the two steps use themselves, which the researcher's fields cannot take.
pub const RESERVED: &[&str] = &[ENTRIES, PAGE_NOTE, TITLE, PAGES, CONTINUES];

fn fixed(key: &str, label: &str, instructions: &str) -> Field {
    Field {
        key: key.to_string(),
        label: label.to_string(),
        instructions: instructions.to_string(),
        kind: FieldKind::Text,
        reason: false,
    }
}

/// Checks the researcher's entry fields: a valid schema whose keys leave the fixed ones free.
pub fn validate(entry: &AnalysisSchema) -> Result<(), SchemaError> {
    entry.validate()?;
    match entry
        .fields
        .iter()
        .find(|f| RESERVED.contains(&f.key.as_str()))
    {
        Some(f) => Err(SchemaError::Duplicate(f.label.clone())),
        None => Ok(()),
    }
}

/// The first step's answer for one page: its entries, each with a title, where it continues,
/// and the researcher's fields for the part on this page; and what else the page holds.
pub fn page_schema(entry: &AnalysisSchema) -> AnalysisSchema {
    let mut fields = vec![
        fixed(
            TITLE,
            "Title",
            "The title as printed, with its section in square brackets before it when it has \
             one ([Editorial] …). For the end of an article or item begun on an earlier page \
             whose title is not repeated, \"Continued: \" and the subject.",
        ),
        fixed(
            CONTINUES,
            "Continues",
            "Where it continues or comes from, as the page says (\"continued on page 5\", \"to be \
             continued\"), or empty.",
        ),
    ];
    fields.extend(entry.fields.iter().cloned());
    AnalysisSchema {
        fields: vec![
            Field {
                key: ENTRIES.to_string(),
                label: "Entries".to_string(),
                instructions: "Every article or item on this page, in reading order; none for a \
                               cover, a blank page or a page of advertisements."
                    .to_string(),
                kind: FieldKind::Entries { fields },
                reason: false,
            },
            fixed(
                PAGE_NOTE,
                "Other content",
                "What else the page holds, briefly (cover, advertisements, notices, a table of \
                 contents, a blank page), or empty.",
            ),
        ],
    }
}

/// The second step's answer for the whole document: one entry per article or item, with every
/// page it is on and the researcher's fields for all of it.
pub fn merge_schema(entry: &AnalysisSchema) -> AnalysisSchema {
    let mut fields = vec![
        fixed(
            TITLE,
            "Title",
            "The full title as printed, with its section in square brackets before it when it has \
             one, obvious OCR errors corrected.",
        ),
        fixed(
            PAGES,
            "Pages",
            "Every page the article or item is on, as page numbers and ranges (\"1, 3-5\").",
        ),
    ];
    fields.extend(entry.fields.iter().cloned());
    AnalysisSchema {
        fields: vec![Field {
            key: ENTRIES.to_string(),
            label: "Entries".to_string(),
            instructions: "One entry per article or item of the whole document, in the order \
                           they begin."
                .to_string(),
            kind: FieldKind::Entries { fields },
            reason: false,
        }],
    }
}

/// The researcher's entry fields, from either step's schema.
pub fn entry_fields(schema: &AnalysisSchema) -> Vec<Field> {
    schema
        .fields
        .iter()
        .find_map(|f| match &f.kind {
            FieldKind::Entries { fields } => Some(
                fields
                    .iter()
                    .filter(|f| !RESERVED.contains(&f.key.as_str()))
                    .cloned()
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

/// The entries of an answer (a page's or the whole document's).
pub fn entries(answers: &[FieldAnswer]) -> &[Vec<FieldAnswer>] {
    answers
        .iter()
        .find(|a| a.key == ENTRIES)
        .map(|a| a.value.entries())
        .unwrap_or(&[])
}

/// The answers for a table of contents, from its entries.
pub fn answers(entries: Vec<Vec<FieldAnswer>>) -> Vec<FieldAnswer> {
    vec![FieldAnswer {
        key: ENTRIES.to_string(),
        value: FieldValue::Entries(entries),
        reason: None,
    }]
}

/// The pages' readings as the second step reads them: one JSON object per page, labelled as
/// the source numbers it, in reading order. Plain JSON, so any text model reads it.
pub fn merge_input(pages: &[(String, &[FieldAnswer])]) -> String {
    let pages: Vec<Value> = pages
        .iter()
        .map(|(label, answers)| {
            let mut page = serde_json::Map::new();
            page.insert("page".into(), json!(label));
            for answer in *answers {
                page.insert(answer.key.clone(), answer_json(&answer.value));
                if let Some(reason) = &answer.reason {
                    page.insert(reason_key(&answer.key), json!(reason));
                }
            }
            Value::Object(page)
        })
        .collect();
    serde_json::to_string_pretty(&json!({ "pages": pages })).unwrap_or_default()
}

fn answer_json(value: &FieldValue) -> Value {
    match value {
        FieldValue::Text(t) => json!(t),
        FieldValue::List(items) => json!(items),
        FieldValue::Entries(entries) => Value::Array(
            entries
                .iter()
                .map(|entry| {
                    Value::Object(
                        entry
                            .iter()
                            .map(|a| (a.key.clone(), answer_json(&a.value)))
                            .collect(),
                    )
                })
                .collect(),
        ),
    }
}

/// The first page an entry's pages name (`"3-5, 9"` → 3), for going to it in the review.
pub fn first_page(pages: &str) -> Option<u32> {
    let digits: String = pages
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn entry() -> AnalysisSchema {
        let mut schema = AnalysisSchema {
            fields: vec![
                Field {
                    key: String::new(),
                    label: "Authors".into(),
                    instructions: String::new(),
                    kind: FieldKind::List { max: None },
                    reason: false,
                },
                Field {
                    key: String::new(),
                    label: "Summary".into(),
                    instructions: String::new(),
                    kind: FieldKind::Text,
                    reason: false,
                },
            ],
        };
        schema.assign_keys();
        schema
    }

    #[test]
    fn both_steps_keep_the_title_and_the_researchers_fields() {
        let page = page_schema(&entry());
        let merge = merge_schema(&entry());
        assert!(page.validate().is_ok());
        assert!(merge.validate().is_ok());
        let keys = |s: &AnalysisSchema| -> Vec<String> {
            match &s.fields[0].kind {
                FieldKind::Entries { fields } => fields.iter().map(|f| f.key.clone()).collect(),
                _ => Vec::new(),
            }
        };
        assert_eq!(keys(&page), ["title", "continues", "authors", "summary"]);
        assert_eq!(keys(&merge), ["title", "pages", "authors", "summary"]);
        assert_eq!(page.fields[1].key, PAGE_NOTE);
        let fields: Vec<_> = entry_fields(&merge).into_iter().map(|f| f.key).collect();
        assert_eq!(fields, ["authors", "summary"]);
    }

    #[test]
    fn a_field_cannot_take_a_fixed_name() {
        let mut taken = entry();
        taken.fields[1].key = "pages".into();
        assert!(validate(&taken).is_err());
        assert!(validate(&entry()).is_ok());
    }

    #[test]
    fn answers_read_back_and_feed_the_second_step() {
        let page = page_schema(&entry());
        let answers = page
            .read_answer(&json!({
                "entries": [{
                    "title": " [Éditorial] Le Hajj ",
                    "continues": "suite page 3",
                    "authors": ["La Rédaction"],
                    "summary": "Sur le pèlerinage."
                }],
                "page_note": ""
            }))
            .unwrap();
        let found = entries(&answers);
        assert_eq!(found.len(), 1);
        assert_eq!(
            crate::analysis::answer_text(&found[0], TITLE),
            "[Éditorial] Le Hajj"
        );
        let input = merge_input(&[("1".to_string(), answers.as_slice())]);
        let read: Value = serde_json::from_str(&input).unwrap();
        assert_eq!(read["pages"][0]["page"], "1");
        assert_eq!(read["pages"][0]["entries"][0]["continues"], "suite page 3");
        assert!(page.render(&answers).contains("[Éditorial] Le Hajj"));

        let empty = page
            .read_answer(&json!({ "entries": [], "page_note": "Cover" }))
            .unwrap();
        assert!(entries(&empty).is_empty());
    }

    #[test]
    fn the_first_page_of_an_entry() {
        assert_eq!(first_page("3-5, 9"), Some(3));
        assert_eq!(first_page("p. 12"), Some(12));
        assert_eq!(first_page(""), None);
    }
}
