//! Whether a text is surely too long for a model to read in one request, measured against the
//! most the catalogue says it reads ([`crate::Limits::max_input_tokens`]).
//!
//! The measure is deliberately lenient: the instructions and the answer are not counted, and a
//! token is taken for four characters whatever the script: about what English or French runs
//! to, where most other scripts hold more tokens. It refuses only what cannot fit; the
//! provider's own refusal, or the offline engine's, stays behind it for the rest.

use crate::ModelInfo;

/// Characters taken for one token: generous, so that no text that fits is refused.
pub const CHARS_PER_TOKEN: u64 = 4;
/// Characters taken for one word, spaces and punctuation included, in what a researcher reads:
/// the offline language models' own figure, so that their sentence and this one agree.
pub const CHARS_PER_WORD: u64 = 6;

/// A text too long for a model, in words a researcher can act on: about how many it holds and
/// about how many the model reads at once, each rounded to two significant figures (the first
/// up, the second down, so the text never seems to fit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooLong {
    pub words: u32,
    pub most_words: u32,
}

impl ModelInfo {
    /// Whether `text` is surely too long for the model to read in one request: its characters
    /// over four exceed the most the model reads. A model the catalogue gives no limit is never
    /// refused.
    pub fn too_long(&self, text: &str) -> Option<TooLong> {
        let most = u64::from(self.limits.max_input_tokens?) * CHARS_PER_TOKEN;
        let chars = text.chars().count() as u64;
        (chars > most).then(|| TooLong {
            words: words(round(chars.div_ceil(CHARS_PER_WORD), true)),
            most_words: words(round(most / CHARS_PER_WORD, false)),
        })
    }
}

/// A count of words as the interface takes it; a text of four billion words is not one.
fn words(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// `n` to two significant figures, up or down.
fn round(n: u64, up: bool) -> u64 {
    let mut step = 1;
    while n / step >= 100 {
        step *= 10;
    }
    if up {
        n.div_ceil(step) * step
    } else {
        n / step * step
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use kalamu_core::Task;

    use super::*;
    use crate::{Catalog, ProviderId};

    fn model(limit: Option<u32>) -> ModelInfo {
        let mut model = Catalog::bundled()
            .models_for(Task::Analysis)
            .first()
            .map(|m| (*m).clone())
            .expect("an analysis model");
        model.limits.max_input_tokens = limit;
        model
    }

    #[test]
    fn a_text_is_too_long_only_past_four_characters_a_token() {
        let model = model(Some(16_500));
        // 66,000 characters fit exactly; one more does not.
        assert_eq!(model.too_long(&"é".repeat(66_000)), None);
        assert_eq!(
            model.too_long(&"é".repeat(66_001)),
            Some(TooLong {
                words: 12_000,
                most_words: 11_000
            })
        );
        // Characters are counted, not bytes.
        assert_eq!(model.too_long(&"a".repeat(66_000)), None);
        let long = model.too_long(&"a".repeat(600_000)).unwrap();
        assert_eq!((long.words, long.most_words), (100_000, 11_000));
    }

    #[test]
    fn a_model_without_a_limit_is_never_refused() {
        assert_eq!(model(None).too_long(&"a".repeat(10_000_000)), None);
        assert_eq!(model(Some(0)).too_long(""), None);
        assert!(model(Some(0)).too_long("a").is_some());
    }

    #[test]
    fn words_are_rounded_so_the_text_never_seems_to_fit() {
        assert_eq!(round(0, true), 0);
        assert_eq!(round(99, true), 99);
        assert_eq!(round(11_001, true), 12_000);
        assert_eq!(round(11_001, false), 11_000);
        assert_eq!(round(699_050, false), 690_000);
        assert_eq!(round(614_666, false), 610_000);
        // A text just past a limit that is a round number still reads as longer.
        let model = model(Some(15_000));
        let long = model.too_long(&"a".repeat(60_001)).unwrap();
        assert!(long.words > long.most_words, "{long:?}");
    }

    #[test]
    fn every_analysis_model_says_how_much_it_reads() {
        let catalog = Catalog::bundled();
        let missing: Vec<&str> = catalog
            .models
            .iter()
            .filter(|m| m.tasks.contains(&Task::Analysis) && m.limits.max_input_tokens.is_none())
            .map(|m| m.id.as_str())
            .collect();
        assert!(missing.is_empty(), "no max_input_tokens: {missing:?}");
        // The offline models read what the offline engine sends them.
        for id in ["gemma-4-e4b", "gemma-4-e2b"] {
            let gemma = catalog.model(&ProviderId::new("local-llm"), id).unwrap();
            assert_eq!(gemma.limits.max_input_tokens, Some(16_500));
        }
    }

    #[test]
    fn an_older_build_reads_a_catalogue_with_the_limit_and_this_one_reads_one_without() {
        let mut json: serde_json::Value = serde_json::from_str(crate::BUNDLED).unwrap();
        for model in json["models"].as_array_mut().unwrap() {
            if let Some(limits) = model["limits"].as_object_mut() {
                limits.remove("max_input_tokens");
            }
        }
        let without = Catalog::from_json(&json.to_string()).unwrap();
        assert!(
            without
                .models
                .iter()
                .all(|m| m.limits.max_input_tokens.is_none())
        );
        // An unknown field is ignored, as an older build ignores this one.
        json["models"][0]["limits"]["max_something_new"] = serde_json::json!(5);
        assert!(Catalog::from_json(&json.to_string()).is_ok());
    }
}
