//! Cloud providers. Each module implements [`Provider`] for one service; [`registry`] is the one
//! place that lists them. Adding a provider means adding a module and a line there.
//!
//! Those that also turn texts into vectors, for the search by meaning, implement [`Embedder`];
//! [`embedders`] lists them the same way.

mod analysis;
mod anthropic;
mod batch;
mod embed;
mod gemini;
mod http;
mod mistral;
mod openai;

use std::sync::Arc;

use kalamu_core::{Embedder, Provider};

pub use anthropic::Anthropic;
pub use gemini::Gemini;
pub use http::client;
pub use mistral::Mistral;
pub use openai::OpenAi;

/// All providers, sharing one HTTP client.
pub fn registry(client: reqwest::Client) -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(Gemini::new(client.clone())),
        Arc::new(OpenAi::new(client.clone())),
        Arc::new(Anthropic::new(client.clone())),
        Arc::new(Mistral::new(client)),
    ]
}

/// The cloud providers that embed texts, sharing one HTTP client. Anthropic offers no
/// embeddings of its own.
pub fn embedders(client: reqwest::Client) -> Vec<Arc<dyn Embedder>> {
    vec![
        Arc::new(Gemini::new(client.clone())),
        Arc::new(OpenAi::new(client.clone())),
        Arc::new(Mistral::new(client)),
    ]
}
