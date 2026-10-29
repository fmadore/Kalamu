//! A table of contents' second step: once every page of a document is read, the pages' entries
//! go to the set-up's merge model in one call, which writes one entry per article or item.
//!
//! The step is a piece like the others (`PieceRef::Whole`, after the pages), so it is cached,
//! retried, resumed and counted in the job's progress. Its key is made from the pages' own keys:
//! reading a page again, or choosing another merge model or prompt, makes it new.

use std::path::Path;
use std::sync::{Arc, Mutex};

use kalamu_core::{
    CacheKey, Job, PieceRef, PieceState, PieceStatus, PreparedInput, ProgressEvent, Setup,
    SourceFile, StopSignal, Timestamp,
};

use crate::run::{Input, Outcome};
use crate::{Engine, EngineError};

impl Engine {
    /// Combines one document's pages under one set-up, when they are all read. Pages still to
    /// read leave the step waiting; a page that failed fails it, until the page is retried.
    pub(crate) async fn merge_pages(
        &self,
        job: &Job,
        source: &SourceFile,
        setup: &Setup,
        spent: &Arc<Mutex<f64>>,
        stop: &StopSignal,
    ) -> Result<Outcome, EngineError> {
        let Some(step) = setup.merge_step() else {
            return Ok(Outcome::Finished);
        };
        let mut states: Vec<PieceState> = self
            .store
            .pieces(job.id)?
            .into_iter()
            .filter(|p| p.setup_id == setup.id && p.source_id == source.id)
            .collect();
        states.sort_by_key(|p| p.index);
        let Some(position) = states.iter().position(|p| p.piece == PieceRef::Whole) else {
            return Ok(Outcome::Finished);
        };
        let mut merge = states.remove(position);
        let pages = states;
        if pages.iter().any(|p| p.status == PieceStatus::Failed) {
            if merge.status != PieceStatus::Failed {
                merge.status = PieceStatus::Failed;
                merge.error = Some(
                    "Some pages could not be read. Retry them, and their pages will be \
                     combined."
                        .into(),
                );
                self.save_merge(&merge)?;
            }
            return Ok(Outcome::Failed);
        }
        if pages.is_empty() || !pages.iter().all(|p| p.status.is_finished()) {
            return Ok(Outcome::Interrupted);
        }

        let mut keys = Vec::new();
        let mut read = Vec::new();
        for page in &pages {
            let Some(key) = &page.cache_key else {
                return Ok(Outcome::Interrupted);
            };
            let Some(result) = self.store.cached_result(&CacheKey(key.clone()))? else {
                return Ok(Outcome::Interrupted);
            };
            keys.push(key.clone());
            read.push((page.piece.label(), result.output.fields));
        }
        let key = CacheKey::merge(&source.sha256, &keys, step, &job.settings.languages);
        if merge.status.is_finished() && merge.cache_key.as_deref() == Some(key.as_str()) {
            return Ok(Outcome::Finished);
        }
        merge.cache_key = Some(key.0.clone());
        merge.status = PieceStatus::Pending;
        merge.error = None;
        self.save_merge(&merge)?;

        let pages: Vec<(String, &[kalamu_core::FieldAnswer])> = read
            .iter()
            .map(|(label, fields)| (label.clone(), fields.as_slice()))
            .collect();
        let stem = Path::new(&source.file_name)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "document".into());
        let input = PreparedInput {
            piece: PieceRef::Whole,
            mime: "text/plain".into(),
            bytes: kalamu_core::contents::merge_input(&pages).into_bytes(),
            file_name: format!("{stem}-pages.json"),
        };
        self.process_piece(
            job,
            source,
            step,
            merge,
            key,
            Input::Ready(input),
            spent.clone(),
            stop,
        )
        .await
    }

    fn save_merge(&self, state: &PieceState) -> Result<(), EngineError> {
        let mut state = state.clone();
        state.updated_at = Timestamp::now();
        self.store.upsert_piece(&state)?;
        self.emit(ProgressEvent::PieceUpdated {
            job_id: state.job_id,
            setup_id: state.setup_id,
            source_id: state.source_id,
            index: state.index,
            piece: state.piece.clone(),
            status: state.status,
            error: state.error.clone(),
            preview: None,
        });
        Ok(())
    }
}
