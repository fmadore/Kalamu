//! A passage the researcher selected in the review, corrected by a model on request: the
//! correction task's request (its preset frozen as a job's would be, its one field, the
//! provider's retries) sent for the passage and a little context, its answer cached under a key
//! of its own and offered as a proposal, through the steps every request from the review goes
//! through ([`crate::asked`]). An answer that is not a correction of the passage is refused and
//! billed. Only the researcher's acceptance writes it, into their corrections of that piece, in
//! the same transaction as the record that says a model proposed it; the model's output is
//! never changed.
//!
//! A passage belongs to the result reviewed, and its cost to that result's job. The job's
//! settings give the languages; the job's task need only write text.

use kalamu_core::passage_correction::{self as passage, PassageError};
use kalamu_core::{
    CacheKey, Correction, Job, JobSettings, JobStore, LineBreaks, PassageAsk, PassageProposal,
    PieceState, PreparedInput, Proposal, ProposalStatus, SetupRequest, StoreError, Task, Timestamp,
};

use crate::asked::{Asked, Call, PassageEstimate, Waiting, kept};
use crate::{Engine, EngineError};

impl Asked for PassageProposal {
    type Effect = Correction;

    fn load(store: &dyn JobStore, id: &str) -> Result<Self, StoreError> {
        store.passage(id)
    }

    fn record(&self, store: &dyn JobStore, effect: Option<&Correction>) -> Result<(), StoreError> {
        store.put_passage(self, effect)
    }

    /// An answer that is not a correction of the passage: empty, or too far from its length.
    fn refusal(&self, answer: &str) -> Option<String> {
        let ask = &self.ask;
        passage::read_answer(&ask.before, &ask.passage, &ask.after, answer)
            .err()
            .map(|e| e.to_string())
    }

    /// Offered when it passes, failed when not.
    fn answered(&mut self, answer: &str) {
        self.answered_at = Some(Timestamp::now());
        let ask = &self.ask;
        match passage::read_answer(&ask.before, &ask.passage, &ask.after, answer) {
            Ok((text, notes)) => {
                self.status = ProposalStatus::Answered;
                self.answer = Some(text);
                self.notes = notes;
            }
            Err(e) => self.lifecycle().fail(e.to_string()),
        }
    }
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
        let state = self.finished_piece(ask.job_id, ask.setup_id, ask.source_id, ask.index)?;
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

    /// What asking would cost: the passage, its context and the prompt read, the passage
    /// written back, at the model's prices; nothing when the answer is stored already.
    pub fn passage_estimate(&self, ask: &PassageAsk) -> Result<PassageEstimate, EngineError> {
        let prepared = self.prepare_passage(ask)?;
        self.estimated(&ask.provider, &prepared.key, |catalog| {
            let m = catalog.model(&ask.provider, &ask.model)?;
            let (input, output) = (m.pricing.input_per_mtok?, m.pricing.output_per_mtok?);
            let read = (prepared.setup.prompt.text.len()
                + prepared.setup.prompt.user_prompt.len()
                + prepared.input.len()) as f64
                / 4.0;
            // Written back as JSON, a little longer than read.
            let written = ask.passage.len() as f64 / 4.0 * 1.1 + 50.0;
            Some((read * input + written * output) / 1_000_000.0)
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
            mut job,
            state,
            setup,
            input,
            key,
        } = self.prepare_passage(&ask)?;
        let api_key = self.ready_key(&ask.provider, &setup.model, &job.settings.languages)?;
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
        if self.asked_or_cached(&mut proposal, &key)? {
            return Ok(proposal);
        }
        let waiting = Waiting::new(&self.passages, &ask.id);
        job.task = Task::TextCorrection;
        let input = PreparedInput {
            piece: state.piece.clone(),
            mime: "text/plain".into(),
            bytes: input.into_bytes(),
            file_name: "passage.txt".into(),
        };
        let call = Call {
            job,
            setup,
            charged: state,
            input,
            key,
            api_key,
        };
        self.call_raced(proposal, call, &waiting.stop).await
    }

    /// Stops waiting for a passage's answer; nothing when it is not waiting.
    pub fn cancel_passage(&self, id: &str) {
        Waiting::cancel(&self.passages, id);
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
        let state = self.finished_piece(ask.job_id, ask.setup_id, ask.source_id, ask.index)?;
        let now = Timestamp::now();
        let (before, after) = self.correction_replacing(
            &state,
            now,
            |shown| passage::replace(shown, &ask, &text),
            EngineError::Passage(PassageError::Changed),
        )?;
        proposal.lifecycle().accept(text, now);
        self.write_decision(&proposal, &after, ask.job_id)?;
        Ok(before)
    }

    /// Leaves a proposal unused.
    pub fn discard_passage(&self, id: &str) -> Result<(), EngineError> {
        self.discard::<PassageProposal>(id)
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
        proposal.lifecycle().undo(Timestamp::now());
        let before = Correction {
            updated_at: Timestamp::now(),
            ..before
        };
        self.write_decision(&proposal, &before, job)
    }

    /// The passages accepted in a source's pieces under a set-up, for its provenance.
    pub(crate) fn passages_accepted(
        &self,
        job: &Job,
        setup: kalamu_core::SetupId,
        source: kalamu_core::SourceId,
        states: &[PieceState],
    ) -> Result<Vec<kalamu_core::provenance::ProvenancePassage>, EngineError> {
        let proposals = kept(
            "the passages corrected by a model",
            self.store.source_passages(job.id, setup, source),
        )?;
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
