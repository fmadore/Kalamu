//! Records (#171) set out as a table: one row per entry across the document's pages, the file,
//! the page and the entry's place on it first, then the group's columns as the researcher named
//! them, from the corrected rows. A page that could not be read stands as one row of its own,
//! saying so, so its entries are never taken for none.
//!
//! `corrected` is a row's own: true for a row the researcher typed into or added
//! ([`ExportPart::rows_corrected`]), false for the model's rows around it. `reviewed` is the
//! page's, as the review marks a page.

use kalamu_core::analysis::answer_text;
use kalamu_core::{CsvCells, ExportDocument, ExportPart, Field, FieldAnswer, Table, Task, records};
use serde_json::{Value, json};

/// The group's columns, for a records analysis; `None` for every other document.
pub(crate) fn columns(doc: &ExportDocument) -> Option<&[Field]> {
    let schema = doc.setup.prompt.schema.as_ref()?;
    (doc.task == Task::Analysis && records::is_records(schema)).then(|| records::columns(schema))
}

/// A page's rows, in reading order.
fn rows<'a>(doc: &ExportDocument, part: &'a ExportPart) -> &'a [Vec<FieldAnswer>] {
    doc.setup
        .prompt
        .schema
        .as_ref()
        .map_or(&[], |schema| records::rows(schema, &part.fields))
}

/// Whether the researcher typed into or added a page's row `index`.
fn corrected(part: &ExportPart, index: usize) -> bool {
    part.rows_corrected.get(index).copied().unwrap_or(false)
}

/// The table for a spreadsheet: guarded, as every task's is.
pub(crate) fn table(doc: &ExportDocument, columns: &[Field]) -> Table {
    let mut headings: Vec<String> = ["file", "page", "position"].map(String::from).to_vec();
    for column in columns {
        headings.push(column.label.clone());
        if column.reason {
            headings.push(format!("{} (reason)", column.label));
        }
    }
    headings.extend(["corrected", "reviewed", "not_read"].map(String::from));
    let mut out = Vec::new();
    for part in &doc.parts {
        let start = [doc.source.file_name.clone(), part.piece.label()];
        if let Some(reason) = &part.failed {
            let mut row = start.to_vec();
            row.push(String::new());
            row.extend(
                columns
                    .iter()
                    .flat_map(|c| std::iter::repeat_n(String::new(), if c.reason { 2 } else { 1 })),
            );
            row.extend(["false".into(), "false".into(), crate::not_read(reason)]);
            out.push(row);
            continue;
        }
        for (i, entry) in rows(doc, part).iter().enumerate() {
            let mut row = start.to_vec();
            row.push((i + 1).to_string());
            for column in columns {
                row.push(answer_text(entry, &column.key));
                if column.reason {
                    let reason = entry.iter().find(|a| a.key == column.key);
                    row.push(reason.and_then(|a| a.reason.clone()).unwrap_or_default());
                }
            }
            row.extend([
                corrected(part, i).to_string(),
                part.reviewed.to_string(),
                String::new(),
            ]);
            out.push(row);
        }
    }
    Table {
        headings,
        rows: out,
        cells: CsvCells::Guarded,
    }
}

/// Every row as an object, for the JSON result: its file, page and place on the page, and its
/// cells by column key, exactly as the researcher left them.
pub(crate) fn json_rows(doc: &ExportDocument) -> Vec<Value> {
    let mut out = Vec::new();
    for part in doc.parts.iter().filter(|p| p.failed.is_none()) {
        for (i, entry) in rows(doc, part).iter().enumerate() {
            let fields: serde_json::Map<String, Value> = entry
                .iter()
                .map(|a| (a.key.clone(), json!(a.value)))
                .collect();
            let mut row = json!({
                "file": doc.source.file_name,
                "page": part.piece.label(),
                "position": i + 1,
                "fields": fields,
                "corrected": corrected(part, i),
                "reviewed": part.reviewed,
            });
            let reasons: serde_json::Map<String, Value> = entry
                .iter()
                .filter_map(|a| Some((a.key.clone(), json!(a.reason.as_ref()?))))
                .collect();
            if !reasons.is_empty() {
                row["reasons"] = json!(reasons);
            }
            out.push(row);
        }
    }
    out
}
