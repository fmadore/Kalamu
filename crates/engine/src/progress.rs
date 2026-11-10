//! A job's progress as its page shows it (#143): each file's pieces counted under each set-up,
//! counted here once rather than in the interface from every piece, twice a second, for each
//! row. Only the files asked for are counted in detail, so a page showing the first two hundred
//! files of a job of five thousand receives two hundred rows, not every piece.

use std::collections::{HashMap, HashSet};

use kalamu_core::{JobId, PieceState, PieceStatus, SetupId, SourceId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// How many different errors a file's row lists under one set-up.
pub const ERRORS_LISTED: usize = 3;

/// One file's pieces under one set-up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileProgress {
    pub source_id: SourceId,
    pub setup_id: SetupId,
    /// Pieces planned so far; none until the file has been read.
    pub total: u32,
    /// Read, now or from the cache.
    pub done: u32,
    pub failed: u32,
    /// The first few different reasons its pieces failed, in the pieces' order.
    pub errors: Vec<String>,
}

/// What a job's page needs of its pieces.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobProgress {
    /// Pieces the provider's batch service holds, over the whole job.
    pub held: u32,
    /// The files asked for that have pieces, under each set-up that has some.
    pub files: Vec<FileProgress>,
}

/// Counts `pieces` (sorted by set-up, source and position, as the store gives them), in detail
/// for the files in `sources` only.
pub fn tally(pieces: &[PieceState], sources: &[SourceId]) -> JobProgress {
    let wanted: HashSet<SourceId> = sources.iter().copied().collect();
    let mut held = 0;
    let mut rows: Vec<FileProgress> = Vec::new();
    let mut at: HashMap<(SourceId, SetupId), usize> = HashMap::new();
    for p in pieces {
        if p.status == PieceStatus::Submitted {
            held += 1;
        }
        if !wanted.contains(&p.source_id) {
            continue;
        }
        let i = *at.entry((p.source_id, p.setup_id)).or_insert_with(|| {
            rows.push(FileProgress {
                source_id: p.source_id,
                setup_id: p.setup_id,
                total: 0,
                done: 0,
                failed: 0,
                errors: Vec::new(),
            });
            rows.len() - 1
        });
        let row = &mut rows[i];
        row.total += 1;
        match p.status {
            PieceStatus::Done | PieceStatus::Cached => row.done += 1,
            PieceStatus::Failed => row.failed += 1,
            _ => {}
        }
        if let Some(e) = &p.error
            && row.errors.len() < ERRORS_LISTED
            && !row.errors.contains(e)
        {
            row.errors.push(e.clone());
        }
    }
    JobProgress { held, files: rows }
}

impl Engine {
    /// The job's progress for its page, the files in `sources` counted one by one.
    pub fn job_progress(
        &self,
        id: JobId,
        sources: &[SourceId],
    ) -> Result<JobProgress, EngineError> {
        Ok(tally(&self.store.pieces(id)?, sources))
    }
}

#[cfg(test)]
mod tests {
    use kalamu_core::{PieceRef, Timestamp};

    use super::*;

    fn piece(
        setup: SetupId,
        source: SourceId,
        index: u32,
        status: PieceStatus,
        error: Option<&str>,
    ) -> PieceState {
        PieceState {
            job_id: JobId::new(),
            setup_id: setup,
            source_id: source,
            index,
            piece: PieceRef::Page {
                number: index + 1,
                half: None,
            },
            status,
            cache_key: None,
            error: error.map(str::to_string),
            attempts: 1,
            updated_at: Timestamp::now(),
        }
    }

    #[test]
    fn counts_the_files_asked_for_and_holds_over_the_whole_job() {
        let (a, b) = (SetupId::new(), SetupId::new());
        let (one, two) = (SourceId::new(), SourceId::new());
        let pieces = vec![
            piece(a, one, 0, PieceStatus::Done, None),
            piece(a, one, 1, PieceStatus::Cached, None),
            piece(a, one, 2, PieceStatus::Failed, Some("refused")),
            piece(a, one, 3, PieceStatus::Failed, Some("refused")),
            piece(a, one, 4, PieceStatus::Failed, Some("timed out")),
            piece(b, one, 0, PieceStatus::Submitted, None),
            piece(a, two, 0, PieceStatus::Submitted, None),
        ];
        let progress = tally(&pieces, &[one]);
        assert_eq!(progress.held, 2);
        assert_eq!(progress.files.len(), 2);
        let first = &progress.files[0];
        assert_eq!((first.setup_id, first.source_id), (a, one));
        assert_eq!((first.total, first.done, first.failed), (5, 2, 3));
        assert_eq!(first.errors, ["refused", "timed out"]);
        let second = &progress.files[1];
        assert_eq!((second.setup_id, second.total, second.done), (b, 1, 0));
        assert!(tally(&pieces, &[]).files.is_empty());
    }

    #[test]
    fn lists_at_most_a_few_different_errors() {
        let (a, one) = (SetupId::new(), SourceId::new());
        let pieces: Vec<PieceState> = (0..6)
            .map(|i| piece(a, one, i, PieceStatus::Failed, Some(&format!("error {i}"))))
            .collect();
        let progress = tally(&pieces, &[one]);
        assert_eq!(progress.files[0].errors.len(), ERRORS_LISTED);
        assert_eq!(progress.files[0].failed, 6);
    }
}
