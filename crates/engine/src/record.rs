//! A saved pipeline as a file to share or cite: its prompts frozen as a job would freeze them,
//! the catalogue's names for its models, and the settings each provider sends.

use scriptorium_core::provenance::ProvenancePrompt;
use scriptorium_prompts::{Pipeline, PipelineFile, Preset, SetupRecord};

use crate::{Engine, EngineError};

impl Engine {
    pub fn pipeline_file(&self, pipeline: Pipeline) -> Result<PipelineFile, EngineError> {
        let catalog = self.catalog();
        let mut setups = Vec::new();
        let mut presets: Vec<Preset> = Vec::new();
        for setup in &pipeline.setups {
            let provider = self.provider(&setup.provider)?;
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
            if takes_prompt && !presets.iter().any(|p| p.id == preset.id) {
                presets.push(preset);
            }
            setups.push(SetupRecord {
                provider_name: catalog
                    .provider(&setup.provider)
                    .map_or_else(|| setup.provider.to_string(), |p| p.name.clone()),
                model_name: catalog
                    .model(&setup.provider, &setup.model)
                    .map_or_else(|| setup.model.clone(), |m| m.name.clone()),
                request: provider.request_settings(pipeline.task, &setup.model),
                prompt: takes_prompt.then(|| ProvenancePrompt::from_snapshot(&prompt)),
            });
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
}
