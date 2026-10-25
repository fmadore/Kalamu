//! Cache keys: a result is reused only when the source, the piece and everything that shapes
//! the model's answer are the same.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LanguageChoice, PIPELINE_VERSION, PieceRef, Setup, Task};

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

    pub fn as_str(&self) -> &str {
        &self.0
    }
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
}
