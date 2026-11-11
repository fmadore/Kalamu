//! A job's progress as its page shows it (#143): each file's pieces counted under each set-up.
//! The store counts them ([`kalamu_core::JobStore::job_progress`]); only the files asked for
//! are counted in detail, so a page showing the first two hundred files of a job of five
//! thousand receives two hundred rows, and reads no piece to make them.

use kalamu_core::{JobId, JobProgress, SourceId};

use crate::{Engine, EngineError};

impl Engine {
    /// The job's progress for its page, the files in `sources` counted one by one.
    pub fn job_progress(
        &self,
        id: JobId,
        sources: &[SourceId],
    ) -> Result<JobProgress, EngineError> {
        Ok(self.store.job_progress(id, sources)?)
    }
}
