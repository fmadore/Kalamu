//! What work is expected to cost before it starts, and what a call cost once answered, from
//! the catalogue's prices.
//!
//! An estimate counts what a model writes and what it thinks before writing, which providers
//! bill as output: at the level Kalamu suggests (the lowest for reading a page, `low` for an
//! analysis), thinking is most of what a newspaper page costs. The figures were measured
//! against what jobs were billed in the 1.10.0 test passes (rc1 and rc2), and lean to the high
//! side; a model that thinks far from the allowance has its own measure in the catalogue
//! ([`ModelInfo::page_thinking`]).

use kalamu_core::{EmbeddingModel, Task};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CHARS_PER_TOKEN, Catalog, ModelInfo};

/// Measured in iwac-ai-pipelines: a PDF page at Gemini's high media resolution.
const INPUT_TOKENS_PER_PAGE: f64 = 1_100.0;
/// A page of small print in columns as sent: Gemini reads a newspaper PDF's text layer beside
/// its picture (about 3,400 tokens a page in the 1.10.0-rc2 test pass), GPT-6.2 Sol a large
/// picture in many tiles (about 11,000).
const DENSE_INPUT_TOKENS_PER_PAGE: f64 = 3_000.0;
/// The text of an ordinary page transcribed in full: a letter, a page of a book or a report.
const TEXT_TOKENS_PER_PAGE: f64 = 900.0;
/// The text of a page of small print in columns (a newspaper's) transcribed in full: a page of
/// the Daily Nebraskan of 1911 holds about 1,200 words, written with its headings marked.
const DENSE_TEXT_TOKENS_PER_PAGE: f64 = 2_000.0;
/// What a model thinks before it writes an ordinary page, at the lowest level it has that is
/// not "none" (a reasoning level's [`thinking`] scales it), unless the catalogue measured its
/// own ([`ModelInfo::page_thinking`]). Claude Sonnet 5.5 was billed about 1,500 tokens of output
/// for a handwritten page of 1755.
const THINKING_TOKENS_PER_PAGE: f64 = 600.0;
/// What a model thinks before it writes a page of small print in columns, at the same level:
/// Gemini 4 Hydrogen and GPT-6.2 Sol were each billed 4,000 to 5,500 tokens of output for a
/// newspaper page (36 pages for $1.21 on Hydrogen), about 2,000 of them its text.
const DENSE_THINKING_TOKENS_PER_PAGE: f64 = 2_500.0;
/// Measured for gemini-3.5-transcribe; used for token-priced audio models.
const INPUT_TOKENS_PER_AUDIO_MINUTE: f64 = 1_500.0;
/// About 150 spoken words a minute.
const OUTPUT_TOKENS_PER_AUDIO_MINUTE: f64 = 250.0;
/// An analysis's answer for one document: a summary of a few paragraphs, lists of names.
const ANALYSIS_ANSWER_TOKENS: f64 = 700.0;
/// What a model thinks before it answers an analysis of one document at `low`: screening ten
/// documents with Gemini 4 Hydrogen was billed about 1,500 tokens of output for each.
const ANALYSIS_THINKING_TOKENS: f64 = 800.0;
/// What each field whose entries describe and quote the text (an interview's themes, its
/// quotations worth keeping) adds to an analysis's answer: entries with fields of their own, and
/// quotations copied word for word. Gemini 4 Hydrogen wrote 2,100 to 2,500 tokens, its thinking
/// included, for an interview of 250 words, where 1,500 were counted.
const CITED_ENTRIES_TOKENS: f64 = 500.0;
/// The least share of a page a region read again is priced as: a strip of two lines still sends
/// the instructions, a picture and some thought.
const MIN_REGION_SHARE: f64 = 0.2;
/// A page's entries for a table of contents: a few titles and short summaries.
const CONTENTS_OUTPUT_TOKENS_PER_PAGE: f64 = 500.0;
/// A page's rows for a records analysis (a directory, a register: one row per entry), as JSON
/// with the columns' names in every row. With the thinking of an ordinary page, Gemini 4
/// Hydrogen's 900 tokens a page on a 4-page sample in the 1.8.0 test pass; a denser register
/// writes more.
const RECORDS_OUTPUT_TOKENS_PER_PAGE: f64 = 400.0;
/// What the second step of a table of contents writes per page combined: about the length of
/// the consolidated summaries.
const MERGE_OUTPUT_TOKENS_PER_PAGE: f64 = 250.0;

/// How much a reasoning level thinks, against the lowest level that thinks at all (`low`,
/// which the figures above were measured at): nothing for a model that does not reason or is
/// told not to, about half at `minimal`, and more at each level above. A level this build does
/// not know is taken for `medium`.
fn thinking(level: Option<&str>) -> f64 {
    match level {
        None | Some("none") => 0.0,
        Some("minimal") => 0.5,
        Some("low") => 1.0,
        Some("high") => 3.0,
        Some("xhigh") => 4.0,
        Some("max") => 5.0,
        // `medium`, or a level this build does not know.
        Some(_) => 2.0,
    }
}

/// What a job reads, as an estimate prices it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Workload<'a> {
    /// Pages read as pictures (scans, a PDF's pages); for a correction or a translation, pages
    /// of text.
    pub pages: u32,
    /// For an analysis, the characters of the texts read as text (a transcript, a corrected
    /// text), all documents together.
    pub text_chars: u64,
    /// For an analysis, the documents answered, each in a call of its own with the
    /// instructions; at least one.
    pub documents: u32,
    pub audio_minutes: f64,
    /// The instructions, sent with every page or document.
    pub prompt_chars: usize,
    /// Pages of small print in columns (a newspaper's), as the prompt says
    /// (`Preset::is_dense`): read page by page, they hold far more text.
    pub dense: bool,
    /// Of `pages`, those known to be small print from their text layer
    /// (`SourceFile::dense_pages`), priced as dense whatever the prompt.
    pub dense_pages: u32,
    /// For a region of a page read again, the share of the page it covers, from 0 to 1: its
    /// picture, text and thinking priced as that share of a page (at least
    /// [`MIN_REGION_SHARE`]); `None` for whole pages. A model priced by the page bills a whole
    /// one.
    pub region: Option<f64>,
    /// For an analysis, its fields whose entries describe and quote the text (an interview's
    /// themes and quotations; not a screening's passages, which are quotations alone), each
    /// writing a longer answer.
    pub cited_entries: u32,
    /// The reasoning level the set-up chose, in the provider's words; `None` for the level
    /// Kalamu suggests for the task.
    pub reasoning: Option<&'a str>,
}

impl ModelInfo {
    /// How much this model thinks for `task` at the `chosen` level (or the suggested one),
    /// against `low`: nothing for a model that does not reason.
    fn thinking_for(&self, task: Task, chosen: Option<&str>) -> f64 {
        thinking(
            self.reasoning
                .as_ref()
                .and_then(|r| r.level(task, chosen))
                .as_deref(),
        )
    }

    /// What it thinks before writing an ordinary page at `low`: its own measure, else the
    /// allowance.
    fn page_thinking_tokens(&self) -> f64 {
        self.page_thinking
            .map_or(THINKING_TOKENS_PER_PAGE, f64::from)
    }
}

/// What a job is expected to cost, before it starts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CostEstimate {
    /// `None` when the model has no published price in the catalogue.
    pub usd: Option<f64>,
    /// The same work in economy mode, through the provider's batch service; `None` when the
    /// model has none.
    #[serde(default)]
    pub economy_usd: Option<f64>,
}

impl Catalog {
    /// A rough cost for what one set-up reads on one model: pages, texts or minutes.
    pub fn estimate(&self, model: &ModelInfo, task: Task, work: &Workload<'_>) -> CostEstimate {
        let p = &model.pricing;
        let prompt_tokens = work.prompt_chars as f64 / CHARS_PER_TOKEN as f64;
        // What the model thinks before it writes, billed as output.
        let thinks = model.thinking_for(task, work.reasoning);
        let usd = match task {
            Task::TextExtraction => {
                let pages = f64::from(work.pages);
                if let Some(per_page) = p.per_page {
                    Some(pages * per_page)
                } else if let (Some(input), Some(output)) = (p.input_per_mtok, p.output_per_mtok) {
                    // Pages of small print, as the prompt or their text layer says; the others
                    // ordinary, or a share of one for a region.
                    let dense = if work.dense {
                        pages
                    } else {
                        f64::from(work.dense_pages.min(work.pages))
                    };
                    let ordinary = pages - dense;
                    let share = work.region.map_or(1.0, |s| s.clamp(MIN_REGION_SHARE, 1.0));
                    let page_thinking = model.page_thinking_tokens();
                    // The instructions are sent with every page.
                    let tokens_in = ordinary * (INPUT_TOKENS_PER_PAGE * share + prompt_tokens)
                        + dense * (DENSE_INPUT_TOKENS_PER_PAGE + prompt_tokens);
                    let tokens_out =
                        ordinary * (TEXT_TOKENS_PER_PAGE + page_thinking * thinks) * share
                            + dense
                                * (DENSE_TEXT_TOKENS_PER_PAGE
                                    + DENSE_THINKING_TOKENS_PER_PAGE.max(page_thinking) * thinks);
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                } else {
                    None
                }
            }
            // Each page read for its entries, the instructions sent with every page.
            Task::Contents => page_by_page(
                model,
                work.pages,
                work.prompt_chars,
                CONTENTS_OUTPUT_TOKENS_PER_PAGE + THINKING_TOKENS_PER_PAGE * thinks,
            ),
            // Each page's text sent and written back corrected or translated, as JSON; a
            // translation runs a little longer than its source in most pairs of languages.
            Task::TextCorrection | Task::Translation => match (p.input_per_mtok, p.output_per_mtok)
            {
                (Some(input), Some(output)) => {
                    let pages = f64::from(work.pages.max(1));
                    let tokens_in = pages * (TEXT_TOKENS_PER_PAGE + prompt_tokens);
                    let longer = if task == Task::Translation { 1.25 } else { 1.1 };
                    let tokens_out =
                        pages * (TEXT_TOKENS_PER_PAGE * longer + THINKING_TOKENS_PER_PAGE * thinks);
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                }
                _ => None,
            },
            // An analysis reads each document once, its pages as pictures and its text as
            // text, with the instructions, and writes a short answer for each; a records
            // analysis, which reads page by page, is priced by `estimate_records`.
            Task::Analysis => match (p.input_per_mtok, p.output_per_mtok) {
                (Some(input), Some(output)) => {
                    let documents = f64::from(work.documents.max(1));
                    let tokens_in = f64::from(work.pages) * INPUT_TOKENS_PER_PAGE
                        + work.text_chars as f64 / CHARS_PER_TOKEN as f64
                        + documents * prompt_tokens;
                    let tokens_out = documents * analysis_output(work.cited_entries, thinks);
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                }
                _ => None,
            },
            Task::AudioTranscription => {
                let minutes = work.audio_minutes;
                if let Some(per_minute) = p.per_minute {
                    Some(minutes * per_minute)
                } else if let (Some(input), Some(output)) = (p.input_per_mtok, p.output_per_mtok) {
                    let tokens_in = minutes * INPUT_TOKENS_PER_AUDIO_MINUTE;
                    let tokens_out = minutes * OUTPUT_TOKENS_PER_AUDIO_MINUTE;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                } else {
                    None
                }
            }
        };
        with_economy(model, usd)
    }

    /// A rough cost for a records analysis of `pages` pages (a directory or a register, one row
    /// per entry; the caller knows it by the analysis's fields) at the `reasoning` level chosen
    /// (`None`: the suggestion): unlike an analysis answered once per document, it reads each
    /// page on its own, the instructions sent with every page, and writes a page of rows for
    /// each.
    pub fn estimate_records(
        &self,
        model: &ModelInfo,
        pages: u32,
        prompt_chars: usize,
        reasoning: Option<&str>,
    ) -> CostEstimate {
        let thinks = model.thinking_for(Task::Analysis, reasoning);
        let usd = page_by_page(
            model,
            pages,
            prompt_chars,
            RECORDS_OUTPUT_TOKENS_PER_PAGE + THINKING_TOKENS_PER_PAGE * thinks,
        );
        with_economy(model, usd)
    }

    /// A rough cost for a table of contents' second step over `pages` pages: one call reading
    /// every page's entries and writing the combined ones. It runs at once, never in a batch.
    pub fn estimate_merge(
        &self,
        model: &ModelInfo,
        pages: u32,
        prompt_chars: usize,
    ) -> CostEstimate {
        let p = &model.pricing;
        let usd = match (p.input_per_mtok, p.output_per_mtok) {
            (Some(input), Some(output)) => {
                let pages = f64::from(pages.max(1));
                let tokens_in = pages * CONTENTS_OUTPUT_TOKENS_PER_PAGE
                    + prompt_chars as f64 / CHARS_PER_TOKEN as f64;
                // The model's thinking, once, at the level suggested for a table of contents.
                let tokens_out = pages * MERGE_OUTPUT_TOKENS_PER_PAGE
                    + ANALYSIS_THINKING_TOKENS * model.thinking_for(Task::Contents, None);
                Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
            }
            _ => None,
        };
        CostEstimate {
            usd,
            economy_usd: None,
        }
    }

    /// A rough cost for a text too long for its model read in `parts` parts, `chars` characters
    /// sent in all (the turns some parts repeat counted again), at the `reasoning` level chosen
    /// (`None`: the suggestion): each part sent with the instructions and answered as an
    /// analysis is, its fields of quoted entries counted ([`Workload::cited_entries`]). Never in
    /// a batch, as no cited analysis is.
    pub fn estimate_parts(
        &self,
        model: &ModelInfo,
        parts: u32,
        chars: u64,
        prompt_chars: usize,
        reasoning: Option<&str>,
        cited_entries: u32,
    ) -> CostEstimate {
        let p = &model.pricing;
        let usd = match (p.input_per_mtok, p.output_per_mtok) {
            (Some(input), Some(output)) => {
                let parts = f64::from(parts.max(1));
                let tokens_in =
                    (chars as f64 + parts * prompt_chars as f64) / CHARS_PER_TOKEN as f64;
                let tokens_out = parts
                    * analysis_output(cited_entries, model.thinking_for(Task::Analysis, reasoning));
                Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
            }
            _ => None,
        };
        CostEstimate {
            usd,
            economy_usd: None,
        }
    }

    /// A rough cost for the step that brings `parts` parts together, at the `reasoning` level
    /// chosen (`None`: the suggestion): one call reading every part's answer and writing the
    /// whole one, each as long as its fields of quoted entries make it.
    pub fn estimate_together(
        &self,
        model: &ModelInfo,
        parts: u32,
        prompt_chars: usize,
        reasoning: Option<&str>,
        cited_entries: u32,
    ) -> CostEstimate {
        let p = &model.pricing;
        let usd = match (p.input_per_mtok, p.output_per_mtok) {
            (Some(input), Some(output)) => {
                let tokens_in = f64::from(parts.max(1)) * analysis_output(cited_entries, 0.0)
                    + prompt_chars as f64 / CHARS_PER_TOKEN as f64;
                let tokens_out =
                    analysis_output(cited_entries, model.thinking_for(Task::Analysis, reasoning));
                Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
            }
            _ => None,
        };
        CostEstimate {
            usd,
            economy_usd: None,
        }
    }

    /// Cost of a finished call from its reported usage: cached input at its own price, and the
    /// batch discount on everything when the call went through a batch, as providers bill it.
    pub fn actual_cost(&self, model: &ModelInfo, usage: &kalamu_core::Usage) -> Option<f64> {
        let p = &model.pricing;
        let factor = p.batch_factor(usage.batch);
        if let Some(per_minute) = p.per_minute
            && usage.audio_seconds > 0.0
        {
            return Some(usage.audio_seconds / 60.0 * per_minute * factor);
        }
        if let Some(per_page) = p.per_page
            && usage.pages > 0
        {
            return Some(f64::from(usage.pages) * per_page * factor);
        }
        let (input, output) = (p.input_per_mtok?, p.output_per_mtok?);
        let cached = usage.cached_input_tokens.min(usage.input_tokens);
        let written = usage.cache_write_tokens.min(usage.input_tokens - cached);
        let plain = usage.input_tokens - cached - written;
        let tokens = plain as f64 * input
            + cached as f64 * p.cached_input_per_mtok.unwrap_or(input)
            + written as f64 * p.cache_write_per_mtok.unwrap_or(input)
            + usage.output_tokens as f64 * output;
        Some(tokens / 1_000_000.0 * factor)
    }
}

/// What embedding `input_tokens` tokens with `model` costs, in US dollars: nothing for a model
/// run on this computer, `None` for a cloud model with no published price.
pub fn embedding_cost(model: &EmbeddingModel, input_tokens: u64) -> Option<f64> {
    if model.local {
        return Some(0.0);
    }
    model
        .input_price
        .map(|per_mtok| input_tokens as f64 * per_mtok / 1_000_000.0)
}

/// Pages read one at a time, the instructions sent with each, and `output_per_page` tokens
/// written for each; `None` for a model not priced by the token.
fn page_by_page(
    model: &ModelInfo,
    pages: u32,
    prompt_chars: usize,
    output_per_page: f64,
) -> Option<f64> {
    let p = &model.pricing;
    let (input, output) = (p.input_per_mtok?, p.output_per_mtok?);
    let pages = f64::from(pages.max(1));
    let prompt_tokens = prompt_chars as f64 / CHARS_PER_TOKEN as f64;
    let tokens_in = pages * (INPUT_TOKENS_PER_PAGE + prompt_tokens);
    let tokens_out = pages * output_per_page;
    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
}

/// What an analysis writes for one document, its answer longer by each of its `cited_entries`
/// fields of quoted entries, thinking `thinks` times what it thinks at `low`.
fn analysis_output(cited_entries: u32, thinks: f64) -> f64 {
    ANALYSIS_ANSWER_TOKENS
        + CITED_ENTRIES_TOKENS * f64::from(cited_entries)
        + ANALYSIS_THINKING_TOKENS * thinks
}

/// `usd` at once, and in economy mode where the model has a batch service.
fn with_economy(model: &ModelInfo, usd: Option<f64>) -> CostEstimate {
    let p = &model.pricing;
    CostEstimate {
        usd,
        economy_usd: usd
            .filter(|_| p.batch_discount.is_some())
            .map(|usd| usd * p.batch_factor(true)),
    }
}
