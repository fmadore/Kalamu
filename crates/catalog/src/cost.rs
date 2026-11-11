//! What work is expected to cost before it starts, and what a call cost once answered, from
//! the catalogue's prices.

use kalamu_core::Task;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Catalog, ModelInfo};

/// Measured in iwac-ai-pipelines: a PDF page at Gemini's high media resolution.
const INPUT_TOKENS_PER_PAGE: f64 = 1_100.0;
/// A dense printed page transcribed in full.
const OUTPUT_TOKENS_PER_PAGE: f64 = 900.0;
/// Measured for gemini-3.5-transcribe; used for token-priced audio models.
const INPUT_TOKENS_PER_AUDIO_MINUTE: f64 = 1_500.0;
/// About 150 spoken words a minute.
const OUTPUT_TOKENS_PER_AUDIO_MINUTE: f64 = 250.0;
/// An analysis's answer (a summary of a few paragraphs, lists of names), with some thinking.
const ANALYSIS_OUTPUT_TOKENS: f64 = 1_500.0;
/// A page's entries for a table of contents: a few titles and short summaries.
const CONTENTS_OUTPUT_TOKENS_PER_PAGE: f64 = 500.0;
/// What the second step of a table of contents writes per page combined: about the length of
/// the consolidated summaries.
const MERGE_OUTPUT_TOKENS_PER_PAGE: f64 = 250.0;

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
    /// A rough cost for `pages` pages or `audio_minutes` minutes on one model.
    pub fn estimate(
        &self,
        model: &ModelInfo,
        task: Task,
        pages: u32,
        audio_minutes: f64,
        prompt_chars: usize,
    ) -> CostEstimate {
        let p = &model.pricing;
        let usd = match task {
            Task::TextExtraction => {
                let pages = f64::from(pages);
                if let Some(per_page) = p.per_page {
                    Some(pages * per_page)
                } else if let (Some(input), Some(output)) = (p.input_per_mtok, p.output_per_mtok) {
                    // The instructions are sent with every page.
                    let prompt_tokens = prompt_chars as f64 / 4.0;
                    let tokens_in = pages * (INPUT_TOKENS_PER_PAGE + prompt_tokens);
                    let tokens_out = pages * OUTPUT_TOKENS_PER_PAGE;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                } else {
                    None
                }
            }
            // Each page read for its entries, the instructions sent with every page.
            Task::Contents => match (p.input_per_mtok, p.output_per_mtok) {
                (Some(input), Some(output)) => {
                    let pages = f64::from(pages.max(1));
                    let prompt_tokens = prompt_chars as f64 / 4.0;
                    let tokens_in = pages * (INPUT_TOKENS_PER_PAGE + prompt_tokens);
                    let tokens_out = pages * CONTENTS_OUTPUT_TOKENS_PER_PAGE;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                }
                _ => None,
            },
            // Each page's text sent and written back corrected or translated, as JSON; a
            // translation runs a little longer than its source in most pairs of languages.
            Task::TextCorrection | Task::Translation => match (p.input_per_mtok, p.output_per_mtok)
            {
                (Some(input), Some(output)) => {
                    let pages = f64::from(pages.max(1));
                    let prompt_tokens = prompt_chars as f64 / 4.0;
                    let tokens_in = pages * (OUTPUT_TOKENS_PER_PAGE + prompt_tokens);
                    let longer = if task == Task::Translation { 1.25 } else { 1.1 };
                    let tokens_out = pages * OUTPUT_TOKENS_PER_PAGE * longer;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                }
                _ => None,
            },
            // An analysis reads the whole document once and writes a short answer.
            Task::Analysis => match (p.input_per_mtok, p.output_per_mtok) {
                (Some(input), Some(output)) => {
                    let pages = f64::from(pages.max(1));
                    let prompt_tokens = prompt_chars as f64 / 4.0;
                    let tokens_in = pages * INPUT_TOKENS_PER_PAGE + prompt_tokens;
                    Some((tokens_in * input + ANALYSIS_OUTPUT_TOKENS * output) / 1_000_000.0)
                }
                _ => None,
            },
            Task::AudioTranscription => {
                if let Some(per_minute) = p.per_minute {
                    Some(audio_minutes * per_minute)
                } else if let (Some(input), Some(output)) = (p.input_per_mtok, p.output_per_mtok) {
                    let tokens_in = audio_minutes * INPUT_TOKENS_PER_AUDIO_MINUTE;
                    let tokens_out = audio_minutes * OUTPUT_TOKENS_PER_AUDIO_MINUTE;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                } else {
                    None
                }
            }
        };
        CostEstimate {
            usd,
            economy_usd: usd
                .filter(|_| p.batch_discount.is_some())
                .map(|usd| usd * p.batch_factor(true)),
        }
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
                let tokens_in = pages * CONTENTS_OUTPUT_TOKENS_PER_PAGE + prompt_chars as f64 / 4.0;
                let tokens_out = pages * MERGE_OUTPUT_TOKENS_PER_PAGE;
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
