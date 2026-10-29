//! Prompt presets: the defaults shipped with the app, the user's own presets and edited
//! defaults, variables, and import/export for sharing across a team.
//!
//! Defaults are never changed on disk. Editing one saves an override under the same id;
//! resetting deletes the override. User presets are one JSON file each in the presets folder.

mod defaults;
mod pipelines;
mod record;

pub use pipelines::{Pipeline, PipelineLibrary, PipelineSetup};
pub use record::{PipelineFile, SetupRecord};

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use kalamu_core::{
    AnalysisSchema, FieldKind, LanguageChoice, LineBreaks, PromptSnapshot, SchemaError, Task,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Preset {
    pub id: String,
    pub task: Task,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Sent as the system prompt. May use `{languages}` and `{speakers}`.
    pub instructions: String,
    /// Sent with each page or segment.
    pub user_prompt: String,
    /// Shipped with the app (possibly edited, see `modified`).
    #[serde(default)]
    pub builtin: bool,
    /// A default the user has edited; resetting restores the shipped text.
    #[serde(default)]
    pub modified: bool,
    /// For an analysis, the fields to fill in for each document; for a table of contents, the
    /// fields of each entry, beside its title and pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub schema: Option<AnalysisSchema>,
    /// For a table of contents, the instructions of the second step, which combines the pages'
    /// readings into entries. May use `{languages}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge_instructions: Option<String>,
}

/// The request sent with a table of contents' second step.
pub const MERGE_REQUEST: &str = "Combine the pages above into one entry per article or item, \
following the instructions given.";

/// Appended to a correction's instructions: where the corrected text goes in the answer.
const CORRECTION_ANSWER: &str = "\n\n## Answer\n\nPut the whole corrected text, and nothing \
else, in the field `text`. When the text needs no correction, return it unchanged.";

/// Values for a preset's variables.
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    pub languages: LanguageChoice,
    /// English names for the chosen languages, in order (`French`, `Mooré`).
    pub language_names: Vec<String>,
    pub speakers: Option<u32>,
    /// The researcher's choice for a text extraction's lines; it overrides the preset's own.
    pub line_breaks: Option<LineBreaks>,
    /// The languages are the ones the model writes its answers in (an analysis, a table of
    /// contents), not the source's.
    pub answers: bool,
}

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Preset not found: {0}")]
    NotFound(String),
    #[error("The preset needs a name and instructions")]
    Incomplete,
    #[error("Could not read or save presets: {0}")]
    Io(String),
    #[error("This is not a Kalamu presets file: {0}")]
    InvalidImport(String),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error("A pipeline needs a name and at least one model")]
    IncompletePipeline,
}

/// The file format used to share presets.
#[derive(Debug, Serialize, Deserialize)]
struct PresetFile {
    format: String,
    version: u32,
    presets: Vec<Preset>,
}

const FILE_FORMAT: &str = "kalamu-prompts";
/// Files shared before the app was renamed from Scriptorium.
const LEGACY_FILE_FORMAT: &str = "scriptorium-prompts";

pub struct PromptLibrary {
    dir: Option<PathBuf>,
    /// User presets and edited defaults, by id.
    saved: BTreeMap<String, Preset>,
}

impl PromptLibrary {
    /// Loads saved presets from `dir`, creating it if needed. Unreadable files are skipped so one
    /// broken preset never hides the others.
    pub fn open(dir: &Path) -> Result<Self, PromptError> {
        fs::create_dir_all(dir).map_err(io)?;
        let mut saved = BTreeMap::new();
        for entry in fs::read_dir(dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(preset) = fs::read_to_string(&path)
                .map_err(io)
                .and_then(|s| serde_json::from_str::<Preset>(&s).map_err(io))
            {
                saved.insert(preset.id.clone(), preset);
            }
        }
        Ok(Self {
            dir: Some(dir.to_path_buf()),
            saved,
        })
    }

    /// Defaults only, nothing saved; for tests.
    pub fn in_memory() -> Self {
        Self {
            dir: None,
            saved: BTreeMap::new(),
        }
    }

    /// Defaults first, in their shipped order, then the user's presets by name.
    pub fn list(&self, task: Option<Task>) -> Vec<Preset> {
        let mut out: Vec<Preset> = defaults::DEFAULTS
            .iter()
            .map(|d| self.saved.get(d.id).cloned().unwrap_or_else(|| builtin(d)))
            .collect();
        let mut own: Vec<Preset> = self
            .saved
            .values()
            .filter(|p| !is_builtin_id(&p.id))
            .cloned()
            .collect();
        own.sort_by_key(|p| p.name.to_lowercase());
        out.extend(own);
        out.retain(|p| task.is_none_or(|t| p.task == t));
        out
    }

    pub fn get(&self, id: &str) -> Result<Preset, PromptError> {
        if let Some(preset) = self.saved.get(id) {
            return Ok(preset.clone());
        }
        defaults::DEFAULTS
            .iter()
            .find(|d| d.id == id)
            .map(builtin)
            .ok_or_else(|| PromptError::NotFound(id.to_string()))
    }

    /// The default to use for a task when the user has not chosen one.
    pub fn default_for(&self, task: Task) -> Preset {
        self.list(Some(task))
            .into_iter()
            .next()
            .unwrap_or_else(|| builtin(&defaults::DEFAULTS[0]))
    }

    /// Saves a preset. A new preset (empty id) gets a fresh id; saving a default stores an edited
    /// copy under the default's id.
    pub fn save(&mut self, mut preset: Preset) -> Result<Preset, PromptError> {
        if preset.name.trim().is_empty() || preset.instructions.trim().is_empty() {
            return Err(PromptError::Incomplete);
        }
        // An analysis is its fields, a table of contents its entries' fields and the
        // instructions that combine its pages; other presets have neither.
        match preset.task {
            Task::Analysis => {
                let schema = preset.schema.as_mut().ok_or(SchemaError::Empty)?;
                schema.assign_keys();
                schema.validate()?;
                preset.merge_instructions = None;
            }
            Task::Contents => {
                let schema = preset.schema.as_mut().ok_or(SchemaError::Empty)?;
                schema.assign_keys();
                kalamu_core::contents::validate(schema)?;
                if preset
                    .merge_instructions
                    .as_deref()
                    .is_none_or(|m| m.trim().is_empty())
                {
                    return Err(PromptError::Incomplete);
                }
            }
            Task::TextExtraction | Task::AudioTranscription | Task::TextCorrection => {
                preset.schema = None;
                preset.merge_instructions = None;
            }
        }
        if preset.id.is_empty() {
            preset.id = new_user_id();
        }
        let is_default = is_builtin_id(&preset.id);
        if !is_default && !is_user_id(&preset.id) {
            preset.id = new_user_id();
        }
        preset.builtin = is_default;
        preset.modified = is_default;
        if let Some(dir) = &self.dir {
            write_atomic(&dir.join(format!("{}.json", preset.id)), &preset)?;
        }
        self.saved.insert(preset.id.clone(), preset.clone());
        Ok(preset)
    }

    /// Deletes a user preset, or resets an edited default to its shipped text.
    pub fn delete(&mut self, id: &str) -> Result<(), PromptError> {
        if self.saved.remove(id).is_none() && !is_builtin_id(id) {
            return Err(PromptError::NotFound(id.to_string()));
        }
        if let Some(dir) = &self.dir {
            let path = dir.join(format!("{id}.json"));
            if path.exists() {
                fs::remove_file(path).map_err(io)?;
            }
        }
        Ok(())
    }

    /// A user copy of any preset, as a starting point.
    pub fn duplicate(&mut self, id: &str, name: &str) -> Result<Preset, PromptError> {
        let mut copy = self.get(id)?;
        copy.id = String::new();
        copy.name = name.to_string();
        self.save(copy)
    }

    /// A JSON file with the given presets, for sharing.
    pub fn export(&self, ids: &[String]) -> Result<String, PromptError> {
        let presets = ids
            .iter()
            .map(|id| self.get(id))
            .collect::<Result<Vec<_>, _>>()?;
        serde_json::to_string_pretty(&PresetFile {
            format: FILE_FORMAT.into(),
            version: 1,
            presets,
        })
        .map_err(io)
    }

    /// Adds the presets in a shared file as new user presets, so they never overwrite the
    /// user's own.
    pub fn import(&mut self, json: &str) -> Result<Vec<Preset>, PromptError> {
        let file: PresetFile =
            serde_json::from_str(json).map_err(|e| PromptError::InvalidImport(e.to_string()))?;
        if file.format != FILE_FORMAT && file.format != LEGACY_FILE_FORMAT {
            return Err(PromptError::InvalidImport(file.format));
        }
        file.presets
            .into_iter()
            .map(|mut preset| {
                preset.id = String::new();
                self.save(preset)
            })
            .collect()
    }
}

impl Preset {
    /// Fills in variables and freezes the result for a job.
    pub fn snapshot(&self, ctx: &RenderContext) -> PromptSnapshot {
        // The schema the model answers in: the fields themselves for an analysis, each page's
        // entries for a table of contents, the corrected text for a correction.
        let schema = match self.task {
            Task::Analysis => self.schema.clone(),
            Task::Contents => self.schema.as_ref().map(kalamu_core::contents::page_schema),
            Task::TextCorrection => Some(kalamu_core::correction::schema()),
            Task::TextExtraction | Task::AudioTranscription => None,
        };
        // The fields are described in words as well as sent as a schema: the schema fixes the
        // answer's shape, the words carry what belongs in each field.
        let instructions = match (self.task, &schema) {
            (Task::TextCorrection, _) => {
                format!("{}{CORRECTION_ANSWER}", self.instructions.trim_end())
            }
            (_, Some(schema)) => format!("{}{}", self.instructions.trim_end(), describe(schema)),
            (_, None) => self.instructions.clone(),
        };
        PromptSnapshot {
            preset_id: Some(self.id.clone()),
            preset_name: self.name.clone(),
            template: self.instructions.clone(),
            text: render(&instructions, ctx),
            user_prompt: render(&self.user_prompt, ctx),
            schema,
        }
    }

    /// A table of contents' second step, frozen for a job: the instructions that combine the
    /// pages, with the entry's fields. `None` for every other task. Lines are not its concern.
    pub fn merge_snapshot(&self, ctx: &RenderContext) -> Option<PromptSnapshot> {
        if self.task != Task::Contents {
            return None;
        }
        let template = self.merge_instructions.clone()?;
        let schema = kalamu_core::contents::merge_schema(self.schema.as_ref()?);
        let ctx = RenderContext {
            line_breaks: None,
            ..ctx.clone()
        };
        Some(PromptSnapshot {
            preset_id: Some(self.id.clone()),
            preset_name: self.name.clone(),
            text: render(
                &format!("{}{}", template.trim_end(), describe(&schema)),
                &ctx,
            ),
            template,
            user_prompt: MERGE_REQUEST.to_string(),
            schema: Some(schema),
        })
    }
}

/// Appended when the researcher chose how lines are kept. Stated as overriding the preset, whose
/// own instructions may say the opposite (the HTR presets join lines; the diplomatic one keeps
/// them), so one choice works with every preset, the user's own included.
const AS_ON_PAGE: &str = "\n\n## Line breaks\n\nKeep the lines of the source: start a new line \
wherever a line ends on the page, keep end-of-line hyphens exactly as they are, and do not join \
lines or rejoin words split across lines. Separate paragraphs with a blank line. This overrides \
any instruction above about joining lines or removing hyphens.";

const RUNNING_TEXT: &str = "\n\n## Line breaks\n\nWrite running text: join the lines of each \
paragraph, rejoin a word split across two lines and drop its line-break hyphen, but keep real \
compound hyphens (as in arc-en-ciel or peut-être). Separate paragraphs with a blank line. This \
overrides any instruction above about keeping line breaks or end-of-line hyphens.";

/// `{languages}` for answers left in the source's language. Said outright, because a model
/// reading English instructions drifts into English on some pages and not on others.
const ANSWER_LANGUAGE_AUTO: &str = "the language the source is written in (the one most of it \
is in, when it mixes several), even though these instructions are in English";

/// Replaces `{languages}` and `{speakers}`. When the user chose languages but the template does
/// not mention them, a short note is appended so the choice still reaches the model; so is the
/// choice of line breaks.
pub fn render(template: &str, ctx: &RenderContext) -> String {
    let languages = match (ctx.language_names.is_empty(), ctx.answers) {
        (true, true) => ANSWER_LANGUAGE_AUTO.to_string(),
        (true, false) => "the language(s) of the source, detected automatically".to_string(),
        (false, _) => ctx.language_names.join(", "),
    };
    let speakers = ctx
        .speakers
        .map(|n| n.to_string())
        .unwrap_or_else(|| "an unknown number of".into());
    let mut out = template
        .replace("{languages}", &languages)
        .replace("{speakers}", &speakers);
    let explicit = matches!(ctx.languages, LanguageChoice::Explicit(_));
    if explicit && !ctx.language_names.is_empty() && !template.contains("{languages}") {
        out.push_str(&if ctx.answers {
            format!(
                "\n\n## Language\n\nWrite your answers in {languages}, whatever the language of \
                 the source; keep names as the source spells them."
            )
        } else {
            format!("\n\n## Language\n\nThe source is in {languages}.")
        });
    }
    match ctx.line_breaks {
        Some(LineBreaks::AsOnPage) => out.push_str(AS_ON_PAGE),
        Some(LineBreaks::RunningText) => out.push_str(RUNNING_TEXT),
        None => {}
    }
    out
}

/// An analysis's fields in words, appended to its instructions: each field's name in the answer,
/// what kind of answer it takes, and what belongs in it.
pub fn describe(schema: &AnalysisSchema) -> String {
    let mut out = String::from(
        "\n\n## What to find\n\nAnswer with exactly these fields, filled from the document:\n",
    );
    describe_fields(&schema.fields, "", &mut out);
    out.push_str(
        "\n\nWrite each name or term in a list as the document spells it, and leave a list \
         empty when the document has nothing for it. Choose only from the values given, spelled \
         exactly as listed.",
    );
    out
}

/// One numbered line per field; an entries field's own fields indented under it.
fn describe_fields(fields: &[kalamu_core::Field], indent: &str, out: &mut String) {
    for (i, field) in fields.iter().enumerate() {
        let kind = match &field.kind {
            FieldKind::Entries { .. } => {
                "a list of entries, each with the fields below".to_string()
            }
            FieldKind::Text => "text".to_string(),
            FieldKind::List { max: Some(max) } => {
                format!("a list of names or terms, at most {max}, most important first")
            }
            FieldKind::List { max: None } => {
                "a list of names or terms, most important first".to_string()
            }
            FieldKind::Choice { values } => format!("exactly one of: {}", values.join(", ")),
            FieldKind::Choices { values } => {
                format!("any of: {}; none if none applies", values.join(", "))
            }
        };
        out.push_str(&format!(
            "\n{indent}{}. {} (`{}`, {kind})",
            i + 1,
            field.label.trim(),
            field.key
        ));
        if !field.instructions.trim().is_empty() {
            out.push_str(&format!(": {}", field.instructions.trim()));
        }
        if field.reason {
            out.push_str(&format!(
                " Give the reason for your answer in one sentence in `{}`.",
                kalamu_core::analysis::reason_key(&field.key)
            ));
        }
        if let FieldKind::Entries { fields } = &field.kind {
            describe_fields(fields, &format!("{indent}   "), out);
        }
    }
}

fn builtin(d: &defaults::Default) -> Preset {
    Preset {
        id: d.id.to_string(),
        task: d.task,
        name: d.name.to_string(),
        description: d.description.to_string(),
        instructions: d.instructions.to_string(),
        user_prompt: d.user_prompt.to_string(),
        builtin: true,
        modified: false,
        schema: d.schema.map(|make| {
            let mut schema = make();
            schema.assign_keys();
            schema
        }),
        merge_instructions: d.merge.map(str::to_string),
    }
}

fn is_builtin_id(id: &str) -> bool {
    defaults::DEFAULTS.iter().any(|d| d.id == id)
}

fn is_user_id(id: &str) -> bool {
    id.strip_prefix("user-")
        .is_some_and(|rest| uuid::Uuid::parse_str(rest).is_ok())
}

fn new_user_id() -> String {
    format!("user-{}", uuid::Uuid::new_v4())
}

pub(crate) fn io(err: impl std::fmt::Display) -> PromptError {
    PromptError::Io(err.to_string())
}

pub(crate) fn write_atomic(path: &Path, value: &impl Serialize) -> Result<(), PromptError> {
    let json = serde_json::to_string_pretty(value).map_err(io)?;
    let tmp = path.with_extension("json.pending");
    fs::write(&tmp, json).map_err(io)?;
    fs::rename(&tmp, path).map_err(io)
}

#[cfg(test)]
mod tests;
