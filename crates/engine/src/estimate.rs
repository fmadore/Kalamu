//! What a job will cost before it starts: each set-up's pages or minutes priced from the
//! catalogue, a records analysis page by page, a table of contents' second step apart, and the
//! totals at each pace; and, for a cited analysis, a text too long for its model.

use kalamu_catalog::CostEstimate;
use kalamu_core::{InputKind, MergeModel, ProviderId, SourceFile, Task};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, SetupRequest, TextTooLong};

/// Characters in a page of text to correct, for a text file without pages.
const TEXT_PAGE_CHARS: u64 = 2_500;
/// Instructions assumed for a preset that cannot be found.
const PROMPT_CHARS: usize = 2_000;

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct EstimateRequest {
    pub task: Task,
    pub sources: Vec<SourceFile>,
    pub setups: Vec<SetupRequest>,
    pub sample_pages: Option<u32>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EstimateRow {
    pub provider: ProviderId,
    pub model: String,
    pub pages: u32,
    pub estimate: CostEstimate,
    /// A table of contents' second step, which combines the pages.
    pub merge: bool,
    /// A cited analysis's text surely too long for this model to read at once, as the job would
    /// be refused for it when created ([`crate::Engine::check_length`]); absent otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub too_long: Option<TextTooLong>,
    /// For a cited analysis of texts too long for this model to read at once, how many parts
    /// they will be read in, all texts together (this row prices the parts, the row after it,
    /// `merge`, the step that brings them together); absent when every text is read whole.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub parts: Option<u32>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Estimate {
    pub rows: Vec<EstimateRow>,
    /// The whole job at once; `None` when a row has no price, or a video's length is not yet
    /// known.
    pub total_usd: Option<f64>,
    /// The whole job in economy mode, a model without a batch service at its own price.
    pub total_economy_usd: Option<f64>,
}

/// The texts of a cited analysis some of which are too long for its model: how many parts those
/// are read in, the characters the parts send, and the pages of the texts read whole.
#[derive(Debug, Clone, Copy)]
struct InParts {
    parts: u32,
    chars: u64,
    whole_pages: u32,
}

impl Engine {
    /// How a cited set-up would read its text sources, when some are read in parts; `None` when
    /// every text is read whole.
    fn in_parts(
        &self,
        catalog: &kalamu_catalog::Catalog,
        setup: &SetupRequest,
        sources: &[SourceFile],
    ) -> Option<InParts> {
        let model = catalog.model(&setup.provider, &setup.model)?;
        let windows = crate::windows::windows_for(model)?;
        let mut read = InParts {
            parts: 0,
            chars: 0,
            whole_pages: 0,
        };
        for source in sources {
            let parts = (source.kind == InputKind::Text)
                .then(|| self.quotable_text(source).ok())
                .flatten()
                .and_then(|text| {
                    let sizes = text.sizes();
                    let plan =
                        kalamu_core::analysis::cited::windows::plan(&sizes, &windows).ok()??;
                    let chars = plan
                        .iter()
                        .map(|&(first, last)| {
                            sizes[first as usize - 1..last as usize]
                                .iter()
                                .map(|&s| u64::from(s))
                                .sum::<u64>()
                        })
                        .sum::<u64>();
                    Some((plan.len() as u32, chars))
                });
            match parts {
                Some((parts, chars)) => {
                    read.parts += parts;
                    read.chars += chars;
                }
                None => read.whole_pages += (source.size_bytes / TEXT_PAGE_CHARS).max(1) as u32,
            }
        }
        (read.parts > 0).then_some(read)
    }

    pub fn estimate(&self, request: &EstimateRequest) -> Estimate {
        let catalog = self.current_catalog();
        let pages: u32 = request
            .sources
            .iter()
            .map(|s| {
                let n = match s.kind {
                    InputKind::Text => (s.size_bytes / TEXT_PAGE_CHARS).max(1) as u32,
                    _ => s.page_count.unwrap_or(1) * if s.split_spreads { 2 } else { 1 },
                };
                request.sample_pages.map_or(n, |cap| n.min(cap))
            })
            .sum();
        // A YouTube video's length is known only once the provider has read it.
        let unknown_length = request.sources.iter().any(|s| s.kind == InputKind::YouTube);
        let minutes: f64 = request
            .sources
            .iter()
            .map(|s| s.duration_ms.unwrap_or(0) as f64 / 60_000.0)
            .sum();
        let unpriced = CostEstimate {
            usd: None,
            economy_usd: None,
        };
        let mut rows = Vec::new();
        for setup in &request.setups {
            let preset = setup
                .preset_id
                .as_ref()
                .and_then(|id| self.with_prompts(|p| p.get(id)).ok());
            let prompt_chars = preset
                .as_ref()
                .map_or(PROMPT_CHARS, |p| p.instructions.len());
            // A records analysis (one row per entry) reads page by page, as the job will run it:
            // known by the preset's fields, as `reads_by_page` knows it.
            let records = request.task == Task::Analysis
                && preset
                    .as_ref()
                    .and_then(|p| p.schema.as_ref())
                    .is_some_and(kalamu_core::records::is_records);
            let estimate = match catalog.model(&setup.provider, &setup.model) {
                Some(model) if !unknown_length => {
                    let mut estimate = if records {
                        catalog.estimate_records(model, pages, prompt_chars)
                    } else {
                        catalog.estimate(model, request.task, pages, minutes, prompt_chars)
                    };
                    // Economy mode only where Kalamu sends this model's work as a batch.
                    let batches = self
                        .provider(&setup.provider)
                        .is_ok_and(|p| p.batches(request.task, &setup.model));
                    if !batches {
                        estimate.economy_usd = None;
                    }
                    estimate
                }
                _ => unpriced.clone(),
            };
            let cites = self.request_cites(request.task, setup).unwrap_or(false);
            let too_long = cites
                .then(|| self.text_too_long(&catalog, setup, &request.sources))
                .flatten();
            // Texts read in parts are priced part by part, and the step after them apart.
            let in_parts = cites
                .then(|| self.in_parts(&catalog, setup, &request.sources))
                .flatten()
                .filter(|_| too_long.is_none());
            let model = catalog.model(&setup.provider, &setup.model);
            let (estimate, together) = match (in_parts, model) {
                (Some(read), Some(model)) => {
                    // The texts that fit, read whole as any analysis is.
                    let whole = if read.whole_pages > 0 {
                        catalog
                            .estimate(model, request.task, read.whole_pages, 0.0, prompt_chars)
                            .usd
                    } else {
                        Some(0.0)
                    };
                    let parts = catalog.estimate_parts(model, read.parts, read.chars, prompt_chars);
                    let usd = whole.zip(parts.usd).map(|(a, b)| a + b);
                    (
                        CostEstimate {
                            usd,
                            economy_usd: None,
                        },
                        Some(catalog.estimate_together(model, read.parts, prompt_chars)),
                    )
                }
                _ => (estimate, None),
            };
            rows.push(EstimateRow {
                provider: setup.provider.clone(),
                model: setup.model.clone(),
                pages,
                estimate,
                merge: false,
                too_long,
                parts: in_parts.map(|r| r.parts),
            });
            if let Some(estimate) = together {
                rows.push(EstimateRow {
                    provider: setup.provider.clone(),
                    model: setup.model.clone(),
                    pages,
                    estimate,
                    merge: true,
                    too_long: None,
                    parts: None,
                });
            }
            // A table of contents' second step, on its own model and never in a batch.
            if request.task == Task::Contents {
                let merge = setup.merge.clone().unwrap_or_else(|| MergeModel {
                    provider: setup.provider.clone(),
                    model: setup.model.clone(),
                });
                let merge_chars = preset
                    .as_ref()
                    .and_then(|p| p.merge_instructions.as_ref())
                    .map_or(PROMPT_CHARS, String::len);
                let estimate = catalog.model(&merge.provider, &merge.model).map_or_else(
                    || unpriced.clone(),
                    |m| catalog.estimate_merge(m, pages, merge_chars),
                );
                rows.push(EstimateRow {
                    provider: merge.provider,
                    model: merge.model,
                    pages,
                    estimate,
                    merge: true,
                    too_long: None,
                    parts: None,
                });
            }
        }
        let total = |economy: bool| {
            rows.iter().try_fold(0.0, |sum, r| {
                let usd = if economy {
                    r.estimate.economy_usd.or(r.estimate.usd)
                } else {
                    r.estimate.usd
                };
                usd.map(|usd| sum + usd)
            })
        };
        Estimate {
            total_usd: total(false),
            total_economy_usd: total(true),
            rows,
        }
    }
}
