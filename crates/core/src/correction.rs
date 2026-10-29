//! Correcting existing text: each page or passage is sent as text and comes back corrected, as
//! the one field of an answer, so every provider's structured output holds it whole and an empty
//! or broken answer is retried rather than stored.

use crate::{AnalysisSchema, Field, FieldAnswer, FieldKind};

/// The field that holds the corrected text.
pub const TEXT: &str = "text";

pub fn schema() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![Field {
            key: TEXT.to_string(),
            label: "Corrected text".to_string(),
            instructions: "The whole page or passage, corrected, and nothing else.".to_string(),
            kind: FieldKind::Text,
            reason: false,
        }],
    }
}

/// The corrected text in an answer.
pub fn text(answers: &[FieldAnswer]) -> String {
    crate::analysis::answer_text(answers, TEXT)
}
