use super::*;
use crate::{PageHalf, TextLine, TextRegion, TextWord, plain_words};

/// A word of `text` at `x` on a line at `y`, as Tesseract places it.
fn word(text: &str, x: f32, y: f32) -> TextWord {
    TextWord {
        x,
        y,
        width: 0.05,
        height: 0.02,
        text: text.to_owned(),
        confidence: Some(0.9),
    }
}

/// A block of lines of words, each line's words spaced along it, as Tesseract places them.
fn tesseract_block(x: f32, y: f32, lines: &[&str]) -> TextRegion {
    let lines: Vec<TextLine> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let ly = y + i as f32 * 0.03;
            TextLine {
                x,
                y: ly,
                width: 0.4,
                height: 0.02,
                words: line
                    .split_whitespace()
                    .enumerate()
                    .map(|(k, w)| word(w, x + k as f32 * 0.06, ly))
                    .collect(),
            }
        })
        .collect();
    TextRegion {
        x,
        y,
        width: 0.4,
        height: 0.03 * lines.len() as f32,
        text: lines
            .iter()
            .map(TextLine::text)
            .collect::<Vec<_>>()
            .join("\n"),
        lines,
    }
}

/// The text a span covers in `text`, counted in UTF-16 units as the interface counts them.
fn spanned(text: &str, span: Option<TextSpan>) -> Option<String> {
    let span = span?;
    let units: Vec<u16> = text.encode_utf16().collect();
    String::from_utf16(&units[span.start as usize..span.end as usize]).ok()
}

/// The Tesseract text of `regions`: blocks apart by a blank line, as `layout` joins them.
fn joined(regions: &[TextRegion]) -> String {
    regions
        .iter()
        .map(|r| r.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[test]
fn tesseract_text_places_every_region_by_position_a_repeated_phrase_included() {
    let regions = vec![
        tesseract_block(0.1, 0.1, &["Le cercle de Kita", "Le cercle de Kita"]),
        tesseract_block(0.1, 0.5, &["Fin du rapport."]),
    ];
    let text = joined(&regions);
    let page = PageRegions::of(&regions, &text, &text, false);
    assert_eq!(page.mapping, TextMapping::Exact);
    assert_eq!(
        (page.blocks.len(), page.lines.len(), page.words.len()),
        (2, 3, 11)
    );
    // The two lines read alike, yet each has its own place: the text is their words in order.
    let first = page.lines[0].span.unwrap();
    let second = page.lines[1].span.unwrap();
    assert_ne!(first, second);
    assert_eq!(
        spanned(&text, Some(first)).as_deref(),
        Some("Le cercle de Kita")
    );
    assert_eq!(
        spanned(&text, Some(second)).as_deref(),
        Some("Le cercle de Kita")
    );
    assert!(second.start > first.end);
    assert_eq!(
        spanned(&text, page.words[4].span).as_deref(),
        Some("Le"),
        "the second line's first word"
    );
    assert_eq!(page.words[4].span.unwrap().start, second.start);
    assert_eq!(
        spanned(&text, page.blocks[1].span).as_deref(),
        Some("Fin du rapport.")
    );
    assert!(page.words.iter().all(|w| !w.estimated));
}

#[test]
fn a_repeated_phrase_has_no_place_once_the_text_differs_from_the_regions() {
    // Mistral OCR: blocks of plain words, the text in Markdown around them, with a footnote no
    // block holds.
    let model = "# Rapport\n\nLe cercle de Kita\n\n**Le cercle de Kita**\n\nFin du rapport.\n\n[^1]: Copie.";
    let regions = vec![
        TextRegion::block(0.1, 0.05, 0.3, 0.05, plain_words("# Rapport")),
        TextRegion::block(0.1, 0.2, 0.4, 0.05, "Le cercle de Kita"),
        TextRegion::block(0.1, 0.3, 0.4, 0.05, plain_words("**Le cercle de Kita**")),
        TextRegion::block(0.1, 0.4, 0.4, 0.05, "Fin du rapport."),
    ];
    let page = PageRegions::of(&regions, model, model, false);
    assert_eq!(page.mapping, TextMapping::Unique);
    assert_eq!(
        spanned(model, page.blocks[0].span).as_deref(),
        Some("Rapport")
    );
    assert_eq!(page.blocks[1].span, None, "found twice: no guess");
    assert_eq!(page.blocks[2].span, None);
    assert_eq!(
        spanned(model, page.blocks[3].span).as_deref(),
        Some("Fin du rapport.")
    );
    // The estimated words that stand once are placed; those of the repeated phrase are not.
    let kita: Vec<_> = page.words.iter().filter(|w| w.text == "Kita").collect();
    assert_eq!(kita.len(), 2);
    assert!(kita.iter().all(|w| w.span.is_none()));
    let fin = page.words.iter().find(|w| w.text == "Fin").unwrap();
    assert_eq!(spanned(model, fin.span).as_deref(), Some("Fin"));
    assert!(page.lines.iter().chain(&page.words).all(|r| r.estimated));
    assert!(page.blocks.iter().all(|r| !r.estimated));
}

#[test]
fn a_correction_keeps_places_only_where_the_words_still_stand_once() {
    let regions = vec![
        tesseract_block(0.1, 0.1, &["Dakar, le 3 rnai 1957"]),
        tesseract_block(0.1, 0.3, &["Le Gouverneur a reçu les délégués."]),
        tesseract_block(0.1, 0.5, &["la mission", "la mission"]),
    ];
    let model = joined(&regions);
    // The researcher corrects a misread word and takes out one of the two lines that read alike.
    let shown =
        "Dakar, le 3 mai 1957\n\nLe Gouverneur a reçu les délégués.\n\nla mission".to_owned();
    let page = PageRegions::of(&regions, &model, &shown, true);
    assert_eq!(page.mapping, TextMapping::Unique);
    assert!(page.corrected);
    assert_eq!(page.blocks[0].span, None, "its words no longer match");
    assert_eq!(
        spanned(&shown, page.blocks[1].span).as_deref(),
        Some("Le Gouverneur a reçu les délégués.")
    );
    // Once in the text shown, but twice in the model's: which one is left cannot be told.
    assert_eq!(page.lines[2].span, None);
    assert_eq!(page.lines[3].span, None);
    let rnai = page.words.iter().find(|w| w.text == "rnai").unwrap();
    assert_eq!(rnai.span, None);
    let dakar = page.words.iter().find(|w| w.text == "Dakar,").unwrap();
    assert_eq!(spanned(&shown, dakar.span).as_deref(), Some("Dakar,"));
}

#[test]
fn french_accents_match_however_they_were_typed_and_spans_count_utf16() {
    let regions = vec![tesseract_block(0.1, 0.1, &["Réception à Bamako"])];
    let model = joined(&regions);
    // The same words, the accents typed as separate marks (NFD): one unit more each.
    let shown = "Re\u{301}ception a\u{300} Bamako";
    let page = PageRegions::of(&regions, &model, shown, true);
    assert_eq!(page.mapping, TextMapping::Exact);
    assert_eq!(page.words[0].span, Some(TextSpan { start: 0, end: 10 }));
    assert_eq!(page.words[1].span, Some(TextSpan { start: 11, end: 13 }));
    assert_eq!(
        spanned(shown, page.words[2].span).as_deref(),
        Some("Bamako")
    );
}

#[test]
fn arabic_is_placed_in_reading_order_and_wide_characters_count_twice() {
    let regions = vec![
        tesseract_block(0.5, 0.1, &["بسم الله الرحمن الرحيم"]),
        tesseract_block(0.5, 0.2, &["كتاب 𝔄 الصلاة"]),
    ];
    let text = joined(&regions);
    let page = PageRegions::of(&regions, &text, &text, false);
    assert_eq!(page.mapping, TextMapping::Exact);
    assert_eq!(spanned(&text, page.words[1].span).as_deref(), Some("الله"));
    // A character outside the Basic Multilingual Plane is two UTF-16 units.
    let wide = &page.words[5];
    assert_eq!(wide.text, "𝔄");
    let span = wide.span.unwrap();
    assert_eq!(span.end - span.start, 2);
    assert_eq!(spanned(&text, page.words[6].span).as_deref(), Some("الصلاة"));
    // In the Markdown of a model that wraps it differently, each block is still found once.
    // A page number the model wrote that no block holds.
    let markdown = "## بسم الله الرحمن الرحيم\n\nكتاب 𝔄 الصلاة\n\n١٢";
    let blocks = vec![
        TextRegion::block(0.5, 0.1, 0.4, 0.05, "بسم الله الرحمن الرحيم"),
        TextRegion::block(0.5, 0.2, 0.4, 0.05, "كتاب 𝔄 الصلاة"),
    ];
    let page = PageRegions::of(&blocks, markdown, markdown, false);
    assert_eq!(page.mapping, TextMapping::Unique);
    assert_eq!(
        spanned(markdown, page.blocks[0].span).as_deref(),
        Some("بسم الله الرحمن الرحيم")
    );
}

#[test]
fn two_columns_are_placed_whatever_order_the_text_reads_them_in() {
    let left = TextRegion::block(0.05, 0.1, 0.42, 0.6, "La réunion tenue à la mosquée");
    let right = TextRegion::block(0.53, 0.1, 0.42, 0.6, "Le commandant de cercle assistait");
    let regions = vec![left, right];
    // Read across in the regions' order: exact.
    let text = "La réunion tenue à la mosquée\n\nLe commandant de cercle assistait";
    let page = PageRegions::of(&regions, text, text, false);
    assert_eq!(page.mapping, TextMapping::Exact);
    assert_eq!(
        spanned(text, page.blocks[1].span).as_deref(),
        Some("Le commandant de cercle assistait")
    );
    // The model's text gives the right column first: each block found once, where it stands.
    let swapped = "Le commandant de cercle assistait\n\nLa réunion tenue à la mosquée";
    let page = PageRegions::of(&regions, swapped, swapped, false);
    assert_eq!(page.mapping, TextMapping::Unique);
    assert_eq!(page.blocks[0].span, Some(TextSpan { start: 35, end: 64 }));
    assert_eq!(page.blocks[1].span, Some(TextSpan { start: 0, end: 33 }));
}

#[test]
fn html_tables_and_markdown_marks_are_left_out_of_the_comparison() {
    let model = "<table><tr><td>Nom</td><td>Lieu</td></tr><tr><td>Diallo</td><td>Kankan</td></tr></table>\n\n| a | b |\n|---|---|\n| **c** | d |";
    let regions = vec![
        TextRegion::block(
            0.1,
            0.1,
            0.8,
            0.2,
            plain_words(
                "<table><tr><td>Nom</td><td>Lieu</td></tr><tr><td>Diallo</td><td>Kankan</td></tr></table>",
            ),
        ),
        TextRegion::block(0.1, 0.4, 0.8, 0.2, "a b\nc d"),
    ];
    let page = PageRegions::of(&regions, model, model, false);
    let diallo = page.words.iter().find(|w| w.text == "Diallo").unwrap();
    assert_eq!(spanned(model, diallo.span).as_deref(), Some("Diallo"));
    // `**c**`: the emphasis is not part of the word's place.
    let c = page.words.iter().find(|w| w.text == "c").unwrap();
    assert_eq!(spanned(model, c.span).as_deref(), Some("c"));
    // A `<` in prose is not a tag.
    let prose = vec![tesseract_block(0.1, 0.1, &["a < b"])];
    let text = joined(&prose);
    let page = PageRegions::of(&prose, &text, &text, false);
    assert_eq!(page.mapping, TextMapping::Exact);
}

#[test]
fn a_block_placed_whole_gets_estimated_lines_and_words() {
    let region = TextRegion::block(0.1, 0.2, 0.4, 0.1, "Dakar, le\n3 mai");
    let lines = region.estimated_lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1].y, 0.25);
    assert_eq!(lines[0].words.len(), 2);
    assert_eq!(lines[0].words[0].x, 0.1);
    let last = lines[0].words.last().map(|w| w.x + w.width).unwrap_or(0.0);
    assert!((last - 0.5).abs() < 1e-5, "the words fill the line");
    let page = PageRegions::of(
        std::slice::from_ref(&region),
        "Dakar, le\n3 mai",
        "x",
        false,
    );
    assert_eq!(page.lines.len(), 2);
    assert!(page.lines[0].estimated && !page.blocks[0].estimated);
    assert_eq!((page.lines[1].x, page.lines[1].y), (0.1, 0.25));
}

#[test]
fn a_split_spread_keeps_its_half_s_fractions_and_exports_put_them_on_the_page() {
    // Read from the right half of a spread: fractions of that half, which the review shows alone.
    let region = tesseract_block(0.2, 0.1, &["Kankan"]);
    let page = PageRegions::of(std::slice::from_ref(&region), "Kankan", "Kankan", false);
    assert_eq!(page.blocks[0].x, 0.2);
    assert_eq!(page.words[0].x, 0.2);
    // On the whole page: the right half begins at the middle and is half as wide.
    let whole = region.onto_whole_page(Some(PageHalf::Right));
    assert!((whole.x - 0.6).abs() < 1e-6);
    assert!((whole.width - 0.2).abs() < 1e-6);
    assert!((whole.lines[0].words[0].x - 0.6).abs() < 1e-6);
    assert!((whole.lines[0].words[0].width - 0.025).abs() < 1e-6);
    assert_eq!(whole.y, region.y, "heights are the page's own");
    let left = region.onto_whole_page(Some(PageHalf::Left));
    assert!((left.x - 0.1).abs() < 1e-6);
    assert_eq!(region.onto_whole_page(None), region);
}

#[test]
fn a_page_without_regions_has_none() {
    let page = PageRegions::of(&[], "Une page.", "Une page.", false);
    assert!(page.blocks.is_empty() && page.lines.is_empty() && page.words.is_empty());
    assert!(PageRegions::none(true).corrected);
}
