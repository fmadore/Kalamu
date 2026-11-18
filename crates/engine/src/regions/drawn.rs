//! The researcher's own regions over a scan: drawn, or an engine's region given another box,
//! kept by the store in a layer of their own and laid over the engine's when a page is shown.
//! Nothing here calls a provider or writes a stored result.
//!
//! A box adjusting an engine's region belongs to the reading of the page it was made on (its
//! cache key). Shown with that reading, it takes the region's place. Shown with another reading
//! that some set-up still shows (comparison: each set-up reads the page its own way), it is that
//! set-up's, and left out. Once no set-up shows its reading any more (the page read again), it
//! is shown everywhere as one the researcher drew, so nothing they did is lost.

use std::collections::HashSet;

use kalamu_core::provenance::ProvenanceRegion;
use kalamu_core::regions::{self, RegionError};
use kalamu_core::{
    AdjustedRegion, DrawnRegion, InputKind, Job, JobId, PageHalf, PageRegions, PieceRef,
    PieceState, RegionDraft, SourceFile, SourceId, StoreError, Timestamp,
};

use crate::{Engine, EngineError};

/// A store's "not found" for a region, as the region's own error.
fn missing(e: StoreError) -> EngineError {
    match e {
        StoreError::NotFound => EngineError::Region(RegionError::NotFound),
        e => e.into(),
    }
}

impl Engine {
    /// `regions` with the researcher's own on the same page laid over them.
    pub(super) fn with_researcher(
        &self,
        job_id: JobId,
        source_id: SourceId,
        piece: &PieceRef,
        regions: PageRegions,
    ) -> Result<PageRegions, EngineError> {
        let PieceRef::Page { number, half } = *piece else {
            return Ok(regions);
        };
        let own = self.page_drawn(job_id, source_id, number, half, regions.result.as_deref())?;
        Ok(regions.with_drawn(&own))
    }

    /// The researcher's regions on a page with no text piece of its own (an analysis's review,
    /// a table of contents'): those drawn there, and the boxes adjusting a reading no set-up
    /// shows any more. No engine's regions.
    pub fn drawn_regions(
        &self,
        job_id: JobId,
        source_id: SourceId,
        page: u32,
        half: Option<PageHalf>,
    ) -> Result<PageRegions, EngineError> {
        self.source_in(job_id, source_id)?;
        let own = self.page_drawn(job_id, source_id, page, half, None)?;
        Ok(PageRegions::none(false).with_drawn(&own))
    }

    /// Saves a region the researcher drew, or the box they gave an engine's region, or a
    /// change to either, and gives it back as kept. Refused for a box outside the page, a page
    /// the source does not have, or an engine's region the page's readings do not hold. A box
    /// for an engine's region that already has one replaces it, so a region has one box.
    pub fn save_region(&self, draft: RegionDraft) -> Result<DrawnRegion, EngineError> {
        let draft = regions::check(&draft).map_err(EngineError::Region)?;
        let (job, source) = self.source_in(draft.job_id, draft.source_id)?;
        on_page(&source, draft.page, draft.half)?;
        let target = PieceRef::Page {
            number: draft.page,
            half: draft.half,
        };
        if let Some(adjusts) = &draft.adjusts {
            self.check_adjusts(&job, source.id, &target, adjusts)?;
        }
        let existing = match &draft.id {
            Some(id) => {
                let kept = self.store.region(id).map_err(missing)?;
                if kept.job_id != draft.job_id || kept.source_id != draft.source_id {
                    return Err(EngineError::Region(RegionError::NotFound));
                }
                Some(kept)
            }
            None => match &draft.adjusts {
                Some(adjusts) => self
                    .store
                    .source_regions(job.id, source.id)?
                    .into_iter()
                    .find(|r| r.adjusts.as_ref() == Some(adjusts)),
                None => None,
            },
        };
        let now = Timestamp::now();
        let region = DrawnRegion {
            id: existing
                .as_ref()
                .map_or_else(DrawnRegion::new_id, |r| r.id.clone()),
            job_id: draft.job_id,
            source_id: draft.source_id,
            page: draft.page,
            half: draft.half,
            x: draft.x,
            y: draft.y,
            width: draft.width,
            height: draft.height,
            text: draft.text,
            adjusts: draft.adjusts,
            source_hash: source.sha256.clone(),
            created_at: existing.map_or(now, |r| r.created_at),
            updated_at: now,
        };
        self.store.put_region(&region)?;
        self.write_later(region.job_id);
        Ok(region)
    }

    /// Deletes a region for good and gives it back as it was, for *Undo*
    /// ([`Engine::restore_region`]).
    pub fn remove_region(&self, id: &str) -> Result<DrawnRegion, EngineError> {
        let removed = self.store.remove_region(id).map_err(missing)?;
        self.write_later(removed.job_id);
        Ok(removed)
    }

    /// Puts back a region removed, as it was: its id, box, text and dates.
    pub fn restore_region(&self, region: DrawnRegion) -> Result<DrawnRegion, EngineError> {
        let draft = RegionDraft {
            id: Some(region.id.clone()),
            job_id: region.job_id,
            source_id: region.source_id,
            page: region.page,
            half: region.half,
            x: region.x,
            y: region.y,
            width: region.width,
            height: region.height,
            text: String::new(),
            adjusts: None,
        };
        let checked = regions::check(&draft).map_err(EngineError::Region)?;
        let (_, source) = self.source_in(region.job_id, region.source_id)?;
        on_page(&source, region.page, region.half)?;
        let region = DrawnRegion {
            x: checked.x,
            y: checked.y,
            width: checked.width,
            height: checked.height,
            ..region
        };
        self.store.put_region(&region)?;
        self.write_later(region.job_id);
        Ok(region)
    }

    /// The regions a set-up's `provenance.json` lists: those drawn on the source, the boxes
    /// adjusting this set-up's readings, and those adjusting a reading no set-up shows any
    /// more. Not the boxes adjusting another set-up's reading.
    pub(crate) fn regions_recorded(
        &self,
        job: &Job,
        source: SourceId,
        states: &[PieceState],
    ) -> Result<Vec<ProvenanceRegion>, EngineError> {
        let kept = self.store.source_regions(job.id, source)?;
        if kept.is_empty() {
            return Ok(Vec::new());
        }
        let own: HashSet<&str> = states
            .iter()
            .filter(|s| s.status.is_finished())
            .filter_map(|s| s.cache_key.as_deref())
            .collect();
        let live = self.readings(job, source, None)?;
        Ok(kept
            .iter()
            .filter(|r| match &r.adjusts {
                None => true,
                Some(a) => own.contains(a.result.as_str()) || !live.contains(&a.result),
            })
            .map(ProvenanceRegion::of)
            .collect())
    }

    /// The researcher's regions on page `number` (or that half of it), less the boxes adjusting
    /// a reading other than `shown` that a set-up still shows.
    fn page_drawn(
        &self,
        job_id: JobId,
        source_id: SourceId,
        number: u32,
        half: Option<PageHalf>,
        shown: Option<&str>,
    ) -> Result<Vec<DrawnRegion>, EngineError> {
        let mut own: Vec<DrawnRegion> = self
            .store
            .source_regions(job_id, source_id)?
            .into_iter()
            .filter(|r| r.on(number, half))
            .collect();
        let other = |r: &DrawnRegion| {
            r.adjusts
                .as_ref()
                .is_some_and(|a| Some(a.result.as_str()) != shown)
        };
        if own.iter().any(other) {
            let job = self.store.job(job_id)?;
            let page = PieceRef::Page { number, half };
            let live = self.readings(&job, source_id, Some(&page))?;
            own.retain(|r| match &r.adjusts {
                Some(a) if other(r) => !live.contains(&a.result),
                _ => true,
            });
        }
        Ok(own)
    }

    /// The cache keys of the readings the job's set-ups show of a source, or of one of its
    /// pieces.
    fn readings(
        &self,
        job: &Job,
        source: SourceId,
        piece: Option<&PieceRef>,
    ) -> Result<HashSet<String>, EngineError> {
        let mut keys = HashSet::new();
        for setup in &job.setups {
            for state in self.store.source_pieces(job.id, setup.id, source)? {
                if state.status.is_finished() && piece.is_none_or(|p| *p == state.piece) {
                    keys.extend(state.cache_key);
                }
            }
        }
        Ok(keys)
    }

    /// Whether a reading of the page some set-up shows holds the region a box adjusts.
    fn check_adjusts(
        &self,
        job: &Job,
        source: SourceId,
        page: &PieceRef,
        adjusts: &AdjustedRegion,
    ) -> Result<(), EngineError> {
        for setup in &job.setups {
            for state in self.store.source_pieces(job.id, setup.id, source)? {
                let reading = state.status.is_finished()
                    && state.piece == *page
                    && state.cache_key.as_deref() == Some(adjusts.result.as_str());
                if reading
                    && self
                        .engine_regions(job.id, source, setup.id, &state)?
                        .holds(&adjusts.region)
                {
                    return Ok(());
                }
            }
        }
        Err(EngineError::Region(RegionError::Adjusts))
    }

    /// The job and its source, or "not found".
    pub(crate) fn source_in(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<(Job, SourceFile), EngineError> {
        let job = self.store.job(job_id)?;
        let source = job
            .sources
            .iter()
            .find(|s| s.id == source_id)
            .cloned()
            .ok_or(EngineError::Store(StoreError::NotFound))?;
        Ok((job, source))
    }
}

/// Whether the source has page `page`, or that half of it: a scan or an image, the page within
/// its count where it is known, a half only where its spreads were split.
pub(crate) fn on_page(
    source: &SourceFile,
    page: u32,
    half: Option<PageHalf>,
) -> Result<(), EngineError> {
    let pages = matches!(source.kind, InputKind::Pdf | InputKind::Image);
    let within = page >= 1 && source.page_count.is_none_or(|n| page <= n);
    let halves = half.is_none() || source.split_spreads;
    if pages && within && halves {
        Ok(())
    } else {
        Err(EngineError::Region(RegionError::NoPage))
    }
}
