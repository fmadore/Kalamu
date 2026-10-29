use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    LanguageChoice, LanguageCode, ProviderId, RequestSetting, SetupOptions, Task, Timestamp, Usage,
};

/// Written as `<source>.provenance.json` next to every output: enough to tell how a result was
/// produced and to reproduce it.
///
/// Modelled on zmo-ai-pipelines' `provenance.json` (schema 2): source hash, full prompt text and
/// its hash, per-run usage. Kalamu starts its own numbering at 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Provenance {
    /// Bumped when fields change meaning; follows zmo-ai-pipelines' `schema_version`.
    pub schema_version: u32,
    pub generator: GeneratorInfo,
    pub created_at: Timestamp,
    pub task: Task,
    pub source: ProvenanceSource,
    /// `cloud` or `offline`.
    pub engine: String,
    pub provider: ProviderId,
    pub model: String,
    /// The versions the provider reported, when they differ from `model`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_versions: Vec<String>,
    /// What the requests carried besides the prompt (reasoning effort, output limit, media
    /// resolution), as recorded with each piece's result. A name listed twice means pieces were
    /// read with different values; results stored before these were kept add nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request: Vec<RequestSetting>,
    pub prompt: ProvenancePrompt,
    /// For a table of contents, the model and prompt that combined the pages' readings; the
    /// fields above are those that read each page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge: Option<ProvenanceMerge>,
    pub languages: LanguageChoice,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detected_languages: Vec<LanguageCode>,
    pub options: SetupOptions,
    pub pieces: u32,
    /// Pages or segments the user corrected (numbered as in the export).
    #[serde(default)]
    pub corrected: Vec<u32>,
    /// Pages or segments the user marked as reviewed.
    #[serde(default)]
    pub reviewed: Vec<u32>,
    /// Pages or segments whose answer stopped at the model's output limit, so their end may be
    /// missing; for a table of contents, the pages read and the combined entries alike.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub truncated: Vec<u32>,
    /// Pages or segments that could not be read after their retries, marked in the files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<u32>,
    /// Fixes applied to working copies before processing (`split_spread`, `rotate`…).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocessing: Vec<String>,
    pub usage: Usage,
    pub cost_usd: Option<f64>,
}

/// A table of contents' second step, as provenance records it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceMerge {
    pub provider: ProviderId,
    pub model: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_versions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request: Vec<RequestSetting>,
    pub prompt: ProvenancePrompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeneratorInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceSource {
    pub file_name: String,
    pub sha256: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// A YouTube video's address: what was read, since there is no file to keep.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub url: Option<String>,
    /// The video's title, as YouTube gave it when the link was added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    /// When the provider first read the video at that address; it may change or be removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub read_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenancePrompt {
    pub preset_id: Option<String>,
    pub preset_name: String,
    /// The full text sent to the model, so results stay reproducible after a preset changes.
    pub text: String,
    pub user_prompt: String,
    /// SHA-256 of `text` followed by `user_prompt`.
    pub sha256: String,
    /// For an analysis, the fields asked for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::AnalysisSchema>,
}

impl ProvenancePrompt {
    pub fn from_snapshot(prompt: &crate::PromptSnapshot) -> Self {
        Self {
            preset_id: prompt.preset_id.clone(),
            preset_name: prompt.preset_name.clone(),
            text: prompt.text.clone(),
            user_prompt: prompt.user_prompt.clone(),
            sha256: crate::fingerprint::sha256_hex(
                format!("{}\n{}", prompt.text, prompt.user_prompt).as_bytes(),
            ),
            schema: prompt.schema.clone(),
        }
    }
}
