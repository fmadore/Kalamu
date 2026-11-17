//! A job's agreement between set-ups, read from the store for the job's page. The statistics
//! are `core`'s ([`kalamu_core::agreement`]); this reads each piece's answers once and feeds them
//! in, document by document.

use std::collections::HashMap;

use kalamu_core::{AgreementTally, FieldAnswer, JobAgreement, JobId, SourceId, Task};

use crate::{Engine, EngineError};

impl Engine {
    /// How far the set-ups of an analysis agree, field by field, over every piece each pair
    /// answered. Reads the models' own answers, not the researcher's corrections, as the
    /// comparison's agreement between set-ups does: a correction is the researcher's reading,
    /// not a model's. Empty for a job with one set-up or a task other than an analysis.
    pub fn job_agreement(&self, job_id: JobId) -> Result<JobAgreement, EngineError> {
        let job = self.store.job(job_id)?;
        if job.task != Task::Analysis || job.setups.len() < 2 {
            return Ok(JobAgreement::default());
        }
        let mut tally = AgreementTally::new(&job.setups);
        if tally.is_empty() {
            return Ok(tally.finish());
        }
        let position: HashMap<_, usize> = job
            .setups
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();
        // Each piece's answers by set-up, only the fields kept: a job of thousands of documents
        // holds their answers, never their texts.
        let mut answers: HashMap<(SourceId, u32), Vec<Option<Vec<FieldAnswer>>>> = HashMap::new();
        for piece in self.store.pieces(job.id)? {
            let Some(&at) = position.get(&piece.setup_id) else {
                continue;
            };
            let Some(result) = self.store.finished_result(&piece)? else {
                continue;
            };
            let slot = answers
                .entry((piece.source_id, piece.index))
                .or_insert_with(|| vec![None; job.setups.len()]);
            if let Some(slot) = slot.get_mut(at) {
                *slot = Some(result.output.fields);
            }
        }
        for piece in answers.values() {
            let of: Vec<Option<&[FieldAnswer]>> = piece.iter().map(Option::as_deref).collect();
            tally.add(&of);
        }
        Ok(tally.finish())
    }
}
