//! Output formats. Each format is one [`Exporter`]; adding a format means adding a module and a
//! line in [`all`], nothing else.
//!
//! Exporters read stored results with the user's corrections applied, so changing format later
//! never calls a provider again.

mod contents;
mod csv_export;
mod docx;
mod json;
mod markdown;
mod searchable_pdf;
mod subtitles;
mod text;
mod timeline;

use kalamu_core::{ExportDocument, ExportFormat, Exporter};

pub use timeline::{cues, segments_from_text, timed_lines, timeline};

/// Every exporter, in the order formats are offered.
pub fn all() -> Vec<Box<dyn Exporter>> {
    vec![
        Box::new(text::TextExporter),
        Box::new(markdown::MarkdownExporter),
        Box::new(docx::DocxExporter),
        Box::new(json::JsonExporter),
        Box::new(subtitles::SrtExporter),
        Box::new(subtitles::VttExporter),
        Box::new(csv_export::CsvExporter),
        Box::new(searchable_pdf::SearchablePdfExporter),
    ]
}

pub fn exporter(format: ExportFormat) -> Option<Box<dyn Exporter>> {
    all().into_iter().find(|e| e.format() == format)
}

/// The label printed before each page or segment, or `None` when markers are off or there is
/// only one part.
pub(crate) fn marker(doc: &ExportDocument, index: usize) -> Option<String> {
    if !doc.options.piece_markers || doc.parts.len() < 2 {
        return None;
    }
    let part = &doc.parts[index];
    match part.piece {
        kalamu_core::PieceRef::Page { .. } => Some(format!("Page {}", part.piece.label())),
        // Recordings and long texts are cut only to fit provider limits; the cuts mean nothing
        // to a reader.
        kalamu_core::PieceRef::Segment { .. }
        | kalamu_core::PieceRef::Whole
        | kalamu_core::PieceRef::Passage { .. } => None,
    }
}

/// The text of one part as it should appear in a plain-text format: built from timed segments
/// when there are any (from the model, or read from `[hh:mm:ss]` markers in a transcript, so
/// times run from the start of the recording), otherwise the model's text as is.
///
/// `ExportPart::segments` are already relative to the start of the recording.
///
/// An analysis's text is its answers already set out as text, with the researcher's
/// corrections.
pub(crate) fn part_text(doc: &ExportDocument, index: usize) -> String {
    body_text(doc, index)
}

/// What stands in a document's files for a piece that could not be read.
pub(crate) fn not_read(reason: &str) -> String {
    if reason.is_empty() {
        "[Not read]".into()
    } else {
        format!("[Not read: {reason}]")
    }
}

fn body_text(doc: &ExportDocument, index: usize) -> String {
    let part = &doc.parts[index];
    if let Some(reason) = &part.failed {
        return not_read(reason);
    }
    let parsed;
    let segments = if part.segments.is_empty() {
        let offset = match part.piece {
            kalamu_core::PieceRef::Segment { start_ms, .. } => start_ms,
            kalamu_core::PieceRef::Page { .. }
            | kalamu_core::PieceRef::Whole
            | kalamu_core::PieceRef::Passage { .. } => {
                return part.text.trim().to_string();
            }
        };
        parsed = segments_from_text(&part.text, offset);
        if parsed.is_empty() {
            return part.text.trim().to_string();
        }
        &parsed
    } else {
        &part.segments
    };
    segments
        .iter()
        .map(|s| {
            let mut line = String::new();
            if doc.options.timestamps {
                line.push_str(&format!("[{}] ", kalamu_core::result::clock(s.start_ms)));
            }
            if doc.options.speaker_labels
                && let Some(speaker) = &s.speaker
            {
                line.push_str(&format!("{speaker}: "));
            }
            line.push_str(s.text.trim());
            line
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests;
