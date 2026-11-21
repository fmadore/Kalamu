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
    /// A cited analysis's texts this model cannot read even in parts, one passage of each alone
    /// too long, in the order given: a job holding one is refused when created
    /// ([`crate::Engine::check_length`]), so this row prices the other sources only, as a job
    /// that leaves them out would cost; absent when every text can be read.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<TextTooLong>>")]
    pub too_long: Vec<TextTooLong>,
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
/// are read in, the characters the parts send, and the sources read whole.
#[derive(Debug, Clone)]
struct InParts {
    parts: u32,
    chars: u64,
    whole: Vec<SourceFile>,
}

/// What sources give an estimate to price, as the job reads them.
#[derive(Debug, Clone, Copy, Default)]
struct Read {
    /// Pages as New job counts them: a text's pages of [`TEXT_PAGE_CHARS`].
    pages: u32,
    /// For an analysis, the pages read as pictures, and the characters of the texts read as
    /// text; for any other task, every page, a text's included.
    picture_pages: u32,
    text_chars: u64,
    documents: u32,
}

/// What `sources` give `task` to read, each capped at `sample_pages` when the job tries a few
/// pages first.
fn read_of(task: Task, sources: &[SourceFile], sample_pages: Option<u32>) -> Read {
    let mut read = Read::default();
    for s in sources {
        let n = match s.kind {
            InputKind::Text => (s.size_bytes / TEXT_PAGE_CHARS).max(1) as u32,
            _ => s.page_count.unwrap_or(1) * if s.split_spreads { 2 } else { 1 },
        };
        let n = sample_pages.map_or(n, |cap| n.min(cap));
        read.pages += n;
        read.documents += 1;
        if task == Task::Analysis && s.kind == InputKind::Text {
            let cap = sample_pages.map_or(u64::MAX, |cap| u64::from(cap) * TEXT_PAGE_CHARS);
            read.text_chars += s.size_bytes.min(cap);
        } else {
            read.picture_pages += n;
        }
    }
    read
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
            whole: Vec::new(),
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
                None => read.whole.push(source.clone()),
            }
        }
        (read.parts > 0).then_some(read)
    }

    pub fn estimate(&self, request: &EstimateRequest) -> Estimate {
        self.estimate_beyond(request, 0)
    }

    /// [`Engine::estimate`], with `cached` of the pages already read under the same key and
    /// taken from the cache at no cost: the rest of a sample ([`crate::rest`]). A table of
    /// contents still combines every page.
    pub(crate) fn estimate_beyond(&self, request: &EstimateRequest, cached: u32) -> Estimate {
        let catalog = self.current_catalog();
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
            // A cited analysis's texts its model cannot read even in parts are left out of the
            // price, as a job that leaves them out would cost.
            let cites = self.request_cites(request.task, setup).unwrap_or(false);
            let unreadable = if cites {
                self.too_long_at(&catalog, setup, &request.sources)
            } else {
                Vec::new()
            };
            let sources: Vec<SourceFile> = request
                .sources
                .iter()
                .enumerate()
                .filter(|(at, _)| !unreadable.iter().any(|(u, _)| u == at))
                .map(|(_, s)| s.clone())
                .collect();
            let too_long: Vec<TextTooLong> = unreadable.into_iter().map(|(_, l)| l).collect();
            let read = read_of(request.task, &sources, request.sample_pages);
            let all_pages = read.pages;
            let pages = all_pages.saturating_sub(cached);
            let preset = setup
                .preset_id
                .as_ref()
                .and_then(|id| self.with_prompts(|p| p.get(id)).ok());
            let prompt_chars = preset
                .as_ref()
                .map_or(PROMPT_CHARS, |p| p.instructions.len());
            let reasoning = setup.options.reasoning.as_deref();
            // What the set-up reads, the pages already read taken from the cache left out.
            let work = |read: Read, cached: u32| kalamu_catalog::Workload {
                pages: read.picture_pages.saturating_sub(cached),
                text_chars: read.text_chars,
                documents: read.documents,
                audio_minutes: minutes,
                prompt_chars,
                // Pages full of small print hold far more text, and take far more thought.
                dense: preset.as_ref().is_some_and(|p| p.is_dense()),
                reasoning,
            };
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
                        catalog.estimate_records(model, pages, prompt_chars, reasoning)
                    } else {
                        catalog.estimate(model, request.task, &work(read, cached))
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
            // Texts read in parts are priced part by part, and the step after them apart.
            let in_parts = cites
                .then(|| self.in_parts(&catalog, setup, &sources))
                .flatten();
            let model = catalog.model(&setup.provider, &setup.model);
            let parts = in_parts.as_ref().map(|r| r.parts);
            let (estimate, together) = match (in_parts, model) {
                (Some(split), Some(model)) => {
                    // The texts that fit, read whole as any analysis is.
                    let whole = if split.whole.is_empty() {
                        Some(0.0)
                    } else {
                        let whole = read_of(request.task, &split.whole, request.sample_pages);
                        catalog.estimate(model, request.task, &work(whole, 0)).usd
                    };
                    let parts = catalog.estimate_parts(
                        model,
                        split.parts,
                        split.chars,
                        prompt_chars,
                        reasoning,
                    );
                    let usd = whole.zip(parts.usd).map(|(a, b)| a + b);
                    (
                        CostEstimate {
                            usd,
                            economy_usd: None,
                        },
                        Some(catalog.estimate_together(
                            model,
                            split.parts,
                            prompt_chars,
                            reasoning,
                        )),
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
                parts,
            });
            if let Some(estimate) = together {
                rows.push(EstimateRow {
                    provider: setup.provider.clone(),
                    model: setup.model.clone(),
                    pages,
                    estimate,
                    merge: true,
                    too_long: Vec::new(),
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
                    |m| catalog.estimate_merge(m, all_pages, merge_chars),
                );
                rows.push(EstimateRow {
                    provider: merge.provider,
                    model: merge.model,
                    pages: all_pages,
                    estimate,
                    merge: true,
                    too_long: Vec::new(),
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
