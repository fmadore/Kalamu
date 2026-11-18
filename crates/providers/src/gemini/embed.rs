//! Gemini's embeddings, through `batchEmbedContents`: several texts in one call, each its own
//! request, the vectors back in the order asked.
//!
//! Gemini Embedding 2 takes no task type: what a text is embedded as goes in front of the text
//! itself, as Google's guide words it (`task: search result | query: …` for what the researcher
//! looks for, `title: none | text: …` for a passage kept in the index). A shorter length is
//! asked with `outputDimensionality`, from 128 to 3,072; Google brings shortened vectors to
//! length 1, and Kalamu does so again whatever comes back.

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, EmbedLimits, EmbedRequest, EmbedRole, Embedded, Embedder, ProviderError, ProviderId,
};
use serde_json::{Value, json};

use super::Gemini;
use crate::embed::{chars_for, check, finish, vector};
use crate::http::send_json;

/// Google states no limit on the requests in one `batchEmbedContents` call; its batch services
/// have long refused more than a hundred.
const TEXTS_PER_CALL: usize = 100;
/// Gemini Embedding 2 reads up to 8,192 tokens in one text; the prefix takes a few of them.
const TOKENS_PER_TEXT: usize = 8_192;
const PREFIX_TOKENS: usize = 32;

/// The text as sent, its role in front of it.
fn prefixed(role: EmbedRole, text: &str) -> String {
    match role {
        EmbedRole::Query => format!("task: search result | query: {text}"),
        EmbedRole::Passage => format!("title: none | text: {text}"),
    }
}

fn body(request: &EmbedRequest<'_>) -> Value {
    let model = format!("models/{}", request.model);
    let requests: Vec<Value> = request
        .texts
        .iter()
        .map(|text| {
            let mut one = json!({
                "model": model,
                "content": { "parts": [{ "text": prefixed(request.role, text) }] },
            });
            if let Some(dimensions) = request.dimensions {
                one["outputDimensionality"] = json!(dimensions);
            }
            one
        })
        .collect();
    json!({ "requests": requests })
}

#[async_trait]
impl Embedder for Gemini {
    fn id(&self) -> ProviderId {
        ProviderId::new("gemini")
    }

    fn limits(&self, _model: &str) -> EmbedLimits {
        EmbedLimits {
            texts: TEXTS_PER_CALL,
            chars: chars_for(TOKENS_PER_TEXT, PREFIX_TOKENS),
        }
    }

    async fn embed(
        &self,
        key: Option<&ApiKey>,
        request: EmbedRequest<'_>,
    ) -> Result<Embedded, ProviderError> {
        if let Some(nothing) = check(&request, self.limits(request.model))? {
            return Ok(nothing);
        }
        let key = key.ok_or_else(|| ProviderError::Auth("no API key was given".into()))?;
        let url = format!(
            "{}/models/{}:batchEmbedContents",
            self.base_url, request.model
        );
        let response = send_json(
            self.client
                .post(url)
                .header("x-goog-api-key", key.expose())
                .json(&body(&request)),
        )
        .await?;
        let vectors = response
            .get("embeddings")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::Other("no vectors in the response".into()))?
            .iter()
            .enumerate()
            .map(|(i, e)| {
                vector(e.get("values")).ok_or_else(|| {
                    ProviderError::Other(format!("vector {} is not a list of numbers", i + 1))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        // The whole call's count where the answer gives one, else each text's.
        let tokens = response
            .pointer("/usageMetadata/promptTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| {
                response
                    .get("embeddings")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.pointer("/usageMetadata/promptTokenCount"))
                    .filter_map(Value::as_u64)
                    .sum()
            });
        finish(&request, vectors, tokens)
    }
}
