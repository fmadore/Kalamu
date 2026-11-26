//! What work started now would send, and to which provider: read before a job is started,
//! resumed or retried, or a workflow step approved, so the researcher's consent is asked for
//! every provider it may reach; and before a server of the researcher's own is changed or
//! removed, so one that running work still sends to is left as it is
//! ([decision](../../../docs/decisions/2026-11-26-a-server-of-the-researchers-own.md)).
//!
//! A job sends to the providers of its set-ups and of the steps that combine their answers.
//! A job that belongs to a workflow run also starts, when it ends, the run's steps that have no
//! job yet, so their providers are ahead of it too; a step already given a job is that job's to
//! start, and a cancelled run starts nothing more.

use crate::{Job, JobId, JobStore, ProviderId, RunId, Setup, StoreError, WorkflowRun};

/// A model that work sends to, on its provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sends {
    pub provider: ProviderId,
    pub model: String,
}

/// Adds what `setups` send to `out`, each model once.
fn add<'a>(out: &mut Vec<Sends>, setups: impl IntoIterator<Item = &'a Setup>) {
    for setup in setups {
        let combining = setup
            .merge
            .as_ref()
            .map(|m| (m.provider.clone(), m.model.clone()));
        for (provider, model) in
            std::iter::once((setup.provider.clone(), setup.model.clone())).chain(combining)
        {
            let sends = Sends { provider, model };
            if !out.contains(&sends) {
                out.push(sends);
            }
        }
    }
}

impl Job {
    /// What this job sends: each set-up's model, and the model that combines its answers.
    pub fn sends(&self) -> Vec<Sends> {
        let mut out = Vec::new();
        add(&mut out, &self.setups);
        out
    }
}

impl WorkflowRun {
    /// What the run may still start on its own: the steps that have no job yet. None once the
    /// run is cancelled.
    pub fn sends_to_come(&self) -> Vec<Sends> {
        let mut out = Vec::new();
        if !self.cancelled {
            add(
                &mut out,
                self.steps
                    .iter()
                    .filter(|s| s.progress.job_id.is_none())
                    .flat_map(|s| &s.setups),
            );
        }
        out
    }
}

/// What starting `job` now would send: its own models, then those of the steps of its workflow
/// run that may start when it ends.
pub fn job_sends(store: &dyn JobStore, job: JobId) -> Result<Vec<Sends>, StoreError> {
    let mut out = store.job(job)?.sends();
    if let Some((run, _)) = store.run_of_job(job)? {
        for sends in store.run(run)?.sends_to_come() {
            if !out.contains(&sends) {
                out.push(sends);
            }
        }
    }
    Ok(out)
}

/// What going on with `run` (a step approved) may start: its steps that have no job yet.
pub fn run_sends(store: &dyn JobStore, run: RunId) -> Result<Vec<Sends>, StoreError> {
    Ok(store.run(run)?.sends_to_come())
}

/// The providers among `sends`, each once, in order.
pub fn providers(sends: &[Sends]) -> Vec<ProviderId> {
    let mut out: Vec<ProviderId> = Vec::new();
    for s in sends {
        if !out.contains(&s.provider) {
            out.push(s.provider.clone());
        }
    }
    out
}

/// The models of `provider` that the jobs in `running` send to, or may start a step that sends
/// to: none means the provider can be changed or removed without changing work under way. A job
/// no longer in the store (deleted as it stopped) sends nothing.
pub fn models_in_use(
    store: &dyn JobStore,
    running: impl IntoIterator<Item = JobId>,
    provider: &ProviderId,
) -> Result<Vec<String>, StoreError> {
    let mut out: Vec<String> = Vec::new();
    for job in running {
        let sends = match job_sends(store, job) {
            Ok(sends) => sends,
            Err(StoreError::NotFound) => continue,
            Err(e) => return Err(e),
        };
        for s in sends {
            if &s.provider == provider && !out.contains(&s.model) {
                out.push(s.model);
            }
        }
    }
    Ok(out)
}
