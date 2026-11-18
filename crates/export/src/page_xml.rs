//! PAGE XML (PRImA's PAGE content schema, version 2019-07-15): the text of each page with where
//! every region, line and word sits, as Transkribus, eScriptorium and Aletheia read it. The
//! schema holds one page a file, so a document is written as one file per page, named
//! `{name}.p0001.page.xml` and on ([`ExportFormat::page_suffix`]).
//!
//! It is written from the same pages, pixels and corrected text as ALTO ([`crate::layout`]), and
//! says only what the stored regions hold:
//!
//! - each region the engine placed is a `TextRegion` with no `type`, which the engine does not
//!   give; its lines are `TextLine`s, and a line's words `Word`s where the engine placed each
//!   word (Tesseract);
//! - a block placed whole (Mistral OCR, LightOnOCR) is written with the lines its shape holds, as
//!   ALTO estimates them, each marked as estimated in its `comments`, and no words: the words'
//!   places would be guesses within a guess;
//! - positions are rectangles, written as four points in whole pixels within the page; there are
//!   no baselines, which no engine reports, and a confidence only where the engine gave a word
//!   one;
//! - `TextEquiv` at every level holds the corrected text: a word's own, a line's words joined by
//!   spaces, a region's lines joined by line breaks;
//! - ids are those the review gives the same regions (`r1`, `r1_l2`, `r1_l2_w3`, from
//!   [`kalamu_core::regions::block_id`] and its neighbours), counted across the whole page where a
//!   spread was split, where the review counts each half on its own;
//! - reading order is the order the engine gave, in `ReadingOrder` and, as Transkribus writes
//!   it, in each region's and line's `custom`;
//! - `Created` and `LastChange` are the times the page was read and last corrected, never the
//!   time of writing, so writing the files again gives the same files.

use std::fmt::Write as _;

use kalamu_core::regions::{block_id, line_id, word_id};
use kalamu_core::{
    ExportDocument, ExportError, ExportFile, ExportFormat, Exporter, InputKind, PieceRef, Task,
    TextLine, Timestamp,
};

use crate::layout::{LaidPage, Layout, error, esc, esc_content, written};

pub struct PageXmlExporter;

const FORMAT: &str = "PAGE XML";

const NAMESPACE: &str = "http://schema.primaresearch.org/PAGE/gts/pagecontent/2019-07-15";

impl Exporter for PageXmlExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::PageXml
    }

    fn supports(&self, task: Task) -> bool {
        task == Task::TextExtraction
    }

    /// The one page of a document of one page; a longer document is written as one file per
    /// page ([`Exporter::export_files`]).
    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let mut files = self.export_files(doc)?;
        match files.pop() {
            Some(file) if files.is_empty() => Ok(file.bytes),
            _ => Err(error(FORMAT)(
                &"a document of several pages is written as one file per page",
            )),
        }
    }

    fn export_files(&self, doc: &ExportDocument) -> Result<Vec<ExportFile>, ExportError> {
        let layout = Layout::of(doc, FORMAT)?;
        Ok(layout
            .pages
            .iter()
            .map(|laid| ExportFile {
                suffix: ExportFormat::PageXml.page_suffix(laid.number),
                bytes: page(doc, laid).into_bytes(),
            })
            .collect())
    }
}

/// One page's file.
fn page(doc: &ExportDocument, laid: &LaidPage) -> String {
    let number = laid.number;
    let parts: Vec<_> = doc
        .parts
        .iter()
        .filter(|p| matches!(p.piece, PieceRef::Page { number: n, .. } if n == number))
        .collect();
    // When the page was read, and last changed: read again, or corrected. A page that could not
    // be read takes the time the document's first page was read; only a document none of whose
    // results records its time (none the engine writes) falls back on the time of writing.
    let created = parts.iter().filter_map(|p| p.read_at).min();
    let changed = parts
        .iter()
        .flat_map(|p| [p.read_at, p.corrected_at])
        .flatten()
        .max();
    let created = created
        .or_else(|| doc.parts.iter().filter_map(|p| p.read_at).min())
        .unwrap_or(doc.provenance.created_at);
    let changed = changed.unwrap_or(created).max(created);

    let mut comments = format!(
        "{} {}",
        doc.provenance.provider.as_str(),
        doc.provenance.model
    );
    if let Some(dpi) = laid.dpi {
        let _ = write!(comments, "; PDF page measured at {} dpi", written(dpi));
    }
    if laid.regions.iter().any(|r| r.lines.is_empty()) {
        comments.push_str("; line positions estimated within blocks the model placed whole");
    }
    if parts.iter().any(|p| p.corrected) {
        comments.push_str("; text corrected by the researcher");
    }
    if let Some(reason) = parts.iter().find_map(|p| p.failed.as_deref()) {
        let _ = write!(comments, "; not read: {reason}");
    }

    let mut x = String::new();
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        x,
        "<PcGts xmlns=\"{NAMESPACE}\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" \
xsi:schemaLocation=\"{NAMESPACE} {NAMESPACE}/pagecontent.xsd\">"
    );
    let _ = writeln!(
        x,
        "  <Metadata>\n    <Creator>{} {}</Creator>\n    <Created>{}</Created>\n    \
<LastChange>{}</LastChange>\n    <Comments>{}</Comments>\n  </Metadata>",
        esc_content(&doc.provenance.generator.name),
        esc_content(&doc.provenance.generator.version),
        date_time(created),
        date_time(changed),
        esc_content(&comments),
    );
    let resolution = laid
        .dpi
        .map(|dpi| {
            let dpi = written(dpi);
            format!(
                " imageXResolution=\"{dpi}\" imageYResolution=\"{dpi}\" imageResolutionUnit=\"PPI\""
            )
        })
        .unwrap_or_default();
    let _ = writeln!(
        x,
        "  <Page imageFilename=\"{}\" imageWidth=\"{}\" imageHeight=\"{}\"{resolution}>",
        esc(&image_name(doc, number)),
        laid.width,
        laid.height,
    );
    if !laid.regions.is_empty() {
        x.push_str("    <ReadingOrder>\n      <OrderedGroup id=\"ro\">\n");
        for r in 0..laid.regions.len() {
            let _ = writeln!(
                x,
                "        <RegionRefIndexed index=\"{r}\" regionRef=\"{}\"/>",
                block_id(r + 1)
            );
        }
        x.push_str("      </OrderedGroup>\n    </ReadingOrder>\n");
    }
    let points = |rx: f32, ry: f32, rw: f32, rh: f32| coords(rx, ry, rw, rh, laid);
    for (r, region) in laid.regions.iter().enumerate() {
        let id = block_id(r + 1);
        let _ = writeln!(
            x,
            "    <TextRegion id=\"{id}\" custom=\"readingOrder {{index:{r};}}\">\n      \
<Coords points=\"{}\"/>",
            points(region.x, region.y, region.width, region.height)
        );
        let (lines, estimated) = laid.lines(region);
        let mut texts = Vec::with_capacity(lines.len());
        for (l, line) in lines.iter().enumerate() {
            let lid = line_id(r + 1, l + 1);
            let note = if estimated {
                " comments=\"position estimated within a block the model placed whole\""
            } else {
                ""
            };
            let _ = writeln!(
                x,
                "      <TextLine id=\"{lid}\" custom=\"readingOrder {{index:{l};}}\"{note}>\n        \
<Coords points=\"{}\"/>",
                points(line.x, line.y, line.width, line.height)
            );
            if !estimated {
                for (w, word) in line.words.iter().enumerate() {
                    let conf = word
                        .confidence
                        .map(|c| format!(" conf=\"{:.2}\"", c.clamp(0.0, 1.0)))
                        .unwrap_or_default();
                    let _ = writeln!(
                        x,
                        "        <Word id=\"{}\">\n          <Coords points=\"{}\"/>\n          \
<TextEquiv{conf}>\n            <Unicode>{}</Unicode>\n          </TextEquiv>\n        </Word>",
                        word_id(r + 1, l + 1, w + 1),
                        points(word.x, word.y, word.width, word.height),
                        esc_content(&word.text),
                    );
                }
            }
            let text = line_text(line);
            let _ = writeln!(
                x,
                "        <TextEquiv>\n          <Unicode>{}</Unicode>\n        </TextEquiv>\n      \
</TextLine>",
                esc_content(&text)
            );
            texts.push(text);
        }
        let text = if lines.is_empty() {
            region.text.trim().to_owned()
        } else {
            texts.join("\n")
        };
        let _ = writeln!(
            x,
            "      <TextEquiv>\n        <Unicode>{}</Unicode>\n      </TextEquiv>\n    </TextRegion>",
            esc_content(&text)
        );
    }
    x.push_str("  </Page>\n</PcGts>\n");
    x
}

/// The image the page's positions refer to: the image itself, or the file of a document of
/// images the page is in; for a PDF, the PDF and the page drawn from it (`letter.pdf#page=3`, as
/// a link to a PDF's page is written).
fn image_name(doc: &ExportDocument, number: u32) -> String {
    match doc.source.kind {
        InputKind::Pdf => format!("{}#page={number}", doc.source.file_name),
        _ => doc.source.page_file(number).map_or_else(
            || doc.source.file_name.clone(),
            |(p, _)| p.file_name.clone(),
        ),
    }
}

/// A line's text: its words joined by spaces.
fn line_text(line: &TextLine) -> String {
    let words: Vec<&str> = line
        .words
        .iter()
        .map(|w| w.text.trim())
        .filter(|w| !w.is_empty())
        .collect();
    words.join(" ")
}

/// A box given in fractions of the page as its four corners in whole pixels, clockwise from the
/// top left, each within the page.
fn coords(x: f32, y: f32, w: f32, h: f32, laid: &LaidPage) -> String {
    let (pw, ph) = (laid.width, laid.height);
    let px = |v: f32, size: f32| (v * size).round().clamp(0.0, (size - 1.0).max(0.0)) as u32;
    let (x0, x1) = (px(x, pw), px(x + w, pw));
    let (y0, y1) = (px(y, ph), px(y + h, ph));
    format!("{x0},{y0} {x1},{y0} {x1},{y1} {x0},{y1}")
}

/// A time as the schema's `dateTime`, in UTC.
fn date_time(at: Timestamp) -> String {
    at.utc_date_time()
}
