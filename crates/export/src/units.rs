//! The list of a table of contents' document units (#50), written beside their PDFs when the
//! researcher exports them: each unit's id, title, pages in the original, file and fields, so
//! the files can be catalogued elsewhere and traced back to their source.

use kalamu_core::{CsvCells, DocumentUnit, ExportError, Field, Table};
use serde_json::{Value, json};

/// One exported unit and where its pages went.
pub struct Row<'a> {
    pub unit: &'a DocumentUnit,
    /// The pages as typed: `1, 3-5, 8 (left)`.
    pub pages: String,
    /// The PDF of its pages, beside the list.
    pub file_name: String,
    /// The file its pages were cut from, and its hash.
    pub original: String,
    pub original_sha256: String,
}

/// One row per unit, a column per entry field, labelled as the researcher named them.
pub fn csv(rows: &[Row<'_>], fields: &[Field]) -> Result<Vec<u8>, ExportError> {
    let mut headings: Vec<String> = ["id", "number", "title", "pages", "file"]
        .map(String::from)
        .to_vec();
    headings.extend(fields.iter().map(|f| f.label.clone()));
    headings.extend(["original".into(), "original_sha256".into()]);
    let rows = rows
        .iter()
        .map(|row| {
            let mut record = vec![
                row.unit.id.to_string(),
                (row.unit.position + 1).to_string(),
                row.unit.title.clone(),
                row.pages.clone(),
                row.file_name.clone(),
            ];
            record.extend(fields.iter().map(|f| {
                row.unit
                    .fields
                    .iter()
                    .find(|a| a.key == f.key)
                    .map(|a| a.value.joined())
                    .unwrap_or_default()
            }));
            record.extend([row.original.clone(), row.original_sha256.clone()]);
            record
        })
        .collect();
    // Titles and fields come from a model's reading of the document, opened in a spreadsheet;
    // the JSON written beside it is the list for another program.
    let table = Table {
        headings,
        rows,
        cells: CsvCells::Guarded,
    };
    Ok(crate::table_csv(&table)?.into_bytes())
}

/// The units with their pages as the original numbers them (`{"kind": "page", "number": 3}`),
/// for another program to read.
pub fn json(rows: &[Row<'_>]) -> Result<Vec<u8>, ExportError> {
    let units: Vec<Value> = rows
        .iter()
        .map(|row| {
            let fields: serde_json::Map<String, Value> = row
                .unit
                .fields
                .iter()
                .map(|a| (a.key.clone(), json!(a.value)))
                .collect();
            json!({
                "id": row.unit.id,
                "number": row.unit.position + 1,
                "title": row.unit.title,
                "pages": row.unit.pages,
                "pages_text": row.pages,
                "file": row.file_name,
                "fields": fields,
                "entry": row.unit.entry,
                "revision": row.unit.revision,
                "original": { "file_name": row.original, "sha256": row.original_sha256 },
            })
        })
        .collect();
    serde_json::to_vec_pretty(&json!({ "units": units })).map_err(|e| ExportError::Write {
        format: "JSON",
        message: e.to_string(),
    })
}
