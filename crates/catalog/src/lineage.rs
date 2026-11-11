//! A model's line over time: the ids the catalogue once gave a model (aliases), and the newer
//! model its provider names in place of one replaced or retired. Nothing that names an older
//! model loses it: the catalogue keeps its entry, hides it from New job's choices and points to
//! its successor.

use kalamu_core::ProviderId;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Catalog, ModelInfo};

/// An id the catalogue once used, and the model it names now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelAlias {
    pub provider: ProviderId,
    /// The id as it was written.
    pub id: String,
    /// The id of the same model in `models`.
    pub model: String,
}

impl Catalog {
    /// A model by its id, or by an id the catalogue once gave it.
    pub fn model(&self, provider: &ProviderId, id: &str) -> Option<&ModelInfo> {
        let find = |id: &str| {
            self.models
                .iter()
                .find(|m| &m.provider == provider && m.id == id)
        };
        find(id).or_else(|| {
            self.aliases
                .iter()
                .find(|a| &a.provider == provider && a.id == id)
                .and_then(|a| find(&a.model))
        })
    }

    /// The id to send for a model: its id now when `id` is one the catalogue once gave it, else
    /// `id` as it is (a model the catalogue does not list is left to its provider).
    pub fn canonical_id<'a>(&'a self, provider: &ProviderId, id: &'a str) -> &'a str {
        self.model(provider, id).map_or(id, |m| m.id.as_str())
    }

    /// The model offered in place of one that was replaced or retired: its successor's
    /// successor, and so on, to the first still offered. `None` for a model still offered, or
    /// one whose line ends without a successor.
    pub fn replacement(&self, model: &ModelInfo) -> Option<&ModelInfo> {
        if model.is_offered() {
            return None;
        }
        let mut current = model;
        // A chain longer than the catalogue has models is a loop; a test keeps it from shipping.
        for _ in 0..self.models.len() {
            let next = self.model(&current.provider, current.successor.as_deref()?)?;
            if next.is_offered() {
                return Some(next);
            }
            current = next;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use kalamu_core::{ProviderId, Task};

    use crate::{BUNDLED, Catalog, CatalogError};

    /// Every model id the bundled catalogue has listed since successors were introduced. A model
    /// is never taken out: an id here must keep resolving, through its entry or an alias, so that
    /// a pipeline, a job or a comparison that names it keeps its name, prices and settings. Add
    /// new ids; never remove one.
    const SHIPPED: &[(&str, &str)] = &[
        ("gemini", "gemini-3.8-flash"),
        ("gemini", "gemini-3.5-transcribe"),
        ("gemini", "gemini-4-carbon"),
        ("gemini", "gemini-4-argon"),
        ("gemini", "gemini-3.1-pro-preview"),
        ("gemini", "gemini-3.5-flash-lite"),
        ("openai", "gpt-6.1-sol"),
        ("openai", "gpt-6.1-terra"),
        ("openai", "gpt-6-astra"),
        ("openai", "gpt-6-luna"),
        ("openai", "gpt-transcribe"),
        ("openai", "gpt-4o-transcribe-diarize"),
        ("anthropic", "claude-fable-5-2"),
        ("anthropic", "claude-fable-5-1"),
        ("anthropic", "claude-opus-5-5"),
        ("anthropic", "claude-sonnet-5-5"),
        ("anthropic", "claude-haiku-5-5"),
        ("mistral", "mistral-ocr-4-1"),
        ("mistral", "voxtral-small-transcribe-3-0"),
        ("mistral", "voxtral-mini-transcribe-3-0"),
        ("mistral", "mistral-large-4-0"),
        ("mistral", "mistral-medium-3-5"),
        ("mistral", "mistral-small-2603"),
        ("mistral", "zai-glm-5-3"),
        ("mistral", "zai-glm-5-3-flash"),
        ("mistral", "deepseek-v4-1-flash-1017"),
        ("mistral", "deepseek-v4p1-flash-1017"),
        ("whisper", "whisper-large-v3-turbo"),
        ("whisper", "whisper-large-v3"),
        ("whisper", "whisper-small"),
        ("whisper", "whisper-base"),
        ("tesseract", "tesseract-5"),
        ("local-ocr", "lightonocr-3-1b"),
        ("local-ocr", "glm-ocr"),
        ("local-llm", "gemma-4-e4b"),
        ("local-llm", "gemma-4-e2b"),
    ];

    fn id(provider: &str) -> ProviderId {
        ProviderId::new(provider)
    }

    #[test]
    fn no_model_once_shipped_is_lost() -> Result<(), CatalogError> {
        let catalog = Catalog::from_json(BUNDLED)?;
        for (provider, model) in SHIPPED {
            assert!(
                catalog.model(&id(provider), model).is_some(),
                "{provider}/{model} no longer resolves: keep its entry and give it a successor"
            );
        }
        for model in &catalog.models {
            assert!(
                SHIPPED.contains(&(model.provider.as_str(), model.id.as_str())),
                "add {}/{} to SHIPPED",
                model.provider.as_str(),
                model.id
            );
        }
        Ok(())
    }

    #[test]
    fn successors_and_aliases_point_to_offered_models_of_the_same_provider()
    -> Result<(), CatalogError> {
        let catalog = Catalog::from_json(BUNDLED)?;
        for model in &catalog.models {
            if let Some(next) = &model.successor {
                assert!(
                    catalog.model(&model.provider, next).is_some(),
                    "{}'s successor {next} is unknown",
                    model.id
                );
                // Its line ends in a model still offered, which does all it was chosen for.
                let replacement = catalog.replacement(model);
                assert!(
                    replacement.is_some_and(|r| model.tasks.iter().all(|t| r.tasks.contains(t))),
                    "{}'s line loops, ends in no model offered, or loses a task",
                    model.id
                );
            }
            if !model.is_offered() {
                assert!(
                    model.recommended_for.is_empty() && model.provider_default_for.is_empty(),
                    "{} is replaced or retired but still a default",
                    model.id
                );
            }
        }
        for alias in &catalog.aliases {
            let listed = |id: &str| {
                catalog
                    .models
                    .iter()
                    .any(|m| m.provider == alias.provider && m.id == id)
            };
            assert!(
                !listed(&alias.id),
                "{} is both an alias and a model",
                alias.id
            );
            assert!(listed(&alias.model), "alias {} names no model", alias.id);
        }
        Ok(())
    }

    #[test]
    fn a_replaced_model_still_resolves_and_points_to_the_newest() {
        let catalog = Catalog::bundled();
        let fable = catalog.model(&id("anthropic"), "claude-fable-5-1");
        assert_eq!(fable.map(|m| m.name.as_str()), Some("Claude Fable 5.1"));
        let newest = fable.and_then(|m| catalog.replacement(m));
        assert_eq!(newest.map(|m| m.id.as_str()), Some("claude-fable-5-2"));
        // A model still offered has no replacement, and the defaults pass over those replaced.
        let opus = catalog.model(&id("anthropic"), "claude-opus-5-5");
        assert!(opus.is_some_and(|m| catalog.replacement(m).is_none()));
        let analysis = catalog.recommended(Task::Analysis, &[id("mistral")]);
        assert_eq!(analysis.map(|m| m.id.as_str()), Some("mistral-large-4-0"));
    }

    #[test]
    fn an_old_id_resolves_to_the_model_and_is_sent_as_it_is_now() {
        let catalog = Catalog::bundled();
        let old = "deepseek-v4p1-flash-1017";
        let model = catalog.model(&id("mistral"), old);
        assert_eq!(
            model.map(|m| m.id.as_str()),
            Some("deepseek-v4-1-flash-1017")
        );
        assert_eq!(
            catalog.canonical_id(&id("mistral"), old),
            "deepseek-v4-1-flash-1017"
        );
        // An id the catalogue never listed is left as it is, for its provider to judge.
        assert_eq!(
            catalog.canonical_id(&id("mistral"), "something-new"),
            "something-new"
        );
    }

    #[test]
    fn a_line_follows_successors_to_the_first_offered_and_survives_a_loop() {
        let mut catalog = Catalog::bundled();
        for model in catalog.models.iter_mut() {
            if model.id == "claude-fable-5-2" {
                model.retired = true;
                model.successor = Some("claude-opus-5-5".into());
            }
        }
        let fable = catalog.model(&id("anthropic"), "claude-fable-5-1").cloned();
        let end = fable.as_ref().and_then(|m| catalog.replacement(m));
        assert_eq!(end.map(|m| m.id.as_str()), Some("claude-opus-5-5"));

        for model in catalog.models.iter_mut() {
            if model.id == "claude-opus-5-5" {
                model.successor = Some("claude-fable-5-1".into());
            }
        }
        let fable = catalog.model(&id("anthropic"), "claude-fable-5-1").cloned();
        assert!(
            fable
                .as_ref()
                .and_then(|m| catalog.replacement(m))
                .is_none()
        );
    }

    /// A catalogue without successors or aliases (as older builds write them) still reads, and
    /// older builds read one that has them, since serde skips fields it does not know: all are
    /// optional, and no new status or schema version was needed.
    #[test]
    fn successors_and_aliases_are_optional() -> Result<(), CatalogError> {
        let mut value: serde_json::Value =
            serde_json::from_str(BUNDLED).map_err(|e| CatalogError::Invalid(e.to_string()))?;
        if let Some(object) = value.as_object_mut() {
            object.remove("aliases");
        }
        if let Some(models) = value["models"].as_array_mut() {
            for model in models.iter_mut().filter_map(|m| m.as_object_mut()) {
                model.remove("successor");
                model.remove("retired");
            }
        }
        let old = Catalog::from_json(&value.to_string())?;
        assert!(old.models.iter().all(|m| m.is_offered()));
        assert!(old.aliases.is_empty());
        Ok(())
    }
}
