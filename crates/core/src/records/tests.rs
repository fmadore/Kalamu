//! Records: what makes an analysis one, the checks on its group, a page's rows read back as the
//! model wrote them, and the correction layer's changes to them.

#![allow(clippy::unwrap_used)]

use serde_json::json;

use super::*;
use crate::SchemaError;

fn column(label: &str, kind: FieldKind) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn group(label: &str, columns: Vec<Field>) -> Field {
    column(label, FieldKind::Entries { fields: columns })
}

/// A register's rows: a name, a trade from a fixed list, and the children named.
fn register() -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![group(
            "Entries",
            vec![
                column("Surname", FieldKind::Text),
                column(
                    "Trade",
                    FieldKind::Choice {
                        values: vec!["Tailor".into(), "Trader".into(), "Farmer".into()],
                    },
                ),
                column("Children", FieldKind::List { max: None }),
            ],
        )],
    };
    schema.assign_keys();
    schema
}

fn text(s: &str) -> FieldValue {
    FieldValue::Text(s.into())
}

#[test]
fn an_analysis_is_records_by_its_fields_and_its_columns_get_keys() {
    let schema = register();
    assert!(is_records(&schema));
    let keys: Vec<&str> = columns(&schema).iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["surname", "trade", "children"]);
    assert_eq!(schema.validate_analysis(), Ok(()));
    let summary = AnalysisSchema {
        fields: vec![column("Summary", FieldKind::Text)],
    };
    assert!(!is_records(&summary));
    assert!(columns(&summary).is_empty());
}

#[test]
fn a_group_needs_columns_and_stands_alone() {
    let mut empty = AnalysisSchema {
        fields: vec![group("Entries", vec![])],
    };
    empty.assign_keys();
    assert_eq!(
        empty.validate_analysis(),
        Err(SchemaError::EmptyGroup("Entries".into()))
    );

    let mut nested = AnalysisSchema {
        fields: vec![group(
            "Entries",
            vec![group("Children", vec![column("Name", FieldKind::Text)])],
        )],
    };
    nested.assign_keys();
    assert_eq!(
        nested.validate_analysis(),
        Err(SchemaError::NestedGroup("Children".into()))
    );

    let mut beside = register();
    beside.fields.push(column("Street", FieldKind::Text));
    beside.assign_keys();
    assert_eq!(
        beside.validate_analysis(),
        Err(SchemaError::GroupNotAlone("Street".into()))
    );
    // The general check leaves that to the analysis: a table of contents' page has its entries
    // beside a note.
    assert_eq!(beside.validate(), Ok(()));

    let mut two = register();
    two.fields
        .push(group("Others", vec![column("Name", FieldKind::Text)]));
    two.assign_keys();
    assert_eq!(two.validate_analysis(), Err(SchemaError::TwoGroups));

    // Columns are named apart within their group, as fields are.
    let mut twice = register();
    if let FieldKind::Entries { fields } = &mut twice.fields[0].kind {
        fields[1].label = "surname".into();
    }
    assert_eq!(
        twice.validate_analysis(),
        Err(SchemaError::Duplicate("surname".into()))
    );
}

/// A register page with ditto marks: the rows come back in reading order, each mark kept as
/// written, nothing carried down from the row above.
#[test]
fn a_register_page_keeps_its_rows_and_ditto_marks_as_written() {
    let schema = register();
    let answers = schema
        .read_answer(&json!({ "entries": [
            { "surname": "Diallo", "trade": "Tailor", "children": ["Awa", "Moussa"] },
            { "surname": "\"", "trade": "tailor", "children": [] },
            { "surname": "do.", "trade": "Trader", "children": ["Fanta"] },
            { "surname": "Diallo", "trade": "Tailor", "children": ["Awa", "Moussa"] }
        ]}))
        .unwrap();
    let found = rows(&schema, &answers);
    assert_eq!(
        found.len(),
        4,
        "a row repeated on the page is a row of its own"
    );
    assert_eq!(found[1][0].value, text("\""));
    assert_eq!(found[2][0].value, text("do."));
    assert_eq!(
        found[1][1].value,
        text("Tailor"),
        "a choice spelled as listed"
    );
}

/// A right-to-left list: the text stays as written, in the order read.
#[test]
fn a_right_to_left_list_is_kept_as_written() {
    let mut schema = AnalysisSchema {
        fields: vec![group(
            "Members",
            vec![
                column("Name", FieldKind::Text),
                column("Town", FieldKind::Text),
            ],
        )],
    };
    schema.assign_keys();
    let answers = schema
        .read_answer(&json!({ "members": [
            { "name": "عبد الله بن محمد", "town": "تمبكتو" },
            { "name": "أحمد بابا", "town": "جني" }
        ]}))
        .unwrap();
    let found = rows(&schema, &answers);
    assert_eq!(found[0][0].value, text("عبد الله بن محمد"));
    assert_eq!(found[1][1].value, text("جني"));
}

fn page() -> Vec<FieldAnswer> {
    register()
        .read_answer(&json!({ "entries": [
            { "surname": "Diallo", "trade": "Tailor", "children": ["Awa"] },
            { "surname": "Traoré", "trade": "Farmer", "children": [] }
        ]}))
        .unwrap()
}

#[test]
fn a_cell_is_written_as_its_column_reads_it() {
    let schema = register();
    let edited = edit(
        &schema,
        &page(),
        &RowEdit::Cell {
            row: 1,
            key: "children".into(),
            value: " Ali ; Binta;ali ".into(),
        },
    )
    .unwrap();
    let found = rows(&schema, &edited);
    assert_eq!(
        found[1][2].value,
        FieldValue::List(vec!["Ali".into(), "Binta".into()])
    );
    assert_eq!(
        found[0],
        rows(&schema, &page())[0],
        "the other rows as they were"
    );

    let choice = edit(
        &schema,
        &edited,
        &RowEdit::Cell {
            row: 0,
            key: "trade".into(),
            value: "trader".into(),
        },
    )
    .unwrap();
    assert_eq!(rows(&schema, &choice)[0][1].value, text("Trader"));
    // A choice left empty is the researcher's to leave; one outside the list is refused.
    let cleared = edit(
        &schema,
        &choice,
        &RowEdit::Cell {
            row: 0,
            key: "trade".into(),
            value: " ".into(),
        },
    )
    .unwrap();
    assert_eq!(rows(&schema, &cleared)[0][1].value, text(""));
    assert_eq!(
        edit(
            &schema,
            &choice,
            &RowEdit::Cell {
                row: 0,
                key: "trade".into(),
                value: "Smith".into(),
            },
        ),
        Err(RowError::NotAChoice {
            value: "Smith".into(),
            label: "Trade".into()
        })
    );
}

#[test]
fn rows_are_added_and_removed_where_asked() {
    let schema = register();
    let added = edit(&schema, &page(), &RowEdit::Add { at: 1 }).unwrap();
    let found = rows(&schema, &added);
    assert_eq!(found.len(), 3);
    assert_eq!(found[1][0].value, text(""));
    assert_eq!(found[1][2].value, FieldValue::List(Vec::new()));
    assert_eq!(found[2][0].value, text("Traoré"));
    // Past the end, after the last.
    let last = edit(&schema, &page(), &RowEdit::Add { at: 99 }).unwrap();
    assert_eq!(rows(&schema, &last)[2][0].value, text(""));

    let removed = edit(&schema, &added, &RowEdit::Remove { row: 0 }).unwrap();
    let found = rows(&schema, &removed);
    assert_eq!(found.len(), 2);
    assert_eq!(found[1][0].value, text("Traoré"));
    assert_eq!(
        edit(&schema, &removed, &RowEdit::Remove { row: 2 }),
        Err(RowError::NoRow(2))
    );
    // Every row removed leaves the page with none, not without its answer.
    let none = edit(&schema, &page(), &RowEdit::Remove { row: 0 })
        .and_then(|a| edit(&schema, &a, &RowEdit::Remove { row: 0 }))
        .unwrap();
    assert_eq!(none.len(), 1);
    assert!(rows(&schema, &none).is_empty());
    // And a row can be added to a page that had none.
    let first = edit(&schema, &none, &RowEdit::Add { at: 0 }).unwrap();
    assert_eq!(rows(&schema, &first).len(), 1);
}

#[test]
fn edits_are_refused_where_there_is_nothing_to_change() {
    let schema = register();
    assert_eq!(
        edit(
            &schema,
            &page(),
            &RowEdit::Cell {
                row: 5,
                key: "surname".into(),
                value: "x".into(),
            },
        ),
        Err(RowError::NoRow(5))
    );
    assert_eq!(
        edit(
            &schema,
            &page(),
            &RowEdit::Cell {
                row: 0,
                key: "street".into(),
                value: "x".into(),
            },
        ),
        Err(RowError::NoColumn("street".into()))
    );
    let summary = AnalysisSchema {
        fields: vec![column("Summary", FieldKind::Text)],
    };
    assert_eq!(
        edit(&summary, &[], &RowEdit::Add { at: 0 }),
        Err(RowError::NotRecords)
    );
}

/// Applies the review's changes one after another, as the correction layer keeps them.
fn edited(start: Vec<FieldAnswer>, changes: &[RowEdit]) -> Vec<FieldAnswer> {
    let schema = register();
    changes.iter().fold(start, |answers, change| {
        edit(&schema, &answers, change).unwrap()
    })
}

fn typed(row: u32, key: &str, value: &str) -> RowEdit {
    RowEdit::Cell {
        row,
        key: key.into(),
        value: value.into(),
    }
}

#[test]
fn only_the_rows_typed_into_or_added_are_corrected() {
    let schema = register();
    let model = page();
    let marks = |changes: &[RowEdit]| corrected_rows(&schema, &model, &edited(page(), changes));
    // One cell typed anew: that row alone.
    assert_eq!(marks(&[typed(1, "surname", "Traore")]), [false, true]);
    // A row added above: it alone, the model's rows below it not.
    assert_eq!(marks(&[RowEdit::Add { at: 0 }]), [true, false, false]);
    // A row removed above: the row that moved up is still the model's.
    assert_eq!(marks(&[RowEdit::Remove { row: 0 }]), [false]);
    // Removed, then one added in its place and typed into.
    assert_eq!(
        marks(&[
            RowEdit::Remove { row: 0 },
            RowEdit::Add { at: 0 },
            typed(0, "surname", "Keïta"),
        ]),
        [true, false]
    );
    // A cell typed back as the model wrote it leaves its row the model's.
    assert_eq!(
        marks(&[typed(0, "surname", "Diallo"), typed(1, "children", "Ali")]),
        [false, true]
    );
    // Nothing changed: no row.
    assert_eq!(corrected_rows(&schema, &model, &model), [false, false]);
}

#[test]
fn a_row_typed_to_match_another_of_the_models_is_still_corrected() {
    let schema = register();
    let model = register()
        .read_answer(&json!({ "entries": [
            { "surname": "Diallo", "trade": "Tailor", "children": [] },
            { "surname": "Sow", "trade": "Farmer", "children": [] },
            { "surname": "Ba", "trade": "Trader", "children": [] }
        ]}))
        .unwrap();
    // The first row typed into until it reads as the third: the third is still the model's.
    let changed = edited(
        model.clone(),
        &[typed(0, "surname", "Ba"), typed(0, "trade", "Trader")],
    );
    assert_eq!(
        corrected_rows(&schema, &model, &changed),
        [true, false, false]
    );
}
