//! A document's second step: once its parts are read, their answers go to the set-up's merge
//! step in one call. A table of contents combines its pages' entries into one entry per article
//! or item; a long interview read in parts brings their answers together ([`crate::windows`]).
//!
//! The step is a piece like the others (`PieceRef::Whole`, after the parts), so it is cached,
//! retried, resumed and counted in the job's progress. Its key is made from the parts' own keys:
//! reading a part again, or choosing another merge model or prompt, makes it new.

use std::sync::{Arc, Mutex};

use kalamu_core::{
    CacheKey, Job, PieceRef, PieceState, PieceStatus, PreparedInput, ProgressEvent, Setup,
    SourceFile, StopSignal, StoredResult, Timestamp,
};

use crate::piece::{Input, Outcome};
use crate::{Engine, EngineError};

/// What one kind of second step does with a document's parts.
pub(crate) trait SecondStep: Sync {
    /// The name its key is made under.
    fn kind(&self) -> &'static str;

    /// Whether a part that failed fails the step until it is read again, with the reason it
    /// fails with (a table of contents, which has nothing to write without every page); `None`
    /// when the step goes on with the parts read, naming the others (an interview).
    fn waits_for_failed(&self) -> Option<&'static str>;

    /// The reason the step fails with when no part could be read.
    fn nothing_read(&self) -> &'static str;

    /// The call's input, from the parts read (with their stored answers) and those that failed.
    fn input(
        &self,
        read: &[(PieceState, StoredResult)],
        failed: &[PieceState],
    ) -> Result<Input, String>;
}

/// A table of contents' pages, combined into entries.
pub(crate) struct CombinePages<'a> {
    pub(crate) source: &'a SourceFile,
}

impl SecondStep for CombinePages<'_> {
    fn kind(&self) -> &'static str {
        "contents_merge"
    }

    fn waits_for_failed(&self) -> Option<&'static str> {
        Some("Some pages could not be read. Retry them, and their pages will be combined.")
    }

    fn nothing_read(&self) -> &'static str {
        "No page could be read."
    }

    fn input(
        &self,
        read: &[(PieceState, StoredResult)],
        _failed: &[PieceState],
    ) -> Result<Input, String> {
        let pages: Vec<(String, &[kalamu_core::FieldAnswer])> = read
            .iter()
            .map(|(page, result)| (page.piece.label(), result.output.fields.as_slice()))
            .collect();
        let stem = crate::source_stem(&self.source.file_name, "document");
        Ok(Input::Ready(PreparedInput {
            piece: PieceRef::Whole,
            mime: "text/plain".into(),
            bytes: kalamu_core::contents::merge_input(&pages).into_bytes(),
            file_name: format!("{stem}-pages.json"),
        }))
    }
}

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
        self.second_step(job, source, setup, spent, stop, &CombinePages { source })
            .await
    }

    /// Runs a document's second step under one set-up once its parts are settled: parts still
    /// to read leave it waiting, and parts that failed fail it or are named in its input, as
    /// the kind of step says. A step already answered under the same key is not run again.
    pub(crate) async fn second_step(
        &self,
        job: &Job,
        source: &SourceFile,
        setup: &Setup,
        spent: &Arc<Mutex<f64>>,
        stop: &StopSignal,
        how: &dyn SecondStep,
    ) -> Result<Outcome, EngineError> {
        let Some(step) = setup.merge_step() else {
            return Ok(Outcome::Finished);
        };
        let mut states: Vec<PieceState> = self.store.source_pieces(job.id, setup.id, source.id)?;
        let Some(position) = states.iter().position(|p| p.piece == PieceRef::Whole) else {
            return Ok(Outcome::Finished);
        };
        let mut merge = states.remove(position);
        let parts = states;
        let failed: Vec<PieceState> = parts
            .iter()
            .filter(|p| p.status == PieceStatus::Failed)
            .cloned()
            .collect();
        if let Some(reason) = how.waits_for_failed().filter(|_| !failed.is_empty()) {
            return self.fail_step(merge, reason);
        }
        let settled = |p: &PieceState| p.status.is_finished() || p.status == PieceStatus::Failed;
        if parts.is_empty() || !parts.iter().all(settled) {
            return Ok(Outcome::Interrupted);
        }

        let mut keys = Vec::new();
        let mut read = Vec::new();
        for part in parts.iter().filter(|p| p.status.is_finished()) {
            let Some(key) = &part.cache_key else {
                return Ok(Outcome::Interrupted);
            };
            let Some(result) = self.store.cached_result(&CacheKey(key.clone()))? else {
                return Ok(Outcome::Interrupted);
            };
            keys.push(key.clone());
            read.push((part.clone(), result));
        }
        if read.is_empty() {
            return self.fail_step(merge, how.nothing_read());
        }
        let missing: Vec<String> = failed.iter().map(|p| p.piece.descriptor()).collect();
        let key = CacheKey::combined(
            how.kind(),
            &source.sha256,
            &keys,
            &missing,
            step,
            &job.settings.languages,
        );
        if merge.status.is_finished() && merge.cache_key.as_deref() == Some(key.as_str()) {
            return Ok(Outcome::Finished);
        }
        let input = match how.input(&read, &failed) {
            Ok(input) => input,
            Err(reason) => return self.fail_step(merge, &reason),
        };
        merge.cache_key = Some(key.0.clone());
        merge.status = PieceStatus::Pending;
        merge.error = None;
        self.save_merge(&merge)?;
        self.process_piece(job, source, step, merge, key, input, spent.clone(), stop)
            .await
    }

    /// Fails the second step with a reason, unless it already failed.
    fn fail_step(&self, mut merge: PieceState, reason: &str) -> Result<Outcome, EngineError> {
        if merge.status != PieceStatus::Failed || merge.error.as_deref() != Some(reason) {
            merge.status = PieceStatus::Failed;
            merge.error = Some(reason.into());
            self.save_merge(&merge)?;
        }
        Ok(Outcome::Failed)
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
