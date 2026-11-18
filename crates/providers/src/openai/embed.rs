//! OpenAI's embeddings endpoint: all the texts of a call as `input`, a shorter length through
//! `dimensions` (the `text-embedding-3` models). OpenAI's vectors come back of length 1; Kalamu
//! checks it, and brings a shortened one to length 1 all the same. The models embed a question
//! and a passage alike, so the role is not sent.

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, EmbedLimits, EmbedRequest, Embedded, Embedder, ProviderError, ProviderId,
};

use super::OpenAi;
use crate::embed::{chars_for, check, embeddings_endpoint};

/// Each text may hold 8,192 tokens, and one call 300,000 summed over its texts (and at most
/// 2,048 texts): 32 texts at the most tokens Kalamu lets a text make stay below it.
const TOKENS_PER_TEXT: usize = 8_192;
const TEXTS_PER_CALL: usize = 32;

#[async_trait]
impl Embedder for OpenAi {
    fn id(&self) -> ProviderId {
        ProviderId::new("openai")
    }

    fn limits(&self, _model: &str) -> EmbedLimits {
        EmbedLimits {
            texts: TEXTS_PER_CALL,
            chars: chars_for(TOKENS_PER_TEXT, 0),
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
        embeddings_endpoint(&self.client, &self.base_url, key, &request, "dimensions").await
    }
}
