//! ALTO XML (version 4): the text of each page with where every block, line and word sits, as
//! digital libraries, eScriptorium and IWAC's pipelines read it.
//!
//! Only models that report where text sits can feed it. Tesseract places each word, which is
//! written as it came, corrections laid back onto the words' places by the engine. Mistral OCR
//! places paragraphs only: their lines share the block's height and their words its width,
//! and the file says those positions are estimated.
//!
//! Positions are in pixels: those of the image for an image source, and of the page drawn at
//! 300 dpi for a PDF, as Tesseract reads it. A page split into halves is put back together, as
//! in the searchable PDF.

use std::fmt::Write as _;

use kalamu_core::{
    ExportDocument, ExportError, ExportFormat, Exporter, InputKind, Task, TextLine, TextRegion,
    TextWord,
};

use crate::searchable_pdf::{pdf_page_sizes, regions_by_page};

pub struct AltoExporter;

/// PDF pages are measured as drawn at this resolution.
const PDF_DPI: f32 = 300.0;

fn error(message: impl std::fmt::Display) -> ExportError {
    ExportError::Write {
        format: "ALTO",
        message: message.to_string(),
    }
}

impl Exporter for AltoExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Alto
    }

    fn supports(&self, task: Task) -> bool {
        task == Task::TextExtraction
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let pages = regions_by_page(doc);
        if pages.is_empty() {
            return Err(error(
                "this model reported no text positions; choose Tesseract or Mistral OCR for ALTO",
            ));
        }
        let sizes: Vec<(u32, (f32, f32))> = match doc.source.kind {
            InputKind::Pdf => {
                let bytes = std::fs::read(&doc.source.path).map_err(error)?;
                pdf_page_sizes(&bytes)?
                    .into_iter()
                    .map(|(n, (w, h))| {
                        let px = |points: f32| (points / 72.0 * PDF_DPI).round();
                        (n, (px(w), px(h)))
                    })
                    .collect()
            }
            InputKind::Image => {
                let (w, h) = image_size(&doc.source.path)?;
                pages.keys().map(|&n| (n, (w, h))).collect()
            }
            _ => return Err(ExportError::Unsupported("ALTO (from a PDF or an image)")),
        };
        let estimated = pages.values().flatten().any(|r| r.lines.is_empty());

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
            settings.push_str("; PDF pages measured at 300 dpi");
        }
        if estimated {
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
            iso_8601(doc.provenance.created_at.0),
            esc(&settings),
            esc(&doc.provenance.generator.name),
            esc(&doc.provenance.generator.version),
        );
        x.push_str("  </Description>\n  <Layout>\n");
        // The pages read, a page that held no text included; not those a sample left out.
        let read: std::collections::BTreeSet<u32> = doc
            .parts
            .iter()
            .filter_map(|p| match p.piece {
                kalamu_core::PieceRef::Page { number, .. } => Some(number),
                _ => None,
            })
            .collect();
        for (number, (width, height)) in sizes.into_iter().filter(|(n, _)| read.contains(n)) {
            let regions = pages.get(&number).map(Vec::as_slice).unwrap_or_default();
            page(&mut x, number, width, height, regions);
        }
        x.push_str("  </Layout>\n</alto>\n");
        Ok(x.into_bytes())
    }
}

/// An image's width and height as shown, after its EXIF orientation.
fn image_size(path: &str) -> Result<(f32, f32), ExportError> {
    use image::ImageDecoder;
    let mut decoder = image::ImageReader::open(path)
        .and_then(image::ImageReader::with_guessed_format)
        .map_err(error)?
        .into_decoder()
        .map_err(error)?;
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

fn page(x: &mut String, number: u32, pw: f32, ph: f32, regions: &[TextRegion]) {
    let _ = writeln!(
        x,
        "    <Page ID=\"page_{number}\" PHYSICAL_IMG_NR=\"{number}\" WIDTH=\"{}\" HEIGHT=\"{}\">",
        pw.round(),
        ph.round()
    );
    let _ = writeln!(x, "      <PrintSpace {}>", pos(0.0, 0.0, 1.0, 1.0, pw, ph));
    for (b, region) in regions.iter().enumerate() {
        let id = format!("p{number}_b{}", b + 1);
        let _ = writeln!(
            x,
            "        <TextBlock ID=\"{id}\" {}>",
            pos(region.x, region.y, region.width, region.height, pw, ph)
        );
        let lines = if region.lines.is_empty() {
            estimated_lines(region)
        } else {
            region.lines.clone()
        };
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

/// A block placed whole, as lines sharing its height and words sharing each line's width by
/// their length.
fn estimated_lines(region: &TextRegion) -> Vec<TextLine> {
    let texts: Vec<&str> = region
        .text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let height = region.height / texts.len().max(1) as f32;
    texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let y = region.y + i as f32 * height;
            let words: Vec<&str> = text.split_whitespace().collect();
            let total: usize = words.iter().map(|w| w.chars().count() + 1).sum::<usize>() - 1;
            let unit = region.width / total.max(1) as f32;
            let mut offset = 0.0;
            let words = words
                .iter()
                .map(|w| {
                    let width = w.chars().count() as f32 * unit;
                    let word = TextWord {
                        x: region.x + offset,
                        y,
                        width,
                        height,
                        text: (*w).to_owned(),
                        confidence: None,
                    };
                    offset += width + unit;
                    word
                })
                .collect();
            TextLine {
                x: region.x,
                y,
                width: region.width,
                height,
                words,
            }
        })
        .collect()
}

/// Milliseconds since 1970 as an `xsd:dateTime` in UTC (`2026-11-04T09:30:00Z`).
fn iso_8601(millis: i64) -> String {
    let secs = millis.div_euclid(1000);
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's days-to-civil algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    )
}

/// Text escaped for an XML attribute or element; characters XML 1.0 forbids are dropped.
fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
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
    fn a_block_placed_whole_gets_estimated_lines_and_words() {
        let region = TextRegion::block(0.1, 0.2, 0.4, 0.1, "Dakar, le\n3 mai");
        let lines = estimated_lines(&region);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].y, 0.25);
        assert_eq!(lines[0].words.len(), 2);
        assert_eq!(lines[0].words[0].x, 0.1);
        let last = lines[0].words.last().map(|w| w.x + w.width).unwrap_or(0.0);
        assert!((last - 0.5).abs() < 1e-5, "the words fill the line");
    }

    #[test]
    fn times_are_written_as_xml_dates() {
        assert_eq!(iso_8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_8601(1_793_785_800_000), "2026-11-04T09:50:00Z");
    }

    #[test]
    fn text_is_escaped_for_xml() {
        assert_eq!(esc("a < b & \"c\"\u{1}"), "a &lt; b &amp; &quot;c&quot;");
    }
}
