//! The model catalogue: which providers and models exist, what each can do, the languages it
//! offers and what it costs.
//!
//! It is data, not code. A copy ships inside the app (`catalog.json`); the app can download a
//! newer one from the repository without a release, and [`Catalog::newest`] keeps whichever is
//! more recent. This crate does no network or file I/O itself.

use std::collections::BTreeMap;

use kalamu_core::{LanguageCode, ProviderId, Task};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

mod cost;
pub use cost::CostEstimate;

/// The catalogue shipped with this build.
pub const BUNDLED: &str = include_str!("../catalog.json");

/// Where the app looks for a newer catalogue.
pub const UPDATE_URL: &str =
    "https://raw.githubusercontent.com/fmadore/Kalamu/main/crates/catalog/catalog.json";

/// Catalogue files this build understands.
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Catalog {
    pub schema_version: u32,
    /// `YYYY-MM-DD`; the newer of two catalogues wins.
    pub updated: String,
    #[serde(default)]
    pub notes: String,
    pub providers: Vec<ProviderInfo>,
    pub models: Vec<ModelInfo>,
    /// Languages offered for models that accept any language, with English names for prompts.
    /// The interface shows names in its own language.
    pub languages: Vec<LanguageInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProviderInfo {
    pub id: ProviderId,
    pub name: String,
    /// Where users create an API key.
    pub key_url: String,
    /// A prefix every key starts with, to catch keys pasted for the wrong provider.
    pub key_prefix: Option<String>,
    /// Runs on this PC: needs no key and sends nothing anywhere.
    #[serde(default)]
    pub local: bool,
    /// The name in other interface languages, by language (`fr`); English is `name`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(optional, as = "Option<BTreeMap<String, Translation>>")]
    pub translations: BTreeMap<String, Translation>,
}

/// A provider's or model's words in another interface language; what is missing stays in
/// English. The catalogue is data refreshed from the repository, so its words cannot live in
/// the interface's translation files.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Translation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ModelStatus {
    Stable,
    Preview,
    /// Still selectable but being retired; the app warns before it stops working.
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum LanguageSupport {
    /// The literal string `"any"`.
    Any(String),
    List(Vec<LanguageCode>),
}

/// How many languages a model's API can be told: with more chosen than it takes, it detects the
/// language instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LanguageHints {
    /// Every language chosen, for recordings that mix them.
    #[default]
    Several,
    /// A single language (Whisper, OpenAI's diarizing model).
    One,
    /// A single language, and none when speakers or timestamps are asked for (Voxtral).
    OneWithoutTimestamps,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Pricing {
    pub input_per_mtok: Option<f64>,
    pub output_per_mtok: Option<f64>,
    pub per_minute: Option<f64>,
    pub per_page: Option<f64>,
    pub note: Option<String>,
    /// Input read from the provider's prompt cache; the normal input price when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cached_input_per_mtok: Option<f64>,
    /// Input written to the prompt cache, where the provider charges for it (Claude).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cache_write_per_mtok: Option<f64>,
    /// The share taken off every price through the provider's batch service (`0.5` for half
    /// price). Absent when the model has no batch service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub batch_discount: Option<f64>,
}

impl Pricing {
    /// What is left to pay after the batch discount, in economy mode.
    fn batch_factor(&self, batch: bool) -> f64 {
        match self.batch_discount {
            Some(discount) if batch => (1.0 - discount).clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Limits {
    pub max_request_mb: Option<u32>,
    pub max_pages: Option<u32>,
    pub max_audio_minutes: Option<u32>,
    /// The most the model writes in one answer, as its provider documents it; requests ask
    /// for all of it. `None` when the provider documents no limit: none is sent, and the
    /// model writes until it is done or its context is full.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_output_tokens: Option<u32>,
}

/// How much a model may think before it answers, in its provider's own words (`low`, `high`…).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Reasoning {
    /// The levels the model accepts, from least to most thinking.
    pub levels: Vec<String>,
    /// The provider's default, where it documents one; Kalamu always sends a level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default: Option<String>,
}

/// Every level providers use, from least to most thinking: a level asked of a model that lacks
/// it becomes the nearest one it has.
const REASONING_SCALE: &[&str] = &["none", "minimal", "low", "medium", "high", "xhigh", "max"];

fn rank(level: &str) -> usize {
    REASONING_SCALE
        .iter()
        .position(|l| *l == level)
        .unwrap_or(REASONING_SCALE.len() / 2)
}

impl Reasoning {
    /// The level suggested for a task. Reading a page, a recording or a text to correct needs
    /// little thought, so the lowest; an analysis or a table of contents starts at `low`, or the
    /// lowest where the model has no `low`. The researcher can raise either.
    pub fn suggested(&self, task: Task) -> Option<&str> {
        let lowest = self.levels.first().map(String::as_str);
        match task {
            Task::TextExtraction | Task::AudioTranscription | Task::TextCorrection => lowest,
            Task::Analysis | Task::Contents => self
                .levels
                .iter()
                .find(|l| *l == "low")
                .map(String::as_str)
                .or(lowest),
        }
    }

    /// The level to send: the one chosen when the model has it, else the nearest it has (the
    /// lower of two equally near), else the suggestion for the task.
    pub fn level(&self, task: Task, chosen: Option<&str>) -> Option<String> {
        let Some(chosen) = chosen else {
            return self.suggested(task).map(str::to_string);
        };
        if self.levels.iter().any(|l| l == chosen) {
            return Some(chosen.to_string());
        }
        let want = rank(chosen);
        self.levels
            .iter()
            .min_by_key(|l| (rank(l).abs_diff(want), rank(l)))
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelInfo {
    pub provider: ProviderId,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub tasks: Vec<Task>,
    /// MIME types the model reads.
    pub inputs: Vec<String>,
    pub languages: LanguageSupport,
    pub status: ModelStatus,
    /// Tasks for which this is the app's default choice.
    #[serde(default)]
    pub recommended_for: Vec<Task>,
    /// Tasks for which this is chosen when the user switches to its provider: the model that
    /// suits most material, not the provider's first or most expensive.
    #[serde(default)]
    pub provider_default_for: Vec<Task>,
    /// Dedicated OCR and speech-to-text models take no instructions.
    #[serde(default = "yes")]
    pub accepts_prompt: bool,
    /// Can label who speaks in a transcription. Whisper cannot.
    #[serde(default = "yes")]
    pub speakers: bool,
    /// Uses the languages chosen for a job. Mistral OCR reads any language and takes none.
    #[serde(default = "yes")]
    pub takes_language: bool,
    /// Detects the language when none is chosen. Tesseract does not: New job asks for the
    /// document's languages instead of offering to detect them.
    #[serde(default = "yes")]
    pub detects_language: bool,
    /// How many of the chosen languages its API takes; the provider modules send them so.
    #[serde(default)]
    pub language_hints: LanguageHints,
    /// Its API is told only languages with a two-letter ISO 639-1 code (OpenAI's speech-to-text
    /// models); it detects the others.
    #[serde(default)]
    pub two_letter_codes: bool,
    /// Reports where text sits on the page, which a searchable PDF and ALTO need.
    #[serde(default)]
    pub layout: bool,
    /// Takes a list of names and words to spell as written (Whisper's initial prompt).
    #[serde(default)]
    pub vocabulary: bool,
    /// Can write the speech in English instead of the language spoken (Whisper, except Turbo,
    /// which was trained without translation).
    #[serde(default)]
    pub translates: bool,
    /// A top-tier model priced far above what transcription and extraction usually need; the
    /// interface says so when it is chosen.
    #[serde(default)]
    pub costly: bool,
    /// When its safety classifier refuses a request, the provider can answer it again with
    /// another of its models, on its own side (Claude's server-side fallback, which Opus and
    /// Sonnet have and Haiku has not). Absent, as in catalogues written before it, means no:
    /// this build never reads a catalogue older than the one it ships with ([`Catalog::newest`]).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(optional, as = "Option<bool>")]
    pub refusal_fallback: bool,
    #[serde(default)]
    pub pricing: Pricing,
    #[serde(default)]
    pub limits: Limits,
    /// The thinking levels it accepts; `None` for a model that does not reason (an OCR or a
    /// speech-to-text model).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reasoning: Option<Reasoning>,
    /// Its name and description in other interface languages, by language (`fr`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(optional, as = "Option<BTreeMap<String, Translation>>")]
    pub translations: BTreeMap<String, Translation>,
}

fn yes() -> bool {
    true
}

impl ModelInfo {
    /// Whether the model can do a task: it is listed for the task providers see, and reads the
    /// task's input.
    pub fn does(&self, task: Task) -> bool {
        self.tasks.contains(&task.request_task()) && task.reads(&self.inputs)
    }

    /// What a request for a task sends: the reasoning level (the set-up's choice, or the
    /// suggestion), the model's whole output limit, and the abilities that shape its request,
    /// so that its provider never tells models apart by name.
    pub fn tuning(&self, task: Task, chosen: Option<&str>) -> kalamu_core::Tuning {
        kalamu_core::Tuning {
            reasoning: self.reasoning.as_ref().and_then(|r| r.level(task, chosen)),
            max_output_tokens: self.limits.max_output_tokens,
            refusal_fallback: self.refusal_fallback,
            speakers: self.speakers,
            no_prompt: !self.accepts_prompt,
        }
    }

    /// Whether this is the app's default for a task; the analysis default serves the tasks made
    /// of analyses.
    pub fn is_recommended_for(&self, task: Task) -> bool {
        listed_for(&self.recommended_for, task)
    }

    /// Whether this is its provider's default for a task, read as `is_recommended_for` is.
    pub fn is_provider_default_for(&self, task: Task) -> bool {
        listed_for(&self.provider_default_for, task)
    }
}

/// Whether `task`, or the kind of call it makes, is among `tasks`.
fn listed_for(tasks: &[Task], task: Task) -> bool {
    tasks.contains(&task) || tasks.contains(&task.request_task())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LanguageInfo {
    pub code: LanguageCode,
    /// English name, used in prompts.
    pub name: String,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("The model catalogue could not be read: {0}")]
    Invalid(String),
    #[error("This model catalogue needs a newer version of Kalamu")]
    TooNew,
}

impl Catalog {
    pub fn bundled() -> Self {
        // The bundled file is checked by a test, so failing here means a broken build.
        Self::from_json(BUNDLED).unwrap_or_else(|_| Self::empty())
    }

    pub fn from_json(json: &str) -> Result<Self, CatalogError> {
        let catalog: Catalog =
            serde_json::from_str(json).map_err(|e| CatalogError::Invalid(e.to_string()))?;
        if catalog.schema_version > SCHEMA_VERSION {
            return Err(CatalogError::TooNew);
        }
        Ok(catalog)
    }

    fn empty() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            updated: String::new(),
            notes: String::new(),
            providers: Vec::new(),
            models: Vec::new(),
            languages: Vec::new(),
        }
    }

    /// The more recent of the bundled catalogue and a downloaded one, ignoring a download this
    /// build cannot read.
    pub fn newest(downloaded: Option<&str>) -> Self {
        let bundled = Self::bundled();
        match downloaded.and_then(|json| Self::from_json(json).ok()) {
            Some(other) if other.updated > bundled.updated => other,
            _ => bundled,
        }
    }

    pub fn provider(&self, id: &ProviderId) -> Option<&ProviderInfo> {
        self.providers.iter().find(|p| &p.id == id)
    }

    pub fn model(&self, provider: &ProviderId, id: &str) -> Option<&ModelInfo> {
        self.models
            .iter()
            .find(|m| &m.provider == provider && m.id == id)
    }

    /// Models that can do a task, in catalogue order (recommended first within a provider). A
    /// table of contents and a correction are made of analyses, so they take the analysis models
    /// that read their input: pages, or text.
    pub fn models_for(&self, task: Task) -> Vec<&ModelInfo> {
        self.models.iter().filter(|m| m.does(task)).collect()
    }

    /// Models that can combine a table of contents' pages: analysis models that read text.
    pub fn merge_models(&self) -> Vec<&ModelInfo> {
        self.models
            .iter()
            .filter(|m| m.tasks.contains(&Task::Analysis) && Task::merges(&m.inputs))
            .collect()
    }

    /// The app's default model for a task, preferring providers the user has a key for.
    pub fn recommended(&self, task: Task, available: &[ProviderId]) -> Option<&ModelInfo> {
        let candidates: Vec<&ModelInfo> = self
            .models_for(task)
            .into_iter()
            .filter(|m| available.is_empty() || available.contains(&m.provider))
            .collect();
        candidates
            .iter()
            .find(|m| m.is_recommended_for(task))
            .or_else(|| candidates.first())
            .copied()
    }

    /// English name of a language, for prompts; the code itself when unknown.
    pub fn language_name(&self, code: &LanguageCode) -> String {
        self.languages
            .iter()
            .find(|l| &l.code == code)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| code.as_str().to_string())
    }
}

#[cfg(test)]
mod tests;
