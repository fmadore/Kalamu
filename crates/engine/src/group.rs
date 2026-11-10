//! Several pages of a PDF read in one call, when the researcher asks for it (text extraction,
//! sent at once rather than as a batch).
//!
//! Pages stay pages: each is planned, recorded, stored, reviewed and exported as before. Only
//! the sending changes. Consecutive pages still to read are sent as one PDF, with an
//! instruction to mark where each begins; the answer is split at the marks and each page
//! stored as its own result, the call's cost on the first. A page whose mark is missing, or
//! that the answer may have cut short at the model's output limit, is read again on its own:
//! nothing is guessed, and the review and error rates stay page by page.
//!
//! How many pages go together is capped so that their text fits in what the model may write in
//! one answer, with room for its thinking: a dense newspaper page counts for more than a
//! letter's.

use std::sync::{Arc, Mutex};

use kalamu_core::{
    CacheKey, InputKind, Job, Pace, PieceRef, PieceState, PieceStatus, ProcessOutput,
    RequestSetting, Setup, SourceFile, StopSignal, pages,
};
use kalamu_media::OpenSource;

use crate::piece::{Called, Input, Outcome, finish, preview};
use crate::plan::Work;
use crate::{Engine, EngineError};

/// Words a page of each kind is expected to take in the answer, in tokens: a dense page (a
/// newspaper's, or any preset marked dense), and anything else (a letter, a report, a
/// register).
const NEWSPAPER_PAGE_TOKENS: u32 = 4_000;
const PAGE_TOKENS: u32 = 1_500;
/// The share of the output limit the pages' text may fill; the rest is room for thinking and
/// for pages denser than expected.
const OUTPUT_SHARE: (u32, u32) = (3, 5);
/// For a model whose provider documents no output limit.
const UNKNOWN_OUTPUT: u32 = 16_000;

/// Pages of one set-up gathered for a call, in order.
type Gathering = (Setup, Vec<(PieceState, CacheKey)>);

/// What the run loop sends: one piece, or several pages of one set-up together.
pub(crate) enum Unit {
    One(Work),
    Group(Setup, Vec<(PieceState, CacheKey)>),
}

/// The most pages `setup`'s model reads in one call: as many as its answer has room for, within
/// the provider's own limit on pages; one for a model that takes no instructions or no PDF.
/// `dense` is the preset's hint that its pages carry much text (a newspaper's).
pub(crate) fn pages_cap(catalog: &kalamu_catalog::Catalog, setup: &Setup, dense: bool) -> u32 {
    let Some(model) = catalog.model(&setup.provider, &setup.model) else {
        return 1;
    };
    if !model.accepts_prompt || !model.inputs.iter().any(|i| i == "application/pdf") {
        return 1;
    }
    let per_page = if dense {
        NEWSPAPER_PAGE_TOKENS
    } else {
        PAGE_TOKENS
    };
    let output = model.limits.max_output_tokens.unwrap_or(UNKNOWN_OUTPUT);
    let fits = (output / OUTPUT_SHARE.1 * OUTPUT_SHARE.0 / per_page).max(1);
    fits.min(model.limits.max_pages.unwrap_or(u32::MAX))
}

/// The page number of a whole page; none for a half of a spread or another kind of piece.
fn whole_page(piece: &PieceRef) -> Option<u32> {
    match piece {
        PieceRef::Page { number, half: None } => Some(*number),
        _ => None,
    }
}

impl Engine {
    /// How many pages of `source` the set-up reads in one call: what the job asks for, capped
    /// by what the model can write in one answer. One for anything but a PDF read for its
    /// text, at once, by a model that takes instructions and reads PDFs. A sample reads its
    /// pages as the whole run would, so the run that follows finds them in the cache.
    pub(crate) fn pages_per_call(&self, job: &Job, source: &SourceFile, setup: &Setup) -> u32 {
        let Some(asked) = job.settings.pages_per_call else {
            return 1;
        };
        if !job.task.groups_pages()
            || job.settings.pace == Pace::Economy
            || source.kind != InputKind::Pdf
            || source.split_spreads
        {
            return 1;
        }
        // Settled when the job was created; worked out again only for a job made before, with
        // the density of the preset it names as it is now.
        let cap = setup.pages_cap.unwrap_or_else(|| {
            let dense = setup.prompt.preset_id.as_deref().is_some_and(|id| {
                self.with_prompts(|p| p.get(id))
                    .is_ok_and(|preset| preset.is_dense())
            });
            pages_cap(&self.current_catalog(), setup, dense)
        });
        let pages = source.page_count.unwrap_or(1);
        asked.asked(pages).min(cap).min(pages).max(1)
    }

    /// The work of a source as it is sent: consecutive whole pages of a set-up that reads
    /// several at once are grouped, up to its size; everything else goes alone.
    pub(crate) fn units(&self, job: &Job, source: &SourceFile, work: Vec<Work>) -> Vec<Unit> {
        let mut units = Vec::new();
        let mut open: Option<Gathering> = None;
        let close = |open: &mut Option<Gathering>, units: &mut Vec<Unit>| {
            if let Some((setup, mut items)) = open.take() {
                if items.len() == 1 {
                    let (state, key) = items.remove(0);
                    units.push(Unit::One((setup, state, key)));
                } else {
                    units.push(Unit::Group(setup, items));
                }
            }
        };
        for (setup, state, key) in work {
            let size = self.pages_per_call(job, source, &setup);
            let page = whole_page(&state.piece);
            let (Some(page), true) = (page, size > 1) else {
                close(&mut open, &mut units);
                units.push(Unit::One((setup, state, key)));
                continue;
            };
            let follows = open.as_ref().is_some_and(|(s, items)| {
                s.id == setup.id
                    && items.len() < size as usize
                    && items
                        .last()
                        .and_then(|(last, _)| whole_page(&last.piece))
                        .is_some_and(|last| last + 1 == page)
            });
            if !follows {
                close(&mut open, &mut units);
                open = Some((setup, Vec::new()));
            }
            if let Some((_, items)) = open.as_mut() {
                items.push((state, key));
            }
        }
        close(&mut open, &mut units);
        units
    }

    /// Reads a group of consecutive pages in one call, and each page the answer does not give
    /// on its own.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_group(
        &self,
        job: &Job,
        source: &SourceFile,
        setup: &Setup,
        items: Vec<(PieceState, CacheKey)>,
        opened: Arc<OpenSource>,
        spent: Arc<Mutex<f64>>,
        stop: &StopSignal,
    ) -> Result<Outcome, EngineError> {
        let step = setup.step();
        let mut outcomes = Vec::new();
        // Pages already answered come from the cache, as they would alone.
        let mut left = Vec::new();
        for (state, key) in items {
            if self.store.cached_result(&key)?.is_some() {
                let input = Input::Open(opened.clone());
                outcomes.push(
                    self.process_piece(job, source, step, state, key, input, spent.clone(), stop)
                        .await?,
                );
            } else {
                left.push((state, key));
            }
        }
        let numbers: Vec<u32> = left
            .iter()
            .filter_map(|(s, _)| whole_page(&s.piece))
            .collect();
        let consecutive =
            numbers.len() == left.len() && numbers.windows(2).all(|w| w[0] + 1 == w[1]);
        let (Some(&first), Some(&last), true) = (numbers.first(), numbers.last(), consecutive)
        else {
            return self
                .one_by_one(job, source, setup, left, &opened, &spent, stop, outcomes)
                .await;
        };
        if first == last {
            return self
                .one_by_one(job, source, setup, left, &opened, &spent, stop, outcomes)
                .await;
        }

        let accepted = self.accepted(job, step)?;
        let stem = crate::source_stem(&source.file_name, "pages");
        let prepared = {
            let opened = opened.clone();
            tokio::task::spawn_blocking(move || opened.prepare_pages(first, last, &accepted, &stem))
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?
        };
        let limit = self
            .current_catalog()
            .model(&setup.provider, &setup.model)
            .and_then(|m| m.limits.max_request_mb)
            .map(|mb| mb as usize * 1024 * 1024);
        let input = match prepared {
            Ok(input) if limit.is_none_or(|limit| input.bytes.len() <= limit) => input,
            // Too large to send together, or not to be cut: each page alone.
            _ => {
                return self
                    .one_by_one(job, source, setup, left, &opened, &spent, stop, outcomes)
                    .await;
            }
        };

        let provider = self.provider(&setup.provider)?;
        let api_key = self.api_key(setup)?;
        for (state, _) in &mut left {
            state.status = PieceStatus::Running;
            self.save(state, None)?;
        }
        let user_prompt = format!(
            "{}\n\n{}",
            step.prompt.user_prompt,
            pages::instruction(first, last)
        );
        let mut lead = left[0].0.clone();
        let called = self
            .call(
                job,
                step,
                &provider,
                &api_key,
                &mut lead,
                &input,
                &user_prompt,
                &setup.options,
                &spent,
                stop,
            )
            .await?;
        match called {
            Called::Answer {
                output,
                mut sent,
                duration_ms,
            } => {
                sent.push(RequestSetting::new("pages", format!("{first}-{last}")));
                let mut found = pages::split(&output.text, first, last);
                // Cut off at the output limit: the last page found may be incomplete.
                if output.truncated {
                    found.pop();
                }
                let mut missing = Vec::new();
                let mut usage = Some(output.usage.clone());
                for (mut state, key) in left {
                    let page = whole_page(&state.piece);
                    let Some((_, text)) = found.iter().find(|(n, _)| Some(*n) == page) else {
                        missing.push((state, key));
                        continue;
                    };
                    // The call's usage, and so its cost, goes with the first page stored.
                    let part = finish(
                        job.task,
                        ProcessOutput {
                            text: text.clone(),
                            usage: usage.take().unwrap_or_default(),
                            model_version: output.model_version.clone(),
                            detected_language: output.detected_language.clone(),
                            ..ProcessOutput::default()
                        },
                    );
                    let cost =
                        self.store_answer(&key, step, part.clone(), sent.clone(), duration_ms)?;
                    state.attempts = lead.attempts;
                    state.status = PieceStatus::Done;
                    state.error = None;
                    self.save(&mut state, Some(preview(&part.text)))?;
                    self.add_cost(job.id, &spent, cost);
                    outcomes.push(Outcome::Finished);
                }
                // No page could be kept (the marks out of order, or the answer cut off after
                // its first page): the call was billed all the same.
                if let Some(usage) = &usage {
                    self.bill(&lead, &setup.provider, &setup.model, usage, &spent)?;
                }
                if !missing.is_empty() {
                    tracing::info!(
                        pages = missing.len(),
                        "pages not marked in the answer: read again one by one"
                    );
                }
                self.one_by_one(job, source, setup, missing, &opened, &spent, stop, outcomes)
                    .await
            }
            Called::Interrupted => {
                for (mut state, _) in left {
                    state.status = PieceStatus::Pending;
                    state.error = None;
                    self.save(&mut state, None)?;
                }
                outcomes.push(Outcome::Interrupted);
                Ok(worst(outcomes))
            }
            Called::Fatal(err) => {
                for (mut state, _) in left {
                    state.status = PieceStatus::Failed;
                    state.error = Some(err.to_string());
                    self.save(&mut state, None)?;
                }
                outcomes.push(Outcome::Fatal(err.to_string()));
                Ok(worst(outcomes))
            }
            // The pages together could not be read: each is tried on its own, with its own
            // retries, before any is marked as failed.
            Called::Failed(_) => {
                self.one_by_one(job, source, setup, left, &opened, &spent, stop, outcomes)
                    .await
            }
        }
    }

    /// Reads each page alone, after `outcomes`, and returns how the whole group ended.
    #[allow(clippy::too_many_arguments)]
    async fn one_by_one(
        &self,
        job: &Job,
        source: &SourceFile,
        setup: &Setup,
        items: Vec<(PieceState, CacheKey)>,
        opened: &Arc<OpenSource>,
        spent: &Arc<Mutex<f64>>,
        stop: &StopSignal,
        mut outcomes: Vec<Outcome>,
    ) -> Result<Outcome, EngineError> {
        for (state, key) in items {
            if stop.is_stopped() {
                let mut state = state;
                state.status = PieceStatus::Pending;
                self.save(&mut state, None)?;
                outcomes.push(Outcome::Interrupted);
                continue;
            }
            let input = Input::Open(opened.clone());
            let outcome = self
                .process_piece(
                    job,
                    source,
                    setup.step(),
                    state,
                    key,
                    input,
                    spent.clone(),
                    stop,
                )
                .await?;
            outcomes.push(outcome);
        }
        Ok(worst(outcomes))
    }
}

/// How a group ended, by its worst page: the key failing, then a stop, then a page not read.
fn worst(outcomes: Vec<Outcome>) -> Outcome {
    let mut out = Outcome::Finished;
    for outcome in outcomes {
        out = match (out, outcome) {
            (Outcome::Fatal(m), _) | (_, Outcome::Fatal(m)) => Outcome::Fatal(m),
            (Outcome::Interrupted, _) | (_, Outcome::Interrupted) => Outcome::Interrupted,
            (Outcome::Failed, _) | (_, Outcome::Failed) => Outcome::Failed,
            _ => Outcome::Finished,
        };
    }
    out
}
