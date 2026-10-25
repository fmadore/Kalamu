use scriptorium_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};

use crate::timeline::timeline;

/// One row per page (text extraction), per segment (transcription) or per document (summary),
/// for spreadsheets.
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
            // One row per document, one column per field (and per reason), labelled as the
            // researcher named them, so the file opens as a table in a spreadsheet.
            Task::Analysis => {
                let fields = doc
                    .setup
                    .prompt
                    .schema
                    .as_ref()
                    .map(|s| s.fields.as_slice())
                    .unwrap_or_default();
                let mut header = vec!["file".to_string()];
                for field in fields {
                    header.push(field.label.clone());
                    if field.reason {
                        header.push(format!("{} (reason)", field.label));
                    }
                }
                header.extend(["corrected".into(), "reviewed".into()]);
                writer.write_record(&header).map_err(err)?;
                for part in &doc.parts {
                    let mut row = vec![doc.source.file_name.clone()];
                    for field in fields {
                        let answer = part.fields.iter().find(|a| a.key == field.key);
                        row.push(answer.map(|a| a.value.joined()).unwrap_or_default());
                        if field.reason {
                            row.push(answer.and_then(|a| a.reason.clone()).unwrap_or_default());
                        }
                    }
                    row.extend([part.corrected.to_string(), part.reviewed.to_string()]);
                    writer.write_record(&row).map_err(err)?;
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
