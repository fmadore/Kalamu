//! Where a page's text sits, as the review draws it over the scan (#84): the boxes an engine
//! stored ([`TextRegion`]) at each granularity the stored data gives, each with the place of its
//! words in the page's text where that place is certain.
//!
//! # Coordinates
//!
//! A box is given in fractions (0 to 1) of the width and height of the piece as the review shows
//! it, measured from its top-left corner. That piece is what `piece_preview` sends, which is the
//! piece as the engines read it: a PDF page with its own `/Rotate` applied, as drawn by pdf.js or
//! hayro; one half of a split spread as that half alone (a PDF page cut to it, an image cropped
//! to it); an image turned upright by its EXIF orientation. These are the fractions the engines
//! stored, unchanged: they do not depend on the resolution a PDF page was drawn at for Tesseract
//! (`page_resolution`), which only ALTO's pixels need, nor on the review's zoom. The reader's
//! quarter turns are laid over them as the scan is drawn turned. Exports put a spread's halves
//! back on the whole page ([`TextRegion::onto_whole_page`]) and measure pixels at the recorded
//! resolution; the review needs neither.
//!
//! # Granularity
//!
//! Blocks are always the engine's own (Tesseract's paragraphs, Mistral OCR's and LightOnOCR's
//! blocks). Lines and words are the engine's where it placed them (Tesseract); within a block
//! placed whole they are estimated, as ALTO estimates them ([`TextRegion::estimated_lines`]),
//! their origin [`RegionOrigin::Estimated`]: the block's text wrapped into as many lines as its
//! shape on the page holds, which needs the page's proportions (`page_aspect`). Without them
//! such a block is drawn alone. A region the researcher draws is a block
//! ([`Granularity::Block`]).
//!
//! A box the researcher draws or adjusts is in the same fractions, of the page or of the half
//! the review shows. Pointer arithmetic may overshoot an edge by a hair: up to [`SLACK`] is
//! clamped to the page, more is refused, and so is a box smaller than [`MIN_SIZE`] either way.
//!
//! # Identities
//!
//! Each region an engine stored has an id that stays the same for one stored result: `r{n}` for
//! block n, `r{n}_l{m}` for its line m and `r{n}_l{m}_w{k}` for that line's word k, all counted
//! from 1 in reading order ([`block_id`], [`line_id`], [`word_id`]). PAGE XML writes the same ids
//! for a whole page; a split spread's halves are numbered each on its own here, as the review
//! shows them, and across the page there. A result read again is another revision, named by its
//! cache key ([`PageRegions::result`]): its ids say nothing about the old one's.
//!
//! # The researcher's regions
//!
//! Regions the researcher draws, and the boxes they give an engine's regions, are kept apart
//! from the stored result, which never changes ([`DrawnRegion`]).
//!
//! # The place of a region's words in the text
//!
//! See [`TextMapping`]: a region is given a place in the text shown only where it is certain,
//! never guessed.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{PageHalf, TextLine, TextRegion, TextWord};

mod drawn;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
mod text;

pub use drawn::{AdjustedRegion, DrawnRegion, MIN_SIZE, RegionDraft, RegionError, SLACK, check};
use text::Words;

/// The id of an engine's block `block` (from 1) on a page.
pub fn block_id(block: usize) -> String {
    format!("r{block}")
}

/// The id of line `line` (from 1) of block `block`.
pub fn line_id(block: usize, line: usize) -> String {
    format!("r{block}_l{line}")
}

/// The id of word `word` (from 1) of line `line` of block `block`.
pub fn word_id(block: usize, line: usize, word: usize) -> String {
    format!("r{block}_l{line}_w{word}")
}

/// What a region outlines: a block of text, one of its lines, or one word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Granularity {
    Block,
    Line,
    Word,
}

/// Who placed a region. More will come (regions imported from ALTO or PAGE XML, a model's
/// reading aligned on an engine's geometry), so code outside this crate cannot take the list
/// for closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
#[non_exhaustive]
pub enum RegionOrigin {
    /// The engine placed it (Tesseract's lines and words, Mistral OCR's blocks).
    Provider,
    /// Kalamu placed it within a block the engine placed whole, as ALTO estimates lines:
    /// approximate.
    Estimated,
    /// The researcher drew it, or gave an engine's region this box.
    Manual,
}

/// A stretch of the text the review shows, in UTF-16 code units, as the interface's strings
/// count them; `end` is past the last unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TextSpan {
    pub start: u32,
    pub end: u32,
}

/// A block, line or word drawn over the scan in the review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ShownRegion {
    /// An engine's region: its place in the result ([`block_id`]), the same whenever that
    /// result is shown. The researcher's own: the id it was saved under.
    pub id: String,
    /// Fractions of the piece as shown; see the module's notes on coordinates.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// The region's own text, as the engine read it, or as the researcher gave it.
    pub text: String,
    pub granularity: Granularity,
    /// Who placed its box: the engine, Kalamu within a block placed whole (approximate), or the
    /// researcher.
    pub origin: RegionOrigin,
    /// Where its words stand in the text shown, only where that place is certain.
    pub span: Option<TextSpan>,
}

/// How regions were given their place in the text shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TextMapping {
    /// The text shown is the regions' words, in their order (Tesseract's text, uncorrected): each
    /// region's place follows from its position, a repeated phrase included.
    Exact,
    /// The text shown differs from the regions' words (Markdown around them, or a correction): a
    /// region has a place only where its words stand exactly once in the model's text and once
    /// in the text shown. A repeated phrase, or words a correction changed, have none.
    Unique,
}

/// One piece's regions at each granularity, in reading order, for the review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PageRegions {
    pub blocks: Vec<ShownRegion>,
    pub lines: Vec<ShownRegion>,
    pub words: Vec<ShownRegion>,
    pub mapping: TextMapping,
    /// The text shown is the researcher's correction.
    pub corrected: bool,
    /// The cache key of the stored result the engine's regions come from: the revision their
    /// ids belong to. None for a piece not read yet, or a review with no text piece.
    pub result: Option<String>,
    /// The researcher's own regions on this page: drawn where the engine placed nothing they
    /// adjust, or adjusting a region of a reading of the page that is no longer shown.
    pub drawn: Vec<ShownRegion>,
}

impl PageRegions {
    /// None: a result whose engine placed no text, or a piece not read yet.
    pub fn none(corrected: bool) -> Self {
        Self {
            blocks: Vec::new(),
            lines: Vec::new(),
            words: Vec::new(),
            mapping: TextMapping::Exact,
            corrected,
            result: None,
            drawn: Vec::new(),
        }
    }

    /// The regions an engine stored for a piece, at each granularity, each placed in `shown`
    /// (the text the review shows: the correction, else the model's text) where that place is
    /// certain. `model` is the model's own text; `corrected` whether `shown` is a correction.
    /// `page_aspect` is the piece's height over its width as shown, which a block placed whole
    /// needs for its estimated lines ([`TextRegion::estimated_lines`]); without it such a block
    /// is drawn alone, with no lines or words, rather than with lines guessed from nothing.
    pub fn of(
        regions: &[TextRegion],
        model: &str,
        shown: &str,
        corrected: bool,
        page_aspect: Option<f32>,
    ) -> Self {
        // Every region's words, normalised, in reading order; each region keeps the stretch of
        // them it holds.
        let mut tokens: Vec<String> = Vec::new();
        let mut blocks = Vec::new();
        let mut lines = Vec::new();
        let mut words = Vec::new();
        for (b, region) in regions.iter().enumerate() {
            let block = b + 1;
            let block_start = tokens.len();
            let estimated = region.lines.is_empty();
            let placed = match (estimated, page_aspect) {
                (false, _) => region.lines.clone(),
                (true, Some(aspect)) => region.estimated_lines(aspect),
                (true, None) => {
                    // The block's words still hold their place in the text, for the block.
                    for word in region.text.split_whitespace() {
                        tokens.extend(Words::of(word).normal);
                    }
                    Vec::new()
                }
            };
            let origin = if estimated {
                RegionOrigin::Estimated
            } else {
                RegionOrigin::Provider
            };
            for (l, line) in placed.iter().enumerate() {
                let line_start = tokens.len();
                for (w, word) in line.words.iter().enumerate() {
                    let start = tokens.len();
                    tokens.extend(Words::of(&word.text).normal);
                    let id = word_id(block, l + 1, w + 1);
                    words.push((shown_word(id, word, origin), start..tokens.len()));
                }
                let id = line_id(block, l + 1);
                lines.push((shown_line(id, line, origin), line_start..tokens.len()));
            }
            let shown = ShownRegion {
                id: block_id(block),
                x: region.x,
                y: region.y,
                width: region.width,
                height: region.height,
                text: region.text.clone(),
                granularity: Granularity::Block,
                origin: RegionOrigin::Provider,
                span: None,
            };
            blocks.push((shown, block_start..tokens.len()));
        }

        let page = Words::of(shown);
        let exact = page.normal == tokens;
        let model_words = if exact { None } else { Some(Words::of(model)) };
        let page_index = page.index();
        let model_index = model_words.as_ref().map(Words::index);
        let place = |range: std::ops::Range<usize>| -> Option<TextSpan> {
            if range.is_empty() {
                return None;
            }
            if exact {
                return page.span(range.start, range.end);
            }
            let run = tokens.get(range)?;
            let once_in_model = model_index.as_ref()?.places(run, 2).len() == 1;
            let at = page_index.places(run, 2);
            match (once_in_model, at.as_slice()) {
                (true, [only]) => page.span(*only, only + run.len()),
                _ => None,
            }
        };
        let placed = |list: Vec<(ShownRegion, std::ops::Range<usize>)>| -> Vec<ShownRegion> {
            list.into_iter()
                .map(|(region, range)| ShownRegion {
                    span: place(range),
                    ..region
                })
                .collect()
        };
        Self {
            blocks: placed(blocks),
            lines: placed(lines),
            words: placed(words),
            mapping: if exact {
                TextMapping::Exact
            } else {
                TextMapping::Unique
            },
            corrected,
            result: None,
            drawn: Vec::new(),
        }
    }
}

fn shown_line(id: String, line: &TextLine, origin: RegionOrigin) -> ShownRegion {
    ShownRegion {
        id,
        x: line.x,
        y: line.y,
        width: line.width,
        height: line.height,
        text: line.text(),
        granularity: Granularity::Line,
        origin,
        span: None,
    }
}

fn shown_word(id: String, word: &TextWord, origin: RegionOrigin) -> ShownRegion {
    ShownRegion {
        id,
        x: word.x,
        y: word.y,
        width: word.width,
        height: word.height,
        text: word.text.clone(),
        granularity: Granularity::Word,
        origin,
        span: None,
    }
}

/// A character's advance, spaces counted, as a share of the text's height: about half in book
/// and newspaper faces.
const ADVANCE: f32 = 0.5;
/// The distance from one line to the next, as a share of the text's height.
const LEADING: f32 = 1.2;

impl TextRegion {
    /// A block placed whole, as estimated lines and words: what ALTO writes, marked estimated,
    /// and what the review draws, marked approximate. `page_aspect` is the height over the
    /// width of the page the region's fractions measure (a split spread's half alone, as the
    /// review shows it; the whole page, as ALTO writes it): it gives the block its shape.
    ///
    /// The block is taken as filled with text of one size, whose characters advance half their
    /// height (`ADVANCE`) and whose lines stand 1.2 heights apart (`LEADING`). `n`
    /// characters filling `L` lines of a block `W` wide and `H` tall put `n / L` characters on
    /// a line, each `ADVANCE × g` wide for a text height `g = H / (LEADING × L)`; so
    /// `n / L × ADVANCE × H / (LEADING × L) = W`, and `L = √(n × ADVANCE × H / (LEADING × W))`.
    /// A line of the text at least one and a half times as long as the block's lines is a
    /// paragraph written on one line (Mistral OCR writes Markdown paragraphs): it is wrapped by
    /// its characters into as many lines as its length fills. A shorter line stays as the engine
    /// broke it, so text whose own lines fit the block is laid out line for line. The lines
    /// share the block's height and their words a line's width by their length, a wrapped
    /// paragraph's last line ending where its words do. No text, no lines.
    pub fn estimated_lines(&self, page_aspect: f32) -> Vec<TextLine> {
        let texts: Vec<Vec<&str>> = self
            .text
            .lines()
            .map(|l| l.split_whitespace().collect::<Vec<_>>())
            .filter(|words| !words.is_empty())
            .collect();
        let total: usize = texts.iter().map(|words| chars(words)).sum();
        let per_line = chars_per_line(total, self.width, self.height * page_aspect);
        // The rows, each with the width of one character: a wrapped line's rows share that of
        // its longest row, so its last row ends early.
        let mut rows: Vec<(Vec<&str>, f32)> = Vec::new();
        for words in &texts {
            let length = chars(words);
            let count = per_line
                .map_or(1, |c| (length as f32 / c).round() as usize)
                .clamp(1, words.len());
            let wrapped = wrap(words, count, length);
            let longest = wrapped.iter().map(|row| chars(row)).max().unwrap_or(1);
            let unit = self.width / longest.max(1) as f32;
            rows.extend(wrapped.into_iter().map(|row| (row, unit)));
        }
        let height = self.height / rows.len().max(1) as f32;
        rows.iter()
            .enumerate()
            .map(|(i, (words, unit))| {
                let unit = *unit;
                let y = self.y + i as f32 * height;
                let extent = (chars(words) as f32 * unit).min(self.width);
                let mut offset = 0.0;
                let words = words
                    .iter()
                    .map(|w| {
                        let width = w.chars().count() as f32 * unit;
                        let word = TextWord {
                            x: self.x + offset,
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
                    x: self.x,
                    y,
                    width: extent,
                    height,
                    words,
                }
            })
            .collect()
    }

    /// The region placed on the whole page it was read from: a half of a split spread is
    /// stored in fractions of that half (the left half from 0 to 0.5 of the page's width, the
    /// right from 0.5 to 1); a whole page's region is unchanged. What the searchable PDF and ALTO
    /// write; the review shows the half alone and needs none of it.
    pub fn onto_whole_page(&self, half: Option<PageHalf>) -> TextRegion {
        let mut r = self.clone();
        let start = match half {
            Some(PageHalf::Left) => 0.0,
            Some(PageHalf::Right) => 0.5,
            None => return r,
        };
        let halve = |x: &mut f32, width: &mut f32| {
            *x = start + *x * 0.5;
            *width *= 0.5;
        };
        halve(&mut r.x, &mut r.width);
        for line in &mut r.lines {
            halve(&mut line.x, &mut line.width);
            for word in &mut line.words {
                halve(&mut word.x, &mut word.width);
            }
        }
        r
    }
}

/// The characters of `words` written with one space between them.
fn chars(words: &[&str]) -> usize {
    words
        .iter()
        .map(|w| w.chars().count() + 1)
        .sum::<usize>()
        .saturating_sub(1)
}

/// How many characters a line holds in a block `width` wide and `height` tall (in one unit)
/// that `total` characters fill; see [`TextRegion::estimated_lines`]. None for a block without
/// a shape, whose text then keeps its own lines.
fn chars_per_line(total: usize, width: f32, height: f32) -> Option<f32> {
    let shaped = width > 0.0 && height > 0.0 && width.is_finite() && height.is_finite();
    if total == 0 || !shaped {
        return None;
    }
    let lines = (total as f32 * ADVANCE * height / (LEADING * width)).sqrt();
    Some(total as f32 / lines.max(1.0))
}

/// `words`, `length` characters in all, as `count` rows of about equal length: each word in the
/// row its middle falls in, so no row is ever empty and there are never more than `count`.
fn wrap<'a>(words: &[&'a str], count: usize, length: usize) -> Vec<Vec<&'a str>> {
    let mut rows: Vec<Vec<&'a str>> = Vec::new();
    let mut current = None;
    let mut offset = 0;
    for &word in words {
        let n = word.chars().count();
        let middle = offset as f32 + n as f32 / 2.0;
        let row =
            ((middle * count as f32 / length.max(1) as f32) as usize).min(count.saturating_sub(1));
        if current != Some(row) {
            rows.push(Vec::new());
            current = Some(row);
        }
        if let Some(last) = rows.last_mut() {
            last.push(word);
        }
        offset += n + 1;
    }
    rows
}
