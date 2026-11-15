//! Where a page's text sits, for the review's source pane (#84): read from the store, never from
//! a provider. The coordinates and how a region finds its place in the text are core's
//! ([`kalamu_core::regions`]).

use kalamu_core::{Correction, JobId, PageRegions, SetupId, SourceId};

use crate::{Engine, EngineError};

impl Engine {
    /// The regions the engine stored for piece `index` of a source under a set-up, each placed in
    /// the text the review shows (the correction, else the model's text) where that is certain.
    /// None for a piece not read, or read by a model that reports no layout.
    pub fn page_regions(
        &self,
        job_id: JobId,
        source_id: SourceId,
        setup_id: SetupId,
        index: u32,
    ) -> Result<PageRegions, EngineError> {
        let pieces = self.store.source_pieces(job_id, setup_id, source_id)?;
        let Some(piece) = pieces.into_iter().find(|p| p.index == index) else {
            return Ok(PageRegions::none(false));
        };
        let corrections = self.store.source_corrections(job_id, setup_id, source_id)?;
        let corrected = Correction::of(&corrections, &piece).is_some_and(|c| c.text.is_some());
        let Some(result) = self.store.finished_result(&piece)? else {
            return Ok(PageRegions::none(corrected));
        };
        if result.output.regions.is_empty() {
            return Ok(PageRegions::none(corrected));
        }
        let speakers = self.store.speakers(job_id)?;
        // The text as the review shows it: `PieceView`'s corrected text, else its model text.
        let shown = self
            .store
            .text(&piece, &corrections, &speakers)?
            .unwrap_or_default();
        let model = kalamu_core::speakers::rename_text(
            &result.output.text,
            speakers.of(setup_id, source_id),
        );
        Ok(PageRegions::of(
            &result.output.regions,
            &model,
            &shown,
            corrected,
        ))
    }
}
