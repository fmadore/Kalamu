use super::*;
use crate::PieceRef;

fn sorted(names: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    natural_sort(&mut v, |s| s.as_str());
    v
}

#[test]
fn page_2_comes_before_page_10() {
    assert_eq!(
        sorted(&["page 10.jpg", "page 2.jpg", "page 1.jpg"]),
        ["page 1.jpg", "page 2.jpg", "page 10.jpg"]
    );
}

#[test]
fn capitals_do_not_change_the_order() {
    assert_eq!(
        sorted(&["Scan 3.jpg", "scan 1.jpg", "SCAN 2.jpg"]),
        ["scan 1.jpg", "SCAN 2.jpg", "Scan 3.jpg"]
    );
}

#[test]
fn leading_zeros_are_numbers_too() {
    assert_eq!(
        sorted(&["p010.jpg", "p9.jpg", "p001.jpg"]),
        ["p001.jpg", "p9.jpg", "p010.jpg"]
    );
    // Equal as numbers: fewer zeros first, every time.
    assert_eq!(sorted(&["p01.jpg", "p1.jpg"]), ["p1.jpg", "p01.jpg"]);
    assert_eq!(sorted(&["p1.jpg", "p01.jpg"]), ["p1.jpg", "p01.jpg"]);
}

#[test]
fn separators_are_one_kind() {
    assert_eq!(
        sorted(&["page-10.jpg", "page_2.jpg", "page 3.jpg", "page.1.jpg"]),
        ["page.1.jpg", "page_2.jpg", "page 3.jpg", "page-10.jpg"]
    );
}

#[test]
fn digits_of_other_scripts_are_left_as_letters() {
    // Arabic-Indic digits are not read as numbers: they sort as characters, after the
    // Latin digits, and keep their own order among themselves.
    assert_eq!(
        sorted(&["ص ٣.jpg", "ص ١.jpg", "ص 2.jpg"]),
        ["ص 2.jpg", "ص ١.jpg", "ص ٣.jpg"]
    );
    assert_eq!(number_in_name("ص ٣.jpg"), None);
}

#[test]
fn numbers_come_before_letters_and_long_numbers_stay_exact() {
    assert_eq!(sorted(&["a.jpg", "1.jpg"]), ["1.jpg", "a.jpg"]);
    assert_eq!(
        sorted(&["IMG_20230101_0012.jpg", "IMG_20230101_0002.jpg"]),
        ["IMG_20230101_0002.jpg", "IMG_20230101_0012.jpg"]
    );
    let huge = "99999999999999999999999.jpg";
    assert_eq!(sorted(&[huge, "2.jpg"]), ["2.jpg", huge]);
}

fn page(name: &str, sha: &str) -> PageFile {
    PageFile {
        id: page_id(&format!("/scans/{name}"), sha),
        path: format!("/scans/{name}"),
        file_name: name.into(),
        sha256: sha.into(),
        size_bytes: 10,
        modified_ms: None,
        frames: 1,
        label: None,
    }
}

#[test]
fn gaps_and_repeats_are_pointed_out() {
    let pages = [
        page("p1.jpg", "a"),
        page("p2.jpg", "b"),
        page("p5.jpg", "c"),
        page("p05 copy.jpg", "d"),
        page("cover.jpg", "e"),
    ];
    let notes = page_notes(&pages);
    assert_eq!(notes[0].number, Some(1));
    assert!(!notes[0].duplicate && notes[0].missing_from.is_none());
    assert_eq!(notes[1].missing_from, None);
    assert_eq!(
        (notes[2].missing_from, notes[2].missing_to),
        (Some(3), Some(4))
    );
    // "p05 copy" carries 5 as well.
    assert!(notes[2].duplicate && notes[3].duplicate);
    assert_eq!(notes[4].number, None);
    assert!(!notes[4].duplicate);
}

#[test]
fn identity_is_not_position() {
    let a = page("1.jpg", "aa");
    let b = page("2.jpg", "bb");
    let doc = SourceFile::image_document("Letters", vec![a.clone(), b.clone()]);
    let moved = SourceFile::image_document("Letters", vec![b.clone(), a.clone()]);
    assert_ne!(doc.sha256, moved.sha256, "the order is the document's");
    assert_eq!(doc.cache_identity(&PieceRef::page(1)).0, "aa");
    assert_eq!(moved.cache_identity(&PieceRef::page(2)).0, "aa");
    assert_eq!(
        doc.cache_identity(&PieceRef::page(1)),
        moved.cache_identity(&PieceRef::page(2)),
        "the same page, wherever it stands"
    );
    assert_eq!(doc.page_count, Some(2));
    assert!(doc.names_itself());
    assert_eq!(
        Path::new(&doc.path),
        Path::new("/scans").join("Letters"),
        "named in its first page's folder"
    );
}

#[test]
fn a_multi_page_file_gives_its_pages_in_order() {
    let mut tiff = page("bundle.tif", "tt");
    tiff.frames = 3;
    let doc = SourceFile::image_document("Box", vec![page("a.jpg", "aa"), tiff]);
    assert_eq!(doc.page_count, Some(4));
    let found = doc.page_file(4).map(|(p, f)| (p.file_name.as_str(), f));
    assert_eq!(found, Some(("bundle.tif", 3)));
    assert_eq!(
        doc.cache_identity(&PieceRef::page(3)),
        ("tt", PieceRef::page(2))
    );
    assert!(doc.page_file(5).is_none());
}

#[test]
fn names_are_files_both_systems_accept() {
    assert_eq!(file_name_of("Box 3: letters/1923?"), "Box 3- letters-1923-");
    assert_eq!(file_name_of("  notes.  "), "notes");
    assert_eq!(file_name_of("con"), "con-");
    assert_eq!(file_name_of("COM1.letters"), "COM1.letters-");
    assert_eq!(file_name_of("..."), "document");
    assert_eq!(file_name_of("Box 3.2"), "Box 3.2");
}

#[test]
fn a_source_reads_its_files() {
    let doc = SourceFile::image_document("D", vec![page("a.jpg", "a"), page("b.jpg", "b")]);
    assert_eq!(doc.files(), ["/scans/a.jpg", "/scans/b.jpg"]);
    assert_eq!(doc.file_of(&PieceRef::page(2)), Some("/scans/b.jpg"));
    assert_eq!(doc.file_of(&PieceRef::Whole), None);
}
