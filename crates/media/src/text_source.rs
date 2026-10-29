//! Text inputs for summaries: plain text, Markdown, CSV and Word files, read as UTF-8 text.

use std::path::{Path, PathBuf};

use docx_rs::{
    DocumentChild, InsertChild, Paragraph, ParagraphChild, RunChild, TableCellContent, TableChild,
    TableRowChild,
};

use kalamu_core::{PageHalf, PieceRef};

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

/// At most this many characters in a passage of a text file sent for correction: a few pages,
/// well within every model's output limit when the passage comes back corrected.
pub const PASSAGE_CHARS: usize = 12_000;

impl TextSource {
    /// The text in the pieces a correction sends: the pages of a text that marks them as
    /// Kalamu and zmo-ai-pipelines write them (`--- Page 3 ---`), otherwise passages of at most
    /// [`PASSAGE_CHARS`] cut at paragraph breaks. The same text always gives the same pieces.
    pub fn sections(&self) -> Result<Vec<(PieceRef, String)>, MediaError> {
        Ok(sections(&self.text()?))
    }

    /// The text of one piece of [`TextSource::sections`].
    pub fn section(&self, piece: &PieceRef) -> Result<String, MediaError> {
        self.sections()?
            .into_iter()
            .find(|(p, _)| p == piece)
            .map(|(_, text)| text)
            .ok_or_else(|| MediaError::NoSuchPage(piece.label()))
    }
}

pub(crate) fn sections(text: &str) -> Vec<(PieceRef, String)> {
    let mut pages: Vec<(PieceRef, String)> = Vec::new();
    for line in text.lines() {
        if let Some(piece) = page_marker(line) {
            pages.push((piece, String::new()));
        } else if let Some((_, page)) = pages.last_mut() {
            page.push_str(line);
            page.push('\n');
        } else if !line.trim().is_empty() {
            // Text before the first marker: not a paged text after all.
            pages.clear();
            break;
        }
    }
    if !pages.is_empty() {
        return pages
            .into_iter()
            .map(|(piece, page)| (piece, page.trim().to_string()))
            .collect();
    }
    passages(text)
        .into_iter()
        .enumerate()
        .map(|(i, passage)| {
            (
                PieceRef::Passage {
                    number: i as u32 + 1,
                },
                passage,
            )
        })
        .collect()
}

/// `--- Page 3 ---` or `--- Page 3 (left) ---`.
fn page_marker(line: &str) -> Option<PieceRef> {
    let inner = line
        .trim()
        .strip_prefix("--- Page ")?
        .strip_suffix(" ---")?
        .trim();
    let (number, half) = match inner.split_once(' ') {
        Some((n, "(left)")) => (n, Some(PageHalf::Left)),
        Some((n, "(right)")) => (n, Some(PageHalf::Right)),
        Some(_) => return None,
        None => (inner, None),
    };
    Some(PieceRef::Page {
        number: number.parse().ok()?,
        half,
    })
}

/// Paragraphs gathered into passages of at most [`PASSAGE_CHARS`]; a longer paragraph is cut
/// at a line break, else at a space.
fn passages(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    for paragraph in text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
        for part in cut(paragraph) {
            if !current.is_empty()
                && current.chars().count() + part.chars().count() + 2 > PASSAGE_CHARS
            {
                out.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(&part);
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn cut(paragraph: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = paragraph;
    while rest.chars().count() > PASSAGE_CHARS {
        let limit = rest
            .char_indices()
            .nth(PASSAGE_CHARS)
            .map_or(rest.len(), |(i, _)| i);
        let head = &rest[..limit];
        let at = head
            .rfind('\n')
            .or_else(|| head.rfind(' '))
            .filter(|&i| i > 0)
            .unwrap_or(limit);
        out.push(rest[..at].trim().to_string());
        rest = rest[at..].trim_start();
    }
    if !rest.is_empty() {
        out.push(rest.to_string());
    }
    out
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

    #[test]
    fn a_paged_text_is_cut_at_its_page_markers() {
        let text = "--- Page 1 ---\n\nLe Hajj\n\n--- Page 2 (left) ---\n\nSuite.\n";
        let found = sections(text);
        assert_eq!(
            found,
            vec![
                (PieceRef::page(1), "Le Hajj".to_string()),
                (
                    PieceRef::Page {
                        number: 2,
                        half: Some(PageHalf::Left)
                    },
                    "Suite.".to_string()
                ),
            ]
        );
        // Text before the first marker: passages instead.
        assert_eq!(
            sections("Préface\n\n--- Page 1 ---\nx")[0].0,
            PieceRef::Passage { number: 1 }
        );
    }

    #[test]
    fn a_long_text_is_cut_into_passages_at_paragraphs() {
        let paragraph = "mot ".repeat(1_000);
        let text = [paragraph.trim(); 8].join("\n\n");
        let found = sections(&text);
        assert!(found.len() > 2);
        assert!(
            found
                .iter()
                .all(|(_, p)| p.chars().count() <= PASSAGE_CHARS)
        );
        assert_eq!(found[1].0, PieceRef::Passage { number: 2 });
        let words: usize = found
            .iter()
            .map(|(_, p)| p.split_whitespace().count())
            .sum();
        assert_eq!(words, 8_000, "nothing lost or repeated");
        // One paragraph longer than a passage is cut at a space.
        let long = "a ".repeat(PASSAGE_CHARS);
        assert!(sections(&long).len() == 2);
    }
}
