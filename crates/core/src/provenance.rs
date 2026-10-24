use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{LanguageChoice, LanguageCode, ProviderId, SetupOptions, Task, Timestamp, Usage};

/// Written as `<source>.provenance.json` next to every output: enough to tell how a result was
/// produced and to reproduce it.
///
/// Modelled on zmo-ai-pipelines' `provenance.json` (schema 2): source hash, full prompt text and
/// its hash, per-run usage. Scriptorium starts its own numbering at 1.
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
    pub prompt: ProvenancePrompt,
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
    /// Fixes applied to working copies before processing (`split_spread`, `rotate`…).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocessing: Vec<String>,
    pub usage: Usage,
    pub cost_usd: Option<f64>,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenancePrompt {
    pub preset_id: Option<String>,
    pub preset_name: String,
    /// The full text sent to the model, so results stay reproducible after a preset changes.
    pub text: String,
    pub sha256: String,
}
