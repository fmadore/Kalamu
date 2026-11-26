//! Cloud providers. Each module implements [`Provider`] for one service; [`registry`] is the one
//! place that lists them. Adding a provider means adding a module and a line there.
//!
//! Those that also turn texts into vectors, for the search by meaning, implement [`Embedder`];
//! [`embedders`] lists them the same way.
//!
//! A server of the researcher's own (Ollama, or any OpenAI-style address) is [`own`]: one
//! provider per server added, listed by [`own::providers`] from the values the shell hands in.

mod analysis;
mod anthropic;
mod batch;
mod embed;
mod gemini;
mod http;
mod mistral;
mod openai;
pub mod own;

use std::collections::HashMap;
use std::sync::Arc;

use kalamu_core::{Embedder, Provider, ProviderId};

pub use anthropic::Anthropic;
pub use gemini::Gemini;
pub use http::client;
pub use mistral::Mistral;
pub use openai::OpenAi;

/// Where each provider is reached when not at its own service's address, as values handed to
/// [`registry_at`] and [`embedders_at`]. Empty (the default), every provider is reached where it
/// always is. The shell fills it only in its test build, to reach the stand-in provider; a
/// provider at an address of the researcher's own would be handed in the same way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Addresses(HashMap<ProviderId, String>);

impl Addresses {
    /// These addresses, with `provider` reached at `base_url`: the address its own service's
    /// paths are written after (`…/v1`, `…/v1beta`).
    pub fn with(mut self, provider: &str, base_url: impl Into<String>) -> Self {
        self.0.insert(ProviderId::new(provider), base_url.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The address given for `provider`, if one was.
    pub fn of(&self, provider: &str) -> Option<&str> {
        self.0.get(&ProviderId::new(provider)).map(String::as_str)
    }

    fn gemini(&self, client: reqwest::Client) -> Gemini {
        match self.of("gemini") {
            Some(url) => Gemini::with_base_url(client, url),
            None => Gemini::new(client),
        }
    }

    fn openai(&self, client: reqwest::Client) -> OpenAi {
        match self.of("openai") {
            Some(url) => OpenAi::with_base_url(client, url),
            None => OpenAi::new(client),
        }
    }

    fn anthropic(&self, client: reqwest::Client) -> Anthropic {
        match self.of("anthropic") {
            Some(url) => Anthropic::with_base_url(client, url),
            None => Anthropic::new(client),
        }
    }

    fn mistral(&self, client: reqwest::Client) -> Mistral {
        match self.of("mistral") {
            Some(url) => Mistral::with_base_url(client, url),
            None => Mistral::new(client),
        }
    }
}

/// All providers, sharing one HTTP client.
pub fn registry(client: reqwest::Client) -> Vec<Arc<dyn Provider>> {
    registry_at(client, &Addresses::default())
}

/// All providers, sharing one HTTP client, each reached at the address `addresses` gives it,
/// else at its own.
pub fn registry_at(client: reqwest::Client, addresses: &Addresses) -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(addresses.gemini(client.clone())),
        Arc::new(addresses.openai(client.clone())),
        Arc::new(addresses.anthropic(client.clone())),
        Arc::new(addresses.mistral(client)),
    ]
}

/// The cloud providers that embed texts, sharing one HTTP client. Anthropic offers no
/// embeddings of its own.
pub fn embedders(client: reqwest::Client) -> Vec<Arc<dyn Embedder>> {
    embedders_at(client, &Addresses::default())
}

/// The cloud providers that embed texts, each reached at the address `addresses` gives it,
/// else at its own.
pub fn embedders_at(client: reqwest::Client, addresses: &Addresses) -> Vec<Arc<dyn Embedder>> {
    vec![
        Arc::new(addresses.gemini(client.clone())),
        Arc::new(addresses.openai(client.clone())),
        Arc::new(addresses.mistral(client)),
    ]
}
