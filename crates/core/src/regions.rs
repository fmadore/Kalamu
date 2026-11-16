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
//! and marked so.
//!
//! # The place of a region's words in the text
//!
//! See [`TextMapping`]: a region is given a place in the text shown only where it is certain,
//! never guessed.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{PageHalf, TextLine, TextRegion, TextWord};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
mod text;

use text::Words;

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
    /// Fractions of the piece as shown; see the module's notes on coordinates.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// The region's own text, as the engine read it.
    pub text: String,
    /// Placed by Kalamu within a block the engine placed whole, not by the engine: approximate.
    pub estimated: bool,
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
        }
    }

    /// The regions an engine stored for a piece, at each granularity, each placed in `shown`
    /// (the text the review shows: the correction, else the model's text) where that place is
    /// certain. `model` is the model's own text; `corrected` whether `shown` is a correction.
    pub fn of(regions: &[TextRegion], model: &str, shown: &str, corrected: bool) -> Self {
        // Every region's words, normalised, in reading order; each region keeps the stretch of
        // them it holds.
        let mut tokens: Vec<String> = Vec::new();
        let mut blocks = Vec::new();
        let mut lines = Vec::new();
        let mut words = Vec::new();
        for region in regions {
            let block_start = tokens.len();
            let estimated = region.lines.is_empty();
            let placed = if estimated {
                region.estimated_lines()
            } else {
                region.lines.clone()
            };
            for line in &placed {
                let line_start = tokens.len();
                for word in &line.words {
                    let start = tokens.len();
                    tokens.extend(Words::of(&word.text).normal);
                    words.push((shown_word(word, estimated), start..tokens.len()));
                }
                lines.push((shown_line(line, estimated), line_start..tokens.len()));
            }
            let block = ShownRegion {
                x: region.x,
                y: region.y,
                width: region.width,
                height: region.height,
                text: region.text.clone(),
                estimated: false,
                span: None,
            };
            blocks.push((block, block_start..tokens.len()));
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
        }
    }
}

fn shown_line(line: &TextLine, estimated: bool) -> ShownRegion {
    ShownRegion {
        x: line.x,
        y: line.y,
        width: line.width,
        height: line.height,
        text: line.text(),
        estimated,
        span: None,
    }
}

fn shown_word(word: &TextWord, estimated: bool) -> ShownRegion {
    ShownRegion {
        x: word.x,
        y: word.y,
        width: word.width,
        height: word.height,
        text: word.text.clone(),
        estimated,
        span: None,
    }
}

impl TextRegion {
    /// A block placed whole, as lines sharing its height and words sharing each line's width
    /// by their length: what ALTO writes, marked estimated, and what the review draws, marked
    /// approximate.
    pub fn estimated_lines(&self) -> Vec<TextLine> {
        let texts: Vec<&str> = self
            .text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let height = self.height / texts.len().max(1) as f32;
        texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let y = self.y + i as f32 * height;
                let words: Vec<&str> = text.split_whitespace().collect();
                let total: usize = words.iter().map(|w| w.chars().count() + 1).sum::<usize>() - 1;
                let unit = self.width / total.max(1) as f32;
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
                    width: self.width,
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
