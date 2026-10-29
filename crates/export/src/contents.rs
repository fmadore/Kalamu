//! A table of contents set out for each format: one entry per article or item, its title and
//! pages first, then the researcher's fields, in the order the preset lists them. The entries
//! come from the combining step, with the researcher's corrections.

use kalamu_core::contents::{PAGES, TITLE, entries};
use kalamu_core::{ExportDocument, Field, FieldAnswer, FieldKind};

/// The fields of each entry (title, pages, then the researcher's), from the combining step's
/// schema.
pub(crate) fn fields(doc: &ExportDocument) -> Vec<Field> {
    doc.setup
        .merge
        .as_ref()
        .and_then(|m| m.prompt.schema.as_ref())
        .and_then(|s| s.fields.first())
        .and_then(|f| match &f.kind {
            FieldKind::Entries { fields } => Some(fields.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// The document's entries, in order.
pub(crate) fn list(doc: &ExportDocument) -> &[Vec<FieldAnswer>] {
    doc.parts
        .iter()
        .find(|p| p.piece == kalamu_core::PieceRef::Whole)
        .map(|p| entries(&p.fields))
        .unwrap_or(&[])
}

pub(crate) fn value(entry: &[FieldAnswer], key: &str) -> String {
    kalamu_core::analysis::answer_text(entry, key)
}

/// The researcher's fields with an answer in this entry, as (label, value).
pub(crate) fn details<'a>(fields: &'a [Field], entry: &[FieldAnswer]) -> Vec<(&'a str, String)> {
    fields
        .iter()
        .filter(|f| f.key != TITLE && f.key != PAGES)
        .map(|f| (f.label.as_str(), value(entry, &f.key)))
        .filter(|(_, v)| !v.trim().is_empty())
        .collect()
}

/// Plain text: each entry numbered, with its pages, then its fields.
pub(crate) fn text(doc: &ExportDocument) -> String {
    let fields = fields(doc);
    let mut out = String::new();
    for (i, entry) in list(doc).iter().enumerate() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("{}. {}\n", i + 1, value(entry, TITLE)));
        let pages = value(entry, PAGES);
        if !pages.is_empty() {
            out.push_str(&format!("   Pages: {pages}\n"));
        }
        for (label, v) in details(&fields, entry) {
            out.push_str(&format!("   {label}: {v}\n"));
        }
    }
    out
}

/// Markdown: a heading per entry, its pages and fields as a list.
pub(crate) fn markdown(doc: &ExportDocument) -> String {
    let fields = fields(doc);
    let mut out = String::new();
    for entry in list(doc) {
        out.push_str(&format!("\n## {}\n\n", value(entry, TITLE)));
        let pages = value(entry, PAGES);
        if !pages.is_empty() {
            out.push_str(&format!("- **Pages:** {pages}\n"));
        }
        for (label, v) in details(&fields, entry) {
            out.push_str(&format!("- **{label}:** {v}\n"));
        }
    }
    out
}
