//! After a sample (#215): the job that reads the rest of its documents with one of its set-ups,
//! and the comparison of another model on the same pages.
//!
//! Both are jobs like any other, made here from the sample rather than from New job, so that
//! the set-ups the sample ran are copied as they were frozen: the same provider, model, prompt
//! and options give the same cache keys ([`kalamu_core::CacheKey::compute`] leaves out the
//! set-up's id and the sample), and the pages the sample read are taken from the cache at no
//! cost when the new job runs. A prompt edited since the sample does not change them.

use kalamu_core::{
    CacheKey, Job, JobId, PieceRef, Setup, SetupId, SetupOptions, SetupRequest, StoreError,
};
use serde::Serialize;
use ts_rs::TS;

use crate::{Engine, EngineError, Estimate, EstimateRequest, NewJob};

/// What reading every page of a sample's documents with one of its set-ups would cost: the
/// pages the sample read come from the cache, priced at nothing.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct RestEstimate {
    /// The whole job, the pages already read left out of its price.
    pub estimate: Estimate,
    /// Pages the sample read under this set-up whose answers the cache still holds.
    pub cached_pages: u32,
}

impl Engine {
    /// The set-up `setup` of the job `sample`, as it was frozen.
    fn sample_setup(&self, sample: JobId, setup: SetupId) -> Result<(Job, Setup), EngineError> {
        let job = self.store.job(sample)?;
        let found = job
            .setups
            .iter()
            .find(|s| s.id == setup)
            .cloned()
            .ok_or(EngineError::Store(StoreError::NotFound))?;
        Ok((job, found))
    }

    /// How many pages `setup` of `sample` read whose answers the cache still holds: those the
    /// job reading the rest takes from it.
    fn cached_pages(&self, sample: &Job, setup: SetupId) -> Result<u32, EngineError> {
        let mut cached = 0;
        for piece in self.store.pieces(sample.id)? {
            let page = matches!(
                piece.piece,
                PieceRef::Page { .. } | PieceRef::Passage { .. }
            );
            if piece.setup_id != setup || !piece.status.is_finished() || !page {
                continue;
            }
            let Some(key) = piece.cache_key else { continue };
            if self.store.cached_result(&CacheKey(key))?.is_some() {
                cached += 1;
            }
        }
        Ok(cached)
    }

    /// What [`Engine::rest_of_sample`] would cost before it is made.
    pub fn rest_estimate(
        &self,
        sample: JobId,
        setup: SetupId,
    ) -> Result<RestEstimate, EngineError> {
        let (job, chosen) = self.sample_setup(sample, setup)?;
        let cached_pages = self.cached_pages(&job, setup)?;
        let request = EstimateRequest {
            task: job.task,
            sources: job.sources.clone(),
            setups: vec![request_of(job.task, &chosen)],
            sample_pages: None,
        };
        Ok(RestEstimate {
            estimate: self.estimate_beyond(&request, cached_pages),
            cached_pages,
        })
    }

    /// A job named `name` that reads every page of the sample's documents with its set-up
    /// `setup`, as frozen, and the sample's other settings; not started. The pages the sample
    /// read come from the cache when it runs.
    pub fn rest_of_sample(
        &self,
        sample: JobId,
        setup: SetupId,
        name: String,
    ) -> Result<Job, EngineError> {
        let (job, mut chosen) = self.sample_setup(sample, setup)?;
        self.provider(&chosen.provider)?;
        chosen.id = SetupId::new();
        let mut settings = job.settings.clone();
        settings.sample_pages = None;
        settings.sample_spread = false;
        // The whole document's files take over the sample's names, which were a first look.
        self.insert_job_continuing(
            name,
            job.task,
            job.sources,
            vec![chosen],
            settings,
            job.output,
            Some(sample),
        )
    }

    /// A job named `name` that reads the sample's pages again with its set-ups, as frozen, and
    /// beside them `model` of `provider` asked the same thing (the sample's prompt and
    /// options, the reasoning level left to the model): a comparison whose first set-ups come
    /// from the cache, so only the new model is paid. Not started.
    pub fn compare_sample(
        &self,
        sample: JobId,
        provider: kalamu_core::ProviderId,
        model: String,
        name: String,
    ) -> Result<Job, EngineError> {
        let job = self.store.job(sample)?;
        let first = job.setups.first().ok_or(EngineError::EmptyJob)?;
        // The prompt a set-up that takes one was given: a model that takes no prompt records
        // none.
        let preset_id = job.setups.iter().find_map(|s| s.prompt.preset_id.clone());
        let request = SetupRequest {
            provider,
            model,
            preset_id,
            label: None,
            options: SetupOptions {
                reasoning: None,
                ..first.options.clone()
            },
            merge: None,
        };
        // Checked as New job's comparison would be: the files the task reads, a model that
        // does it and reads them, analyses read alike.
        let mut setups: Vec<SetupRequest> =
            job.setups.iter().map(|s| request_of(job.task, s)).collect();
        setups.push(request.clone());
        let new = NewJob {
            name: name.clone(),
            task: job.task,
            sources: job.sources.clone(),
            setups,
            settings: job.settings.clone(),
            output: job.output.clone(),
        };
        let catalog = self.current_catalog();
        self.check_new(&catalog, &new)?;
        self.check_length(&catalog, &new)?;
        let mut added = self.freeze_setups(&catalog, job.task, &job.settings, &[request])?;
        let mut setups: Vec<Setup> = job
            .setups
            .iter()
            .cloned()
            .map(|mut s| {
                s.id = SetupId::new();
                s
            })
            .collect();
        for mut setup in added.drain(..) {
            setup.label = distinct(&setup.label, &setups);
            setups.push(setup);
        }
        let mut output = job.output;
        crate::rules::writable_formats(&catalog, job.task, &new.setups, &mut output.formats);
        self.insert_job(name, job.task, job.sources, setups, job.settings, output)
    }
}

/// A frozen set-up as New job would have asked for it, for the estimate and the checks: a
/// table of contents with the model that combines its pages.
fn request_of(task: kalamu_core::Task, setup: &Setup) -> SetupRequest {
    SetupRequest {
        provider: setup.provider.clone(),
        model: setup.model.clone(),
        preset_id: setup.prompt.preset_id.clone(),
        label: Some(setup.label.clone()),
        options: setup.options.clone(),
        merge: setup
            .merge
            .as_ref()
            .filter(|_| task == kalamu_core::Task::Contents)
            .map(|m| kalamu_core::MergeModel {
                provider: m.provider.clone(),
                model: m.model.clone(),
            }),
    }
}

/// `label`, or `label-2`, `label-3`… when another set-up has it: labels name files.
fn distinct(label: &str, setups: &[Setup]) -> String {
    let taken = |l: &str| setups.iter().any(|s| s.label == l);
    let mut out = label.to_string();
    let mut n = 2;
    while taken(&out) {
        out = format!("{label}-{n}");
        n += 1;
    }
    out
}
