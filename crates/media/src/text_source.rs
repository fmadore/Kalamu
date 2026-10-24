//! Text inputs for summaries: plain text, Markdown, CSV and Word files, read as UTF-8 text.

use std::path::{Path, PathBuf};

use docx_rs::{
    DocumentChild, InsertChild, Paragraph, ParagraphChild, RunChild, TableCellContent, TableChild,
    TableRowChild,
};

use crate::MediaError;

pub struct TextSource {
    path: PathBuf,
}

impl TextSource {
    pub fn open(path: &Path) -> Result<Self, MediaError> {
        let ext = extension(path);
        if !matches!(ext.as_str(), "txt" | "md" | "csv" | "docx") {
            return Err(MediaError::UnsupportedKind);
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// The document's text: Word files flattened to paragraphs, a byte-order mark dropped, and
    /// stray bytes that are not UTF-8 replaced rather than refused.
    pub fn text(&self) -> Result<String, MediaError> {
        let bytes = std::fs::read(&self.path).map_err(|e| MediaError::Read {
            path: self.path.display().to_string(),
            message: e.to_string(),
        })?;
        if extension(&self.path) == "docx" {
            return docx_text(&bytes);
        }
        let text = String::from_utf8_lossy(&bytes);
        Ok(text.trim_start_matches('\u{feff}').to_string())
    }
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Paragraphs and table cells of a Word file, one per line, in reading order. The archive is
/// read in memory; nothing is extracted to disk.
fn docx_text(bytes: &[u8]) -> Result<String, MediaError> {
    let docx = docx_rs::read_docx(bytes).map_err(|e| MediaError::Read {
        path: "Word file".into(),
        message: e.to_string(),
    })?;
    let mut lines = Vec::new();
    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(p) => lines.push(paragraph_text(p)),
            DocumentChild::Table(table) => {
                for TableChild::TableRow(row) in &table.rows {
                    let cells: Vec<String> = row
                        .cells
                        .iter()
                        .map(|TableRowChild::TableCell(cell)| {
                            cell.children
                                .iter()
                                .filter_map(|c| match c {
                                    TableCellContent::Paragraph(p) => Some(paragraph_text(p)),
                                    _ => None,
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .collect();
                    lines.push(cells.join("\t"));
                }
            }
            _ => {}
        }
    }
    Ok(lines.join("\n").trim().to_string())
}

fn paragraph_text(paragraph: &Paragraph) -> String {
    let mut out = String::new();
    for child in &paragraph.children {
        match child {
            ParagraphChild::Run(run) => run_text(&run.children, &mut out),
            ParagraphChild::Insert(insert) => {
                for c in &insert.children {
                    if let InsertChild::Run(run) = c {
                        run_text(&run.children, &mut out);
                    }
                }
            }
            ParagraphChild::Hyperlink(link) => {
                for c in &link.children {
                    if let ParagraphChild::Run(run) = c {
                        run_text(&run.children, &mut out);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn run_text(children: &[RunChild], out: &mut String) {
    for child in children {
        match child {
            RunChild::Text(t) => out.push_str(&t.text),
            RunChild::Tab(_) => out.push('\t'),
            RunChild::Break(_) => out.push('\n'),
            _ => {}
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reads_word_paragraphs_and_tables_and_plain_text() {
        use docx_rs::{Docx, Run, Table, TableCell, TableRow};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lettre.docx");
        let file = std::fs::File::create(&path).unwrap();
        Docx::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Bamako, le 3 mai")))
            .add_table(Table::new(vec![TableRow::new(vec![
                TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Nom"))),
                TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Imam"))),
            ])]))
            .build()
            .pack(file)
            .unwrap();
        let text = TextSource::open(&path).unwrap().text().unwrap();
        assert_eq!(text, "Bamako, le 3 mai\nNom\tImam");

        let txt = dir.path().join("notes.txt");
        std::fs::write(&txt, "\u{feff}Entretien\n").unwrap();
        assert_eq!(
            TextSource::open(&txt).unwrap().text().unwrap(),
            "Entretien\n"
        );
        assert!(TextSource::open(&dir.path().join("table.xlsx")).is_err());
    }
}
