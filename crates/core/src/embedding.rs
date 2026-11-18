//! Embeddings: a text turned into a list of numbers by a model, so that texts that mean alike
//! lie close together. The project's search by meaning and its map of documents are made from
//! them.
//!
//! An [`Embedder`] is to embeddings what a [`Provider`](crate::Provider) is to a job's pieces:
//! one implementation per cloud provider in `providers`, one for the models run on this
//! computer in `offline`. The engine never matches on an embedder's name: what a model needs
//! (its size, its limits, its price) is a catalogue field. Nothing here does I/O.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ApiKey, ProviderError, ProviderId, StopSignal};

/// What a text is embedded as. Most models embed a question and the passages that answer it
/// differently; an embedder whose model does not tell them apart ignores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedRole {
    /// A stretch of a document, kept in the index.
    Passage,
    /// What the researcher looks for.
    Query,
}

/// One call: some texts, embedded with one model.
#[derive(Debug)]
pub struct EmbedRequest<'a> {
    pub model: &'a str,
    pub role: EmbedRole,
    /// At most [`EmbedLimits::texts`] texts, each of at most [`EmbedLimits::chars`]
    /// characters: the caller cuts longer texts before it asks.
    pub texts: &'a [String],
    /// The length of the vectors asked for, where the model offers several; none for its own.
    pub dimensions: Option<u32>,
    pub stop: &'a StopSignal,
}

/// What one call may hold, for one model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbedLimits {
    /// The most texts in one call.
    pub texts: usize,
    /// The most characters in one text, set low enough that the model's own limit, counted in
    /// tokens, is not passed in any script.
    pub chars: usize,
}

/// The answer to one call.
#[derive(Debug, Clone, PartialEq)]
pub struct Embedded {
    /// One vector for each text, in the order asked, all of one length, each of length 1
    /// (normalised), so that a dot product is a cosine.
    pub vectors: Vec<Vec<f32>>,
    /// The tokens billed, where the provider says; 0 for a model run on this computer.
    pub input_tokens: u64,
}

/// A provider of embeddings, cloud or local.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// The provider's id, as in the catalogue and the keychain.
    fn id(&self) -> ProviderId;

    /// Whether a call needs an API key; a model run on this computer needs none.
    fn needs_key(&self) -> bool {
        true
    }

    /// Whether `model` can run now: for a local model, that its file is downloaded and its
    /// engine shipped with this copy.
    fn check_ready(&self, model: &str) -> Result<(), ProviderError> {
        let _ = model;
        Ok(())
    }

    /// What one call may hold for `model`.
    fn limits(&self, model: &str) -> EmbedLimits;

    /// Embeds the texts. `key` is none only where [`needs_key`](Self::needs_key) is false.
    /// Long local work checks `request.stop` between texts and returns
    /// [`ProviderError::Interrupted`] once it is set.
    async fn embed(
        &self,
        key: Option<&ApiKey>,
        request: EmbedRequest<'_>,
    ) -> Result<Embedded, ProviderError>;
}

/// One embedding model, as the catalogue lists it: what the engine and the interface need to
/// know to offer it, cut texts for it and say what it costs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbeddingModel {
    /// The id sent to the provider, or a local model's id in the list of downloads.
    pub id: String,
    pub provider: ProviderId,
    /// The name shown.
    pub name: String,
    /// The lengths of vector it gives, the one used first. Vectors of two lengths, or of two
    /// models, are never compared.
    pub dimensions: Vec<u32>,
    /// The most characters sent as one text: [`EmbedLimits::chars`] for this model.
    pub max_chars: u32,
    /// US dollars for a million tokens; none for a model run on this computer.
    pub input_price: Option<f64>,
    /// Run on this computer: nothing is sent.
    pub local: bool,
    /// Whether its provider says it reads many languages, not English alone.
    pub multilingual: bool,
}
