//! A corrected page keeps its words' places: the researcher's text is laid back onto the boxes
//! of the words the engine read, so ALTO and the searchable PDF stay true after a correction.
//!
//! Ported from iwac-ai-pipelines' `02_correct_alto_xml.py`, which corrects ALTO line by line and
//! writes the corrected words back into the original `String` elements, leaving their positions
//! untouched. Here the correction is a whole page typed by the researcher rather than a model's
//! line, so the words are matched by a diff instead of line by line: a word kept keeps its box,
//! a word changed takes the box of the word it replaces, words added share the box of the word
//! before them, and a word removed leaves its box empty.

use kalamu_core::export::RegionPath;
use kalamu_core::{TextLine, TextRegion, TextWord};
use similar::{Algorithm, DiffOp, capture_diff_slices};

/// The most corrected words one engine word's box may take. More means the researcher added
/// text the engine never placed (a skipped column, a rewritten passage): its words would be
/// slivers of one box, presented as measured, so the page is placed as a block instead.
const MOST_PER_BOX: usize = 3;

/// The regions with `corrected` laid onto their words, or `None` when the engine placed blocks
/// only (Mistral OCR), read no words, or the correction adds more than the engine's boxes can
/// honestly hold, for the caller to place the text over the whole page, marked as estimated.
#[cfg(test)]
pub(crate) fn realign(regions: &[TextRegion], corrected: &str) -> Option<Vec<TextRegion>> {
    realign_traced(regions, corrected).map(|r| r.regions)
}

/// Where a corrected line came from: the place of the line, and for each of its words the place
/// of the word whose box it took.
type LineFrom = (usize, Vec<usize>);

/// Corrected regions, and where each of their blocks, lines and words came from in the regions
/// realigned: so a box the researcher gave one of those follows it into the corrected page.
pub(crate) struct Realigned {
    pub regions: Vec<TextRegion>,
    /// For each block of `regions`, its place in the regions given; for each of its lines, the
    /// place of the line it was there; for each of a line's words, the place of the word whose
    /// box it took (several words may share one, a word split by the correction).
    from: Vec<(usize, Vec<LineFrom>)>,
}

impl Realigned {
    /// The places in `regions` of what stood at `old` before: none where the correction removed
    /// it, several for a word it split.
    pub(crate) fn places(&self, old: RegionPath) -> Vec<RegionPath> {
        let mut out = Vec::new();
        for (b, (block, lines)) in self.from.iter().enumerate() {
            if *block != old.block {
                continue;
            }
            let Some(old_line) = old.line else {
                out.push(RegionPath {
                    block: b,
                    line: None,
                    word: None,
                });
                continue;
            };
            for (l, (line, words)) in lines.iter().enumerate() {
                if *line != old_line {
                    continue;
                }
                match old.word {
                    None => out.push(RegionPath {
                        block: b,
                        line: Some(l),
                        word: None,
                    }),
                    Some(old_word) => out.extend(
                        words
                            .iter()
                            .enumerate()
                            .filter(|(_, w)| **w == old_word)
                            .map(|(w, _)| RegionPath {
                                block: b,
                                line: Some(l),
                                word: Some(w),
                            }),
                    ),
                }
            }
        }
        out
    }
}

/// [`realign`], with where each block, line and word came from.
pub(crate) fn realign_traced(regions: &[TextRegion], corrected: &str) -> Option<Realigned> {
    if regions.iter().any(|r| r.lines.is_empty()) {
        return None;
    }
    let old: Vec<&str> = regions
        .iter()
        .flat_map(|r| &r.lines)
        .flat_map(|l| &l.words)
        .map(|w| w.text.as_str())
        .collect();
    if old.is_empty() {
        return None;
    }
    let new: Vec<&str> = corrected.split_whitespace().collect();

    // The corrected words each original word now holds.
    let mut held: Vec<Vec<&str>> = vec![Vec::new(); old.len()];
    let mut before_first: Vec<&str> = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &old, &new) {
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                for i in 0..len {
                    held[old_index + i].push(new[new_index + i]);
                }
            }
            DiffOp::Delete { .. } => {}
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                let added = &new[new_index..new_index + new_len];
                match old_index.checked_sub(1) {
                    Some(previous) => held[previous].extend_from_slice(added),
                    None => before_first.extend_from_slice(added),
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                // Shared in order, earlier boxes first, so a word put back together from
                // pieces sits where it began.
                for i in 0..old_len {
                    let start = (i * new_len).div_ceil(old_len);
                    let end = ((i + 1) * new_len).div_ceil(old_len);
                    held[old_index + i].extend_from_slice(&new[new_index + start..new_index + end]);
                }
            }
        }
    }
    if !before_first.is_empty() {
        before_first.append(&mut held[0]);
        held[0] = before_first;
    }
    if held.iter().any(|h| h.len() > MOST_PER_BOX) {
        return None;
    }

    let mut held = held.into_iter();
    let mut out = Vec::new();
    let mut from = Vec::new();
    for (b, region) in regions.iter().enumerate() {
        let mut lines = Vec::new();
        let mut lines_from = Vec::new();
        for (l, line) in region.lines.iter().enumerate() {
            let mut words = Vec::new();
            let mut words_from = Vec::new();
            for (w, word) in line.words.iter().enumerate() {
                let tokens = held.next().unwrap_or_default();
                let placed = place(word, &tokens);
                words_from.extend(std::iter::repeat_n(w, placed.len()));
                words.extend(placed);
            }
            if let Some(line) = fit_line(words) {
                lines.push(line);
                lines_from.push((l, words_from));
            }
        }
        if lines.is_empty() {
            continue;
        }
        from.push((b, lines_from));
        let text = lines
            .iter()
            .map(TextLine::text)
            .collect::<Vec<_>>()
            .join("\n");
        out.push(TextRegion {
            text,
            lines,
            ..region.clone()
        });
    }
    Some(Realigned { regions: out, from })
}

/// The words `tokens` set in `word`'s box: the word itself, unchanged; one corrected word in
/// its place; several sharing its width by their length, in reading order.
fn place(word: &TextWord, tokens: &[&str]) -> Vec<TextWord> {
    match tokens {
        [] => Vec::new(),
        [only] => vec![TextWord {
            text: (*only).to_owned(),
            // The engine's confidence was in what it read, not in the correction.
            confidence: word.confidence.filter(|_| *only == word.text),
            ..word.clone()
        }],
        many => {
            // Each word's characters and the space after it.
            let total: usize = many.iter().map(|t| t.chars().count() + 1).sum::<usize>() - 1;
            let unit = word.width / total.max(1) as f32;
            let rtl = many.iter().any(|t| is_rtl(t));
            let mut offset = 0.0;
            many.iter()
                .map(|t| {
                    let width = t.chars().count() as f32 * unit;
                    let x = if rtl {
                        word.x + word.width - offset - width
                    } else {
                        word.x + offset
                    };
                    offset += width + unit;
                    TextWord {
                        x,
                        y: word.y,
                        width,
                        height: word.height,
                        text: (*t).to_owned(),
                        confidence: None,
                    }
                })
                .collect()
        }
    }
}

/// A line drawn around its remaining words, or none when all were removed.
fn fit_line(words: Vec<TextWord>) -> Option<TextLine> {
    let first = words.first()?;
    let (mut x0, mut y0) = (first.x, first.y);
    let (mut x1, mut y1) = (first.x + first.width, first.y + first.height);
    for w in &words {
        x0 = x0.min(w.x);
        y0 = y0.min(w.y);
        x1 = x1.max(w.x + w.width);
        y1 = y1.max(w.y + w.height);
    }
    Some(TextLine {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
        words,
    })
}

/// Written right to left: Hebrew, Arabic, Syriac, Thaana, N'Ko and their presentation forms.
fn is_rtl(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}'))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn word(x: f32, text: &str) -> TextWord {
        TextWord {
            x,
            y: 0.1,
            width: 0.1,
            height: 0.02,
            text: text.into(),
            confidence: Some(0.9),
        }
    }

    fn page(lines: &[&[(f32, &str)]]) -> Vec<TextRegion> {
        let lines: Vec<TextLine> = lines
            .iter()
            .enumerate()
            .map(|(i, words)| {
                let words: Vec<TextWord> = words
                    .iter()
                    .map(|&(x, t)| TextWord {
                        y: 0.1 + i as f32 * 0.05,
                        ..word(x, t)
                    })
                    .collect();
                fit_line(words).unwrap()
            })
            .collect();
        let text = lines
            .iter()
            .map(TextLine::text)
            .collect::<Vec<_>>()
            .join("\n");
        vec![TextRegion {
            lines,
            ..TextRegion::block(0.0, 0.0, 1.0, 1.0, text)
        }]
    }

    fn words(regions: &[TextRegion]) -> Vec<(String, f32, Option<f32>)> {
        regions
            .iter()
            .flat_map(|r| &r.lines)
            .flat_map(|l| &l.words)
            .map(|w| (w.text.clone(), w.x, w.confidence))
            .collect()
    }

    #[test]
    fn a_corrected_word_takes_the_box_of_the_word_it_replaces() {
        let regions = page(&[&[(0.1, "Dakar,"), (0.3, "le"), (0.5, "3"), (0.7, "rnai")]]);
        let out = realign(&regions, "Dakar, le 3 mai").unwrap();
        assert_eq!(
            words(&out),
            vec![
                ("Dakar,".into(), 0.1, Some(0.9)),
                ("le".into(), 0.3, Some(0.9)),
                ("3".into(), 0.5, Some(0.9)),
                // Corrected: the box is kept, the engine's confidence is not.
                ("mai".into(), 0.7, None),
            ]
        );
        assert_eq!(out[0].text, "Dakar, le 3 mai");
    }

    #[test]
    fn words_split_share_the_box_and_words_removed_leave_it() {
        let regions = page(&[
            &[(0.1, "Lagouverneur"), (0.4, "a"), (0.6, "re~u")],
            &[(0.1, "~~")],
        ]);
        let out = realign(&regions, "La gouverneur a reçu").unwrap();
        let w = words(&out);
        assert_eq!(w[0].0, "La");
        assert_eq!(w[1].0, "gouverneur");
        assert!(w[0].1 == 0.1 && w[1].1 > w[0].1 && w[1].1 < 0.2);
        assert_eq!(w[3].0, "reçu");
        // The second line held only noise, now removed: it is gone.
        assert_eq!(out[0].lines.len(), 1);
    }

    #[test]
    fn words_added_go_with_the_word_before_them_or_the_first() {
        let regions = page(&[&[(0.1, "le"), (0.5, "mai")]]);
        let out = realign(&regions, "Dakar, le 3 mai 1957").unwrap();
        let texts: Vec<String> = words(&out).into_iter().map(|w| w.0).collect();
        assert_eq!(texts, ["Dakar,", "le", "3", "mai", "1957"]);
    }

    #[test]
    fn right_to_left_words_are_split_from_the_right() {
        let regions = page(&[&[(0.5, "بسمالله")]]);
        let out = realign(&regions, "بسم الله").unwrap();
        let w = words(&out);
        assert!(w[0].1 > w[1].1, "the first word sits to the right");
    }

    #[test]
    fn text_the_engine_never_placed_is_left_to_the_caller() {
        // A column the engine skipped, typed in by the researcher: its words would all be
        // squeezed into the box of the word before them.
        let regions = page(&[&[(0.1, "Le"), (0.3, "cercle"), (0.5, "de"), (0.7, "Dakar.")]]);
        let added = "Le cercle de Dakar. Le commandant de cercle a réuni les notables du quartier.";
        assert!(realign(&regions, added).is_none());
        // A page rewritten in other words, five for each the engine read.
        let regions = page(&[&[(0.1, "xxxx")]]);
        assert!(realign(&regions, "une phrase toute neuve ici").is_none());
    }

    #[test]
    fn each_block_line_and_word_says_where_it_came_from() {
        let regions = page(&[
            &[(0.1, "Lagouverneur"), (0.4, "a")],
            &[(0.1, "~~")],
            &[(0.1, "fin")],
        ]);
        let out = realign_traced(&regions, "La gouverneur a fin").unwrap();
        let at = |block, line, word| RegionPath { block, line, word };
        assert_eq!(out.places(at(0, None, None)), [at(0, None, None)]);
        // The word split in two: both halves came from it.
        assert_eq!(
            out.places(at(0, Some(0), Some(0))),
            [at(0, Some(0), Some(0)), at(0, Some(0), Some(1))]
        );
        assert_eq!(
            out.places(at(0, Some(0), Some(1))),
            [at(0, Some(0), Some(2))]
        );
        // The second line is gone; the third is now the second.
        assert!(out.places(at(0, Some(1), None)).is_empty());
        assert_eq!(out.places(at(0, Some(2), None)), [at(0, Some(1), None)]);
        assert!(out.places(at(1, None, None)).is_empty());
    }

    #[test]
    fn blocks_without_words_are_left_to_the_caller() {
        let regions = vec![TextRegion::block(0.1, 0.1, 0.8, 0.1, "Dakar")];
        assert!(realign(&regions, "Dakar").is_none());
    }
}
