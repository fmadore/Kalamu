//! A synthetic corpus the size of a large project, to measure indexing, query time and index
//! size: `cargo test -p kalamu-search --release -- --ignored --nocapture scale`.

use std::time::Instant;

use super::*;

/// Words of a West African francophone newspaper page, mixed so that every page differs.
const WORDS: &[&str] = &[
    "la",
    "communauté",
    "musulmane",
    "de",
    "Ouagadougou",
    "a",
    "célébré",
    "Tabaski",
    "imam",
    "mosquée",
    "association",
    "des",
    "élèves",
    "et",
    "étudiants",
    "conférence",
    "prêche",
    "Bobo-Dioulasso",
    "zakat",
    "pèlerinage",
    "La Mecque",
    "jeunesse",
    "Coran",
    "école",
    "franco-arabe",
    "ministre",
    "cultes",
    "sermon",
    "fidèles",
    "vendredi",
    "الشيخ",
    "المسجد",
];

fn page_text(seed: usize, chars: usize) -> String {
    let mut out = String::new();
    let mut n = (seed as u64).wrapping_mul(2_654_435_761);
    while out.len() < chars {
        n = n.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        out.push_str(WORDS[(n >> 33) as usize % WORDS.len()]);
        out.push(if n.is_multiple_of(11) { '\n' } else { ' ' });
    }
    out
}

#[test]
#[ignore = "measures a large synthetic corpus; run on demand"]
fn scale() {
    const DOCUMENTS: usize = 500;
    const PAGES: usize = 8;
    const CHARS: usize = 3_000;
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::open(&dir.path().join("jobs.sqlite")).unwrap());
    let index = Arc::new(SqliteIndex::open(&dir.path().join("search.sqlite")).unwrap());
    let f = Fixture::on(store, index);
    for d in 0..DOCUMENTS {
        let doc = f.document(&format!("Issue {d}"), InputKind::Pdf);
        let mut pages: Vec<String> = (0..PAGES)
            .map(|p| page_text(d * PAGES + p, CHARS))
            .collect();
        if d % 50 == 0 {
            pages[3].push_str(" Union culturelle musulmane ");
        }
        let pages: Vec<&str> = pages.iter().map(String::as_str).collect();
        f.extraction(&doc, &pages);
    }
    let text_mb = (DOCUMENTS * PAGES * CHARS) as f64 / 1e6;

    let t = Instant::now();
    let first = f.find(&request("union culturelle musulmane"));
    let indexing = t.elapsed();
    let t = Instant::now();
    let again = f.find(&request("union culturelle musulmane"));
    let query = t.elapsed();
    let t = Instant::now();
    let common = f.find(&request("mosquée"));
    let common_query = t.elapsed();
    let size = f.index.size().unwrap();
    assert_eq!(first.matches as usize, DOCUMENTS / 50);
    assert_eq!(again.matches, first.matches);
    println!(
        "{DOCUMENTS} documents, {} pages, {text_mb:.1} MB of text: first search (indexing) {:.2?}, \
         then {:.2?}; a common word ({} matches, {} listed) {:.2?}; index {} passages, {:.1} MB",
        DOCUMENTS * PAGES,
        indexing,
        query,
        common.matches,
        common.listed,
        common_query,
        size.passages,
        size.bytes as f64 / 1e6
    );
}
