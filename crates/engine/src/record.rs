//! A saved pipeline as a file to share or cite: its prompts frozen as a job would freeze them,
//! the catalogue's names for its models, and the settings each provider sends.

use kalamu_catalog::Catalog;
use kalamu_core::provenance::ProvenancePrompt;
use kalamu_core::{ProviderId, Task};
use kalamu_prompts::{Pipeline, PipelineFile, Preset, SetupRecord};

use crate::{Engine, EngineError, SetupRequest};

impl Engine {
    pub fn pipeline_file(&self, pipeline: Pipeline) -> Result<PipelineFile, EngineError> {
        let catalog = self.catalog();
        let mut setups = Vec::new();
        let mut presets: Vec<Preset> = Vec::new();
        for setup in &pipeline.setups {
            let (prompt, preset) = self.frozen_prompt(
                &catalog,
                pipeline.task,
                &pipeline.settings,
                &setup.provider,
                &setup.model,
                setup.preset_id.as_deref(),
                &setup.options,
            )?;
            let takes_prompt = prompt.preset_id.is_some();
            let mut record = self.record(
                &catalog,
                pipeline.task,
                &setup.provider,
                &setup.model,
                takes_prompt.then(|| ProvenancePrompt::from_snapshot(&prompt)),
            )?;
            // A table of contents' second step, frozen as the job would freeze it.
            if pipeline.task == Task::Contents {
                let request = SetupRequest {
                    provider: setup.provider.clone(),
                    model: setup.model.clone(),
                    preset_id: setup.preset_id.clone(),
                    label: None,
                    options: setup.options.clone(),
                    merge: setup.merge.clone(),
                };
                let merge =
                    self.merge_step(&catalog, &pipeline.settings, &request, preset.clone())?;
                record.merge = Some(Box::new(self.record(
                    &catalog,
                    Task::Contents,
                    &merge.provider,
                    &merge.model,
                    Some(ProvenancePrompt::from_snapshot(&merge.prompt)),
                )?));
            }
            if takes_prompt && !presets.iter().any(|p| p.id == preset.id) {
                presets.push(preset);
            }
            setups.push(record);
        }
        let language_names = Self::language_names(&catalog, &pipeline.settings);
        Ok(PipelineFile::new(
            pipeline,
            &self.config.app_version,
            language_names,
            setups,
            presets,
        ))
    }

    fn record(
        &self,
        catalog: &Catalog,
        task: Task,
        provider: &ProviderId,
        model: &str,
        prompt: Option<ProvenancePrompt>,
    ) -> Result<SetupRecord, EngineError> {
        Ok(SetupRecord {
            provider_name: catalog
                .provider(provider)
                .map_or_else(|| provider.to_string(), |p| p.name.clone()),
            model_name: catalog
                .model(provider, model)
                .map_or_else(|| model.to_string(), |m| m.name.clone()),
            request: self.provider(provider)?.request_settings(task, model),
            prompt,
            merge: None,
        })
    }
}
