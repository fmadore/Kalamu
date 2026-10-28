use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use kalamu_core::PageHalf;
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::MediaError;

/// Page attributes a page inherits from its ancestors in the page tree when it does not set
/// them itself (PDF 1.7, 7.7.3.4).
const INHERITED: &[&[u8]] = &[b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

/// Keys not copied into a single-page PDF: they point back into the rest of the document (links,
/// article threads, the page tree), and the model does not need them.
const SKIPPED: &[&[u8]] = &[b"Parent", b"P", b"Annots", b"B", b"StructParents", b"Dest"];

/// A PDF parsed once; pages are cut out on demand.
pub struct PdfSource {
    doc: Document,
    pages: BTreeMap<u32, ObjectId>,
    /// The file, when opened from one, for sending the whole document (summaries).
    path: Option<std::path::PathBuf>,
}

impl PdfSource {
    pub fn open(path: &Path) -> Result<Self, MediaError> {
        let bytes = std::fs::read(path).map_err(|e| MediaError::Read {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let mut pdf = Self::from_bytes(&bytes)?;
        pdf.path = Some(path.to_path_buf());
        Ok(pdf)
    }

    /// The whole file as it is, for providers that read a complete document.
    pub fn whole(&self) -> Result<Vec<u8>, MediaError> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| MediaError::Pdf("not opened from a file".into()))?;
        std::fs::read(path).map_err(|e| MediaError::Read {
            path: path.display().to_string(),
            message: e.to_string(),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MediaError> {
        let doc = Document::load_mem(bytes).map_err(|e| match e {
            lopdf::Error::Decryption(_) | lopdf::Error::InvalidPassword => MediaError::Encrypted,
            other => MediaError::Pdf(other.to_string()),
        })?;
        if doc.is_encrypted() {
            return Err(MediaError::Encrypted);
        }
        let pages = doc.get_pages();
        Ok(Self {
            doc,
            pages,
            path: None,
        })
    }

    pub fn page_count(&self) -> u32 {
        self.pages.len() as u32
    }

    fn page_id(&self, number: u32) -> Result<ObjectId, MediaError> {
        self.pages
            .get(&number)
            .copied()
            .ok_or_else(|| MediaError::NoSuchPage(number.to_string()))
    }

    /// The page dictionary with inherited attributes filled in.
    fn page_dict(&self, number: u32) -> Result<Dictionary, MediaError> {
        let id = self.page_id(number)?;
        let mut page = self
            .doc
            .get_dictionary(id)
            .map_err(|e| MediaError::Pdf(e.to_string()))?
            .clone();
        let mut parent = page.get(b"Parent").and_then(Object::as_reference).ok();
        let mut seen = BTreeSet::new();
        while let Some(pid) = parent {
            if !seen.insert(pid) {
                break;
            }
            let Ok(node) = self.doc.get_dictionary(pid) else {
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

    /// `[x0, y0, x1, y1]` of the visible area, in points.
    fn page_box(&self, page: &Dictionary) -> Option<[f32; 4]> {
        let value = page
            .get(b"CropBox")
            .or_else(|_| page.get(b"MediaBox"))
            .ok()?;
        let value = match value {
            Object::Reference(id) => self.doc.get_object(*id).ok()?,
            other => other,
        };
        let array = value.as_array().ok()?;
        let mut out = [0f32; 4];
        for (i, v) in array.iter().take(4).enumerate() {
            out[i] = v.as_float().ok()?;
        }
        Some([
            out[0].min(out[2]),
            out[1].min(out[3]),
            out[0].max(out[2]),
            out[1].max(out[3]),
        ])
    }

    fn rotation(page: &Dictionary) -> i64 {
        page.get(b"Rotate")
            .and_then(Object::as_i64)
            .map(|r| r.rem_euclid(360))
            .unwrap_or(0)
    }

    /// Width and height as displayed, after rotation, in points.
    pub fn display_size(&self, number: u32) -> Option<(f32, f32)> {
        let page = self.page_dict(number).ok()?;
        let [x0, y0, x1, y1] = self.page_box(&page)?;
        let (w, h) = (x1 - x0, y1 - y0);
        Some(if Self::rotation(&page) % 180 == 90 {
            (h, w)
        } else {
            (w, h)
        })
    }

    /// Wider than tall by enough to be two pages side by side.
    pub fn is_spread(&self, number: u32) -> bool {
        self.display_size(number)
            .is_some_and(|(w, h)| h > 0.0 && w / h > crate::checks::SPREAD_RATIO)
    }

    /// The highest image resolution on the page, in dots per inch, from image sizes only.
    pub fn image_dpi(&self, number: u32) -> Option<u32> {
        let page = self.page_dict(number).ok()?;
        let (width_pt, _) = self.display_size(number)?;
        let resources = self.resolve_dict(page.get(b"Resources").ok()?)?;
        let xobjects = self.resolve_dict(resources.get(b"XObject").ok()?)?;
        let mut widest = 0i64;
        for (_, value) in xobjects.iter() {
            let Ok(id) = value.as_reference() else {
                continue;
            };
            let Ok(Object::Stream(stream)) = self.doc.get_object(id) else {
                continue;
            };
            let is_image = stream
                .dict
                .get(b"Subtype")
                .and_then(Object::as_name)
                .is_ok_and(|n| n == b"Image");
            if is_image {
                let w = stream
                    .dict
                    .get(b"Width")
                    .and_then(Object::as_i64)
                    .unwrap_or(0);
                widest = widest.max(w);
            }
        }
        (widest > 0 && width_pt > 0.0).then(|| (widest as f32 / (width_pt / 72.0)) as u32)
    }

    /// Whether the page already carries extractable text.
    pub fn has_text(&self, number: u32) -> bool {
        self.doc
            .extract_text(&[number])
            .is_ok_and(|t| t.chars().filter(|c| c.is_alphanumeric()).count() > 40)
    }

    fn resolve_dict<'a>(&'a self, value: &'a Object) -> Option<&'a Dictionary> {
        match value {
            Object::Dictionary(d) => Some(d),
            Object::Reference(id) => self.doc.get_dictionary(*id).ok(),
            _ => None,
        }
    }

    /// A standalone PDF holding one page, or one half of a spread.
    pub fn page_pdf(&self, number: u32, half: Option<PageHalf>) -> Result<Vec<u8>, MediaError> {
        let page_id = self.page_id(number)?;
        let mut page = self.page_dict(number)?;
        for key in SKIPPED {
            page.remove(key);
        }

        // Everything the page refers to, transitively, except back-references into the tree.
        let mut keep = BTreeSet::new();
        let mut stack: Vec<ObjectId> = Vec::new();
        collect_refs(&Object::Dictionary(page.clone()), &mut stack);
        while let Some(id) = stack.pop() {
            if id == page_id || !keep.insert(id) {
                continue;
            }
            if let Ok(object) = self.doc.get_object(id) {
                collect_refs(object, &mut stack);
            }
        }

        let mut out = Document::with_version(self.doc.version.clone());
        for id in &keep {
            if let Ok(object) = self.doc.get_object(*id) {
                out.objects.insert(*id, strip(object.clone()));
            }
        }
        out.max_id = self.doc.max_id;
        let pages_id = out.new_object_id();

        if let Some(half) = half
            && let Some(bounds) = self.page_box(&page)
        {
            let cut = half_box(bounds, Self::rotation(&page), half);
            let array = Object::Array(cut.iter().map(|v| Object::Real(*v)).collect());
            page.set("MediaBox", array.clone());
            page.set("CropBox", array);
        }
        page.set("Parent", Object::Reference(pages_id));
        out.objects.insert(page_id, Object::Dictionary(page));
        out.objects.insert(
            pages_id,
            Object::Dictionary(Dictionary::from_iter([
                ("Type", Object::Name(b"Pages".to_vec())),
                ("Kids", Object::Array(vec![Object::Reference(page_id)])),
                ("Count", Object::Integer(1)),
            ])),
        );
        let catalog_id = out.add_object(Dictionary::from_iter([
            ("Type", Object::Name(b"Catalog".to_vec())),
            ("Pages", Object::Reference(pages_id)),
        ]));
        out.trailer.set("Root", Object::Reference(catalog_id));
        let mut bytes = Vec::new();
        out.save_to(&mut bytes)
            .map_err(|e| MediaError::Pdf(e.to_string()))?;
        Ok(bytes)
    }
}

/// Removes back-references from copied objects so they cannot pull in other pages.
fn strip(object: Object) -> Object {
    match object {
        Object::Dictionary(mut d) => {
            d.remove(b"Parent");
            d.remove(b"P");
            Object::Dictionary(d)
        }
        other => other,
    }
}

fn collect_refs(object: &Object, out: &mut Vec<ObjectId>) {
    match object {
        Object::Reference(id) => out.push(*id),
        Object::Array(items) => items.iter().for_each(|o| collect_refs(o, out)),
        Object::Dictionary(d) => {
            for (key, value) in d.iter() {
                if !SKIPPED.contains(&key.as_slice()) {
                    collect_refs(value, out);
                }
            }
        }
        Object::Stream(s) => {
            for (key, value) in s.dict.iter() {
                if !SKIPPED.contains(&key.as_slice()) {
                    collect_refs(value, out);
                }
            }
        }
        _ => {}
    }
}

/// The half of a page box that appears on the given side of the screen, given the page's
/// rotation (clockwise, in degrees).
fn half_box([x0, y0, x1, y1]: [f32; 4], rotation: i64, half: PageHalf) -> [f32; 4] {
    let (xm, ym) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let left = half == PageHalf::Left;
    match (rotation, left) {
        (90, true) | (270, false) => [x0, y0, x1, ym],
        (90, false) | (270, true) => [x0, ym, x1, y1],
        (180, true) => [xm, y0, x1, y1],
        (180, false) => [x0, y0, xm, y1],
        (_, true) => [x0, y0, xm, y1],
        (_, false) => [xm, y0, x1, y1],
    }
}

/// A small PDF with the given page sizes, for tests here and in other crates.
#[cfg(any(test, feature = "test-support"))]
pub fn sample_pdf(pages: &[(f32, f32)], text: bool) -> Vec<u8> {
    use lopdf::content::{Content, Operation};
    use lopdf::{Stream, dictionary};

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let mut kids = Vec::new();
    for (i, (w, h)) in pages.iter().enumerate() {
        let mut ops = Vec::new();
        if text {
            ops = vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![72.into(), 720.into()]),
                Operation::new(
                    "Tj",
                    vec![Object::string_literal(format!(
                        "Page {} of the Bulletin de l'Afrique occidentale francaise, Dakar.",
                        i + 1
                    ))],
                ),
                Operation::new("ET", vec![]),
            ];
        }
        let content = Content { operations: ops };
        let content_id = doc.add_object(Stream::new(
            dictionary! {},
            content.encode().unwrap_or_default(),
        ));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), Object::Real(*w), Object::Real(*h)],
            "Contents" => content_id,
        });
        kids.push(Object::Reference(page_id));
    }
    // Resources on the tree node, inherited by every page.
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => pages.len() as i64,
            "Resources" => resources_id,
        }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    let _ = doc.save_to(&mut out);
    out
}
