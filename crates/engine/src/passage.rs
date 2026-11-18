//! A passage the researcher selected in the review, corrected by a model on request: the
//! correction task's request (its preset frozen as a job's would be, its one field, the
//! provider's retries) sent for the passage and a little context, recorded in the store before
//! and after the call, its answer cached under a key of its own and offered as a proposal. Only
//! the researcher's acceptance writes it, into their corrections of that piece, in the same
//! transaction as the record that says a model proposed it; the model's output is never
//! changed.
//!
//! No job is made: a passage belongs to the result reviewed, and its cost to that result's
//! job. The job's settings give the languages; the job's task need only write text.

use std::sync::Arc;
use std::time::Duration;

use kalamu_core::passage_correction::{self as passage, PassageError};
use kalamu_core::{
    ApiKey, CacheKey, Correction, Job, JobSettings, LineBreaks, PassageAsk, PassageProposal,
    PieceState, PreparedInput, ProposalStatus, SetupRequest, StopSignal, StoreError, Task,
    Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::control::lock;
use crate::piece::Called;
use crate::{Engine, EngineError};

/// What asking about a passage, or for a region to be read again, would cost, before it is
/// asked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PassageEstimate {
    /// About what the call costs at the model's prices; none for a model without one, or
    /// when the answer is already stored.
    pub usd: Option<f64>,
    /// Read on this computer: nothing is sent.
    pub offline: bool,
    /// The same passage, context, model and prompt (or the same picture of a region, model and
    /// prompt) were asked about before: the stored answer is offered again, for nothing.
    pub cached: bool,
}

/// The passage's request, answered or not, before it is recorded.
struct Prepared {
    job: Job,
    state: PieceState,
    setup: kalamu_core::Setup,
    input: String,
    key: CacheKey,
}

impl Engine {
    /// Checks a passage against the piece's text as the review shows it now, and freezes the
    /// correction preset for the model chosen, as a job would.
    fn prepare_passage(&self, ask: &PassageAsk) -> Result<Prepared, EngineError> {
        passage::check(ask).map_err(EngineError::Passage)?;
        let job = self.store.job(ask.job_id)?;
        // Only a text can be corrected: an analysis's answers or a table of contents' entries
        // are corrected field by field.
        if matches!(job.task, Task::Analysis | Task::Contents) {
            return Err(EngineError::NotText);
        }
        let state = self
            .store
            .source_pieces(ask.job_id, ask.setup_id, ask.source_id)?
            .into_iter()
            .find(|p| p.index == ask.index && p.status.is_finished())
            .ok_or(EngineError::NothingRead)?;
        let text = self.shown_text(&state)?;
        if passage::locate(&text, ask).is_none() {
            return Err(EngineError::Passage(PassageError::Changed));
        }
        let catalog = self.current_catalog();
        let task = Task::TextCorrection;
        if let Some(model) = catalog.model(&ask.provider, &ask.model) {
            if model.retired {
                return Err(EngineError::RetiredModel(model.name.clone()));
            }
            if !model.does(task) {
                return Err(EngineError::UnsupportedModel(ask.model.clone()));
            }
        }
        if self.accepted_for(task, ask)?.is_empty() {
            return Err(EngineError::UnsupportedModel(ask.model.clone()));
        }
        // The lines of the passage are kept, so that it fits back where it was taken from.
        let settings = JobSettings {
            languages: job.settings.languages.clone(),
            line_breaks: Some(LineBreaks::AsOnPage),
            ..JobSettings::default()
        };
        let request = SetupRequest {
            provider: ask.provider.clone(),
            model: ask.model.clone(),
            preset_id: ask.preset_id.clone(),
            label: None,
            options: Default::default(),
            merge: None,
        };
        let mut setup = self
            .freeze_setups(&catalog, task, &settings, std::slice::from_ref(&request))?
            .into_iter()
            .next()
            .ok_or(EngineError::EmptyJob)?;
        setup.prompt.user_prompt = if setup.prompt.user_prompt.trim().is_empty() {
            passage::REQUEST.to_string()
        } else {
            format!(
                "{}\n\n{}",
                setup.prompt.user_prompt.trim_end(),
                passage::REQUEST
            )
        };
        let input = passage::input(&ask.before, &ask.passage, &ask.after);
        let key = CacheKey::passage(&setup, &settings.languages, &input);
        Ok(Prepared {
            job,
            state,
            setup,
            input,
            key,
        })
    }

    /// The formats the provider takes for a correction from this model: text alone.
    fn accepted_for(&self, task: Task, ask: &PassageAsk) -> Result<Vec<&'static str>, EngineError> {
        let provider = self.provider(&ask.provider)?;
        let catalog = self.current_catalog();
        let model = catalog.model(&ask.provider, &ask.model);
        Ok(crate::piece::accepted_inputs(
            provider.as_ref(),
            model,
            task,
        ))
    }

    /// A finished piece's text as the review shows it: the correction, else the model's, with
    /// the speakers' names.
    pub(crate) fn shown_text(&self, state: &PieceState) -> Result<String, EngineError> {
        let corrections =
            self.store
                .source_corrections(state.job_id, state.setup_id, state.source_id)?;
        let speakers = kalamu_core::SpeakerIndex::new(self.store.source_speaker_names(
            state.job_id,
            state.setup_id,
            state.source_id,
        )?);
        Ok(self
            .store
            .text(state, &corrections, &speakers)?
            .unwrap_or_default())
    }

    /// What asking would cost: the passage, its context and the prompt read, the passage
    /// written back, at the model's prices; nothing when the answer is stored already.
    pub fn passage_estimate(&self, ask: &PassageAsk) -> Result<PassageEstimate, EngineError> {
        let prepared = self.prepare_passage(ask)?;
        let offline = !self.provider(&ask.provider)?.needs_key();
        if self.store.cached_result(&prepared.key)?.is_some() {
            return Ok(PassageEstimate {
                usd: None,
                offline,
                cached: true,
            });
        }
        let catalog = self.current_catalog();
        let usd = catalog.model(&ask.provider, &ask.model).and_then(|m| {
            let (input, output) = (m.pricing.input_per_mtok?, m.pricing.output_per_mtok?);
            let read = (prepared.setup.prompt.text.len()
                + prepared.setup.prompt.user_prompt.len()
                + prepared.input.len()) as f64
                / 4.0;
            // Written back as JSON, a little longer than read.
            let written = ask.passage.len() as f64 / 4.0 * 1.1 + 50.0;
            Some((read * input + written * output) / 1_000_000.0)
        });
        Ok(PassageEstimate {
            usd: usd.filter(|_| !offline),
            offline,
            cached: false,
        })
    }

    /// Asks the chosen model to correct a passage, through the correction task's request, and
    /// gives back the proposal: recorded before the call and after it; taken from the cache
    /// when the same was asked before; refused when the answer is empty or too different from
    /// the passage, and billed all the same. Ends early with the proposal cancelled when
    /// [`Engine::cancel_passage`] is called meanwhile; an answer that arrives after is still
    /// stored, so asking again costs nothing.
    pub async fn ask_passage(&self, ask: PassageAsk) -> Result<PassageProposal, EngineError> {
        let Prepared {
            job,
            state,
            setup,
            input,
            key,
        } = self.prepare_passage(&ask)?;
        let provider = self.provider(&ask.provider)?;
        let api_key = if provider.needs_key() {
            self.keys
                .key(&ask.provider)
                .ok_or_else(|| EngineError::MissingKey(ask.provider.to_string()))?
        } else {
            ApiKey::new("")
        };
        provider
            .check_ready(&setup.model, &job.settings.languages)
            .map_err(EngineError::NotReady)?;

        let mut proposal = PassageProposal {
            ask: ask.clone(),
            status: ProposalStatus::Asked,
            prompt: setup.prompt.clone(),
            request: Vec::new(),
            cache_key: key.0.clone(),
            answer: None,
            notes: Vec::new(),
            cost_usd: None,
            cached: false,
            error: None,
            accepted: None,
            asked_at: Timestamp::now(),
            answered_at: None,
            decided_at: None,
        };
        self.store.put_passage(&proposal, None)?;

        if let Some(stored) = self.store.cached_result(&key)? {
            proposal.cached = true;
            proposal.request = stored.request.clone();
            self.answered(&mut proposal, &stored.output.text);
            self.store.put_passage(&proposal, None)?;
            return Ok(proposal);
        }

        let stop = StopSignal::default();
        lock(&self.passages).insert(ask.id.clone(), stop.clone());
        let me = self
            .me
            .upgrade()
            .ok_or_else(|| EngineError::Output("the engine is closing".into()))?;
        let call = {
            let stop = stop.clone();
            let ask = ask.clone();
            tokio::spawn(async move {
                me.call_passage(job, state, setup, &ask, input, key, api_key, &stop)
                    .await
            })
        };
        let ended = tokio::select! {
            joined = call => Some(joined),
            () = stopped(&stop) => None,
        };
        lock(&self.passages).remove(&ask.id);
        proposal.answered_at = Some(Timestamp::now());
        match ended {
            None => {
                proposal.status = ProposalStatus::Cancelled;
            }
            Some(Err(e)) => {
                proposal.status = ProposalStatus::Failed;
                proposal.error = Some(e.to_string());
            }
            Some(Ok(Err(e))) => {
                proposal.status = ProposalStatus::Failed;
                proposal.error = Some(e.to_string());
            }
            Some(Ok(Ok(PassageCall::Interrupted))) => {
                proposal.status = ProposalStatus::Cancelled;
            }
            Some(Ok(Ok(PassageCall::Failed(reason)))) => {
                proposal.status = ProposalStatus::Failed;
                proposal.error = Some(reason);
            }
            Some(Ok(Ok(PassageCall::Answer {
                text,
                notes,
                cost,
                request,
            }))) => {
                proposal.status = ProposalStatus::Answered;
                proposal.answer = Some(text);
                proposal.notes = notes;
                proposal.cost_usd = cost;
                proposal.request = request;
            }
        }
        self.store.put_passage(&proposal, None)?;
        Ok(proposal)
    }

    /// Reads a stored answer into the proposal: offered when it passes, failed when not.
    fn answered(&self, proposal: &mut PassageProposal, answer: &str) {
        let ask = &proposal.ask;
        proposal.answered_at = Some(Timestamp::now());
        match passage::read_answer(&ask.before, &ask.passage, &ask.after, answer) {
            Ok((text, notes)) => {
                proposal.status = ProposalStatus::Answered;
                proposal.answer = Some(text);
                proposal.notes = notes;
            }
            Err(e) => {
                proposal.status = ProposalStatus::Failed;
                proposal.error = Some(e.to_string());
            }
        }
    }

    /// The call itself, with the provider's retries: its answer checked, then stored under the
    /// passage's key with its cost; refused and billed when it is not a correction of the
    /// passage.
    #[allow(clippy::too_many_arguments)]
    async fn call_passage(
        &self,
        mut job: Job,
        mut state: PieceState,
        setup: kalamu_core::Setup,
        ask: &PassageAsk,
        input: String,
        key: CacheKey,
        api_key: ApiKey,
        stop: &StopSignal,
    ) -> Result<PassageCall, EngineError> {
        job.task = Task::TextCorrection;
        let provider = self.provider(&setup.provider)?;
        let prepared = PreparedInput {
            piece: state.piece.clone(),
            mime: "text/plain".into(),
            bytes: input.into_bytes(),
            file_name: "passage.txt".into(),
        };
        let spent = Arc::new(std::sync::Mutex::new(self.store.spent(job.id)?));
        let called = self
            .call(
                &job,
                setup.step(),
                &provider,
                &api_key,
                &mut state,
                &prepared,
                &setup.prompt.user_prompt,
                &setup.options,
                &spent,
                stop,
                None,
            )
            .await?;
        match called {
            Called::Answer {
                output,
                sent,
                duration_ms,
            } => {
                let output = crate::piece::finish(Task::TextCorrection, output);
                match passage::read_answer(&ask.before, &ask.passage, &ask.after, &output.text) {
                    Ok((text, notes)) => {
                        let cost = self.store_answer(
                            &key,
                            setup.step(),
                            output,
                            sent.clone(),
                            duration_ms,
                        )?;
                        Ok(PassageCall::Answer {
                            text,
                            notes,
                            cost,
                            request: sent,
                        })
                    }
                    Err(e) => {
                        // Paid for, though nothing is kept: charged to the piece reviewed.
                        self.bill(&state, &setup.provider, &setup.model, &output.usage, &spent)?;
                        Ok(PassageCall::Failed(e.to_string()))
                    }
                }
            }
            Called::Interrupted => Ok(PassageCall::Interrupted),
            Called::Fatal(e) => Ok(PassageCall::Failed(e.to_string())),
            Called::Failed(e) => Ok(PassageCall::Failed(
                e.map_or_else(|| "The model did not answer".to_string(), |e| e.to_string()),
            )),
        }
    }

    /// Stops waiting for a passage's answer; nothing when it is not waiting.
    pub fn cancel_passage(&self, id: &str) {
        if let Some(stop) = lock(&self.passages).get(id) {
            stop.stop();
        }
    }

    /// Writes a proposal into the researcher's corrections of its piece, exactly as a typed
    /// correction of the passage would be: `text` (the answer, or the researcher's edit of it)
    /// in place of the passage in the text as the review shows it now, the piece left reviewed
    /// or not as it was. Refused when the text under the passage changed since it was asked
    /// about. Gives back the piece's correction as it stood, to put back with
    /// [`Engine::undo_passage`].
    pub fn accept_passage(&self, id: &str, text: String) -> Result<Correction, EngineError> {
        let mut proposal = self.store.passage(id)?;
        if proposal.status != ProposalStatus::Answered {
            return Err(EngineError::Passage(PassageError::NotWaiting));
        }
        let ask = proposal.ask.clone();
        let state = self
            .store
            .source_pieces(ask.job_id, ask.setup_id, ask.source_id)?
            .into_iter()
            .find(|p| p.index == ask.index && p.status.is_finished())
            .ok_or(EngineError::NothingRead)?;
        let corrections = self
            .store
            .source_corrections(ask.job_id, ask.setup_id, ask.source_id)?;
        let before = Correction::of(&corrections, &state)
            .cloned()
            .unwrap_or(Correction {
                job_id: ask.job_id,
                setup_id: ask.setup_id,
                source_id: ask.source_id,
                index: ask.index,
                text: None,
                fields: None,
                reviewed: false,
                updated_at: Timestamp::now(),
            });
        let shown = self.shown_text(&state)?;
        let corrected = passage::replace(&shown, &ask, &text)
            .ok_or(EngineError::Passage(PassageError::Changed))?;
        let now = Timestamp::now();
        let correction = Correction {
            text: Some(corrected),
            updated_at: now,
            ..before.clone()
        };
        proposal.status = ProposalStatus::Accepted;
        proposal.accepted = Some(text);
        proposal.decided_at = Some(now);
        self.store.put_passage(&proposal, Some(&correction))?;
        self.write_later(ask.job_id);
        Ok(before)
    }

    /// Leaves a proposal unused.
    pub fn discard_passage(&self, id: &str) -> Result<(), EngineError> {
        let mut proposal = self.store.passage(id)?;
        if proposal.status != ProposalStatus::Answered {
            return Ok(());
        }
        proposal.status = ProposalStatus::Discarded;
        proposal.decided_at = Some(Timestamp::now());
        self.store.put_passage(&proposal, None)?;
        Ok(())
    }

    /// Puts back the correction an accepted passage replaced, and records that it was undone,
    /// in one transaction.
    pub fn undo_passage(&self, id: &str, before: Correction) -> Result<(), EngineError> {
        let mut proposal = self.store.passage(id)?;
        let ask = &proposal.ask;
        let same_piece = before.job_id == ask.job_id
            && before.setup_id == ask.setup_id
            && before.source_id == ask.source_id
            && before.index == ask.index;
        if proposal.status != ProposalStatus::Accepted || !same_piece {
            return Err(EngineError::Passage(PassageError::NotWaiting));
        }
        let job = ask.job_id;
        proposal.status = ProposalStatus::Undone;
        proposal.decided_at = Some(Timestamp::now());
        let before = Correction {
            updated_at: Timestamp::now(),
            ..before
        };
        self.store.put_passage(&proposal, Some(&before))?;
        self.write_later(job);
        Ok(())
    }

    /// The passages accepted in a source's pieces under a set-up, for its provenance.
    pub(crate) fn passages_accepted(
        &self,
        job: &Job,
        setup: kalamu_core::SetupId,
        source: kalamu_core::SourceId,
        states: &[PieceState],
    ) -> Result<Vec<kalamu_core::provenance::ProvenancePassage>, EngineError> {
        let proposals = match self.store.source_passages(job.id, setup, source) {
            Ok(p) => p,
            // A store that keeps no passages has none to tell of.
            Err(StoreError::Database(_)) => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        Ok(proposals
            .into_iter()
            .filter(|p| p.status == ProposalStatus::Accepted)
            .filter(|p| states.iter().any(|s| s.index == p.ask.index))
            .map(|p| kalamu_core::provenance::ProvenancePassage {
                piece: p.ask.index + 1,
                provider: p.ask.provider.clone(),
                model: p.ask.model.clone(),
                prompt: kalamu_core::provenance::ProvenancePrompt::from_snapshot(&p.prompt),
                request: p.request.clone(),
                passage_chars: p.ask.passage.chars().count() as u32,
                context_chars: (p.ask.before.chars().count() + p.ask.after.chars().count()) as u32,
                edited: p.accepted.as_ref() != p.answer.as_ref(),
                cost_usd: p.cost_usd,
                asked_at: p.asked_at,
                accepted_at: p.decided_at.unwrap_or(p.asked_at),
            })
            .collect())
    }
}

/// How a passage's call ended.
enum PassageCall {
    Answer {
        text: String,
        notes: Vec<kalamu_core::ProposalNote>,
        cost: Option<f64>,
        request: Vec<kalamu_core::RequestSetting>,
    },
    Interrupted,
    Failed(String),
}

/// Returns once `stop` is set.
pub(crate) async fn stopped(stop: &StopSignal) {
    while !stop.is_stopped() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
