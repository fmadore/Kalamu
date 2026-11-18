//! Mistral's embeddings endpoint, in OpenAI's shape: the texts as `input`, a shorter length
//! through `output_dimension` where a model offers one (Mistral Embed gives 1,024 numbers and
//! no other length). Its vectors come back of length 1; Kalamu checks it. The role is not sent:
//! Mistral Embed embeds a question and a passage alike.

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, EmbedLimits, EmbedRequest, Embedded, Embedder, ProviderError, ProviderId,
};

use super::Mistral;
use crate::embed::{chars_for, check, embeddings_endpoint};

/// Mistral Embed reads 8,000 tokens in one text. Mistral documents no limit for a whole call;
/// clients keep each call below 16,384 tokens, which two texts at the most tokens Kalamu lets a
/// text make stay below.
const TOKENS_PER_TEXT: usize = 8_000;
const TEXTS_PER_CALL: usize = 2;

#[async_trait]
impl Embedder for Mistral {
    fn id(&self) -> ProviderId {
        ProviderId::new("mistral")
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
        embeddings_endpoint(
            &self.client,
            &self.base_url,
            key,
            &request,
            "output_dimension",
        )
        .await
    }
}
