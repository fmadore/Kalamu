//! The model catalogue: which providers and models exist, what each can do, the languages it
//! offers and what it costs.
//!
//! It is data, not code. A copy ships inside the app (`catalog.json`); the app can download a
//! newer one from the repository without a release, and [`Catalog::newest`] keeps whichever is
//! more recent. This crate does no network or file I/O itself.

use scriptorium_core::{LanguageCode, ProviderId, Task};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// The catalogue shipped with this build.
pub const BUNDLED: &str = include_str!("../catalog.json");

/// Where the app looks for a newer catalogue.
pub const UPDATE_URL: &str =
    "https://raw.githubusercontent.com/fmadore/Scriptorium/main/crates/catalog/catalog.json";

/// Catalogue files this build understands.
const SCHEMA_VERSION: u32 = 1;

/// Measured in iwac-ai-pipelines: a PDF page at Gemini's high media resolution.
const INPUT_TOKENS_PER_PAGE: f64 = 1_100.0;
/// A dense printed page transcribed in full.
const OUTPUT_TOKENS_PER_PAGE: f64 = 900.0;
/// Measured for gemini-3.5-transcribe; used for token-priced audio models.
const INPUT_TOKENS_PER_AUDIO_MINUTE: f64 = 1_500.0;
/// About 150 spoken words a minute.
const OUTPUT_TOKENS_PER_AUDIO_MINUTE: f64 = 250.0;
/// An analysis's answer (a summary of a few paragraphs, lists of names), with some thinking.
const ANALYSIS_OUTPUT_TOKENS: f64 = 1_500.0;

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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Pricing {
    pub input_per_mtok: Option<f64>,
    pub output_per_mtok: Option<f64>,
    pub per_minute: Option<f64>,
    pub per_page: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Limits {
    pub max_request_mb: Option<u32>,
    pub max_pages: Option<u32>,
    pub max_audio_minutes: Option<u32>,
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
    /// Dedicated OCR and speech-to-text models take no instructions.
    #[serde(default = "yes")]
    pub accepts_prompt: bool,
    /// Can label who speaks in a transcription. Whisper cannot.
    #[serde(default = "yes")]
    pub speakers: bool,
    /// Reports where text sits on the page, which a searchable PDF needs.
    #[serde(default)]
    pub layout: bool,
    /// A top-tier model priced far above what transcription and extraction usually need; the
    /// interface says so when it is chosen.
    #[serde(default)]
    pub costly: bool,
    #[serde(default)]
    pub pricing: Pricing,
    #[serde(default)]
    pub limits: Limits,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LanguageInfo {
    pub code: LanguageCode,
    /// English name, used in prompts.
    pub name: String,
}

/// What a job is expected to cost, before it starts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CostEstimate {
    /// `None` when the model has no published price in the catalogue.
    pub usd: Option<f64>,
    /// Why the estimate is rough, or why there is none.
    pub note: Option<String>,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("The model catalogue could not be read: {0}")]
    Invalid(String),
    #[error("This model catalogue needs a newer version of Scriptorium")]
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

    /// Models that can do a task, in catalogue order (recommended first within a provider).
    pub fn models_for(&self, task: Task) -> Vec<&ModelInfo> {
        self.models
            .iter()
            .filter(|m| m.tasks.contains(&task))
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
            .find(|m| m.recommended_for.contains(&task))
            .or_else(|| candidates.first())
            .copied()
    }

    /// The languages to offer in the dropdown for a model.
    pub fn languages_for(&self, model: &ModelInfo) -> Vec<LanguageInfo> {
        match &model.languages {
            LanguageSupport::Any(_) => self.languages.clone(),
            LanguageSupport::List(codes) => codes
                .iter()
                .map(|code| LanguageInfo {
                    code: code.clone(),
                    name: self.language_name(code),
                })
                .collect(),
        }
    }

    /// English name of a language, for prompts; the code itself when unknown.
    pub fn language_name(&self, code: &LanguageCode) -> String {
        self.languages
            .iter()
            .find(|l| &l.code == code)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| code.as_str().to_string())
    }

    /// A rough cost for `pages` pages or `audio_minutes` minutes on one model.
    pub fn estimate(
        &self,
        model: &ModelInfo,
        task: Task,
        pages: u32,
        audio_minutes: f64,
        prompt_chars: usize,
    ) -> CostEstimate {
        let p = &model.pricing;
        let usd = match task {
            Task::TextExtraction => {
                let pages = f64::from(pages);
                if let Some(per_page) = p.per_page {
                    Some(pages * per_page)
                } else if let (Some(input), Some(output)) = (p.input_per_mtok, p.output_per_mtok) {
                    // The instructions are sent with every page.
                    let prompt_tokens = prompt_chars as f64 / 4.0;
                    let tokens_in = pages * (INPUT_TOKENS_PER_PAGE + prompt_tokens);
                    let tokens_out = pages * OUTPUT_TOKENS_PER_PAGE;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                } else {
                    None
                }
            }
            // An analysis reads the whole document once and writes a short answer.
            Task::Analysis => match (p.input_per_mtok, p.output_per_mtok) {
                (Some(input), Some(output)) => {
                    let pages = f64::from(pages.max(1));
                    let prompt_tokens = prompt_chars as f64 / 4.0;
                    let tokens_in = pages * INPUT_TOKENS_PER_PAGE + prompt_tokens;
                    Some((tokens_in * input + ANALYSIS_OUTPUT_TOKENS * output) / 1_000_000.0)
                }
                _ => None,
            },
            Task::AudioTranscription => {
                if let Some(per_minute) = p.per_minute {
                    Some(audio_minutes * per_minute)
                } else if let (Some(input), Some(output)) = (p.input_per_mtok, p.output_per_mtok) {
                    let tokens_in = audio_minutes * INPUT_TOKENS_PER_AUDIO_MINUTE;
                    let tokens_out = audio_minutes * OUTPUT_TOKENS_PER_AUDIO_MINUTE;
                    Some((tokens_in * input + tokens_out * output) / 1_000_000.0)
                } else {
                    None
                }
            }
        };
        CostEstimate {
            usd,
            note: if usd.is_none() {
                Some("No published price in the catalogue for this model.".into())
            } else {
                p.note.clone()
            },
        }
    }

    /// Cost of a finished call from its reported usage.
    pub fn actual_cost(&self, model: &ModelInfo, usage: &scriptorium_core::Usage) -> Option<f64> {
        let p = &model.pricing;
        if let Some(per_minute) = p.per_minute
            && usage.audio_seconds > 0.0
        {
            return Some(usage.audio_seconds / 60.0 * per_minute);
        }
        if let Some(per_page) = p.per_page
            && usage.pages > 0
        {
            return Some(f64::from(usage.pages) * per_page);
        }
        let (input, output) = (p.input_per_mtok?, p.output_per_mtok?);
        Some(
            (usage.input_tokens as f64 * input + usage.output_tokens as f64 * output) / 1_000_000.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalogue_is_valid_and_consistent() -> Result<(), CatalogError> {
        let catalog = Catalog::from_json(BUNDLED)?;
        for model in &catalog.models {
            assert!(
                catalog.provider(&model.provider).is_some(),
                "{} names an unknown provider",
                model.id
            );
            assert!(!model.tasks.is_empty(), "{} has no task", model.id);
            assert!(!model.inputs.is_empty(), "{} reads nothing", model.id);
            assert!(
                model
                    .recommended_for
                    .iter()
                    .all(|t| model.tasks.contains(t)),
                "{} is recommended for a task it cannot do",
                model.id
            );
            if let LanguageSupport::Any(word) = &model.languages {
                assert_eq!(word, "any");
            }
        }
        for task in [
            Task::TextExtraction,
            Task::AudioTranscription,
            Task::Analysis,
        ] {
            assert!(catalog.recommended(task, &[]).is_some());
        }
        Ok(())
    }

    #[test]
    fn a_newer_download_replaces_the_bundled_catalogue() {
        let mut newer = Catalog::bundled();
        newer.updated = "2999-01-01".into();
        newer.models.truncate(1);
        let json = serde_json::to_string(&newer).unwrap_or_default();
        assert_eq!(Catalog::newest(Some(&json)).models.len(), 1);

        let mut older = Catalog::bundled();
        older.updated = "2000-01-01".into();
        let json = serde_json::to_string(&older).unwrap_or_default();
        assert_eq!(Catalog::newest(Some(&json)), Catalog::bundled());
        assert_eq!(Catalog::newest(Some("not json")), Catalog::bundled());
    }

    #[test]
    fn recommendation_prefers_providers_with_a_key() {
        let catalog = Catalog::bundled();
        let only_claude = [ProviderId::new("anthropic")];
        let model = catalog.recommended(Task::TextExtraction, &only_claude);
        assert_eq!(model.map(|m| m.provider.as_str()), Some("anthropic"));
    }

    #[test]
    fn estimates_scale_with_pages_and_minutes() {
        let catalog = Catalog::bundled();
        let flash = catalog.model(&ProviderId::new("gemini"), "gemini-3.8-flash");
        let flash = flash.cloned().unwrap_or_else(|| catalog.models[0].clone());
        let ten = catalog
            .estimate(&flash, Task::TextExtraction, 10, 0.0, 0)
            .usd;
        let twenty = catalog
            .estimate(&flash, Task::TextExtraction, 20, 0.0, 0)
            .usd;
        assert!(matches!((ten, twenty), (Some(a), Some(b)) if (b - 2.0 * a).abs() < 1e-9));

        let ocr = catalog
            .model(&ProviderId::new("mistral"), "mistral-ocr-4-1")
            .cloned();
        if let Some(ocr) = ocr {
            let usd = catalog.estimate(&ocr, Task::TextExtraction, 5, 0.0, 0).usd;
            assert!(usd.is_some_and(|u| (u - 0.02).abs() < 1e-9), "{usd:?}");
        }
    }
}
