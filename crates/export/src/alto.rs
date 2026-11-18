//! ALTO XML (version 4): the text of each page with where every block, line and word sits, as
//! digital libraries, eScriptorium and IWAC's pipelines read it.
//!
//! Only models that report where text sits can feed it. Tesseract places each word, which is
//! written as it came, corrections laid back onto the words' places by the engine. Mistral OCR
//! places paragraphs only: their text is wrapped into the lines the block's size holds, which
//! share its height, and their words share each line's width
//! ([`kalamu_core::TextRegion::estimated_lines`]); the file says those positions are estimated.
//! The pages, their pixels and the corrected view are shared with PAGE XML ([`crate::layout`]).

use std::fmt::Write as _;

use kalamu_core::{ExportDocument, ExportError, ExportFormat, Exporter, InputKind, Task};

use crate::layout::{LaidPage, Layout, esc};

pub struct AltoExporter;

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
        x.push_str("  </Description>\n  <Layout>\n");
        for laid in &layout.pages {
            page(&mut x, laid);
        }
        x.push_str("  </Layout>\n</alto>\n");
        Ok(x.into_bytes())
    }
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
        let id = format!("p{number}_b{}", b + 1);
        let _ = writeln!(
            x,
            "        <TextBlock ID=\"{id}\" {}>",
            pos(region.x, region.y, region.width, region.height, pw, ph)
        );
        // The region is on the whole page, measured `pw` by `ph`: the same block, in pixels, as
        // the review estimates within (a split spread's half there), so both draw the same lines.
        let (lines, _) = laid.lines(region);
        for (l, line) in lines.iter().enumerate() {
            let lid = format!("{id}_l{}", l + 1);
            let _ = writeln!(
                x,
                "          <TextLine ID=\"{lid}\" {}>",
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
                    "            <String ID=\"{lid}_w{}\" CONTENT=\"{}\" {}{confidence}/>",
                    w + 1,
                    esc(&word.text),
                    pos(word.x, word.y, word.width, word.height, pw, ph)
                );
            }
            x.push_str("          </TextLine>\n");
        }
        x.push_str("        </TextBlock>\n");
    }
    x.push_str("      </PrintSpace>\n    </Page>\n");
}
