//! PAGE XML: one file per page from the regions ALTO writes, as golden files, and a parse of
//! the output that checks what the 2019-07-15 schema asks of its structure.

use std::collections::{HashMap, HashSet};

use kalamu_core::{ExportFile, PageHalf, PieceRef, TextLine, TextRegion};

use super::*;

/// Tesseract's line of words on a page, one of them without a confidence.
fn placed() -> TextRegion {
    TextRegion {
        lines: vec![TextLine {
            x: 0.1,
            y: 0.1,
            width: 0.4,
            height: 0.02,
            words: vec![
                word(0.1, 0.1, 0.12, "Dakar,", Some(0.96)),
                word(0.24, 0.1, 0.04, "le", Some(0.91)),
                word(0.3, 0.1, 0.02, "3", Some(0.88)),
                word(0.34, 0.1, 0.06, "mai", None),
            ],
        }],
        ..TextRegion::block(0.1, 0.1, 0.4, 0.02, "Dakar, le 3 mai")
    }
}

fn part(number: u32, text: &str, regions: Vec<TextRegion>) -> ExportPart {
    ExportPart {
        piece: PieceRef::page(number),
        text: text.into(),
        segments: vec![],
        corrected: false,
        reviewed: false,
        fields: vec![],
        regions,
        failed: None,
        drawn_dpi: None,
        // 2026-10-27, then a correction three days later.
        read_at: Some(Timestamp(1_793_096_727_000)),
        corrected_at: None,
    }
}

/// A PDF of `pages` (width and height in points) at a path of its own.
fn pdf(dir: &tempfile::TempDir, name: &str, pages: &[(i64, i64)]) -> SourceFile {
    let path = dir.path().join(name);
    pdf_of_pages(&path, pages);
    let mut src = source(name, InputKind::Pdf);
    src.path = path.display().to_string();
    src
}

fn files(doc: &ExportDocument) -> Vec<ExportFile> {
    exporter(ExportFormat::PageXml)
        .unwrap_or_else(|| panic!("no PAGE XML exporter"))
        .export_files(doc)
        .unwrap_or_else(|e| panic!("{e}"))
}

fn text(file: &ExportFile) -> String {
    String::from_utf8(file.bytes.clone()).unwrap_or_else(|e| panic!("{e}"))
}

/// A Tesseract page with its words, and a corrected page whose words the engine laid back on
/// their places: one file each, named by its page, each with the times it was read and changed.
#[test]
fn page_xml_writes_each_page_with_its_words_as_golden_files() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut corrected = part(
        2,
        "Dakar, le 5 mai",
        vec![TextRegion {
            lines: vec![TextLine {
                x: 0.1,
                y: 0.1,
                width: 0.4,
                height: 0.02,
                words: vec![
                    word(0.1, 0.1, 0.12, "Dakar,", Some(0.96)),
                    word(0.24, 0.1, 0.04, "le", Some(0.91)),
                    // The corrected word, without the engine's confidence.
                    word(0.3, 0.1, 0.02, "5", None),
                    word(0.34, 0.1, 0.06, "mai", None),
                ],
            }],
            ..TextRegion::block(0.1, 0.1, 0.4, 0.02, "Dakar, le 5 mai")
        }],
    );
    corrected.corrected = true;
    corrected.corrected_at = Some(Timestamp(1_793_355_927_000));
    let doc = document(
        Task::TextExtraction,
        pdf(&dir, "lettre.pdf", &[(595, 842), (595, 842)]),
        vec![part(1, "Dakar, le 3 mai", vec![placed()]), corrected],
    );
    let out = files(&doc);
    let names: Vec<&str> = out.iter().map(|f| f.suffix.as_str()).collect();
    assert_eq!(names, ["p0001.page.xml", "p0002.page.xml"]);
    check("letter.p0001.page.xml", &out[0].bytes);
    check("letter.p0002.page.xml", &out[1].bytes);
    for file in &out {
        valid(&text(file));
    }
    let second = text(&out[1]);
    assert!(second.contains("<Created>2026-10-27T"), "{second}");
    assert!(second.contains("<LastChange>2026-10-30T"), "{second}");
    assert!(
        second.contains("text corrected by the researcher"),
        "{second}"
    );

    // Written again later, the files are the same: no clock in them.
    let mut later = doc.clone();
    later.provenance.created_at = Timestamp(1_900_000_000_000);
    assert_eq!(files(&later), out);

    // A document of several pages is not one file.
    assert!(exporter(ExportFormat::PageXml).is_some_and(|e| e.export(&doc).is_err()));
}

/// Mistral OCR's blocks: a region each, with the lines ALTO estimates, marked as estimated, and
/// no words; text escaped, characters XML forbids dropped.
#[test]
fn page_xml_of_blocks_placed_whole_estimates_lines_and_no_words() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let doc = document(
        Task::TextExtraction,
        pdf(&dir, "journal.pdf", &[(595, 842)]),
        vec![part(
            1,
            "Le Gouverneur & les délégués <réunis>",
            vec![
                TextRegion::block(
                    0.1,
                    0.1,
                    0.8,
                    0.04,
                    "Le Gouverneur & les délégués\n<réunis>\u{1}",
                ),
                TextRegion::block(0.1, 0.2, 0.8, 0.02, "Kita"),
            ],
        )],
    );
    let out = files(&doc);
    assert_eq!(out.len(), 1);
    check("journal.p0001.page.xml", &out[0].bytes);
    let xml = text(&out[0]);
    valid(&xml);
    assert!(!xml.contains("<Word "), "{xml}");
    assert!(xml.contains("position estimated within a block"), "{xml}");
    assert!(
        xml.contains("line positions estimated within blocks"),
        "{xml}"
    );
    assert!(xml.contains("Le Gouverneur &amp; les délégués"), "{xml}");
    assert!(xml.contains("&lt;réunis&gt;</Unicode>"), "{xml}");
    // A document of one page is also one file.
    assert!(
        exporter(ExportFormat::PageXml)
            .is_some_and(|e| e.export(&doc).is_ok_and(|b| b == out[0].bytes))
    );
}

/// A poster drawn below 300 dpi for Tesseract: measured, and said to be, at that resolution.
#[test]
fn page_xml_measures_a_drawn_pdf_page_at_its_resolution() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut poster = part(2, "Bamako", vec![placed()]);
    poster.drawn_dpi = Some(252.0);
    let doc = document(
        Task::TextExtraction,
        pdf(&dir, "affiche.pdf", &[(595, 842), (2000, 1000)]),
        vec![part(1, "Dakar, le 3 mai", vec![placed()]), poster],
    );
    let out = files(&doc);
    check("poster.p0002.page.xml", &out[1].bytes);
    let xml = text(&out[1]);
    valid(&xml);
    assert!(
        xml.contains(
            "imageFilename=\"affiche.pdf#page=2\" imageWidth=\"7000\" imageHeight=\"3500\" \
imageXResolution=\"252\" imageYResolution=\"252\" imageResolutionUnit=\"PPI\""
        ),
        "{xml}"
    );
    assert!(xml.contains("PDF page measured at 252 dpi"), "{xml}");
    let first = text(&out[0]);
    assert!(
        first.contains("imageWidth=\"2479\" imageHeight=\"3508\""),
        "{first}"
    );
}

/// A page read without text is a page with no regions, as ALTO writes it; a page that failed
/// says so; and a model that reported no positions gives no file at all.
#[test]
fn page_xml_of_a_page_without_text_has_no_regions() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut failed = part(3, "", vec![]);
    failed.failed = Some("the provider timed out".into());
    failed.read_at = None;
    let doc = document(
        Task::TextExtraction,
        pdf(&dir, "lettre.pdf", &[(595, 842), (595, 842), (595, 842)]),
        vec![
            part(1, "Dakar, le 3 mai", vec![placed()]),
            part(2, "", vec![]),
            failed,
        ],
    );
    let out = files(&doc);
    assert_eq!(out.len(), 3);
    check("blank.p0002.page.xml", &out[1].bytes);
    check("blank.p0003.page.xml", &out[2].bytes);
    let blank = text(&out[1]);
    valid(&blank);
    assert!(!blank.contains("<TextRegion"), "{blank}");
    assert!(!blank.contains("<ReadingOrder"), "{blank}");
    let not_read = text(&out[2]);
    valid(&not_read);
    assert!(
        not_read.contains("not read: the provider timed out"),
        "{not_read}"
    );
    // Dated by the document's reading, not the time of writing.
    assert!(not_read.contains("<Created>2026-10-27T"), "{not_read}");

    let mut plain = doc;
    for p in &mut plain.parts {
        p.regions.clear();
    }
    assert!(exporter(ExportFormat::PageXml).is_some_and(|e| e.export_files(&plain).is_err()));
}

/// The right half of a spread is put back on the right of the whole page, and each page of a
/// document of images names its own image and is measured in it.
#[test]
fn page_xml_places_halves_on_the_whole_page_and_names_each_image() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut half = part(1, "Dakar, le 3 mai", vec![placed()]);
    half.piece = PieceRef::Page {
        number: 1,
        half: Some(PageHalf::Right),
    };
    let doc = document(
        Task::TextExtraction,
        pdf(&dir, "lettre.pdf", &[(595, 842)]),
        vec![half],
    );
    let xml = text(&files(&doc)[0]);
    // 0.5 + 0.1 / 2 of 2479 pixels.
    assert!(
        xml.contains("<Word id=\"r1_l1_w1\">\n          <Coords points=\"1363,"),
        "{xml}"
    );

    let mut pages = Vec::new();
    for (name, width) in [("p1.png", 1000), ("p2.png", 2000)] {
        let path = dir.path().join(name);
        image::DynamicImage::ImageRgb8(image::RgbImage::new(width, 1500))
            .save(&path)
            .unwrap_or_else(|e| panic!("{e}"));
        pages.push(kalamu_core::PageFile {
            id: name.into(),
            path: path.display().to_string(),
            file_name: name.into(),
            sha256: "00".repeat(32),
            size_bytes: 1,
            modified_ms: None,
            frames: 1,
            label: None,
        });
    }
    let source = SourceFile::image_document("Pages", pages);
    // A block reaching past the page's edge stays within it.
    let wide = TextRegion::block(0.5, 0.1, 0.7, 0.02, "Mot");
    let doc = document(
        Task::TextExtraction,
        source,
        vec![
            part(1, "Mot", vec![wide.clone()]),
            part(2, "Mot", vec![wide]),
        ],
    );
    let out = files(&doc);
    check("pages.p0001.page.xml", &out[0].bytes);
    let (first, second) = (text(&out[0]), text(&out[1]));
    valid(&first);
    valid(&second);
    assert!(
        first.contains("imageFilename=\"p1.png\" imageWidth=\"1000\" imageHeight=\"1500\">"),
        "{first}"
    );
    assert!(
        second.contains("imageFilename=\"p2.png\" imageWidth=\"2000\""),
        "{second}"
    );
    assert!(
        first.contains("<Coords points=\"500,150 999,150 999,180 500,180\"/>"),
        "{first}"
    );
}

/// An element of the parsed file.
#[derive(Debug, Default)]
struct El {
    name: String,
    attrs: HashMap<String, String>,
    children: Vec<El>,
    text: String,
}

impl El {
    fn named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a El> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    fn unicode(&self) -> String {
        let equiv: Vec<&El> = self.named("TextEquiv").collect();
        assert_eq!(equiv.len(), 1, "{} has one TextEquiv", self.name);
        let unicode: Vec<&El> = equiv[0].named("Unicode").collect();
        assert_eq!(unicode.len(), 1);
        unicode[0].text.clone()
    }
}

/// The file parsed: well-formed XML of elements, attributes in double quotes, text and the five
/// entities the exporters write. Panics where it is not.
fn parse(xml: &str) -> El {
    let body = xml
        .strip_prefix("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
        .unwrap_or_else(|| panic!("no XML declaration"));
    let unescape = |s: &str| {
        assert!(
            !s.replace("&amp;", "")
                .replace("&lt;", "")
                .replace("&gt;", "")
                .replace("&quot;", "")
                .contains('&'),
            "unknown entity in {s:?}"
        );
        s.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
    };
    let mut stack: Vec<El> = vec![El::default()];
    let mut rest = body;
    while !rest.is_empty() {
        if let Some(tag) = rest.strip_prefix('<') {
            let end = tag.find('>').unwrap_or_else(|| panic!("unclosed tag"));
            let inner = &tag[..end];
            rest = &tag[end + 1..];
            if let Some(name) = inner.strip_prefix('/') {
                let el = stack.pop().unwrap_or_else(|| panic!("stray end tag"));
                assert_eq!(el.name, name, "end tag matches");
                stack
                    .last_mut()
                    .unwrap_or_else(|| panic!("no parent"))
                    .children
                    .push(el);
                continue;
            }
            let closed = inner.ends_with('/');
            let inner = inner.trim_end_matches('/');
            let (name, mut attrs_text) = inner.split_once(' ').unwrap_or((inner, ""));
            let mut el = El {
                name: name.into(),
                ..El::default()
            };
            loop {
                attrs_text = attrs_text.trim_start();
                if attrs_text.is_empty() {
                    break;
                }
                let (key, after) = attrs_text
                    .split_once("=\"")
                    .unwrap_or_else(|| panic!("attribute without a quoted value: {attrs_text}"));
                let close = after.find('"').unwrap_or_else(|| panic!("unclosed value"));
                let value = &after[..close];
                assert!(!value.contains('<'), "< in an attribute");
                let fresh = el.attrs.insert(key.into(), unescape(value)).is_none();
                assert!(fresh, "attribute {key} given once");
                attrs_text = &after[close + 1..];
            }
            if closed {
                stack
                    .last_mut()
                    .unwrap_or_else(|| panic!("no parent"))
                    .children
                    .push(el);
            } else {
                stack.push(el);
            }
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            let text = unescape(&rest[..end]);
            if let Some(el) = stack.last_mut() {
                el.text.push_str(&text);
            }
            rest = &rest[end..];
        }
    }
    let mut root = stack.pop().unwrap_or_else(|| panic!("nothing parsed"));
    assert!(stack.is_empty(), "every element closed");
    assert_eq!(root.children.len(), 1, "one root element");
    root.children.remove(0)
}

/// An XML NCName as the exporters make them: a letter or underscore, then letters, digits,
/// underscores, hyphens and dots.
fn ncname(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// A `points` value as the schema's `PointsType` and the exporter write it: four corners of a
/// rectangle, whole non-negative pixels, within the page.
fn rectangle(points: &str, width: u32, height: u32) {
    let corners: Vec<(u32, u32)> = points
        .split(' ')
        .map(|p| {
            let (x, y) = p.split_once(',').unwrap_or_else(|| panic!("point {p}"));
            let n = |v: &str| v.parse::<u32>().unwrap_or_else(|e| panic!("{v}: {e}"));
            (n(x), n(y))
        })
        .collect();
    assert_eq!(corners.len(), 4, "{points}");
    // Clockwise from the top left: top right, bottom right, bottom left.
    let [(ax, ay), (bx, by), (cx, cy), (dx, dy)] = corners[..] else {
        panic!("{points}");
    };
    assert!(by == ay && cx == bx && dy == cy && dx == ax, "{points}");
    assert!(ax <= bx && ay <= cy, "{points}");
    for (x, y) in corners {
        assert!(
            x < width && y < height,
            "{points} within {width} by {height}"
        );
    }
}

/// What the schema asks of the file's structure, as far as the exporter writes it, and the
/// text agreeing from one level to the next.
fn valid(xml: &str) {
    let root = parse(xml);
    assert_eq!(root.name, "PcGts");
    assert_eq!(
        root.attrs.get("xmlns").map(String::as_str),
        Some("http://schema.primaresearch.org/PAGE/gts/pagecontent/2019-07-15")
    );
    let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Metadata", "Page"]);
    let meta: Vec<&str> = root.children[0]
        .children
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(meta, ["Creator", "Created", "LastChange", "Comments"]);
    for at in &root.children[0].children[1..3] {
        // xs:dateTime in UTC.
        assert_eq!(at.text.len(), 20, "{}", at.text);
        assert!(at.text.ends_with('Z') && at.text.as_bytes()[10] == b'T');
    }
    let page = &root.children[1];
    let size = |key: &str| -> u32 {
        page.attrs
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("Page needs {key}"))
    };
    let (width, height) = (size("imageWidth"), size("imageHeight"));
    assert!(page.attrs.contains_key("imageFilename"));

    let mut ids = HashSet::new();
    let mut id = |el: &El| {
        let value = el
            .attrs
            .get("id")
            .unwrap_or_else(|| panic!("{} needs an id", el.name));
        assert!(ncname(value), "{value} is an NCName");
        assert!(ids.insert(value.clone()), "{value} is unique");
        value.clone()
    };
    let order: Vec<&str> = page.children.iter().map(|c| c.name.as_str()).collect();
    let regions: Vec<&El> = page.named("TextRegion").collect();
    let mut refs = Vec::new();
    if regions.is_empty() {
        assert!(order.is_empty(), "{order:?}");
    } else {
        assert_eq!(order[0], "ReadingOrder");
        assert!(order[1..].iter().all(|n| *n == "TextRegion"), "{order:?}");
        let reading = &page.children[0];
        assert_eq!(reading.children.len(), 1);
        let group = &reading.children[0];
        assert_eq!(group.name, "OrderedGroup");
        id(group);
        for (i, r) in group.children.iter().enumerate() {
            assert_eq!(r.name, "RegionRefIndexed");
            assert_eq!(r.attrs.get("index"), Some(&i.to_string()));
            refs.push(r.attrs.get("regionRef").cloned().unwrap_or_default());
        }
    }
    let mut region_ids = Vec::new();
    for region in regions {
        region_ids.push(id(region));
        let kinds: Vec<&str> = region.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(kinds.first(), Some(&"Coords"), "{kinds:?}");
        assert_eq!(kinds.last(), Some(&"TextEquiv"), "{kinds:?}");
        assert!(kinds[1..kinds.len() - 1].iter().all(|k| *k == "TextLine"));
        rectangle(&region.children[0].attrs["points"], width, height);
        let mut lines = Vec::new();
        for line in region.named("TextLine") {
            id(line);
            let kinds: Vec<&str> = line.children.iter().map(|c| c.name.as_str()).collect();
            assert_eq!(kinds.first(), Some(&"Coords"), "{kinds:?}");
            assert_eq!(kinds.last(), Some(&"TextEquiv"), "{kinds:?}");
            assert!(kinds[1..kinds.len() - 1].iter().all(|k| *k == "Word"));
            rectangle(&line.children[0].attrs["points"], width, height);
            let mut words = Vec::new();
            for word in line.named("Word") {
                id(word);
                let kinds: Vec<&str> = word.children.iter().map(|c| c.name.as_str()).collect();
                assert_eq!(kinds, ["Coords", "TextEquiv"]);
                rectangle(&word.children[0].attrs["points"], width, height);
                if let Some(conf) = word.children[1].attrs.get("conf") {
                    let conf: f32 = conf.parse().unwrap_or(-1.0);
                    assert!((0.0..=1.0).contains(&conf));
                }
                words.push(word.unicode());
            }
            let text = line.unicode();
            if !words.is_empty() {
                assert_eq!(text, words.join(" "));
            }
            lines.push(text);
        }
        if !lines.is_empty() {
            assert_eq!(region.unicode(), lines.join("\n"));
        }
    }
    assert_eq!(
        refs, region_ids,
        "the reading order names every region once"
    );
}
