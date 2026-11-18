//! The researcher's decision on a region's reading: written where its target says (the text
//! of a region they drew, or the page's text in place of the region's words) with the record
//! that a model read it, in one transaction; discarded; undone as it was; and the readings
//! accepted, as a set-up's provenance lists them.

use kalamu_core::provenance::{ProvenancePrompt, ProvenanceRegionReading, ReadingPlace};
use kalamu_core::region_reading as reading;
use kalamu_core::{
    Correction, DrawnRegion, Job, PieceState, ProposalStatus, ReadingEffect, ReadingTarget,
    RegionError, RegionReadingError, SetupId, SourceId, StoreError, Timestamp,
};

use super::{missing, refused};
use crate::{Engine, EngineError};

impl Engine {
    /// Writes a reading where its target says: `text` (the answer, or the researcher's edit
    /// of it) as the text of the region they drew, or in place of the region's words in the
    /// page's text as the review shows it now, the piece left reviewed or not as it was.
    /// Refused for a reading with no place to go, and when those words changed since it was
    /// asked for. Gives back what it replaced, to put back with
    /// [`Engine::undo_region_reading`].
    pub fn accept_region_reading(
        &self,
        id: &str,
        text: String,
    ) -> Result<ReadingEffect, EngineError> {
        let mut proposal = self.store.region_reading(id)?;
        if proposal.status != ProposalStatus::Answered {
            return Err(refused(RegionReadingError::NotWaiting));
        }
        let ask = proposal.ask.clone();
        let now = Timestamp::now();
        let (before, after) = match &ask.target {
            ReadingTarget::Nowhere => return Err(refused(RegionReadingError::Nowhere)),
            ReadingTarget::Region => {
                let region = self.store.region(&ask.region_id).map_err(missing)?;
                if region.job_id != ask.job_id || region.source_id != ask.source_id {
                    return Err(EngineError::Region(RegionError::NotFound));
                }
                let after = DrawnRegion {
                    text: text.clone(),
                    updated_at: now,
                    ..region.clone()
                };
                (ReadingEffect::Region(region), ReadingEffect::Region(after))
            }
            ReadingTarget::Text {
                setup_id, index, ..
            } => {
                let state = self.finished_piece(&ask, *setup_id, *index)?;
                let corrections =
                    self.store
                        .source_corrections(ask.job_id, *setup_id, ask.source_id)?;
                let before = Correction::of(&corrections, &state)
                    .cloned()
                    .unwrap_or(Correction {
                        job_id: ask.job_id,
                        setup_id: *setup_id,
                        source_id: ask.source_id,
                        index: *index,
                        text: None,
                        fields: None,
                        reviewed: false,
                        updated_at: now,
                    });
                let shown = self.shown_text(&state)?;
                let corrected = reading::replace(&shown, &ask.target, &text)
                    .ok_or(refused(RegionReadingError::Changed))?;
                let after = Correction {
                    text: Some(corrected),
                    updated_at: now,
                    ..before.clone()
                };
                (
                    ReadingEffect::Correction(before),
                    ReadingEffect::Correction(after),
                )
            }
        };
        proposal.status = ProposalStatus::Accepted;
        proposal.accepted = Some(text);
        proposal.decided_at = Some(now);
        self.store.put_region_reading(&proposal, Some(&after))?;
        self.write_later(ask.job_id);
        Ok(before)
    }

    /// Leaves a reading unused.
    pub fn discard_region_reading(&self, id: &str) -> Result<(), EngineError> {
        let mut proposal = self.store.region_reading(id)?;
        if proposal.status != ProposalStatus::Answered {
            return Ok(());
        }
        proposal.status = ProposalStatus::Discarded;
        proposal.decided_at = Some(Timestamp::now());
        self.store.put_region_reading(&proposal, None)?;
        Ok(())
    }

    /// Puts back what an accepted reading replaced (the page's correction, or the region as it
    /// was), and records that it was undone, in one transaction.
    pub fn undo_region_reading(&self, id: &str, before: ReadingEffect) -> Result<(), EngineError> {
        let mut proposal = self.store.region_reading(id)?;
        let ask = &proposal.ask;
        let fits = match (&ask.target, &before) {
            (ReadingTarget::Region, ReadingEffect::Region(r)) => {
                r.id == ask.region_id && r.job_id == ask.job_id && r.source_id == ask.source_id
            }
            (
                ReadingTarget::Text {
                    setup_id, index, ..
                },
                ReadingEffect::Correction(c),
            ) => {
                c.job_id == ask.job_id
                    && c.setup_id == *setup_id
                    && c.source_id == ask.source_id
                    && c.index == *index
            }
            _ => false,
        };
        if proposal.status != ProposalStatus::Accepted || !fits {
            return Err(refused(RegionReadingError::NotWaiting));
        }
        let job = ask.job_id;
        let now = Timestamp::now();
        proposal.status = ProposalStatus::Undone;
        proposal.decided_at = Some(now);
        let before = match before {
            ReadingEffect::Correction(c) => ReadingEffect::Correction(Correction {
                updated_at: now,
                ..c
            }),
            ReadingEffect::Region(r) => ReadingEffect::Region(DrawnRegion {
                updated_at: now,
                ..r
            }),
        };
        self.store.put_region_reading(&proposal, Some(&before))?;
        self.write_later(job);
        Ok(())
    }

    /// The readings accepted on a source that a set-up's `provenance.json` lists: those written
    /// into a region the researcher drew, which every set-up shows, and those written into this
    /// set-up's text of one of `states`.
    pub(crate) fn readings_accepted(
        &self,
        job: &Job,
        setup: SetupId,
        source: SourceId,
        states: &[PieceState],
    ) -> Result<Vec<ProvenanceRegionReading>, EngineError> {
        let readings = match self.store.source_region_readings(job.id, source) {
            Ok(r) => r,
            // A store that keeps no readings has none to tell of.
            Err(StoreError::Database(_)) => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        Ok(readings
            .into_iter()
            .filter(|r| r.status == ProposalStatus::Accepted)
            .filter_map(|r| {
                let went_to = match &r.ask.target {
                    ReadingTarget::Region => ReadingPlace::Region,
                    ReadingTarget::Text {
                        setup_id, index, ..
                    } if *setup_id == setup && states.iter().any(|s| s.index == *index) => {
                        ReadingPlace::Text
                    }
                    _ => return None,
                };
                let ask = &r.ask;
                Some(ProvenanceRegionReading {
                    page: ask.page,
                    half: ask.half,
                    region: ask.region_id.clone(),
                    x: ask.x,
                    y: ask.y,
                    width: ask.width,
                    height: ask.height,
                    provider: ask.provider.clone(),
                    model: ask.model.clone(),
                    prompt: ProvenancePrompt::from_snapshot(&r.prompt),
                    request: r.request.clone(),
                    went_to,
                    edited: r.accepted.as_ref() != r.answer.as_ref(),
                    cost_usd: r.cost_usd,
                    asked_at: r.asked_at,
                    accepted_at: r.decided_at.unwrap_or(r.asked_at),
                })
            })
            .collect())
    }
}
