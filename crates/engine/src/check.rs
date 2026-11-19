//! A check an answer must pass before it is kept, given by whoever asks for it
//! ([`Engine::call`](crate::Engine)): what the request asks beyond the set-up's own (other
//! fields, a sentence added to the prompt) and how the answer is read against it.
//!
//! An answer the check refuses is refused like any broken answer: billed and charged to the
//! piece, asked again within `max_attempts` with the same request, and the piece fails with
//! the reason when no answer passes. An answer that passes is kept as the check gives it back,
//! which may be changed or completed from what the caller knows (a translation's lines given
//! back their times). Numbered lines ([`crate::aligned::Passage`]) are the first check.

use kalamu_core::{AnalysisSchema, ProcessOutput, ProviderError};

pub(crate) trait AnswerCheck: Send + Sync {
    /// The fields asked for instead of the set-up's own, if any.
    fn schema(&self) -> Option<AnalysisSchema> {
        None
    }

    /// Added to the request after the prompt's own, after a blank line, if anything.
    fn request(&self) -> Option<&str> {
        None
    }

    /// The answer as it is kept, or why it is refused: a sentence a researcher can read, which
    /// provenance and the piece's error show.
    fn check(&self, output: ProcessOutput) -> Result<ProcessOutput, String>;
}

/// An answer read by a check: kept as the check gives it back, or refused as a broken answer
/// with the check's reason, billed for what the answer used.
pub(crate) fn checked(
    check: &dyn AnswerCheck,
    output: ProcessOutput,
) -> Result<ProcessOutput, ProviderError> {
    let usage = output.usage.clone();
    check
        .check(output)
        .map_err(|reason| ProviderError::InvalidAnswer { reason, usage })
}

#[cfg(test)]
mod tests {
    use kalamu_core::Usage;

    use super::*;

    /// Keeps an answer that says "yes", marked; refuses any other.
    struct Yes;

    impl AnswerCheck for Yes {
        fn check(&self, mut output: ProcessOutput) -> Result<ProcessOutput, String> {
            if output.text == "yes" {
                output.text.push_str(" (checked)");
                Ok(output)
            } else {
                Err(format!("the answer was {:?}", output.text))
            }
        }
    }

    fn answer(text: &str) -> ProcessOutput {
        ProcessOutput {
            text: text.into(),
            usage: Usage {
                input_tokens: 10,
                output_tokens: 2,
                ..Usage::default()
            },
            ..ProcessOutput::default()
        }
    }

    #[test]
    fn a_check_keeps_an_answer_as_it_gives_it_back_or_refuses_it_billed() {
        assert_eq!(Yes.schema(), None);
        assert_eq!(Yes.request(), None);
        let kept = checked(&Yes, answer("yes")).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(kept.text, "yes (checked)");
        match checked(&Yes, answer("no")) {
            Err(ProviderError::InvalidAnswer { reason, usage }) => {
                assert_eq!(reason, "the answer was \"no\"");
                assert_eq!((usage.input_tokens, usage.output_tokens), (10, 2));
            }
            other => panic!("not refused: {other:?}"),
        }
        // A refusal is billed, so the engine charges it and asks again.
        let refused = checked(&Yes, answer("no")).err();
        assert!(refused.as_ref().is_some_and(|e| e.billed().is_some()));
        assert!(refused.is_some_and(|e| e.is_retryable()));
    }
}
