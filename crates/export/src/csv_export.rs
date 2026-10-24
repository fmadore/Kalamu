use scriptorium_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};

use crate::timeline::timeline;

/// One row per page (text extraction) or per segment (transcription), for spreadsheets.
pub struct CsvExporter;

impl Exporter for CsvExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Csv
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let err = |e: csv::Error| ExportError::Write {
            format: "CSV",
            message: e.to_string(),
        };
        // A byte-order mark so Excel opens accented text as UTF-8.
        let mut writer = csv::Writer::from_writer(b"\xEF\xBB\xBF".to_vec());
        match doc.task {
            Task::TextExtraction => {
                writer
                    .write_record(["page", "text", "corrected", "reviewed"])
                    .map_err(err)?;
                for part in &doc.parts {
                    writer
                        .write_record([
                            part.piece.label(),
                            part.text.trim().to_string(),
                            part.corrected.to_string(),
                            part.reviewed.to_string(),
                        ])
                        .map_err(err)?;
                }
            }
            Task::AudioTranscription => {
                writer
                    .write_record(["start", "end", "speaker", "text"])
                    .map_err(err)?;
                for seg in timeline(doc) {
                    writer
                        .write_record([
                            scriptorium_core::result::clock(seg.start_ms),
                            scriptorium_core::result::clock(seg.end_ms),
                            seg.speaker.unwrap_or_default(),
                            seg.text,
                        ])
                        .map_err(err)?;
                }
            }
        }
        writer.into_inner().map_err(|e| ExportError::Write {
            format: "CSV",
            message: e.to_string(),
        })
    }
}
