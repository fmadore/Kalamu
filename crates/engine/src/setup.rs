//! Building a job from what the user chose: set-ups get their prompt frozen and checked against
//! the catalogue.

use kalamu_catalog::Catalog;
use kalamu_core::{
    InputKind, Job, JobId, JobSettings, JobStatus, LanguageChoice, MergeModel, MergeStep,
    OutputDestination, OutputSettings, PagesPerCall, PromptSnapshot, ProviderId, Setup, SetupId,
    SetupOptions, SetupRequest, SourceFile, Task, Timestamp,
};
use kalamu_prompts::{Preset, RenderContext};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

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
    pub fn create_job(&self, mut new: NewJob) -> Result<Job, EngineError> {
        let catalog = self.current_catalog();
        Self::check_new(&catalog, &new)?;
        crate::rules::writable_formats(&catalog, new.task, &new.setups, &mut new.output.formats);
        let setups = self.freeze_setups(&catalog, new.task, &new.settings, &new.setups)?;
        self.insert_job(
            new.name,
            new.task,
            new.sources,
            setups,
            new.settings,
            new.output,
        )
    }

    /// Whether a job can be made of what was chosen: files and set-ups, a folder for what has
    /// none to write beside, sources the task reads and models that do it.
    pub(crate) fn check_new(catalog: &Catalog, new: &NewJob) -> Result<(), EngineError> {
        if new.sources.is_empty() || new.setups.is_empty() {
            return Err(EngineError::EmptyJob);
        }
        // A video, or text frozen from a video's transcript, has no folder to write beside.
        let online = |s: &SourceFile| {
            s.kind == InputKind::YouTube
                || s.derived_from
                    .as_ref()
                    .is_some_and(|d| d.original_path.starts_with("http"))
        };
        if new.output.destination == OutputDestination::BesideSources
            && new.sources.iter().any(online)
        {
            return Err(EngineError::LinkNeedsFolder);
        }
        Self::check_job(catalog, new)
    }

    /// The set-ups as a job runs them: each prompt rendered with the job's settings and frozen,
    /// labels made distinct for file names, a table of contents' second step settled.
    pub(crate) fn freeze_setups(
        &self,
        catalog: &Catalog,
        task: Task,
        settings: &JobSettings,
        requests: &[SetupRequest],
    ) -> Result<Vec<Setup>, EngineError> {
        let mut setups = Vec::new();
        let mut labels: Vec<String> = Vec::new();
        for request in requests {
            let provider = self.provider(&request.provider)?;
            if provider.accepted_inputs(task).is_empty() {
                return Err(EngineError::UnsupportedModel(request.model.clone()));
            }
            let (prompt, preset) = self.frozen_prompt(
                catalog,
                task,
                settings,
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
            let dense = preset.is_dense();
            let merge = if task == Task::Contents {
                Some(self.merge_step(catalog, settings, request, preset)?)
            } else {
                None
            };
            // The reasoning level is settled now, so the job, its cache keys and provenance
            // hold the one sent: the choice, the nearest the model has, or the suggestion.
            let mut options = request.options.clone();
            if let Some(model) = catalog.model(&request.provider, &request.model) {
                options.reasoning = model.tuning(task, options.reasoning.as_deref()).reasoning;
                // Names to spell and translation only for a model that uses them, so another
                // model's results and cache keys are not changed by what it ignores.
                let terms = options.vocabulary_terms();
                options.vocabulary =
                    (model.vocabulary && !terms.is_empty()).then(|| terms.join("\n"));
                options.translate &= model.translates;
            }
            let mut setup = Setup {
                id: SetupId::new(),
                label,
                provider: request.provider.clone(),
                model: request.model.clone(),
                prompt,
                options,
                merge,
                pages_cap: None,
            };
            // Settled now, as the reasoning level is, so a catalogue refreshed before the job
            // resumes never regroups pages already read.
            let several = !matches!(settings.pages_per_call, None | Some(PagesPerCall::One));
            if task.groups_pages() && several {
                setup.pages_cap = Some(crate::group::pages_cap(catalog, &setup, dense));
            }
            setups.push(setup);
        }
        Ok(setups)
    }

    /// Records a new job with set-ups already frozen.
    pub(crate) fn insert_job(
        &self,
        name: String,
        task: Task,
        sources: Vec<SourceFile>,
        setups: Vec<Setup>,
        settings: JobSettings,
        output: OutputSettings,
    ) -> Result<Job, EngineError> {
        let job = build_job(name, task, sources, setups, settings, output);
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
        let ctx = Self::render_context(catalog, task, settings, options);
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

    /// The values a preset's variables take for a job; line breaks only where text is written
    /// out page by page.
    fn render_context(
        catalog: &Catalog,
        task: Task,
        settings: &JobSettings,
        options: &SetupOptions,
    ) -> RenderContext {
        RenderContext {
            languages: settings.languages.clone(),
            language_names: Self::language_names(catalog, settings),
            speakers: options.speakers,
            line_breaks: settings.line_breaks.filter(|_| task.has_line_breaks()),
            answers: matches!(task, Task::Analysis | Task::Contents),
        }
    }

    /// A table of contents' second step: the chosen model, or the pages' own, with the preset's
    /// instructions for combining the pages, frozen.
    pub(crate) fn merge_step(
        &self,
        catalog: &Catalog,
        settings: &JobSettings,
        request: &SetupRequest,
        preset: Preset,
    ) -> Result<MergeStep, EngineError> {
        let chosen = request.merge.clone().unwrap_or_else(|| MergeModel {
            provider: request.provider.clone(),
            model: request.model.clone(),
        });
        let provider = self.provider(&chosen.provider)?;
        let reads_text = catalog
            .model(&chosen.provider, &chosen.model)
            .is_none_or(|m| Task::merges(&m.inputs));
        if !provider
            .accepted_inputs(Task::Contents)
            .contains(&"text/plain")
            || !reads_text
        {
            return Err(EngineError::UnsupportedModel(chosen.model));
        }
        let ctx = Self::render_context(catalog, Task::Contents, settings, &request.options);
        let prompt = preset
            .merge_snapshot(&ctx)
            .ok_or(EngineError::Prompt(kalamu_prompts::PromptError::Incomplete))?;
        Ok(MergeStep {
            provider: chosen.provider,
            model: chosen.model,
            prompt,
        })
    }
}

/// Safe in a file name on Windows and macOS, and short in characters (Windows) and in UTF-8
/// bytes (macOS).
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
    let mut bytes = 0;
    cleaned
        .trim_matches(['-', '.'])
        .chars()
        .take(60)
        .take_while(|c| {
            bytes += c.len_utf8();
            bytes <= 90
        })
        .collect()
}

/// A new job of set-ups already frozen, not yet recorded.
pub(crate) fn build_job(
    name: String,
    task: Task,
    sources: Vec<SourceFile>,
    setups: Vec<Setup>,
    settings: JobSettings,
    mut output: OutputSettings,
) -> Job {
    // Formats the task does not write (from a pipeline of another task) are left out.
    output.formats.retain(|f| task.formats().contains(f));
    Job {
        id: JobId::new(),
        name,
        task,
        created_at: Timestamp::now(),
        sources,
        setups,
        settings,
        output,
        status: JobStatus::Pending,
    }
}
