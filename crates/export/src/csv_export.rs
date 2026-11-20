use kalamu_core::{
    CsvCells, ExportDocument, ExportError, ExportFile, ExportFormat, Exporter, Table, Task,
};

use crate::timeline::timeline;
use crate::{table_csv, table_xlsx};

/// One row per page (text extraction), per segment (transcription), per document (analysis)
/// or per entry (records, a table of contents), as CSV for R, Python and other tools.
pub struct CsvExporter;

/// The same table as [`CsvExporter`]'s, as an Excel workbook, which opens by double-click in a
/// spreadsheet set to any language.
pub struct XlsxExporter;

impl Exporter for CsvExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Csv
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        Ok(table_csv(&table(doc))?.into_bytes())
    }

    fn export_files(&self, doc: &ExportDocument) -> Result<Vec<ExportFile>, ExportError> {
        files(self, doc, |t| Ok(table_csv(t)?.into_bytes()))
    }
}

impl Exporter for XlsxExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Xlsx
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        table_xlsx(&table(doc))
    }

    fn export_files(&self, doc: &ExportDocument) -> Result<Vec<ExportFile>, ExportError> {
        files(self, doc, table_xlsx)
    }
}

/// The table, and for an analysis that cites the text one row per quotation beside it
/// (`<name>.quotations.csv`, `<name>.quotations.xlsx`), each written by `write`.
fn files(
    exporter: &dyn Exporter,
    doc: &ExportDocument,
    write: impl Fn(&Table) -> Result<Vec<u8>, ExportError>,
) -> Result<Vec<ExportFile>, ExportError> {
    let extension = exporter.format().extension();
    let mut files = vec![ExportFile {
        suffix: extension.to_owned(),
        bytes: write(&table(doc))?,
    }];
    if let Some(fields) = crate::cited::fields(doc) {
        files.push(ExportFile {
            suffix: format!("quotations.{extension}"),
            bytes: write(&crate::cited::quotations(doc, fields))?,
        });
    }
    Ok(files)
}

/// Every task's table is guarded in its CSV: the text was read off a document by a model (a list
/// item, a telephone number, a formula quoted, or one planted to run), the file is for a
/// spreadsheet, and the JSON export keeps the text exactly for a program. The workbook holds
/// every cell as text and needs no guarding.
fn table(doc: &ExportDocument) -> Table {
    // Records: one row per entry, however many pages.
    if let Some(columns) = crate::records::columns(doc) {
        return crate::records::table(doc, columns);
    }
    match doc.task {
        Task::TextExtraction | Task::TextCorrection | Task::Translation => Table {
            cells: CsvCells::Guarded,
            headings: strings(["page", "text", "corrected", "reviewed"]),
            rows: doc
                .parts
                .iter()
                .enumerate()
                .map(|(i, part)| {
                    vec![
                        part.piece.label(),
                        crate::part_text(doc, i),
                        part.corrected.to_string(),
                        part.reviewed.to_string(),
                    ]
                })
                .collect(),
        },
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
            let mut headings = vec!["file".to_string()];
            for field in fields {
                headings.push(field.label.clone());
                if field.reason {
                    headings.push(format!("{} (reason)", field.label));
                }
            }
            headings.extend(["corrected".into(), "reviewed".into()]);
            let rows = doc
                .parts
                .iter()
                .map(|part| {
                    let mut row = vec![doc.source.file_name.clone()];
                    for field in fields {
                        let answer = part.fields.iter().find(|a| a.key == field.key);
                        row.push(answer.map(|a| a.value.joined()).unwrap_or_default());
                        if field.reason {
                            row.push(answer.and_then(|a| a.reason.clone()).unwrap_or_default());
                        }
                    }
                    row.extend([part.corrected.to_string(), part.reviewed.to_string()]);
                    row
                })
                .collect();
            Table {
                headings,
                rows,
                cells: CsvCells::Guarded,
            }
        }
        // One row per entry: title, pages, then the researcher's fields.
        Task::Contents => {
            let fields = crate::contents::fields(doc);
            let mut headings = vec!["file".to_string()];
            headings.extend(fields.iter().map(|f| f.label.clone()));
            headings.extend(["corrected".into(), "reviewed".into()]);
            let part = doc
                .parts
                .iter()
                .find(|p| p.piece == kalamu_core::PieceRef::Whole);
            let (corrected, reviewed) = part.map_or((false, false), |p| (p.corrected, p.reviewed));
            let rows = crate::contents::list(doc)
                .iter()
                .map(|entry| {
                    let mut row = vec![doc.source.file_name.clone()];
                    row.extend(fields.iter().map(|f| crate::contents::value(entry, &f.key)));
                    row.extend([corrected.to_string(), reviewed.to_string()]);
                    row
                })
                .collect();
            Table {
                headings,
                rows,
                cells: CsvCells::Guarded,
            }
        }
        Task::AudioTranscription => Table {
            cells: CsvCells::Guarded,
            headings: strings(["start", "end", "speaker", "text"]),
            rows: timeline(doc)
                .into_iter()
                .map(|seg| {
                    vec![
                        kalamu_core::result::clock(seg.start_ms),
                        kalamu_core::result::clock(seg.end_ms),
                        seg.speaker.unwrap_or_default(),
                        seg.text,
                    ]
                })
                .collect(),
        },
    }
}

fn strings<const N: usize>(words: [&str; N]) -> Vec<String> {
    words.map(String::from).to_vec()
}
