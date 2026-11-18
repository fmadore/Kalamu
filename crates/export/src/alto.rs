//! ALTO XML (version 4): the text of each page with where every block, line and word sits, as
//! digital libraries, eScriptorium and IWAC's pipelines read it.
//!
//! Only models that report where text sits can feed it. Tesseract places each word, which is
//! written as it came, corrections laid back onto the words' places by the engine. Mistral OCR
//! places paragraphs only: their text is wrapped into the lines the block's size holds, which
//! share its height, and their words share each line's width
//! ([`kalamu_core::TextRegion::estimated_lines`]); the file says those positions are estimated.
//! The pages, their pixels and the corrected view are shared with PAGE XML ([`crate::layout`]).
//!
//! Where the researcher placed boxes (#84), the file describes a second processing step,
//! `processing_2`, a `contentModification` made in Kalamu's review, and every `TextBlock`,
//! `TextLine` and `String` whose box is the researcher's, or the model's widened to hold one of
//! theirs, refers to it in `PROCESSINGREFS`, which ALTO gives those elements for referring to the
//! processing history since 4.1. The researcher's own regions that hold text are blocks after the
//! model's on their page (`researcher_{id}`), their lines and words estimated within the box, as
//! the step's description says; one without text is left out, since ALTO writes text. A file
//! with nothing of the researcher's is written as before.

use std::fmt::Write as _;

use kalamu_core::export::BoxKind;
use kalamu_core::{
    ExportDocument, ExportError, ExportFormat, Exporter, InputKind, Task, TextLine, TextRegion,
};

use crate::geometry::{Mark, researcher_id};
use crate::layout::{LaidPage, Layout, esc};

pub struct AltoExporter;

/// The processing step the researcher's boxes refer to.
const RESEARCHER_STEP: &str = "processing_2";

impl Exporter for AltoExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Alto
    }

    fn supports(&self, task: Task) -> bool {
        task == Task::TextExtraction
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let layout = Layout::of(doc, "ALTO")?;

        let mut x = String::new();
        x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        x.push_str(
            "<alto xmlns=\"http://www.loc.gov/standards/alto/ns-v4#\" \
xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" \
xsi:schemaLocation=\"http://www.loc.gov/standards/alto/ns-v4# \
http://www.loc.gov/alto/v4/alto-4-4.xsd\">\n",
        );
        x.push_str("  <Description>\n    <MeasurementUnit>pixel</MeasurementUnit>\n");
        let _ = writeln!(
            x,
            "    <sourceImageInformation>\n      <fileName>{}</fileName>\n    </sourceImageInformation>",
            esc(&doc.source.file_name)
        );
        let mut settings = format!(
            "{} {}",
            doc.provenance.provider.as_str(),
            doc.provenance.model
        );
        if doc.source.kind == InputKind::Pdf {
            settings.push_str(&layout.measured());
        }
        if layout.estimated() {
            settings.push_str(
                "; line and word positions estimated within blocks the model placed whole",
            );
        }
        let _ = writeln!(
            x,
            "    <Processing ID=\"processing_1\">\n      \
<processingDateTime>{}</processingDateTime>\n      \
<processingStepSettings>{}</processingStepSettings>\n      \
<processingSoftware>\n        <softwareName>{}</softwareName>\n        \
<softwareVersion>{}</softwareVersion>\n      </processingSoftware>\n    </Processing>",
            doc.provenance.created_at.utc_date_time(),
            esc(&settings),
            esc(&doc.provenance.generator.name),
            esc(&doc.provenance.generator.version),
        );
        if let Some(step) = researcher_step(doc, &layout) {
            x.push_str(&step);
        }
        x.push_str("  </Description>\n  <Layout>\n");
        for laid in &layout.pages {
            page(&mut x, laid);
        }
        x.push_str("  </Layout>\n</alto>\n");
        Ok(x.into_bytes())
    }
}

/// The processing step of the researcher's boxes, when the file holds any: what it did, and
/// when they last placed a box.
fn researcher_step(doc: &ExportDocument, layout: &Layout) -> Option<String> {
    let changed = layout.changed()?;
    let marks = || layout.pages.iter().flat_map(|p| &p.marks);
    let adjusted = marks().any(|m| m.block.is_some() || !m.lines.is_empty() || !m.words.is_empty());
    let widened = marks()
        .flat_map(|m| m.block.iter().chain(m.lines.values()))
        .any(|m| m.kind != BoxKind::Researcher);
    let drawn = layout
        .pages
        .iter()
        .flat_map(|p| &p.drawn)
        .any(|d| !d.region.text.trim().is_empty());
    let mut described = Vec::new();
    if adjusted {
        described.push(
            "positions set by the researcher in the review, where an element refers to this step",
        );
    }
    if widened {
        described
            .push("blocks and lines widened just enough to hold the lines and words within them");
    }
    if drawn {
        described.push(
            "regions placed by the researcher, after the model's on their page, their line and \
             word positions estimated within the region",
        );
    }
    if described.is_empty() {
        return None;
    }
    let mut x = format!(
        "    <Processing ID=\"{RESEARCHER_STEP}\">\n      \
<processingCategory>contentModification</processingCategory>\n      \
<processingDateTime>{}</processingDateTime>\n",
        changed.utc_date_time()
    );
    for line in described {
        let _ = writeln!(
            x,
            "      <processingStepDescription>{}</processingStepDescription>",
            esc(line)
        );
    }
    let _ = writeln!(
        x,
        "      <processingSoftware>\n        <softwareName>{}</softwareName>\n        \
<softwareVersion>{}</softwareVersion>\n      </processingSoftware>\n    </Processing>",
        esc(&doc.provenance.generator.name),
        esc(&doc.provenance.generator.version),
    );
    Some(x)
}

/// ` PROCESSINGREFS="processing_2"` for a box the researcher had a hand in, else nothing.
fn refers(mark: Option<&Mark>) -> String {
    mark.map(|_| format!(" PROCESSINGREFS=\"{RESEARCHER_STEP}\""))
        .unwrap_or_default()
}

/// A box in pixels: HPOS, VPOS, WIDTH and HEIGHT.
fn pos(x: f32, y: f32, w: f32, h: f32, pw: f32, ph: f32) -> String {
    format!(
        "HPOS=\"{}\" VPOS=\"{}\" WIDTH=\"{}\" HEIGHT=\"{}\"",
        (x * pw).round().max(0.0),
        (y * ph).round().max(0.0),
        (w * pw).round().max(0.0),
        (h * ph).round().max(0.0)
    )
}

fn page(x: &mut String, laid: &LaidPage) {
    let (number, pw, ph) = (laid.number, laid.width, laid.height);
    let _ = writeln!(
        x,
        "    <Page ID=\"page_{number}\" PHYSICAL_IMG_NR=\"{number}\" WIDTH=\"{pw}\" HEIGHT=\"{ph}\">",
    );
    let _ = writeln!(x, "      <PrintSpace {}>", pos(0.0, 0.0, 1.0, 1.0, pw, ph));
    for (b, region) in laid.regions.iter().enumerate() {
        let marks = laid.marks(b);
        // The region is on the whole page, measured `pw` by `ph`: the same block, in pixels, as
        // the review estimates within (a split spread's half there), so both draw the same lines.
        let (lines, _) = laid.lines(b);
        let block = Block {
            id: format!("p{number}_b{}", b + 1),
            region,
            lines: &lines,
            mark: marks.and_then(|m| m.block.as_ref()),
        };
        block.write(x, laid, |l, w| {
            let marks = marks?;
            match w {
                None => marks.lines.get(&l),
                Some(w) => marks.words.get(&(l, w)),
            }
        });
    }
    // The researcher's own regions with text, after the model's, their lines estimated.
    let theirs = Mark {
        id: String::new(),
        kind: BoxKind::Researcher,
    };
    for drawn in laid
        .drawn
        .iter()
        .filter(|d| !d.region.text.trim().is_empty())
    {
        let lines = laid.estimate(&drawn.region);
        let block = Block {
            id: researcher_id(&drawn.id),
            region: &drawn.region,
            lines: &lines,
            mark: Some(&theirs),
        };
        block.write(x, laid, |_, _| None);
    }
    x.push_str("      </PrintSpace>\n    </Page>\n");
}

/// A `TextBlock` to write: its id, box, the lines it is written as, and its mark.
struct Block<'a> {
    id: String,
    region: &'a TextRegion,
    lines: &'a [TextLine],
    mark: Option<&'a Mark>,
}

impl Block<'_> {
    /// The block with its lines and words, each referring to the researcher's step where its
    /// mark (`marked(line, word)`, from 0) says so.
    fn write<'m>(
        &self,
        x: &mut String,
        laid: &LaidPage,
        marked: impl Fn(usize, Option<usize>) -> Option<&'m Mark>,
    ) {
        let (pw, ph, id, r) = (laid.width, laid.height, &self.id, self.region);
        let _ = writeln!(
            x,
            "        <TextBlock ID=\"{id}\"{} {}>",
            refers(self.mark),
            pos(r.x, r.y, r.width, r.height, pw, ph)
        );
        for (l, line) in self.lines.iter().enumerate() {
            let lid = format!("{id}_l{}", l + 1);
            let _ = writeln!(
                x,
                "          <TextLine ID=\"{lid}\"{} {}>",
                refers(marked(l, None)),
                pos(line.x, line.y, line.width, line.height, pw, ph)
            );
            for (w, word) in line.words.iter().enumerate() {
                if w > 0 {
                    x.push_str("            <SP/>\n");
                }
                let confidence = word
                    .confidence
                    .map(|c| format!(" WC=\"{c:.2}\""))
                    .unwrap_or_default();
                let _ = writeln!(
                    x,
                    "            <String ID=\"{lid}_w{}\" CONTENT=\"{}\"{} {}{confidence}/>",
                    w + 1,
                    esc(&word.text),
                    refers(marked(l, Some(w))),
                    pos(word.x, word.y, word.width, word.height, pw, ph)
                );
            }
            x.push_str("          </TextLine>\n");
        }
        x.push_str("        </TextBlock>\n");
    }
}
