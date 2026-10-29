use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AnalysisSchema, ProviderId, SetupId};

/// The prompt a set-up uses, frozen at job creation so later edits to a preset never change
/// what a job did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PromptSnapshot {
    /// The preset it came from, if any (`text_extraction.printed`, or a user preset's id).
    pub preset_id: Option<String>,
    pub preset_name: String,
    /// The instructions as written, with `{variables}`.
    pub template: String,
    /// The instructions sent to the model (system prompt), variables filled in.
    pub text: String,
    /// The short request sent with each page or segment.
    #[serde(default)]
    pub user_prompt: String,
    /// For an analysis, the fields its answer must have.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub schema: Option<AnalysisSchema>,
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
    /// For a table of contents, the second step: the model that combines the pages' readings,
    /// which may be another provider's (a large model reads the pages, a small one combines).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge: Option<MergeStep>,
}

/// A table of contents' second step: one call per document, over the first step's readings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MergeStep {
    pub provider: ProviderId,
    pub model: String,
    pub prompt: PromptSnapshot,
}

/// The model chosen for a table of contents' second step, before its prompt is frozen: in a
/// new job's request and in a saved pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MergeModel {
    pub provider: ProviderId,
    pub model: String,
}

/// One call's provider, model and prompt: a set-up's, or its merge step's.
#[derive(Debug, Clone, Copy)]
pub struct Step<'a> {
    pub provider: &'a ProviderId,
    pub model: &'a str,
    pub prompt: &'a PromptSnapshot,
}

impl Setup {
    /// The call each page or segment makes.
    pub fn step(&self) -> Step<'_> {
        Step {
            provider: &self.provider,
            model: &self.model,
            prompt: &self.prompt,
        }
    }

    /// The call that combines a document's pages, for a table of contents.
    pub fn merge_step(&self) -> Option<Step<'_>> {
        self.merge.as_ref().map(|m| Step {
            provider: &m.provider,
            model: &m.model,
            prompt: &m.prompt,
        })
    }
}
