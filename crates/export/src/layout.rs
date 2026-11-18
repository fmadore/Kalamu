//! What ALTO and PAGE XML share: the pages a document's regions are written on, each measured in
//! the pixels of the image its positions refer to, the lines a block is written as, and XML
//! escaping.
//!
//! The text is the corrected view, decided before an exporter sees it: where the researcher
//! corrected a page whose engine placed each word, the engine lays the corrected words back onto
//! the words' places; where it could not, or the engine placed no word, the corrected text is one
//! block over the page (`kalamu_engine`'s export document). Both formats write the regions they
//! are given as they are.
//!
//! Positions are in pixels: those of the image for an image source, and for a PDF those of the
//! page drawn as Tesseract read it, at the resolution its result records (300 dpi, or less for
//! a very large page; 300 for results read before it was recorded). A page split into halves is
//! put back together, as in the searchable PDF.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use kalamu_core::{ExportDocument, ExportError, InputKind, PieceRef, TextLine, TextRegion};

use crate::searchable_pdf::{pdf_page_sizes, regions_by_page};

/// PDF pages are measured as drawn at this resolution when their result records none: what
/// every page was drawn at before the resolution was recorded, and what a model reading the PDF
/// itself is measured at.
pub(crate) const PDF_DPI: f32 = 300.0;

/// A page whose regions are written.
pub(crate) struct LaidPage {
    pub number: u32,
    /// Width and height in pixels, whole numbers.
    pub width: f32,
    pub height: f32,
    /// The regions on the whole page, in reading order; none for a page read without text.
    pub regions: Vec<TextRegion>,
    /// For a PDF page, the resolution it is measured at.
    pub dpi: Option<f32>,
}

/// A document's pages as ALTO and PAGE XML write them.
pub(crate) struct Layout {
    pub pages: Vec<LaidPage>,
    /// Each PDF page's resolution, where its result records one.
    dpi: BTreeMap<u32, f32>,
    read: BTreeSet<u32>,
}

/// An error writing `format`.
pub(crate) fn error(format: &'static str) -> impl Fn(&dyn std::fmt::Display) -> ExportError {
    move |message| ExportError::Write {
        format,
        message: message.to_string(),
    }
}

impl Layout {
    /// The pages of `doc` to write in `format`: for a PDF every page read, a page that held no
    /// text included; for images, each page with text, measured in its own file. An error when
    /// the model reported no positions at all.
    pub(crate) fn of(doc: &ExportDocument, format: &'static str) -> Result<Self, ExportError> {
        let fail = error(format);
        let mut by_page = regions_by_page(doc);
        if by_page.is_empty() {
            return Err(fail(&format!(
                "this model reported no text positions; choose Tesseract or Mistral OCR for {format}"
            )));
        }
        let dpi = page_resolutions(doc);
        let read = read_pages(doc);
        let sizes: Vec<(u32, (f32, f32), Option<f32>)> = match doc.source.kind {
            InputKind::Pdf => {
                let bytes = std::fs::read(&doc.source.path).map_err(|e| fail(&e))?;
                pdf_page_sizes(&bytes)?
                    .into_iter()
                    .filter(|(n, _)| read.contains(n))
                    .map(|(n, (w, h))| {
                        let dpi = dpi.get(&n).copied().unwrap_or(PDF_DPI);
                        let px = |points: f32| (points / 72.0 * dpi).round();
                        (n, (px(w), px(h)), Some(dpi))
                    })
                    .collect()
            }
            // Each page measured in its own file: one image, or a document of images, whose
            // pages are files of their own sizes.
            InputKind::Image => {
                let mut measured: HashMap<&str, (f32, f32)> = HashMap::new();
                let mut sizes = Vec::new();
                for &n in by_page.keys() {
                    let path = doc
                        .source
                        .page_file(n)
                        .map_or(doc.source.path.as_str(), |(p, _)| p.path.as_str());
                    let size = match measured.get(path) {
                        Some(size) => *size,
                        None => {
                            let size = image_size(path).map_err(|e| fail(&e))?;
                            measured.insert(path, size);
                            size
                        }
                    };
                    sizes.push((n, size, None));
                }
                sizes
            }
            _ => return Err(ExportError::Unsupported(format)),
        };
        let pages = sizes
            .into_iter()
            .map(|(number, (width, height), dpi)| LaidPage {
                number,
                width: width.round(),
                height: height.round(),
                regions: by_page.remove(&number).unwrap_or_default(),
                dpi,
            })
            .collect();
        Ok(Self { pages, dpi, read })
    }

    /// Whether any page has a block placed whole, whose lines are estimated.
    pub(crate) fn estimated(&self) -> bool {
        self.pages
            .iter()
            .flat_map(|p| &p.regions)
            .any(|r| r.lines.is_empty())
    }

    /// The note on how PDF pages are measured: `; PDF pages measured at 300 dpi`, or, where
    /// some were drawn smaller, each other resolution with its pages.
    pub(crate) fn measured(&self) -> String {
        let mut others: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for &page in &self.read {
            let value = self.dpi.get(&page).copied().unwrap_or(PDF_DPI);
            if (value - PDF_DPI).abs() > f32::EPSILON {
                others.entry(written(value)).or_default().push(page);
            }
        }
        let mut note = format!("; PDF pages measured at {} dpi", written(PDF_DPI));
        for (value, pages) in others {
            let pages: Vec<String> = pages.iter().map(u32::to_string).collect();
            let label = if pages.len() == 1 { "page" } else { "pages" };
            note.push_str(&format!(", {label} {} at {value} dpi", pages.join(", ")));
        }
        note
    }
}

impl LaidPage {
    /// The lines a region is written as, and whether they are estimated: the engine's own, or,
    /// for a block placed whole, the lines its shape on this page holds
    /// ([`TextRegion::estimated_lines`]), the same the review draws.
    pub(crate) fn lines(&self, region: &TextRegion) -> (Vec<TextLine>, bool) {
        if region.lines.is_empty() {
            (region.estimated_lines(self.height / self.width), true)
        } else {
            (region.lines.clone(), false)
        }
    }
}

/// The pages read, a page that held no text included; not those a sample left out.
fn read_pages(doc: &ExportDocument) -> BTreeSet<u32> {
    doc.parts
        .iter()
        .filter_map(|p| match p.piece {
            PieceRef::Page { number, .. } => Some(number),
            _ => None,
        })
        .collect()
}

/// The resolution each page was drawn at, where its result records one; for a page split into
/// halves, that of its first half.
fn page_resolutions(doc: &ExportDocument) -> BTreeMap<u32, f32> {
    let mut dpi = BTreeMap::new();
    for part in &doc.parts {
        if let (PieceRef::Page { number, .. }, Some(drawn)) = (&part.piece, part.drawn_dpi) {
            dpi.entry(*number).or_insert(drawn);
        }
    }
    dpi
}

/// A resolution as written: two decimals at most, `300` rather than `300.00`.
pub(crate) fn written(dpi: f32) -> String {
    let text = format!("{dpi:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// An image's width and height as shown, after its EXIF orientation.
fn image_size(path: &str) -> Result<(f32, f32), image::ImageError> {
    use image::ImageDecoder;
    let mut decoder = image::ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let (w, h) = decoder.dimensions();
    let turned = decoder.orientation().is_ok_and(|o| {
        use image::metadata::Orientation::*;
        matches!(o, Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH)
    });
    Ok(if turned {
        (h as f32, w as f32)
    } else {
        (w as f32, h as f32)
    })
}

/// Text escaped for an XML attribute: line breaks and tabs become spaces, and characters XML
/// 1.0 forbids are dropped.
pub(crate) fn esc(text: &str) -> String {
    escape(text, false)
}

/// Text escaped for an XML element's content: line breaks are kept (`\r\n` and `\r` as `\n`),
/// and characters XML 1.0 forbids are dropped.
pub(crate) fn esc_content(text: &str) -> String {
    escape(&text.replace("\r\n", "\n"), true)
}

fn escape(text: &str, content: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' | '\t' if content => out.push(c),
            '\r' if content => out.push('\n'),
            '\t' | '\n' | '\r' => out.push(' '),
            c if (c as u32) < 0x20 || matches!(c, '\u{FFFE}' | '\u{FFFF}') => {}
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_escaped_for_xml() {
        assert_eq!(esc("a < b & \"c\"\u{1}"), "a &lt; b &amp; &quot;c&quot;");
        assert_eq!(esc("a\nb"), "a b");
        assert_eq!(esc_content("a\r\nb\u{FFFF}\u{0}"), "a\nb");
    }
}
