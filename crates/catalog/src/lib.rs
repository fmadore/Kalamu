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
    /// suggestion) and the model's whole output limit.
    pub fn tuning(&self, task: Task, chosen: Option<&str>) -> kalamu_core::Tuning {
        kalamu_core::Tuning {
            reasoning: self.reasoning.as_ref().and_then(|r| r.level(task, chosen)),
            max_output_tokens: self.limits.max_output_tokens,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flags New job reads must match what the provider modules send: OpenAI's
    /// transcriptions take two-letter codes, and Whisper, Voxtral and the diarizing model one
    /// language.
    #[test]
    fn transcription_language_hints_match_the_providers() -> Result<(), CatalogError> {
        let catalog = Catalog::from_json(BUNDLED)?;
        for model in catalog
            .models
            .iter()
            .filter(|m| m.tasks.contains(&Task::AudioTranscription))
        {
            let expected = match (model.provider.as_str(), model.id.as_str()) {
                ("whisper", _) | ("openai", "gpt-4o-transcribe-diarize") => LanguageHints::One,
                ("mistral", _) => LanguageHints::OneWithoutTimestamps,
                _ => LanguageHints::Several,
            };
            assert_eq!(model.language_hints, expected, "{}", model.id);
            assert_eq!(
                model.two_letter_codes,
                model.provider.as_str() == "openai",
                "{}",
                model.id
            );
        }
        Ok(())
    }

    /// Translations are for the interface's other languages, and say something.
    #[test]
    fn translations_are_for_interface_languages() -> Result<(), CatalogError> {
        let catalog = Catalog::from_json(BUNDLED)?;
        let all = catalog
            .providers
            .iter()
            .flat_map(|p| &p.translations)
            .chain(catalog.models.iter().flat_map(|m| &m.translations));
        for (language, t) in all {
            assert!(["fr", "de"].contains(&language.as_str()), "{language}");
            assert!(t.name.is_some() || t.description.is_some());
            assert!(
                t.name
                    .iter()
                    .chain(&t.description)
                    .all(|s| !s.trim().is_empty())
            );
        }
        Ok(())
    }

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
            assert!(
                model
                    .provider_default_for
                    .iter()
                    .all(|t| model.tasks.contains(t)),
                "{} is a default for a task it cannot do",
                model.id
            );
            if let LanguageSupport::Any(word) = &model.languages {
                assert_eq!(word, "any");
            }
        }
        // Each provider has exactly one default for each task it offers.
        for provider in &catalog.providers {
            for task in [
                Task::TextExtraction,
                Task::AudioTranscription,
                Task::Analysis,
            ] {
                let models = catalog
                    .models
                    .iter()
                    .filter(|m| m.provider == provider.id && m.tasks.contains(&task));
                let defaults = models
                    .clone()
                    .filter(|m| m.provider_default_for.contains(&task))
                    .count();
                let expected = usize::from(models.count() > 0);
                assert_eq!(defaults, expected, "{} {task:?}", provider.id);
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

    #[test]
    fn cached_input_and_batches_are_billed_at_their_own_prices() {
        let catalog = Catalog::bundled();
        let mut model = catalog.models[0].clone();
        model.pricing = Pricing {
            input_per_mtok: Some(2.0),
            output_per_mtok: Some(10.0),
            cached_input_per_mtok: Some(0.2),
            cache_write_per_mtok: Some(2.5),
            batch_discount: Some(0.5),
            ..Pricing::default()
        };
        let usage = |batch| kalamu_core::Usage {
            input_tokens: 1_000_000,
            cached_input_tokens: 600_000,
            cache_write_tokens: 100_000,
            output_tokens: 100_000,
            batch,
            ..kalamu_core::Usage::default()
        };
        // 0.3 M plain at $2, 0.6 M cached at $0.2, 0.1 M written at $2.5, 0.1 M out at $10.
        let fast = catalog.actual_cost(&model, &usage(false));
        assert!(fast.is_some_and(|c| (c - 1.97).abs() < 1e-9), "{fast:?}");
        let economy = catalog.actual_cost(&model, &usage(true));
        assert!(
            economy.is_some_and(|c| (c - 0.985).abs() < 1e-9),
            "{economy:?}"
        );

        let estimate = catalog.estimate(&model, Task::Analysis, 1, 0.0, 0);
        let (Some(usd), Some(economy)) = (estimate.usd, estimate.economy_usd) else {
            panic!("{estimate:?}");
        };
        assert!((economy - usd / 2.0).abs() < 1e-12);
        model.pricing.batch_discount = None;
        assert_eq!(
            catalog
                .estimate(&model, Task::Analysis, 1, 0.0, 0)
                .economy_usd,
            None
        );
    }

    /// Gemini 3.5 Transcribe is estimated by the minute, as Google estimates it, but its answer
    /// reports tokens, not seconds: the cost comes from them, as the live run of 3 November 2026
    /// was billed (1,308 tokens read and 177 written for 52 seconds).
    #[test]
    fn a_transcriber_billed_by_tokens_is_estimated_by_the_minute() {
        let catalog = Catalog::bundled();
        let model = catalog
            .model(&ProviderId::new("gemini"), "gemini-3.5-transcribe")
            .cloned()
            .unwrap_or_else(|| panic!("no transcribe"));
        let estimate = catalog.estimate(&model, Task::AudioTranscription, 0, 2.0, 0);
        assert_eq!(estimate.usd, Some(0.01));
        let usage = kalamu_core::Usage {
            input_tokens: 1_308,
            output_tokens: 177,
            ..kalamu_core::Usage::default()
        };
        let cost = catalog.actual_cost(&model, &usage);
        assert!(cost.is_some_and(|c| (c - 0.00474).abs() < 1e-6), "{cost:?}");
    }

    #[test]
    fn contents_and_corrections_take_the_analysis_models_that_read_their_input() {
        let catalog = Catalog::bundled();
        let ids = |task| -> Vec<String> {
            catalog
                .models_for(task)
                .iter()
                .map(|m| m.id.clone())
                .collect()
        };
        let contents = ids(Task::Contents);
        assert!(contents.contains(&"gemini-3.8-flash".to_string()));
        assert!(
            !contents.iter().any(|m| m.starts_with("deepseek")),
            "reads text only"
        );
        assert!(
            !contents.iter().any(|m| m.starts_with("mistral-ocr")),
            "no analysis"
        );
        let corrections = ids(Task::TextCorrection);
        assert!(corrections.iter().any(|m| m.starts_with("deepseek")));
        assert!(!corrections.iter().any(|m| m.starts_with("voxtral")));
        assert!(
            catalog
                .merge_models()
                .iter()
                .any(|m| m.id.starts_with("deepseek"))
        );
        assert_eq!(
            catalog
                .recommended(Task::Contents, &[])
                .map(|m| m.id.as_str()),
            catalog
                .recommended(Task::Analysis, &[])
                .map(|m| m.id.as_str())
        );
        let flash = catalog
            .model(&ProviderId::new("gemini"), "gemini-3.8-flash")
            .cloned()
            .unwrap_or_else(|| panic!("no flash"));
        let merge = catalog.estimate_merge(&flash, 40, 0);
        assert!(merge.usd.is_some() && merge.economy_usd.is_none());
    }

    #[test]
    fn reasoning_suggests_the_lowest_for_reading_and_keeps_to_the_levels_a_model_has() {
        let catalog = Catalog::bundled();
        let model = |id: &str| {
            catalog
                .models
                .iter()
                .find(|m| m.id == id)
                .unwrap_or_else(|| panic!("{id}"))
        };
        for m in &catalog.models {
            if let Some(r) = &m.reasoning {
                assert!(!r.levels.is_empty(), "{}", m.id);
                if let Some(d) = &r.default {
                    assert!(r.levels.contains(d), "{} default", m.id);
                }
                let mut ranks = r.levels.iter().map(|l| rank(l));
                assert!(
                    ranks
                        .clone()
                        .zip(ranks.by_ref().skip(1))
                        .all(|(a, b)| a < b),
                    "{} levels out of order",
                    m.id
                );
            }
        }
        let opus = model("claude-opus-5-5");
        assert_eq!(opus.limits.max_output_tokens, Some(128_000));
        let level = |m: &ModelInfo, task, chosen| m.tuning(task, chosen).reasoning;
        // Reading a page: the lowest; an analysis: `low`; a choice the model has: kept.
        assert_eq!(
            level(opus, Task::TextExtraction, None).as_deref(),
            Some("low")
        );
        let terra = model("gpt-6.1-terra");
        assert_eq!(
            level(terra, Task::TextExtraction, None).as_deref(),
            Some("none")
        );
        assert_eq!(level(terra, Task::Analysis, None).as_deref(), Some("low"));
        assert_eq!(
            level(opus, Task::Analysis, Some("xhigh")).as_deref(),
            Some("xhigh")
        );
        // A level the model lacks: the nearest it has, the lower of two equally near.
        let medium = model("mistral-medium-3-5");
        assert_eq!(level(medium, Task::Analysis, None).as_deref(), Some("none"));
        assert_eq!(
            level(medium, Task::Analysis, Some("xhigh")).as_deref(),
            Some("high")
        );
        let large = model("mistral-large-4-0");
        assert_eq!(
            level(large, Task::Analysis, Some("none")).as_deref(),
            Some("low")
        );
        assert_eq!(
            level(large, Task::Analysis, Some("max")).as_deref(),
            Some("high")
        );
        // OCR and speech-to-text models do not reason.
        assert_eq!(
            level(model("mistral-ocr-4-1"), Task::TextExtraction, Some("high")),
            None
        );
        // Where Mistral documents no output limit, none is set.
        assert_eq!(medium.limits.max_output_tokens, None);
    }
}
