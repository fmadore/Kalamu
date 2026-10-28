//! Building a job from what the user chose: set-ups get their prompt frozen and checked against
//! the catalogue.

use kalamu_catalog::Catalog;
use kalamu_core::{
    InputKind, Job, JobId, JobSettings, JobStatus, LanguageChoice, OutputDestination,
    OutputSettings, PromptSnapshot, ProviderId, Setup, SetupId, SetupOptions, SourceFile, Task,
    Timestamp,
};
use kalamu_prompts::{Preset, RenderContext};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// One provider, model and prompt to run, as chosen in the interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupRequest {
    pub provider: ProviderId,
    pub model: String,
    /// A preset id; the task's default when absent.
    pub preset_id: Option<String>,
    /// Shown in comparison columns and file names; derived from the model when absent.
    pub label: Option<String>,
    #[serde(default)]
    pub options: SetupOptions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NewJob {
    pub name: String,
    pub task: Task,
    pub sources: Vec<SourceFile>,
    pub setups: Vec<SetupRequest>,
    #[serde(default)]
    pub settings: JobSettings,
    #[serde(default)]
    pub output: OutputSettings,
}

impl Engine {
    pub fn create_job(&self, new: NewJob) -> Result<Job, EngineError> {
        if new.sources.is_empty() || new.setups.is_empty() {
            return Err(EngineError::EmptyJob);
        }
        if new.output.destination == OutputDestination::BesideSources
            && new.sources.iter().any(|s| s.kind == InputKind::YouTube)
        {
            return Err(EngineError::LinkNeedsFolder);
        }
        let catalog = self.catalog();
        let mut setups = Vec::new();
        let mut labels: Vec<String> = Vec::new();
        for request in &new.setups {
            let provider = self.provider(&request.provider)?;
            if provider.accepted_inputs(new.task).is_empty() {
                return Err(EngineError::UnsupportedModel(request.model.clone()));
            }
            let (prompt, _) = self.frozen_prompt(
                &catalog,
                new.task,
                &new.settings,
                &request.provider,
                &request.model,
                request.preset_id.as_deref(),
                &request.options,
            )?;
            let mut label = request
                .label
                .clone()
                .filter(|l| !l.trim().is_empty())
                .unwrap_or_else(|| request.model.clone());
            label = sanitize_label(&label);
            // Two set-ups with the same model (different prompts) need distinct file names.
            let base = label.clone();
            let mut n = 2;
            while labels.contains(&label) {
                label = format!("{base}-{n}");
                n += 1;
            }
            labels.push(label.clone());
            setups.push(Setup {
                id: SetupId::new(),
                label,
                provider: request.provider.clone(),
                model: request.model.clone(),
                prompt,
                options: request.options.clone(),
            });
        }
        let job = Job {
            id: JobId::new(),
            name: new.name,
            task: new.task,
            created_at: Timestamp::now(),
            sources: new.sources,
            setups,
            settings: new.settings,
            output: new.output,
            status: JobStatus::Pending,
        };
        self.store.create_job(&job)?;
        Ok(job)
    }

    /// English names of the chosen languages, as prompts state them; none for auto-detection.
    pub(crate) fn language_names(catalog: &Catalog, settings: &JobSettings) -> Vec<String> {
        if matches!(settings.languages, LanguageChoice::Auto) {
            return Vec::new();
        }
        settings
            .languages
            .codes()
            .iter()
            .map(|c| catalog.language_name(c))
            .collect()
    }

    /// The prompt a set-up sends, with the job's languages, line breaks and speakers filled in,
    /// and the preset it came from. A model that takes no prompt gets an empty one.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn frozen_prompt(
        &self,
        catalog: &Catalog,
        task: Task,
        settings: &JobSettings,
        provider: &ProviderId,
        model: &str,
        preset_id: Option<&str>,
        options: &SetupOptions,
    ) -> Result<(PromptSnapshot, Preset), EngineError> {
        let accepts_prompt = catalog
            .model(provider, model)
            .is_none_or(|m| m.accepts_prompt);
        let preset = match preset_id {
            Some(id) => self.with_prompts(|p| p.get(id))?,
            None => self.with_prompts(|p| p.default_for(task)),
        };
        let ctx = RenderContext {
            languages: settings.languages.clone(),
            language_names: Self::language_names(catalog, settings),
            speakers: options.speakers,
            line_breaks: settings
                .line_breaks
                .filter(|_| task == Task::TextExtraction),
        };
        let mut prompt = preset.snapshot(&ctx);
        if !accepts_prompt {
            // Dedicated OCR and speech models take no instructions; recording none keeps
            // provenance and the cache honest.
            prompt.text = String::new();
            prompt.user_prompt = String::new();
            prompt.template = String::new();
            prompt.preset_id = None;
            prompt.preset_name = "None (this model takes no prompt)".into();
        }
        Ok((prompt, preset))
    }
}

/// Safe in a Windows file name.
fn sanitize_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();
    cleaned.trim_matches(['-', '.']).chars().take(60).collect()
}
