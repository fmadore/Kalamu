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
//! - `Created` and `LastChange` are the times the page was read and last corrected, or a box last
//!   placed by the researcher, never the time of writing, so writing the files again gives the
//!   same files.
//!
//! The researcher's boxes (#84) are said where the estimated ones are, in the `comments` the
//! schema gives every region, line and word: a box the researcher placed, or the model's widened
//! to hold one of theirs, names the region of theirs it comes from as `provenance.json` lists it.
//! The page's `Metadata` adds a `MetadataItem` of type `processingStep`, the 2019 schema's place
//! for a step of the page's making (where OCR-D records its processors). The model's regions keep
//! their ids; the researcher's own come after them in the reading order, each a `TextRegion`
//! named `researcher_{id}` (an XML id cannot start with a digit, as theirs may): with lines
//! estimated within its box where it holds text, and with its box alone where it holds none, a
//! statement that something is there.

use std::fmt::Write as _;

use kalamu_core::export::BoxKind;
use kalamu_core::regions::{block_id, line_id, word_id};
use kalamu_core::{
    ExportDocument, ExportError, ExportFile, ExportFormat, Exporter, InputKind, PieceRef, Task,
    TextLine, Timestamp,
};

use crate::geometry::{Mark, researcher_id};
use crate::layout::{LaidPage, Layout, error, esc, esc_content, written};

pub struct PageXmlExporter;

const FORMAT: &str = "PAGE XML";

const NAMESPACE: &str = "http://schema.primaresearch.org/PAGE/gts/pagecontent/2019-07-15";

/// The note on a line estimated within a block the model placed whole.
const ESTIMATED: &str = "position estimated within a block the model placed whole";

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
    // When the page was read, and last changed: read again, corrected, or a box placed. A page
    // that could not be read takes the time the document's first page was read; only a
    // document none of whose results records its time (none the engine writes) falls back on
    // the time of writing.
    let created = parts.iter().filter_map(|p| p.read_at).min();
    let changed = parts
        .iter()
        .flat_map(|p| [p.read_at, p.corrected_at])
        .chain([laid.changed])
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
    if laid.estimated() {
        comments.push_str("; line positions estimated within blocks the model placed whole");
    }
    if parts.iter().any(|p| p.corrected) {
        comments.push_str("; text corrected by the researcher");
    }
    let researcher = laid.researcher();
    if researcher {
        comments.push_str("; positions set by the researcher where an element's comments say so");
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
    let _ = write!(
        x,
        "  <Metadata>\n    <Creator>{} {}</Creator>\n    <Created>{}</Created>\n    \
<LastChange>{}</LastChange>\n    <Comments>{}</Comments>\n",
        esc_content(&doc.provenance.generator.name),
        esc_content(&doc.provenance.generator.version),
        date_time(created),
        date_time(changed),
        esc_content(&comments),
    );
    if let (true, Some(at)) = (researcher, laid.changed) {
        let _ = writeln!(
            x,
            "    <MetadataItem type=\"processingStep\" name=\"review\" value=\"positions set and \
regions placed by the researcher in {}\" date=\"{}\"/>",
            esc(&doc.provenance.generator.name),
            date_time(at)
        );
    }
    x.push_str("  </Metadata>\n");
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
    // The model's regions in their order, then the researcher's own in the order made.
    let ids: Vec<String> = (1..=laid.regions.len())
        .map(block_id)
        .chain(laid.drawn.iter().map(|d| researcher_id(&d.id)))
        .collect();
    if !ids.is_empty() {
        x.push_str("    <ReadingOrder>\n      <OrderedGroup id=\"ro\">\n");
        for (r, id) in ids.iter().enumerate() {
            let _ = writeln!(
                x,
                "        <RegionRefIndexed index=\"{r}\" regionRef=\"{id}\"/>"
            );
        }
        x.push_str("      </OrderedGroup>\n    </ReadingOrder>\n");
    }
    for r in 0..laid.regions.len() {
        model_region(&mut x, laid, r);
    }
    for (d, drawn) in laid.drawn.iter().enumerate() {
        let index = laid.regions.len() + d;
        let id = researcher_id(&drawn.id);
        let region = &drawn.region;
        let _ = writeln!(
            x,
            "    <TextRegion id=\"{id}\" custom=\"readingOrder {{index:{index};}}\" \
comments=\"region placed by the researcher (region {})\">\n      <Coords points=\"{}\"/>",
            esc(&drawn.id),
            coords(region.x, region.y, region.width, region.height, laid)
        );
        if !region.text.trim().is_empty() {
            let lines = laid.estimate(region);
            let mut texts = Vec::with_capacity(lines.len());
            for (l, line) in lines.iter().enumerate() {
                let note = "position estimated within a region the researcher drew";
                texts.push(text_line(
                    &mut x,
                    laid,
                    &format!("{id}_l{}", l + 1),
                    l,
                    line,
                    note,
                ));
            }
            let text = if lines.is_empty() {
                region.text.trim().to_owned()
            } else {
                texts.join("\n")
            };
            text_equiv(&mut x, "      ", &text);
        }
        x.push_str("    </TextRegion>\n");
    }
    x.push_str("  </Page>\n</PcGts>\n");
    x
}

/// Block `r` of the model's, from 0, with its lines and words, each with its note.
fn model_region(x: &mut String, laid: &LaidPage, r: usize) {
    let Some(region) = laid.regions.get(r) else {
        return;
    };
    let marks = laid.marks(r);
    let id = block_id(r + 1);
    let note = marks
        .and_then(|m| m.block.as_ref())
        .map(|m| format!(" comments=\"{}\"", esc(&m.note())))
        .unwrap_or_default();
    let _ = writeln!(
        x,
        "    <TextRegion id=\"{id}\" custom=\"readingOrder {{index:{r};}}\"{note}>\n      \
<Coords points=\"{}\"/>",
        coords(region.x, region.y, region.width, region.height, laid)
    );
    let (lines, estimated) = laid.lines(r);
    let mut texts = Vec::with_capacity(lines.len());
    for (l, line) in lines.iter().enumerate() {
        let mark = marks.and_then(|m| m.lines.get(&l));
        let word_mark = |w: usize| marks.and_then(|m| m.words.get(&(l, w)));
        let note = line_note(mark, estimated);
        let lid = line_id(r + 1, l + 1);
        let _ = writeln!(
            x,
            "      <TextLine id=\"{lid}\" custom=\"readingOrder {{index:{l};}}\"{}>\n        \
<Coords points=\"{}\"/>",
            note.map(|n| format!(" comments=\"{}\"", esc(&n)))
                .unwrap_or_default(),
            coords(line.x, line.y, line.width, line.height, laid)
        );
        // Within estimated lines, words only where the researcher placed one of them.
        let words = !estimated || (0..line.words.len()).any(|w| word_mark(w).is_some());
        if words {
            for (w, word) in line.words.iter().enumerate() {
                let conf = word
                    .confidence
                    .map(|c| format!(" conf=\"{:.2}\"", c.clamp(0.0, 1.0)))
                    .unwrap_or_default();
                let note = match word_mark(w) {
                    Some(m) => Some(m.note()),
                    None => estimated.then(|| ESTIMATED.to_owned()),
                };
                let _ = writeln!(
                    x,
                    "        <Word id=\"{}\"{}>\n          <Coords points=\"{}\"/>\n          \
<TextEquiv{conf}>\n            <Unicode>{}</Unicode>\n          </TextEquiv>\n        </Word>",
                    word_id(r + 1, l + 1, w + 1),
                    note.map(|n| format!(" comments=\"{}\"", esc(&n)))
                        .unwrap_or_default(),
                    coords(word.x, word.y, word.width, word.height, laid),
                    esc_content(&word.text),
                );
            }
        }
        let text = line_text(line);
        text_equiv(x, "        ", &text);
        x.push_str("      </TextLine>\n");
        texts.push(text);
    }
    let text = if lines.is_empty() {
        region.text.trim().to_owned()
    } else {
        texts.join("\n")
    };
    text_equiv(x, "      ", &text);
    x.push_str("    </TextRegion>\n");
}

/// A line's note: the researcher's box is theirs, estimated or not; the model's line widened
/// for a word of theirs is still estimated where it was.
fn line_note(mark: Option<&Mark>, estimated: bool) -> Option<String> {
    match (mark, estimated) {
        (Some(m), true) if m.kind == BoxKind::ModelWidened => {
            Some(format!("{ESTIMATED}; {}", m.note()))
        }
        (Some(m), _) => Some(m.note()),
        (None, true) => Some(ESTIMATED.to_owned()),
        (None, false) => None,
    }
}

/// A line estimated within a region of the researcher's, with no words; its text.
fn text_line(
    x: &mut String,
    laid: &LaidPage,
    id: &str,
    index: usize,
    line: &TextLine,
    note: &str,
) -> String {
    let _ = writeln!(
        x,
        "      <TextLine id=\"{id}\" custom=\"readingOrder {{index:{index};}}\" comments=\"{note}\">\n        \
<Coords points=\"{}\"/>",
        coords(line.x, line.y, line.width, line.height, laid)
    );
    let text = line_text(line);
    text_equiv(x, "        ", &text);
    x.push_str("      </TextLine>\n");
    text
}

/// A `TextEquiv` holding `text`, indented by `indent`.
fn text_equiv(x: &mut String, indent: &str, text: &str) {
    let _ = writeln!(
        x,
        "{indent}<TextEquiv>\n{indent}  <Unicode>{}</Unicode>\n{indent}</TextEquiv>",
        esc_content(text)
    );
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
