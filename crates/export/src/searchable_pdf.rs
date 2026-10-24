//! The original PDF with an invisible text layer, so it can be searched and its text copied.
//!
//! Only models that report where text sits (Mistral OCR, at paragraph level) can feed it; the
//! README asks for this format from them only. Each text block is set in invisible text
//! (render mode 3) over its box: the block's words are spread over as many lines as fit its
//! shape, each line stretched to the box's width. The font is a glyph-less Type0 font, as in
//! Tesseract's PDF output: two-byte codes equal to Unicode code points, a ToUnicode map that
//! says so, and a fixed advance, so any script stays searchable and line widths are exact. The
//! source is read from disk; nothing else in it changes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use lopdf::{Dictionary, Document, Object, ObjectId, Stream, dictionary};
use scriptorium_core::{
    ExportDocument, ExportError, ExportFormat, Exporter, InputKind, PageHalf, PieceRef, Task,
    TextRegion,
};

pub struct SearchablePdfExporter;

const FONT: &[u8] = b"ScriptoriumText";
/// The fixed advance of every code, in thousandths of the font size.
const ADVANCE: f32 = 0.5;
const INHERITED: &[&[u8]] = &[b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

fn error(message: impl std::fmt::Display) -> ExportError {
    ExportError::Write {
        format: "searchable PDF",
        message: message.to_string(),
    }
}

impl Exporter for SearchablePdfExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::SearchablePdf
    }

    fn supports(&self, task: Task) -> bool {
        task == Task::TextExtraction
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        if doc.source.kind != InputKind::Pdf {
            return Err(ExportError::Unsupported(
                "Searchable PDF (from a PDF source)",
            ));
        }
        let pages = regions_by_page(doc);
        if pages.is_empty() {
            return Err(error(
                "this model reported no text positions; choose Mistral OCR for a searchable PDF",
            ));
        }
        let bytes = std::fs::read(&doc.source.path).map_err(error)?;
        let mut pdf = Document::load_mem(&bytes).map_err(error)?;
        let font = add_font(&mut pdf);
        let ids = pdf.get_pages();
        for (number, regions) in pages {
            let Some(&page_id) = ids.get(&number) else {
                continue;
            };
            add_text_layer(&mut pdf, page_id, &regions, font)?;
        }
        let mut out = Vec::new();
        pdf.save_to(&mut out).map_err(error)?;
        Ok(out)
    }
}

/// Each page's regions, with halves of split spreads mapped back onto the whole page.
fn regions_by_page(doc: &ExportDocument) -> BTreeMap<u32, Vec<TextRegion>> {
    let mut pages: BTreeMap<u32, Vec<TextRegion>> = BTreeMap::new();
    for part in &doc.parts {
        let PieceRef::Page { number, half } = &part.piece else {
            continue;
        };
        for region in &part.regions {
            let mut r = region.clone();
            match half {
                Some(PageHalf::Left) => {
                    r.x *= 0.5;
                    r.width *= 0.5;
                }
                Some(PageHalf::Right) => {
                    r.x = 0.5 + r.x * 0.5;
                    r.width *= 0.5;
                }
                None => {}
            }
            pages.entry(*number).or_default().push(r);
        }
    }
    pages
}

/// The glyph-less Unicode font, added once and shared by every page.
fn add_font(pdf: &mut Document) -> ObjectId {
    let cmap = b"/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n\
1 beginbfrange\n<0000> <FFFF> <0000>\nendbfrange\n\
endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n";
    let to_unicode = pdf.add_object(Stream::new(dictionary! {}, cmap.to_vec()));
    let descriptor = pdf.add_object(dictionary! {
        "Type" => "FontDescriptor",
        "FontName" => Object::Name(FONT.to_vec()),
        "Flags" => 5,
        "FontBBox" => vec![0.into(), 0.into(), 500.into(), 1000.into()],
        "ItalicAngle" => 0,
        "Ascent" => 1000,
        "Descent" => 0,
        "CapHeight" => 1000,
        "StemV" => 80,
    });
    let cid_font = pdf.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "CIDFontType2",
        "BaseFont" => Object::Name(FONT.to_vec()),
        "CIDSystemInfo" => dictionary! {
            "Registry" => Object::string_literal("Adobe"),
            "Ordering" => Object::string_literal("Identity"),
            "Supplement" => 0,
        },
        "FontDescriptor" => descriptor,
        "DW" => (ADVANCE * 1000.0) as i64,
        "CIDToGIDMap" => "Identity",
    });
    pdf.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type0",
        "BaseFont" => Object::Name(FONT.to_vec()),
        "Encoding" => "Identity-H",
        "DescendantFonts" => vec![cid_font.into()],
        "ToUnicode" => to_unicode,
    })
}

/// The page dictionary with inherited attributes filled in.
fn effective_page(pdf: &Document, page_id: ObjectId) -> Result<Dictionary, ExportError> {
    let mut page = pdf.get_dictionary(page_id).map_err(error)?.clone();
    let mut parent = page.get(b"Parent").and_then(Object::as_reference).ok();
    let mut seen = BTreeSet::new();
    while let Some(id) = parent {
        if !seen.insert(id) {
            break;
        }
        let Ok(node) = pdf.get_dictionary(id) else {
            break;
        };
        for key in INHERITED {
            if !page.has(key)
                && let Ok(value) = node.get(key)
            {
                page.set(key.to_vec(), value.clone());
            }
        }
        parent = node.get(b"Parent").and_then(Object::as_reference).ok();
    }
    Ok(page)
}

fn resolve<'a>(pdf: &'a Document, value: &'a Object) -> &'a Object {
    match value {
        Object::Reference(id) => pdf.get_object(*id).unwrap_or(value),
        other => other,
    }
}

/// `[x0, y0, x1, y1]` of the visible area.
fn page_box(pdf: &Document, page: &Dictionary) -> Option<[f32; 4]> {
    let value = page
        .get(b"CropBox")
        .or_else(|_| page.get(b"MediaBox"))
        .ok()?;
    let array = resolve(pdf, value).as_array().ok()?;
    let mut v = [0f32; 4];
    for (i, n) in array.iter().take(4).enumerate() {
        v[i] = resolve(pdf, n).as_float().ok()?;
    }
    Some([
        v[0].min(v[2]),
        v[1].min(v[3]),
        v[0].max(v[2]),
        v[1].max(v[3]),
    ])
}

/// The matrix from display space (origin at the bottom left of the page as shown, after
/// `/Rotate`) to the page's own space, and the displayed width and height.
fn display_space(page_box: [f32; 4], rotate: i64) -> ([f32; 6], f32, f32) {
    let [x0, y0, x1, y1] = page_box;
    let (w, h) = (x1 - x0, y1 - y0);
    match rotate.rem_euclid(360) {
        90 => ([0.0, 1.0, -1.0, 0.0, x1, y0], h, w),
        180 => ([-1.0, 0.0, 0.0, -1.0, x1, y1], w, h),
        270 => ([0.0, -1.0, 1.0, 0.0, x0, y1], h, w),
        _ => ([1.0, 0.0, 0.0, 1.0, x0, y0], w, h),
    }
}

/// A line of text as two-byte codes, one per Unicode code point in the Basic Multilingual Plane
/// (anything beyond becomes the replacement character).
fn hex(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 4 + 2);
    out.push('<');
    for c in text.chars() {
        let code = u32::from(c);
        let code = if code <= 0xFFFF { code } else { 0xFFFD };
        let _ = write!(out, "{code:04X}");
    }
    out.push('>');
    out
}

/// The block's words over `lines` lines of about equal length.
fn wrap(text: &str, lines: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let total: usize = words.iter().map(|w| w.chars().count() + 1).sum();
    let target = total.div_ceil(lines.max(1));
    let mut out = vec![String::new()];
    for word in words {
        let current = out.last_mut().map_or(0, |l| l.chars().count());
        if current > 0 && current + word.chars().count() > target && out.len() < lines {
            out.push(String::new());
        }
        if let Some(line) = out.last_mut() {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    out.retain(|l| !l.is_empty());
    out
}

/// The content stream that sets `regions` invisibly on a page `width` × `height` (display
/// space, points).
fn text_layer(regions: &[TextRegion], width: f32, height: f32) -> String {
    let mut s = String::from("BT\n3 Tr\n");
    for r in regions {
        let (box_w, box_h) = (r.width * width, r.height * height);
        let chars = r.text.chars().count().max(1) as f32;
        if box_w <= 0.0 || box_h <= 0.0 {
            continue;
        }
        // As many lines as make the characters roughly as wide as they are tall.
        let lines = ((chars * ADVANCE * box_h / box_w).sqrt().round() as usize).max(1);
        let lines = wrap(&r.text, lines);
        let size = box_h / lines.len() as f32;
        let top = height - r.y * height;
        let _ = writeln!(s, "/{} {size:.2} Tf", String::from_utf8_lossy(FONT));
        for (i, line) in lines.iter().enumerate() {
            let n = line.chars().count().max(1) as f32;
            let scale = 100.0 * box_w / (n * ADVANCE * size);
            let baseline = top - size * (i as f32 + 0.8);
            let _ = writeln!(
                s,
                "{scale:.1} Tz 1 0 0 1 {:.2} {baseline:.2} Tm {} Tj",
                r.x * width,
                hex(line)
            );
        }
    }
    s.push_str("ET\n");
    s
}

fn add_text_layer(
    pdf: &mut Document,
    page_id: ObjectId,
    regions: &[TextRegion],
    font: ObjectId,
) -> Result<(), ExportError> {
    let page = effective_page(pdf, page_id)?;
    let page_box = page_box(pdf, &page).ok_or_else(|| error("a page has no size"))?;
    let rotate = page.get(b"Rotate").and_then(Object::as_i64).unwrap_or(0);
    let (m, width, height) = display_space(page_box, rotate);

    // Resources: a copy for this page alone, with the font added.
    let mut resources = match page.get(b"Resources") {
        Ok(value) => resolve(pdf, value).as_dict().cloned().unwrap_or_default(),
        Err(_) => Dictionary::new(),
    };
    let mut fonts = match resources.get(b"Font") {
        Ok(value) => resolve(pdf, value).as_dict().cloned().unwrap_or_default(),
        Err(_) => Dictionary::new(),
    };
    fonts.set(FONT.to_vec(), font);
    resources.set("Font", fonts);

    // The existing content is wrapped in q … Q, so whatever state it leaves cannot shift the
    // text layer drawn after it.
    let layer = format!(
        "q\n{:.4} {:.4} {:.4} {:.4} {:.2} {:.2} cm\n{}Q\n",
        m[0],
        m[1],
        m[2],
        m[3],
        m[4],
        m[5],
        text_layer(regions, width, height)
    );
    let save = pdf.add_object(Stream::new(dictionary! {}, b"q\n".to_vec()));
    let restore = pdf.add_object(Stream::new(dictionary! {}, b"Q\n".to_vec()));
    let layer = pdf.add_object(Stream::new(dictionary! {}, layer.into_bytes()));
    let page = pdf
        .get_object_mut(page_id)
        .and_then(Object::as_dict_mut)
        .map_err(error)?;
    let mut contents = vec![Object::Reference(save)];
    match page.get(b"Contents") {
        Ok(Object::Array(items)) => contents.extend(items.iter().cloned()),
        Ok(other) => contents.push(other.clone()),
        Err(_) => {}
    }
    contents.push(Object::Reference(restore));
    contents.push(Object::Reference(layer));
    page.set("Contents", contents);
    page.set("Resources", resources);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn text_becomes_unicode_codes_and_lines_fit_their_box() {
        assert_eq!(hex("Aé"), "<004100E9>");
        assert_eq!(hex("ب"), "<0628>");
        assert_eq!(hex("😀"), "<FFFD>");
        let lines = wrap("Bamako le trois mai mil neuf cent cinquante-sept", 3);
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines.join(" "),
            "Bamako le trois mai mil neuf cent cinquante-sept"
        );
    }

    #[test]
    fn rotated_pages_map_display_space_back_to_the_page() {
        let (m, w, h) = display_space([0.0, 0.0, 600.0, 800.0], 90);
        assert_eq!((w, h), (800.0, 600.0));
        // The display's bottom-left corner is the page's bottom-right.
        assert_eq!((m[4], m[5]), (600.0, 0.0));
        let (_, w, h) = display_space([0.0, 0.0, 600.0, 800.0], 0);
        assert_eq!((w, h), (600.0, 800.0));
    }

    #[test]
    fn the_layer_is_invisible_text_inside_each_box() {
        let layer = text_layer(
            &[TextRegion {
                x: 0.1,
                y: 0.1,
                width: 0.8,
                height: 0.05,
                text: "Dakar, le 3 mai 1957".into(),
            }],
            600.0,
            800.0,
        );
        assert!(layer.starts_with("BT\n3 Tr\n"));
        assert!(layer.contains(" Tf\n"));
        assert!(layer.contains("1 0 0 1 60.00 "));
        assert!(layer.ends_with("ET\n"));
    }
}
