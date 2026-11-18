//! What every request the researcher makes from the review goes through, whatever it asks (a
//! passage corrected, a region read again): recorded before its call; answered from the cache
//! for nothing when the same was asked before; otherwise called through the engine's
//! [`Engine::call`], with the provider's retries, raced against a stop signal the researcher can
//! raise; recorded after it (answered, failed, cancelled), a refusal the provider billed charged
//! to a piece; then decided, the decision and what it writes recorded in one transaction.
//!
//! Each kind says, through [`Asked`], where it is kept, which answers it refuses before they
//! are stored, and how it reads an answer; what it sends, what it costs beforehand and what
//! accepting writes stay in its own module.
//!
//! No job is made: a request belongs to the result reviewed, and its cost to that result's job.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kalamu_catalog::Catalog;
use kalamu_core::{
    ApiKey, CacheKey, Correction, Job, JobId, JobStore, LanguageChoice, PieceState, PreparedInput,
    Proposal, ProposalStatus, ProviderId, Setup, SetupId, SourceId, StopSignal, StoreError,
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

/// A kind of request made from the review, as the steps every kind goes through need it.
pub(crate) trait Asked: Proposal + Clone + Send + Sync + 'static {
    /// What accepting or undoing it writes, in the same transaction as its record.
    type Effect;
    fn load(store: &dyn JobStore, id: &str) -> Result<Self, StoreError>;
    /// Records it as it stands, by its id, with what a decision writes.
    fn record(&self, store: &dyn JobStore, effect: Option<&Self::Effect>)
    -> Result<(), StoreError>;
    /// Why the model's answer is refused before it is stored: not kept, so asking again calls
    /// again, and billed all the same.
    fn refusal(&self, answer: &str) -> Option<String>;
    /// Reads an answer, the model's or the cache's, into the record: offered, or failed.
    fn answered(&mut self, answer: &str);
}

/// One call of a request: its request frozen as a job's would be.
pub(crate) struct Call {
    /// The job the request belongs to, its task and settings those of the request.
    pub job: Job,
    pub setup: Setup,
    /// The piece a refusal the provider billed is charged to.
    pub charged: PieceState,
    pub input: PreparedInput,
    pub key: CacheKey,
    pub api_key: ApiKey,
}

/// How a call ended.
enum Ended {
    Answer {
        text: String,
        cost: Option<f64>,
        request: Vec<kalamu_core::RequestSetting>,
    },
    Interrupted,
    Failed(String),
}

/// A request waiting for its model, by id, so the researcher can stop waiting; no longer listed
/// once dropped.
pub(crate) struct Waiting<'a> {
    waiting: &'a Mutex<HashMap<String, StopSignal>>,
    id: String,
    pub stop: StopSignal,
}

impl<'a> Waiting<'a> {
    pub fn new(waiting: &'a Mutex<HashMap<String, StopSignal>>, id: &str) -> Self {
        let stop = StopSignal::default();
        lock(waiting).insert(id.to_string(), stop.clone());
        Self {
            waiting,
            id: id.to_string(),
            stop,
        }
    }

    /// Stops waiting for the request `id`; nothing when it is not waiting.
    pub fn cancel(waiting: &Mutex<HashMap<String, StopSignal>>, id: &str) {
        if let Some(stop) = lock(waiting).get(id) {
            stop.stop();
        }
    }
}

impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        lock(self.waiting).remove(&self.id);
    }
}

impl Engine {
    /// The key a request to `provider` needs (none for a model on this computer), once the
    /// provider is ready for `model` and `languages`.
    pub(crate) fn ready_key(
        &self,
        provider: &ProviderId,
        model: &str,
        languages: &LanguageChoice,
    ) -> Result<ApiKey, EngineError> {
        let found = self.provider(provider)?;
        let api_key = if found.needs_key() {
            self.keys
                .key(provider)
                .ok_or_else(|| EngineError::MissingKey(provider.to_string()))?
        } else {
            ApiKey::new("")
        };
        found
            .check_ready(model, languages)
            .map_err(EngineError::NotReady)?;
        Ok(api_key)
    }

    /// What asking would cost: `usd` at the model's prices, nothing to pay on this computer,
    /// nothing when the answer under `key` is stored already.
    pub(crate) fn estimated(
        &self,
        provider: &ProviderId,
        key: &CacheKey,
        usd: impl FnOnce(&Catalog) -> Option<f64>,
    ) -> Result<PassageEstimate, EngineError> {
        let offline = !self.provider(provider)?.needs_key();
        if self.store.cached_result(key)?.is_some() {
            return Ok(PassageEstimate {
                usd: None,
                offline,
                cached: true,
            });
        }
        Ok(PassageEstimate {
            usd: usd(&self.current_catalog()).filter(|_| !offline),
            offline,
            cached: false,
        })
    }

    /// Records a request as asked, then answers it from the cache when the answer under `key`
    /// is stored: read, recorded, and true.
    pub(crate) fn asked_or_cached<P: Asked>(
        &self,
        proposal: &mut P,
        key: &CacheKey,
    ) -> Result<bool, EngineError> {
        proposal.record(&*self.store, None)?;
        let Some(stored) = self.store.cached_result(key)? else {
            return Ok(false);
        };
        let lifecycle = proposal.lifecycle();
        *lifecycle.cached = true;
        *lifecycle.request = stored.request.clone();
        proposal.answered(&stored.output.text);
        proposal.record(&*self.store, None)?;
        Ok(true)
    }

    /// Calls the model for a request recorded as asked, and records how it ended: answered,
    /// failed, or cancelled when `stop` is raised meanwhile. The call goes on alone after a
    /// stop, so an answer that arrives after is still stored and asking again costs nothing.
    pub(crate) async fn call_raced<P: Asked>(
        &self,
        mut proposal: P,
        call: Call,
        stop: &StopSignal,
    ) -> Result<P, EngineError> {
        let me = self
            .me
            .upgrade()
            .ok_or_else(|| EngineError::Output("the engine is closing".into()))?;
        let called = {
            let (stop, judge) = (stop.clone(), proposal.clone());
            tokio::spawn(async move { me.call_once(call, &judge, &stop).await })
        };
        let ended = tokio::select! {
            joined = called => Some(joined),
            () = stopped(stop) => None,
        };
        let mut lifecycle = proposal.lifecycle();
        *lifecycle.answered_at = Some(Timestamp::now());
        match ended {
            None | Some(Ok(Ok(Ended::Interrupted))) => lifecycle.cancel(),
            Some(Err(e)) => lifecycle.fail(e.to_string()),
            Some(Ok(Err(e))) => lifecycle.fail(e.to_string()),
            Some(Ok(Ok(Ended::Failed(reason)))) => lifecycle.fail(reason),
            Some(Ok(Ok(Ended::Answer {
                text,
                cost,
                request,
            }))) => {
                *lifecycle.cost_usd = cost;
                *lifecycle.request = request;
                proposal.answered(&text);
            }
        }
        proposal.record(&*self.store, None)?;
        Ok(proposal)
    }

    /// The call itself, with the provider's retries: its answer stored under the request's key
    /// with its cost, as a job's piece's is; refused by `judge` and billed to the charged piece,
    /// not stored.
    async fn call_once<P: Asked>(
        &self,
        call: Call,
        judge: &P,
        stop: &StopSignal,
    ) -> Result<Ended, EngineError> {
        let Call {
            job,
            setup,
            mut charged,
            input,
            key,
            api_key,
        } = call;
        let provider = self.provider(&setup.provider)?;
        let spent = Arc::new(Mutex::new(self.store.spent(job.id)?));
        let called = self
            .call(
                &job,
                setup.step(),
                &provider,
                &api_key,
                &mut charged,
                &input,
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
                let output = crate::piece::finish(job.task, output);
                if let Some(reason) = judge.refusal(&output.text) {
                    // Paid for, though nothing is kept: charged to the piece reviewed.
                    self.bill(
                        &charged,
                        &setup.provider,
                        &setup.model,
                        &output.usage,
                        &spent,
                    )?;
                    return Ok(Ended::Failed(reason));
                }
                let text = output.text.clone();
                let cost =
                    self.store_answer(&key, setup.step(), output, sent.clone(), duration_ms)?;
                Ok(Ended::Answer {
                    text,
                    cost,
                    request: sent,
                })
            }
            Called::Interrupted => Ok(Ended::Interrupted),
            Called::Fatal(e) => Ok(Ended::Failed(e.to_string())),
            Called::Failed(e) => Ok(Ended::Failed(
                e.map_or_else(|| "The model did not answer".to_string(), |e| e.to_string()),
            )),
        }
    }

    /// Leaves a request unused; nothing when it is not waiting for a decision.
    pub(crate) fn discard<P: Asked>(&self, id: &str) -> Result<(), EngineError> {
        let mut proposal = P::load(&*self.store, id)?;
        if proposal.status() != ProposalStatus::Answered {
            return Ok(());
        }
        proposal.lifecycle().discard(Timestamp::now());
        proposal.record(&*self.store, None)?;
        Ok(())
    }

    /// Records a decision (accepted or undone) with what it writes, in one transaction, and
    /// writes the job's files again a little later.
    pub(crate) fn write_decision<P: Asked>(
        &self,
        proposal: &P,
        effect: &P::Effect,
        job: JobId,
    ) -> Result<(), EngineError> {
        proposal.record(&*self.store, Some(effect))?;
        self.write_later(job);
        Ok(())
    }

    /// A finished piece of a set-up, or "nothing read".
    pub(crate) fn finished_piece(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
        index: u32,
    ) -> Result<PieceState, EngineError> {
        self.store
            .source_pieces(job, setup, source)?
            .into_iter()
            .find(|p| p.index == index && p.status.is_finished())
            .ok_or(EngineError::NothingRead)
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

    /// The piece's correction as it stands (or none yet, as an empty one), and that correction
    /// with `replace` made in the text as the review shows it now, at `now`, the piece left
    /// reviewed or not as it was: `changed` when `replace` finds no longer what it replaces.
    pub(crate) fn correction_replacing(
        &self,
        state: &PieceState,
        now: Timestamp,
        replace: impl FnOnce(&str) -> Option<String>,
        changed: EngineError,
    ) -> Result<(Correction, Correction), EngineError> {
        let corrections =
            self.store
                .source_corrections(state.job_id, state.setup_id, state.source_id)?;
        let before = Correction::of(&corrections, state)
            .cloned()
            .unwrap_or(Correction {
                job_id: state.job_id,
                setup_id: state.setup_id,
                source_id: state.source_id,
                index: state.index,
                text: None,
                fields: None,
                reviewed: false,
                updated_at: now,
            });
        let corrected = replace(&self.shown_text(state)?).ok_or(changed)?;
        let after = Correction {
            text: Some(corrected),
            updated_at: now,
            ..before.clone()
        };
        Ok((before, after))
    }
}

/// The requests of a kind a store keeps, for provenance; none from a store that keeps none.
pub(crate) fn kept<T>(found: Result<Vec<T>, StoreError>) -> Result<Vec<T>, EngineError> {
    match found {
        Ok(found) => Ok(found),
        Err(StoreError::Database(_)) => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

/// Returns once `stop` is set.
pub(crate) async fn stopped(stop: &StopSignal) {
    while !stop.is_stopped() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_waits_until_dropped_and_only_its_own_stop_is_raised() {
        let waiting = Mutex::new(HashMap::new());
        let first = Waiting::new(&waiting, "a");
        let second = Waiting::new(&waiting, "b");
        Waiting::cancel(&waiting, "a");
        assert!(first.stop.is_stopped());
        assert!(!second.stop.is_stopped());
        drop(first);
        assert!(!lock(&waiting).contains_key("a"));
        // No longer waiting: nothing to stop, and nothing breaks.
        Waiting::cancel(&waiting, "a");
        assert!(lock(&waiting).contains_key("b"));
        drop(second);
        assert!(lock(&waiting).is_empty());
    }
}
