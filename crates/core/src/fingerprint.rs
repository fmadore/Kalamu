//! Cache keys: a result is reused only when the source, the piece and everything that shapes
//! the model's answer are the same.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LanguageChoice, PIPELINE_VERSION, PieceRef, PromptSnapshot, Setup, Step, Task};

/// SHA-256 over the source content, the piece, the set-up and the job's languages.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CacheKey(pub String);

impl CacheKey {
    pub fn compute(
        task: Task,
        source_sha256: &str,
        piece: &PieceRef,
        setup: &Setup,
        languages: &LanguageChoice,
        preprocessing: &[String],
    ) -> Self {
        // Only what changes the model's answer goes in: not the set-up's id or label, nor the
        // preset's name.
        let mut material = serde_json::json!({
            "pipeline": PIPELINE_VERSION,
            "task": task.as_str(),
            "source": source_sha256,
            "piece": piece.descriptor(),
            "provider": setup.provider.as_str(),
            "model": setup.model,
            "prompt": setup.prompt.text,
            "user_prompt": setup.prompt.user_prompt,
            "options": setup.options,
            "languages": languages,
        });
        // Added only when there is something, so keys made before fixes existed stay valid.
        if !preprocessing.is_empty() {
            material["preprocessing"] = serde_json::json!(preprocessing);
        }
        // Only analyses have a schema; other keys are unchanged by its addition.
        if let Some(schema) = &setup.prompt.schema {
            material["schema"] = serde_json::json!(schema);
        }
        Self(sha256_hex(material.to_string().as_bytes()))
    }

    /// The key of a table of contents' second step: the pages' own keys stand for what was read,
    /// so a page read again, or another merge model or prompt, makes it new.
    pub fn merge(
        source_sha256: &str,
        page_keys: &[String],
        step: Step<'_>,
        languages: &LanguageChoice,
    ) -> Self {
        Self::combined(
            "contents_merge",
            source_sha256,
            page_keys,
            &[],
            step,
            languages,
        )
    }

    /// The key of a second step over a document's parts ([`CacheKey::merge`]'s, under another
    /// name): the parts' keys stand for what was read, and `missing` names the parts that could
    /// not be, so a part read again later makes the step new. Left out when empty, so a table
    /// of contents' keys stay as they were.
    pub fn combined(
        kind: &str,
        source_sha256: &str,
        part_keys: &[String],
        missing: &[String],
        step: Step<'_>,
        languages: &LanguageChoice,
    ) -> Self {
        let mut material = serde_json::json!({
            "pipeline": PIPELINE_VERSION,
            "task": kind,
            "source": source_sha256,
            "pages": part_keys,
            "provider": step.provider.as_str(),
            "model": step.model,
            "prompt": step.prompt.text,
            "user_prompt": step.prompt.user_prompt,
            "schema": step.prompt.schema,
            "languages": languages,
        });
        if !missing.is_empty() {
            material["missing"] = serde_json::json!(missing);
        }
        Self(sha256_hex(material.to_string().as_bytes()))
    }

    /// The key of a passage corrected from the review: the text sent (the passage and its
    /// context) stands for the source and the piece, so asking the same of the same model and
    /// prompt again reuses the answer.
    pub fn passage(setup: &Setup, languages: &LanguageChoice, input: &str) -> Self {
        let material = serde_json::json!({
            "pipeline": PIPELINE_VERSION,
            "task": "passage_correction",
            "text": sha256_hex(input.as_bytes()),
            "provider": setup.provider.as_str(),
            "model": setup.model,
            "prompt": setup.prompt.text,
            "user_prompt": setup.prompt.user_prompt,
            "options": setup.options,
            "languages": languages,
            "schema": setup.prompt.schema,
        });
        Self(sha256_hex(material.to_string().as_bytes()))
    }

    /// The key of a region read again from the review: the picture cut from the page (its
    /// SHA-256) stands for the source, the page and the box, so reading the same part of the same
    /// file with the same model and prompt again reuses the answer, whatever the request's id.
    pub fn region(setup: &Setup, languages: &LanguageChoice, picture_sha256: &str) -> Self {
        let material = serde_json::json!({
            "pipeline": PIPELINE_VERSION,
            "task": "region_reading",
            "picture": picture_sha256,
            "provider": setup.provider.as_str(),
            "model": setup.model,
            "prompt": setup.prompt.text,
            "user_prompt": setup.prompt.user_prompt,
            "options": setup.options,
            "languages": languages,
            "schema": setup.prompt.schema,
        });
        Self(sha256_hex(material.to_string().as_bytes()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PromptSnapshot {
    /// What a frozen prompt asked, whatever model read it and whatever the preset is called: the
    /// part of a [`CacheKey`] that is the prompt (its instructions, the answer languages filled
    /// in; the request sent with each document, the researcher's question and themes in it; and
    /// an analysis's fields, the choices they offer and what each asks). Two runs of a preset
    /// edited between them keep its id but not this. SHA-256, in hexadecimal.
    pub fn fingerprint(&self) -> String {
        let material = serde_json::json!({
            "prompt": self.text,
            "user_prompt": self.user_prompt,
            "schema": self.schema,
        });
        sha256_hex(material.to_string().as_bytes())
    }

    /// [`PromptSnapshot::fingerprint`]'s first twelve characters: enough to tell two versions
    /// of a prompt apart in a table, short enough to read.
    pub fn short_fingerprint(&self) -> String {
        short_fingerprint(&self.fingerprint())
    }
}

/// A prompt's fingerprint ([`PromptSnapshot::fingerprint`]) cut to the twelve characters a table
/// shows.
pub fn short_fingerprint(fingerprint: &str) -> String {
    fingerprint.chars().take(12).collect()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LanguageCode, PromptSnapshot, ProviderId, SetupId, SetupOptions};

    fn setup(prompt: &str) -> Setup {
        Setup {
            id: SetupId::new(),
            label: "a".into(),
            provider: ProviderId::new("gemini"),
            model: "gemini-flash".into(),
            prompt: PromptSnapshot {
                preset_id: None,
                preset_name: "Printed".into(),
                template: prompt.into(),
                text: prompt.into(),
                user_prompt: "Transcribe this page.".into(),
                schema: None,
            },
            options: SetupOptions::default(),
            merge: None,
            pages_cap: None,
            windows: None,
        }
    }

    #[test]
    fn same_inputs_give_same_key_whatever_the_setup_id_or_label() {
        let page = PieceRef::page(1);
        let mut other = setup("Transcribe.");
        other.label = "b".into();
        let a = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &page,
            &setup("Transcribe."),
            &LanguageChoice::Auto,
            &[],
        );
        let b = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &page,
            &other,
            &LanguageChoice::Auto,
            &[],
        );
        assert_eq!(a, b);
    }

    #[test]
    fn prompt_page_and_language_change_the_key() {
        let page = PieceRef::page(1);
        let base = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &page,
            &setup("A"),
            &LanguageChoice::Auto,
            &[],
        );
        let prompt = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &page,
            &setup("B"),
            &LanguageChoice::Auto,
            &[],
        );
        let other_page = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &PieceRef::page(2),
            &setup("A"),
            &LanguageChoice::Auto,
            &[],
        );
        let language = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &page,
            &setup("A"),
            &LanguageChoice::Explicit(vec![LanguageCode::new("fr")]),
            &[],
        );
        let evened = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &page,
            &setup("A"),
            &LanguageChoice::Auto,
            &["even_out_volume".to_string()],
        );
        assert_ne!(base, prompt);
        assert_ne!(base, other_page);
        assert_ne!(base, language);
        assert_ne!(base, evened, "a fix to the input changes the result");
    }

    /// Keys of results already stored never change: a newspaper page read by Claude Opus, as
    /// keyed before models' abilities and presets' density were recorded (#147). What a
    /// provider learns from the catalogue, and how many pages go together, are not in a key.
    #[test]
    fn keys_of_stored_results_stay_as_they_were() {
        let mut opus = setup("# Newspaper or magazine page");
        opus.provider = ProviderId::new("anthropic");
        opus.model = "claude-opus-5-5".into();
        opus.prompt.preset_id = Some("text_extraction.newspaper".into());
        opus.options.reasoning = Some("low".into());
        let key = CacheKey::compute(
            Task::TextExtraction,
            "abc",
            &PieceRef::page(3),
            &opus,
            &LanguageChoice::Explicit(vec![LanguageCode::new("fr")]),
            &[],
        );
        // Computed by this test on the commit before the change (9b3fd85).
        assert_eq!(
            key.as_str(),
            "ace2d5efb73011e7904c0454ecf2030c1af99251938a5233e75b1e95d7063010"
        );
    }
    /// A prompt's fingerprint follows what it asks, not what it is called: its instructions, its
    /// request and its fields change it; its preset's id and name do not.
    #[test]
    fn a_prompts_fingerprint_follows_what_it_asks() {
        let base = setup("A").prompt;
        let mut renamed = base.clone();
        renamed.preset_id = Some("analysis.screening".into());
        renamed.preset_name = "Another name".into();
        assert_eq!(base.fingerprint(), renamed.fingerprint());
        let mut asked = base.clone();
        asked.user_prompt.push_str(" And this.");
        let mut fields = base.clone();
        fields.schema = Some(crate::AnalysisSchema { fields: Vec::new() });
        assert_ne!(base.fingerprint(), setup("B").prompt.fingerprint());
        assert_ne!(base.fingerprint(), asked.fingerprint());
        assert_ne!(base.fingerprint(), fields.fingerprint());
        assert_eq!(base.short_fingerprint().len(), 12);
        assert!(base.fingerprint().starts_with(&base.short_fingerprint()));
    }

    #[test]
    fn a_region_read_again_is_keyed_by_its_picture_model_and_prompt() {
        let picture = sha256_hex(b"the pixels cut");
        let base = CacheKey::region(&setup("A"), &LanguageChoice::Auto, &picture);
        // The set-up's id or label, which the request's own id is no part of, change nothing.
        let mut relabelled = setup("A");
        relabelled.label = "b".into();
        assert_eq!(
            base,
            CacheKey::region(&relabelled, &LanguageChoice::Auto, &picture)
        );
        let other_picture = CacheKey::region(
            &setup("A"),
            &LanguageChoice::Auto,
            &sha256_hex(b"other pixels"),
        );
        let mut other = setup("A");
        other.model = "gemini-pro".into();
        let other_model = CacheKey::region(&other, &LanguageChoice::Auto, &picture);
        let other_prompt = CacheKey::region(&setup("B"), &LanguageChoice::Auto, &picture);
        assert_ne!(base, other_picture);
        assert_ne!(base, other_model);
        assert_ne!(base, other_prompt);
        // Never the key of a whole page read with the same set-up.
        let page = CacheKey::compute(
            Task::TextExtraction,
            &picture,
            &PieceRef::page(1),
            &setup("A"),
            &LanguageChoice::Auto,
            &[],
        );
        assert_ne!(base, page);
    }
}
