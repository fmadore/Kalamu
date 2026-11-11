//! A job's progress as its page shows it (#143): each file's pieces counted under each set-up.
//! The store counts them ([`crate::JobStore::job_progress`]), so a running job's page, read
//! again twice a second, costs counts rather than every piece of a job of thousands of files.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{SetupId, SourceId};

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
    /// Every file of the job with a failed piece under any set-up, asked for or not, so the
    /// page can show the files that failed however far down a long job they sit.
    pub failed_sources: Vec<SourceId>,
    /// The files asked for that have pieces, under each set-up that has some.
    pub files: Vec<FileProgress>,
}
