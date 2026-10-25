//! Prompt presets: the defaults shipped with the app, the user's own presets and edited
//! defaults, variables, and import/export for sharing across a team.
//!
//! Defaults are never changed on disk. Editing one saves an override under the same id;
//! resetting deletes the override. User presets are one JSON file each in the presets folder.

mod defaults;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use scriptorium_core::{LanguageChoice, LineBreaks, PromptSnapshot, Task};
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
}

/// Values for a preset's variables.
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    pub languages: LanguageChoice,
    /// English names for the chosen languages, in order (`French`, `Mooré`).
    pub language_names: Vec<String>,
    pub speakers: Option<u32>,
    /// The researcher's choice for a text extraction's lines; it overrides the preset's own.
    pub line_breaks: Option<LineBreaks>,
}

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Preset not found: {0}")]
    NotFound(String),
    #[error("The preset needs a name and instructions")]
    Incomplete,
    #[error("Could not read or save presets: {0}")]
    Io(String),
    #[error("This is not a Scriptorium presets file: {0}")]
    InvalidImport(String),
}

/// The file format used to share presets.
#[derive(Debug, Serialize, Deserialize)]
struct PresetFile {
    format: String,
    version: u32,
    presets: Vec<Preset>,
}

const FILE_FORMAT: &str = "scriptorium-prompts";

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
        if file.format != FILE_FORMAT {
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
        PromptSnapshot {
            preset_id: Some(self.id.clone()),
            preset_name: self.name.clone(),
            template: self.instructions.clone(),
            text: render(&self.instructions, ctx),
            user_prompt: render(&self.user_prompt, ctx),
        }
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

/// Replaces `{languages}` and `{speakers}`. When the user chose languages but the template does
/// not mention them, a short note is appended so the choice still reaches the model; so is the
/// choice of line breaks.
pub fn render(template: &str, ctx: &RenderContext) -> String {
    let languages = if ctx.language_names.is_empty() {
        "the language(s) of the source, detected automatically".to_string()
    } else {
        ctx.language_names.join(", ")
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
        out.push_str(&format!("\n\n## Language\n\nThe source is in {languages}."));
    }
    match ctx.line_breaks {
        Some(LineBreaks::AsOnPage) => out.push_str(AS_ON_PAGE),
        Some(LineBreaks::RunningText) => out.push_str(RUNNING_TEXT),
        None => {}
    }
    out
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

fn io(err: impl std::fmt::Display) -> PromptError {
    PromptError::Io(err.to_string())
}

fn write_atomic(path: &Path, preset: &Preset) -> Result<(), PromptError> {
    let json = serde_json::to_string_pretty(preset).map_err(io)?;
    let tmp = path.with_extension("json.pending");
    fs::write(&tmp, json).map_err(io)?;
    fs::rename(&tmp, path).map_err(io)
}

#[cfg(test)]
mod tests;
