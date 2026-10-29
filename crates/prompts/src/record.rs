//! A pipeline as a file to share or cite: everything that decides its results, from the models
//! and the settings each provider sends to the full text of every prompt and its fields. The
//! JSON file can be imported into another Kalamu; the Markdown version reads as a methods
//! appendix. Both are built by the engine, which knows the catalogue and the providers.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use kalamu_core::provenance::{GeneratorInfo, ProvenancePrompt};
use kalamu_core::{
    FieldKind, LanguageChoice, LineBreaks, OutputDestination, RequestSetting, Task, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Pipeline, PipelineLibrary, Preset, PromptError, PromptLibrary};

const FILE_FORMAT: &str = "kalamu-pipeline";
/// Files exported before the app was renamed from Scriptorium.
const LEGACY_FILE_FORMAT: &str = "scriptorium-pipeline";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PipelineFile {
    pub format: String,
    pub version: u32,
    pub generator: GeneratorInfo,
    pub exported_at: Timestamp,
    /// Without its output folder, a local path that means nothing elsewhere.
    pub pipeline: Pipeline,
    /// English names of the chosen languages, as the prompts state them.
    pub language_names: Vec<String>,
    /// One per set-up, in the pipeline's order.
    pub setups: Vec<SetupRecord>,
    /// The presets the set-ups use, so the file runs the same prompts elsewhere.
    pub presets: Vec<Preset>,
}

/// What one set-up sends, in words a methods section can quote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupRecord {
    pub provider_name: String,
    pub model_name: String,
    /// Fixed settings the provider sends with every request.
    pub request: Vec<RequestSetting>,
    /// The prompt as sent, hashed as in each result's provenance. `None` for a model that
    /// takes no prompt.
    pub prompt: Option<ProvenancePrompt>,
    /// For a table of contents, the second step, which combines the pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge: Option<Box<SetupRecord>>,
}

impl PipelineFile {
    pub fn new(
        mut pipeline: Pipeline,
        app_version: &str,
        language_names: Vec<String>,
        setups: Vec<SetupRecord>,
        presets: Vec<Preset>,
    ) -> Self {
        pipeline.destination = OutputDestination::default();
        Self {
            format: FILE_FORMAT.into(),
            version: 1,
            generator: GeneratorInfo {
                name: "Kalamu".into(),
                version: app_version.into(),
            },
            exported_at: Timestamp::now(),
            pipeline,
            language_names,
            setups,
            presets,
        }
    }

    pub fn to_json(&self) -> Result<String, PromptError> {
        serde_json::to_string_pretty(self).map_err(crate::io)
    }

    /// The record as Markdown, for a methods section or an appendix.
    pub fn to_markdown(&self) -> String {
        let p = &self.pipeline;
        let mut out = String::new();
        let _ = writeln!(out, "# {}\n", p.name);
        let _ = writeln!(
            out,
            "A Kalamu pipeline, exported on {} from {} {}. The JSON file exported with \
             it can be imported into Kalamu to run the same pipeline.\n",
            date(self.exported_at),
            self.generator.name,
            self.generator.version
        );
        out.push_str("## Task\n\n");
        let _ = writeln!(out, "- Task: {}", task_name(p.task));
        let languages = match &p.settings.languages {
            LanguageChoice::Explicit(_) if !self.language_names.is_empty() => {
                self.language_names.join(", ")
            }
            _ => "detected automatically".into(),
        };
        let _ = writeln!(out, "- Languages: {languages}");
        if matches!(p.task, Task::TextExtraction | Task::TextCorrection) {
            let lines = match p.settings.line_breaks {
                Some(LineBreaks::AsOnPage) => "kept as on the page",
                Some(LineBreaks::RunningText) => "joined into running text",
                None => "as the prompt says",
            };
            let _ = writeln!(out, "- Lines: {lines}");
        }
        if let Some(n) = p.settings.sample_pages {
            let _ = writeln!(out, "- Sample: the first {n} pages of each document only");
        }
        if !p.formats.is_empty() {
            let formats: Vec<&str> = p.formats.iter().map(|f| f.extension()).collect();
            let _ = writeln!(out, "- Output formats: {}", formats.join(", "));
        }
        let several = self.setups.len() > 1;
        for (i, (setup, record)) in p.setups.iter().zip(&self.setups).enumerate() {
            let title = setup
                .label
                .clone()
                .filter(|l| !l.trim().is_empty())
                .unwrap_or_else(|| record.model_name.clone());
            if several {
                let _ = writeln!(out, "\n## Set-up {}: {title}\n", i + 1);
            } else {
                let _ = writeln!(out, "\n## Model and prompt\n");
            }
            if p.task == Task::Contents {
                out.push_str("Each page is read for the articles or items on it.\n\n");
            }
            let _ = writeln!(out, "- Provider: {}", record.provider_name);
            let _ = writeln!(out, "- Model: {} (`{}`)", record.model_name, setup.model);
            if p.task == Task::AudioTranscription {
                let yes_no = |b: bool| if b { "yes" } else { "no" };
                let _ = writeln!(out, "- Speaker labels: {}", yes_no(setup.options.diarize));
                let _ = writeln!(out, "- Timestamps: {}", yes_no(setup.options.timestamps));
                if let Some(n) = setup.options.speakers {
                    let _ = writeln!(out, "- Expected speakers: {n}");
                }
            }
            request_settings(&mut out, record);
            let Some(prompt) = &record.prompt else {
                out.push_str("- Prompt: none, this model takes no instructions\n");
                continue;
            };
            let origin = self
                .presets
                .iter()
                .find(|preset| Some(&preset.id) == prompt.preset_id.as_ref())
                .map_or("", |preset| match (preset.builtin, preset.modified) {
                    (true, false) => " (built in)",
                    (true, true) => " (built in, edited)",
                    _ => " (the researcher's own)",
                });
            let _ = writeln!(out, "- Prompt: {}{origin}", prompt.preset_name);
            let _ = writeln!(out, "- Prompt SHA-256: `{}`", prompt.sha256);
            out.push_str("\n### Instructions\n\n");
            fenced(&mut out, &prompt.text);
            out.push_str("\n### Request sent with each piece\n\n");
            fenced(&mut out, &prompt.user_prompt);
            if let Some(schema) = &prompt.schema {
                out.push_str("\n### Fields\n\n");
                out.push_str("| Field | Answer | Instructions | Reason |\n");
                out.push_str("| --- | --- | --- | --- |\n");
                fields_table(&mut out, &schema.fields, "");
            }
            if let Some(merge) = &record.merge {
                out.push_str("\n### Combining the pages\n\n");
                out.push_str(
                    "Each document's page readings are then combined into one entry per article \
                     or item, in one call.\n\n",
                );
                let _ = writeln!(out, "- Provider: {}", merge.provider_name);
                let model = setup.merge.as_ref().map_or("", |m| m.model.as_str());
                let _ = writeln!(out, "- Model: {} (`{model}`)", merge.model_name);
                request_settings(&mut out, merge);
                if let Some(prompt) = &merge.prompt {
                    let _ = writeln!(out, "- Prompt SHA-256: `{}`", prompt.sha256);
                    out.push_str("\n#### Instructions\n\n");
                    fenced(&mut out, &prompt.text);
                    out.push_str("\n#### Request\n\n");
                    fenced(&mut out, &prompt.user_prompt);
                }
            }
        }
        out
    }
}

fn request_settings(out: &mut String, record: &SetupRecord) {
    if record.request.is_empty() {
        out.push_str("- Request settings: the provider's defaults\n");
    } else {
        let settings: Vec<String> = record
            .request
            .iter()
            .map(|s| format!("`{}` = `{}`", s.name, s.value))
            .collect();
        let _ = writeln!(out, "- Request settings: {}", settings.join(", "));
    }
}

/// One row per field; an entries field's own fields follow it, marked as belonging to it.
fn fields_table(out: &mut String, fields: &[kalamu_core::Field], parent: &str) {
    for field in fields {
        let answer = match &field.kind {
            FieldKind::Text => "text".to_string(),
            FieldKind::List { max: Some(max) } => format!("list, at most {max}"),
            FieldKind::List { max: None } => "list".to_string(),
            FieldKind::Choice { values } => format!("one of: {}", values.join(", ")),
            FieldKind::Choices { values } => format!("any of: {}", values.join(", ")),
            FieldKind::Entries { .. } => "entries, each with the fields below".to_string(),
        };
        let _ = writeln!(
            out,
            "| {parent}{} (`{}`) | {} | {} | {} |",
            cell(&field.label),
            field.key,
            cell(&answer),
            cell(&field.instructions),
            if field.reason { "yes" } else { "no" }
        );
        if let FieldKind::Entries { fields } = &field.kind {
            fields_table(out, fields, &format!("{parent}{} › ", cell(&field.label)));
        }
    }
}

impl PipelineLibrary {
    /// Adds the pipeline in a shared file, with its presets. A preset identical to one already
    /// here is used as it is; any other becomes a new user preset, so an import never changes
    /// the user's own prompts or the defaults.
    pub fn import(
        &mut self,
        json: &str,
        prompts: &mut PromptLibrary,
    ) -> Result<Pipeline, PromptError> {
        let file: PipelineFile =
            serde_json::from_str(json).map_err(|e| PromptError::InvalidImport(e.to_string()))?;
        if (file.format != FILE_FORMAT && file.format != LEGACY_FILE_FORMAT) || file.version > 1 {
            return Err(PromptError::InvalidImport(file.format));
        }
        let mut ids = BTreeMap::new();
        for preset in file.presets {
            let same = prompts.get(&preset.id).is_ok_and(|local| {
                local.task == preset.task
                    && local.instructions == preset.instructions
                    && local.user_prompt == preset.user_prompt
                    && local.schema == preset.schema
                    && local.merge_instructions == preset.merge_instructions
            });
            let id = if same {
                preset.id.clone()
            } else {
                let mut copy = preset.clone();
                copy.id = String::new();
                copy.builtin = false;
                copy.modified = false;
                if preset.builtin {
                    copy.name = format!("{} ({})", preset.name, file.pipeline.name);
                }
                prompts.save(copy)?.id
            };
            ids.insert(preset.id, id);
        }
        let mut pipeline = file.pipeline;
        for setup in &mut pipeline.setups {
            if let Some(id) = setup.preset_id.as_mut()
                && let Some(new) = ids.get(id.as_str())
            {
                *id = new.clone();
            }
        }
        pipeline.id = String::new();
        pipeline.destination = OutputDestination::default();
        // Never replace a pipeline of the same name.
        let taken = |name: &str| {
            self.list()
                .iter()
                .any(|p| p.name.to_lowercase() == name.to_lowercase())
        };
        let base = pipeline.name.trim().to_string();
        let mut n = 2;
        while taken(&pipeline.name) {
            pipeline.name = format!("{base} ({n})");
            n += 1;
        }
        self.save(pipeline)
    }
}

fn task_name(task: Task) -> &'static str {
    match task {
        Task::TextExtraction => "text extraction",
        Task::AudioTranscription => "audio transcription",
        Task::Analysis => "analysis",
        Task::Contents => "detailed table of contents",
        Task::TextCorrection => "correction of existing text",
    }
}

/// A fence longer than any run of backticks in the text, so a prompt quoting code stays whole.
fn fenced(out: &mut String, text: &str) {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest.max(2) + 1);
    let _ = writeln!(out, "{fence}text\n{}\n{fence}", text.trim_end());
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\r', '\n'], " ")
}

/// `YYYY-MM-DD` in UTC, from Howard Hinnant's `civil_from_days`.
fn date(at: Timestamp) -> String {
    let days = at.0.div_euclid(86_400_000);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kalamu_core::{ExportFormat, JobSettings, ProviderId, SetupOptions};

    use super::*;
    use crate::{PipelineSetup, RenderContext};

    fn file(prompts: &PromptLibrary, preset: &str) -> PipelineFile {
        let preset = prompts.get(preset).unwrap();
        let snapshot = preset.snapshot(&RenderContext::default());
        let pipeline = Pipeline {
            id: "pipeline-1".into(),
            name: "Literature review".into(),
            task: Task::Analysis,
            setups: vec![PipelineSetup {
                provider: ProviderId::new("mistral"),
                model: "mistral-small-2603".into(),
                preset_id: Some(preset.id.clone()),
                label: None,
                options: SetupOptions::default(),
                merge: None,
            }],
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Csv],
            destination: OutputDestination::Folder("C:\\Users\\someone\\results".into()),
            updated_at: Timestamp(0),
        };
        PipelineFile::new(
            pipeline,
            "1.2.3",
            Vec::new(),
            vec![SetupRecord {
                provider_name: "Mistral".into(),
                model_name: "Mistral Small 4".into(),
                request: vec![RequestSetting::new("reasoning_effort", "none")],
                prompt: Some(ProvenancePrompt {
                    preset_id: snapshot.preset_id,
                    preset_name: snapshot.preset_name,
                    text: snapshot.text,
                    user_prompt: snapshot.user_prompt,
                    sha256: "abc".into(),
                    schema: snapshot.schema,
                }),
                merge: None,
            }],
            vec![preset],
        )
    }

    #[test]
    fn the_record_states_the_model_settings_prompt_and_fields_but_no_local_folder() {
        let prompts = PromptLibrary::in_memory();
        let file = file(&prompts, "analysis.tone");
        assert_eq!(file.pipeline.destination, OutputDestination::default());
        let md = file.to_markdown();
        assert!(md.starts_with("# Literature review\n"));
        assert!(md.contains("- Model: Mistral Small 4 (`mistral-small-2603`)"));
        assert!(md.contains("`reasoning_effort` = `none`"));
        assert!(md.contains("- Prompt: Tone of the document (built in)"));
        assert!(md.contains("### Instructions\n\n```text\n# Tone of the document"));
        assert!(md.contains("| Tone (`tone`) | one of: Positive, Neutral, Negative, Mixed |"));
        assert!(!md.contains("someone"));
    }

    #[test]
    fn importing_reuses_identical_presets_and_copies_changed_ones() {
        let mut prompts = PromptLibrary::in_memory();
        let mut pipelines = PipelineLibrary::in_memory();
        let json = file(&prompts, "analysis.tone").to_json().unwrap();
        let first = pipelines.import(&json, &mut prompts).unwrap();
        assert_eq!(first.setups[0].preset_id.as_deref(), Some("analysis.tone"));
        assert_eq!(first.name, "Literature review");

        let mut edited = file(&prompts, "analysis.tone");
        edited.presets[0].instructions = "Judge the tone.".into();
        let second = pipelines
            .import(&edited.to_json().unwrap(), &mut prompts)
            .unwrap();
        assert_eq!(
            second.name, "Literature review (2)",
            "never replaces a pipeline"
        );
        let copy = prompts
            .get(second.setups[0].preset_id.as_deref().unwrap())
            .unwrap();
        assert_eq!(copy.instructions, "Judge the tone.");
        assert!(!copy.builtin);
        assert_eq!(
            prompts.get("analysis.tone").unwrap().instructions,
            PromptLibrary::in_memory()
                .get("analysis.tone")
                .unwrap()
                .instructions,
            "the default is untouched"
        );

        assert!(pipelines.import("{}", &mut prompts).is_err());
    }

    #[test]
    fn pipelines_exported_by_scriptorium_still_import() {
        let mut prompts = PromptLibrary::in_memory();
        let mut pipelines = PipelineLibrary::in_memory();
        let mut old = file(&prompts, "analysis.tone");
        old.format = LEGACY_FILE_FORMAT.into();
        let imported = pipelines
            .import(&old.to_json().unwrap(), &mut prompts)
            .unwrap();
        assert_eq!(imported.name, "Literature review");

        old.format = "another-app".into();
        assert!(
            pipelines
                .import(&old.to_json().unwrap(), &mut prompts)
                .is_err()
        );
    }

    #[test]
    fn dates_are_written_in_utc() {
        assert_eq!(date(Timestamp(0)), "1970-01-01");
        assert_eq!(date(Timestamp(1_793_096_727_000)), "2026-10-27");
    }
}
