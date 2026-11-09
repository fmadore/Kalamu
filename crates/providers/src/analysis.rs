//! What every provider shares for analyses: the JSON schema built from the researcher's fields,
//! the document as the model reads it, and reading the answer back.
//!
//! The document goes first and the request last, as both Google's and Anthropic's long-context
//! guidance advise, and the answer is constrained to a schema with each provider's structured
//! output rather than asked for in prose. The answer is then checked against the fields again
//! (`AnalysisSchema::read_answer`): structured output fixes the shape, not every rule.

use kalamu_core::{
    AnalysisSchema, BatchContext, ProcessOutput, ProcessRequest, ProviderError, Task,
};

pub(crate) use kalamu_core::structured::{document_text, finish, schema, tagged_document};

/// The fields an analysis request carries; a request without them is a bug upstream.
pub(crate) fn fields<'a>(
    request: &ProcessRequest<'a>,
) -> Result<&'a AnalysisSchema, ProviderError> {
    request
        .analysis
        .ok_or_else(|| ProviderError::InvalidRequest("an analysis needs fields".into()))
}

/// An answer as its task reads it: an analysis's, a table of contents' or a correction's is
/// checked against its fields; any other is kept as written.
pub(crate) fn read(
    context: BatchContext<'_>,
    output: ProcessOutput,
) -> Result<ProcessOutput, ProviderError> {
    match (context.task, context.analysis) {
        (Task::Analysis | Task::Contents | Task::TextCorrection, Some(fields)) => {
            finish(output, fields)
        }
        (Task::Analysis | Task::Contents | Task::TextCorrection, None) => Err(
            ProviderError::InvalidRequest("an analysis needs fields".into()),
        ),
        _ => Ok(output),
    }
}
