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

/// Images made one document of pages for New job: the source in natural order, what each
/// page's name says of its number, and the checks of every page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImageDocument {
    pub source: SourceFile,
    pub notes: Vec<kalamu_core::PageNote>,
    pub warnings: Vec<kalamu_core::InputWarning>,
}

impl Engine {
    pub fn create_job(&self, mut new: NewJob) -> Result<Job, EngineError> {
        // A document of pages as the researcher ordered it: its hash, count and place follow
        // from its pages, worked out here rather than trusted from the interface.
        for source in &mut new.sources {
            source.settle_pages();
        }
        let catalog = self.current_catalog();
        self.check_new(&catalog, &new)?;
        self.check_length(&catalog, &new)?;
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
    /// none to write beside, sources the task reads and models that do it and can read them.
    pub(crate) fn check_new(&self, catalog: &Catalog, new: &NewJob) -> Result<(), EngineError> {
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
        let by_page = self.reads_by_page(new)?;
        Self::check_job(catalog, new, by_page)?;
        self.check_cited(new)?;
        self.check_reading(catalog, new, by_page)
    }

    /// Whether a new analysis reads page by page, its presets' fields one repeating group each
    /// ([`crate::plan::reads_by_page`] for a job made); refused when some set-ups would and
    /// others would not, as their results would not line up.
    pub(crate) fn reads_by_page(&self, new: &NewJob) -> Result<bool, EngineError> {
        if new.task != Task::Analysis {
            return Ok(false);
        }
        let mut records = Vec::new();
        for request in &new.setups {
            let preset = match request.preset_id.as_deref() {
                Some(id) => self.with_prompts(|p| p.get(id))?,
                None => self.with_prompts(|p| p.default_for(new.task)),
            };
            records.push(
                preset
                    .schema
                    .as_ref()
                    .is_some_and(kalamu_core::records::is_records),
            );
        }
        match (records.iter().all(|r| *r), records.iter().any(|r| *r)) {
            (true, _) => Ok(!records.is_empty()),
            (false, true) => Err(EngineError::MixedReading),
            (false, false) => Ok(false),
        }
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
            // The question kept with the prompt is settled into the set-up as if it had been
            // typed, so the job, its cache keys and provenance hold the one sent, and the same
            // words cost nothing twice whichever way they came.
            let mut options = with_preset_question(task, &request.options, &preset);
            // Themes that are only blank lines are none, as an empty box is.
            if options.theme_lines().is_empty() {
                options.themes = None;
            }
            let merge = if task == Task::Contents {
                Some(self.merge_step(catalog, settings, request, preset.clone())?)
            } else {
                None
            };
            // The reasoning level is settled now, so the job, its cache keys and provenance
            // hold the one sent: the choice, the nearest the model has, or the suggestion.
            if let Some(model) = catalog.model(&request.provider, &request.model) {
                options.reasoning = model.tuning(task, options.reasoning.as_deref()).reasoning;
                // Names to spell and translation only for a model that uses them, so another
                // model's results and cache keys are not changed by what it ignores.
                let terms = options.vocabulary_terms();
                options.vocabulary =
                    (model.vocabulary && !terms.is_empty()).then(|| terms.join("\n"));
                options.translate &= model.translates;
            }
            // An analysis that cites the text reads one too long for its model in parts, then
            // brings them together on the same model: settled now from the catalogue's limit,
            // used only for a text that does not fit.
            let windows = (task == Task::Analysis)
                .then(|| catalog.model(&request.provider, &request.model))
                .flatten()
                .and_then(crate::windows::windows_for);
            let together = windows
                .and_then(|_| {
                    preset.consolidation_snapshot(&Self::render_context(
                        catalog, task, settings, &options,
                    ))
                })
                .filter(|_| prompt.preset_id.is_some());
            let mut setup = Setup {
                id: SetupId::new(),
                label,
                provider: request.provider.clone(),
                // An id the catalogue once gave the model is sent as it is now.
                model: catalog
                    .canonical_id(&request.provider, &request.model)
                    .to_string(),
                prompt,
                options,
                merge,
                pages_cap: None,
                windows: None,
            };
            if let Some(mut step) = together {
                step.user_prompt = setup.options.with_question(&step.user_prompt);
                setup.merge = Some(MergeStep {
                    provider: setup.provider.clone(),
                    model: setup.model.clone(),
                    prompt: step,
                });
                setup.windows = windows;
            }
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

    /// Records a new job with set-ups already frozen, its file names settled against the jobs
    /// kept ([`Engine::settle_labels`]).
    pub(crate) fn insert_job(
        &self,
        name: String,
        task: Task,
        sources: Vec<SourceFile>,
        setups: Vec<Setup>,
        settings: JobSettings,
        output: OutputSettings,
    ) -> Result<Job, EngineError> {
        self.insert_job_continuing(name, task, sources, setups, settings, output, None)
    }

    /// [`Engine::insert_job`] for a job that carries on `continues` (the rest of a sample),
    /// whose file names it may take over.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_job_continuing(
        &self,
        name: String,
        task: Task,
        sources: Vec<SourceFile>,
        setups: Vec<Setup>,
        settings: JobSettings,
        output: OutputSettings,
        continues: Option<JobId>,
    ) -> Result<Job, EngineError> {
        let mut job = build_job(name, task, sources, setups, settings, output);
        self.settle_labels(&mut job, continues)?;
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
        // The researcher's question goes with an analysis's request: the one typed for the
        // set-up, or else the one its prompt keeps.
        if task == Task::Analysis {
            let options = with_preset_question(task, options, &preset);
            prompt.user_prompt = options.with_question(&prompt.user_prompt);
        }
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
            answers: task.answers_in_language(),
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
        let mut chosen = request.merge.clone().unwrap_or_else(|| MergeModel {
            provider: request.provider.clone(),
            model: request.model.clone(),
        });
        chosen.model = catalog
            .canonical_id(&chosen.provider, &chosen.model)
            .to_string();
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

/// A set-up's options with the question and the themes its analysis prompt keeps
/// ([`Preset::question`], [`Preset::themes`]) when the researcher typed none for it: what is typed
/// replaces the prompt's. Unchanged for another task, or a prompt without them, so those cache
/// keys stay as they were.
pub(crate) fn with_preset_question(
    task: Task,
    options: &SetupOptions,
    preset: &Preset,
) -> SetupOptions {
    let mut out = options.clone();
    if task != Task::Analysis {
        return out;
    }
    let typed = |v: &Option<String>| v.as_deref().is_some_and(|q| !q.trim().is_empty());
    let kept = |v: &Option<String>| {
        v.as_deref()
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(str::to_string)
    };
    if !typed(&options.question)
        && let Some(question) = kept(&preset.question)
    {
        out.question = Some(question);
    }
    if !typed(&options.themes)
        && let Some(themes) = kept(&preset.themes)
    {
        out.themes = Some(themes);
    }
    out
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
    // Formats the task does not write (from a pipeline of another task) are left out, and
    // subtitles of a translation whose text has no times (`Task::writes`).
    output.formats.retain(|f| task.writes(*f, &sources));
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
