use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ProviderId, SetupId};

/// The prompt a set-up uses, frozen at job creation so later edits to a preset never change
/// what a job did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PromptSnapshot {
    /// The preset it came from, if any (`text_extraction.printed`, or a user preset's id).
    pub preset_id: Option<String>,
    pub preset_name: String,
    /// The template as written, with `{variables}`.
    pub template: String,
    /// The text sent to the model, variables filled in.
    pub text: String,
}

/// Provider-independent options for one set-up.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupOptions {
    /// Sampling temperature, where the provider supports it.
    pub temperature: Option<f32>,
    /// Ask for speaker labels in transcriptions.
    #[serde(default)]
    pub diarize: bool,
    /// Ask for timestamps in transcriptions.
    #[serde(default)]
    pub timestamps: bool,
    /// Expected number of speakers, if known.
    pub speakers: Option<u32>,
}

/// A provider, model and prompt combination. A normal job has one; comparison mode has several.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Setup {
    pub id: SetupId,
    /// Shown in comparison columns and export file names (`gemini-flash`).
    pub label: String,
    pub provider: ProviderId,
    pub model: String,
    pub prompt: PromptSnapshot,
    #[serde(default)]
    pub options: SetupOptions,
}
