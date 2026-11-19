//! An analysis whose fields cite the text, as a researcher uses it: a Markdown memo (each
//! field under its heading, each entry's quotations as block quotes with the speaker and the
//! time), and one row per quotation for a spreadsheet (`<name>.quotations.csv`, beside the
//! analysis's own CSV). Both read the corrected answers: an entry or a quotation the researcher
//! removed is not there. The JSON result keeps every quotation's anchor, turn and times in its
//! fields, and names the snapshot read. A document with no cited field is written by the other
//! modules exactly as before.

use kalamu_core::analysis::cited::{self, CitedValue, Evidence};
use kalamu_core::{CsvCells, ExportDocument, Field, FieldAnswer, FieldKind, Table, Task};
use serde_json::{Value, json};

/// The analysis's fields, when one of them cites the text; `None` for every other document.
pub(crate) fn fields(doc: &ExportDocument) -> Option<&[Field]> {
    let schema = doc.setup.prompt.schema.as_ref()?;
    (doc.task == Task::Analysis && cited::has_cited(&schema.fields)).then_some(&schema.fields)
}

/// The text read, for the JSON result: the snapshot's id (none for a text file given directly)
/// and the hash of the text the anchors count in.
pub(crate) fn snapshot(doc: &ExportDocument) -> Value {
    json!({
        "id": doc.source.derived_from.as_ref().map(|d| d.snapshot_id.clone()),
        "sha256": doc.source.sha256,
    })
}

/// A quotation's speaker and time as the memo writes them: `Awa, [00:04:12]`.
fn said_by(evidence: &Evidence) -> String {
    let time = evidence
        .start_ms
        .map(|ms| format!("[{}]", kalamu_core::result::clock(ms)));
    [evidence.speaker.clone(), time]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
}

/// A quotation as a block quote, with who said it and when below it.
fn block_quote(evidence: &Evidence) -> String {
    let mut out: Vec<String> = evidence
        .quote
        .lines()
        .map(|l| format!("> {}", l.trim()))
        .collect();
    let by = said_by(evidence);
    if !by.is_empty() {
        out.push(">".into());
        out.push(format!("> — {by}"));
    }
    out.join("\n")
}

/// One answer of an entry's own fields, or of the document's, as a paragraph.
fn paragraph(field: &Field, answer: &FieldAnswer) -> String {
    let value = answer.value.joined();
    match field.kind {
        FieldKind::Text => value,
        _ => format!("**{}:** {value}", field.label.trim()),
    }
}

/// One entry: with two fields or more, the first as its heading and the others below it, then
/// its quotations; with one field (a quotation's reason), the quotations first and the field
/// after them; with none, its quotations alone.
fn entry(own: &[Field], answers: &[FieldAnswer], evidence: &[Evidence]) -> String {
    let answer = |field: &Field| answers.iter().find(|a| a.key == field.key);
    let quotes: Vec<String> = evidence.iter().map(block_quote).collect();
    let mut blocks = Vec::new();
    match own {
        [] => blocks.extend(quotes),
        [only] => {
            blocks.extend(quotes);
            blocks.extend(answer(only).map(|a| paragraph(only, a)));
        }
        [first, rest @ ..] => {
            let heading = answer(first).map(|a| a.value.joined()).unwrap_or_default();
            blocks.push(format!("### {heading}"));
            blocks.extend(
                rest.iter()
                    .filter_map(|f| answer(f).map(|a| paragraph(f, a))),
            );
            blocks.extend(quotes);
        }
    }
    blocks.retain(|b| !b.trim().is_empty());
    blocks.join("\n\n")
}

/// The memo: the header every Markdown file has, then each field under its heading, cited
/// fields entry by entry, and what was left out at the end of each.
pub(crate) fn markdown(doc: &ExportDocument, fields: &[Field]) -> String {
    let mut out = format!(
        "# {}\n\n*{} · {} · prompt: {}*\n",
        doc.source.file_name, doc.setup.provider, doc.setup.model, doc.setup.prompt.preset_name
    );
    for part in &doc.parts {
        if let Some(reason) = &part.failed {
            out.push_str(&format!("\n{}\n", crate::not_read(reason)));
            continue;
        }
        for field in fields {
            let Some(answer) = part.fields.iter().find(|a| a.key == field.key) else {
                continue;
            };
            let mut blocks = vec![format!("## {}", field.label.trim())];
            match (&field.kind, answer.value.cited()) {
                (FieldKind::Cited { fields: own, .. }, Some(value)) => {
                    blocks.extend(
                        value
                            .entries
                            .iter()
                            .map(|e| entry(own, &e.answers, &e.evidence)),
                    );
                    blocks.extend(cited::left_out_note(value).map(|n| format!("*{n}*")));
                }
                // Under its own heading, a field's value alone.
                _ => blocks.push(answer.value.joined()),
            }
            if let Some(reason) = &answer.reason {
                blocks.push(format!("*Reason: {reason}*"));
            }
            blocks.retain(|b| !b.trim().is_empty());
            out.push('\n');
            out.push_str(&blocks.join("\n\n"));
            out.push('\n');
        }
    }
    out
}

/// One row per quotation kept, in the order of the fields and their entries: the field, the
/// entry (its first field: a theme's name, a quotation's reason), who said it, when, the
/// quotation as the text has it and its turn. Guarded, as every table for a spreadsheet is.
pub(crate) fn quotations(doc: &ExportDocument, fields: &[Field]) -> Table {
    let headings = [
        "file",
        "field",
        "entry",
        "speaker",
        "start",
        "end",
        "quotation",
        "turn",
    ]
    .map(String::from)
    .to_vec();
    let clock = |ms: Option<u64>| ms.map(kalamu_core::result::clock).unwrap_or_default();
    let mut rows = Vec::new();
    for part in doc.parts.iter().filter(|p| p.failed.is_none()) {
        for field in fields {
            let Some(value): Option<&CitedValue> = part
                .fields
                .iter()
                .find(|a| a.key == field.key)
                .and_then(|a| a.value.cited())
            else {
                continue;
            };
            for e in &value.entries {
                let name = e
                    .answers
                    .first()
                    .map(|a| a.value.joined())
                    .unwrap_or_default();
                for evidence in &e.evidence {
                    rows.push(vec![
                        doc.source.file_name.clone(),
                        field.label.trim().to_string(),
                        name.clone(),
                        evidence.speaker.clone().unwrap_or_default(),
                        clock(evidence.start_ms),
                        clock(evidence.end_ms),
                        evidence.quote.clone(),
                        evidence.turn.to_string(),
                    ]);
                }
            }
        }
    }
    Table {
        headings,
        rows,
        cells: CsvCells::Guarded,
    }
}
