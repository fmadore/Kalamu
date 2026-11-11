//! Correcting or translating existing text: each page or passage is sent as text and comes back
//! corrected or translated, as the one field of an answer, so every provider's structured output
//! holds it whole and an empty or broken answer is retried rather than stored.

use crate::{AnalysisSchema, Field, FieldAnswer, FieldKind};

/// The field that holds the corrected or translated text.
pub const TEXT: &str = "text";

/// A correction's answer: the page or passage corrected.
pub fn schema() -> AnalysisSchema {
    one_field(
        "Corrected text",
        "The whole page or passage, corrected, and nothing else.",
    )
}

/// A translation's answer: the page or passage translated, in the same one field, so a
/// translation is stored and read back as a correction is.
pub fn translation_schema() -> AnalysisSchema {
    one_field(
        "Translated text",
        "The whole page or passage, translated, and nothing else.",
    )
}

fn one_field(label: &str, instructions: &str) -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![Field {
            key: TEXT.to_string(),
            label: label.to_string(),
            instructions: instructions.to_string(),
            kind: FieldKind::Text,
            reason: false,
        }],
    }
}

/// The corrected or translated text in an answer.
pub fn text(answers: &[FieldAnswer]) -> String {
    crate::analysis::answer_text(answers, TEXT)
}
