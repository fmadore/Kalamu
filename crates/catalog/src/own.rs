//! Servers of the researcher's own: the entries the researcher added in Settings (*Other
//! servers*), kept by the shell in `servers.json` beside the cached model list and never
//! refreshed, and [`Catalog::with_servers`], which adds them to a catalogue as providers and
//! models ([decision](../../../docs/decisions/2026-11-26-a-server-of-the-researchers-own.md)).
//!
//! A model is offered only for what it did in its trial: its entry is written from
//! [`OwnModel::reads_pages`] and [`OwnModel::fills_forms`], so New job, workflows and every
//! dialog obey it through the rules the bundled models already obey (`does`, `inputs`). The
//! researcher's entries are checked by [`OwnServers::problems`], never by the checks a
//! downloaded list goes through, and a downloaded list may never name a server of one's own.

use std::collections::BTreeSet;

use kalamu_core::{ProviderId, Task};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    Catalog, LanguageSupport, Limits, ModelInfo, ModelStatus, Pricing, ProviderInfo, Tier, checks,
};

/// What every server's provider id starts with: `own-` and eight hexadecimal digits.
pub const ID_PREFIX: &str = "own-";

/// The most requests a server is sent at the same time.
pub const MOST_AT_ONCE: u32 = 8;

/// The mime types a model that reads pages is sent: a page drawn as a picture, as the engine
/// and media make them for every model that reads images.
const PICTURES: [&str; 2] = ["image/png", "image/jpeg"];

/// How a server is spoken to: Ollama at its own address (`/api/chat`), or the OpenAI-style
/// Chat Completions most other servers speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Dialect {
    Ollama,
    OpenAi,
}

/// A server the researcher added: one provider, under an id made once and kept through a
/// rename or a changed address, so that results, the cache, consent and the key hang on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OwnServer {
    /// `own-` and eight hexadecimal digits ([`OwnServers::new_id`]).
    pub id: ProviderId,
    /// The researcher's name for it, shown wherever a provider's name is.
    pub name: String,
    /// `http://` or `https://`, as typed (`http://localhost:11434`).
    pub address: String,
    pub dialect: Dialect,
    /// A key is saved for it in the system's keychain.
    #[serde(default)]
    pub key: bool,
    /// Requests sent at the same time, from 1 to [`MOST_AT_ONCE`].
    #[serde(default = "one")]
    pub at_once: u32,
    /// The models ticked and tried, in the order they were added.
    #[serde(default)]
    pub models: Vec<OwnModel>,
}

fn one() -> u32 {
    1
}

/// A model of a server, as its trial left it and the researcher set it under *More choices*.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OwnModel {
    /// The server's own name for the model (`gemma3:12b`).
    pub id: String,
    /// Its trial's picture was read: offered for reading pages.
    #[serde(default)]
    pub reads_pages: bool,
    /// Its trial's form came back filled: offered for analyses, contents, corrections and
    /// translations.
    #[serde(default)]
    pub fills_forms: bool,
    /// *Reads up to*: the context it is run with, in tokens, the answer included.
    pub context: u32,
    /// *Longest answer*, in tokens.
    pub max_output: u32,
    /// *Price* per million tokens read and written; none, the default, reads as not priced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub input_per_mtok: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output_per_mtok: Option<f64>,
    /// The day of its last trial, `YYYY-MM-DD`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tried_at: Option<String>,
    /// The digest the server gave for the model at its trial (Ollama's), to tell a model
    /// updated under the same name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub digest: Option<String>,
}

/// What `servers.json` holds.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OwnServers {
    #[serde(default)]
    pub servers: Vec<OwnServer>,
}

/// What a catalogue says of a provider that is a server of the researcher's own, for the
/// interface: where it is and how it is reached. Never in a list the app ships or downloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OwnInfo {
    pub address: String,
    /// The address's host, in lower case (`localhost`, `192.168.1.20`).
    pub host: String,
    /// The address is this computer: nothing leaves it, no consent is asked.
    pub this_computer: bool,
    /// A key is saved for it.
    pub key: bool,
    pub dialect: Dialect,
}

/// Why the list of servers was not used. The whole file is refused; the shell decides what to
/// do with it.
#[derive(Debug, Error)]
pub enum OwnServersError {
    #[error("The list of your servers could not be read: {0}")]
    Invalid(String),
    #[error("The list of your servers was refused: {0}")]
    Refused(String),
}

/// Whether `id` has the form of a server's provider id: `own-` and eight lower-case
/// hexadecimal digits.
pub fn is_own_id(id: &str) -> bool {
    id.strip_prefix(ID_PREFIX).is_some_and(|hex| {
        hex.len() == 8
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

impl OwnServers {
    /// Reads `servers.json` and checks it ([`OwnServers::problems`]): refused whole when
    /// anything in it is wrong.
    pub fn from_json(json: &str) -> Result<Self, OwnServersError> {
        if json.len() > crate::MAX_BYTES {
            return Err(OwnServersError::Refused(format!(
                "the file is larger than {} bytes",
                crate::MAX_BYTES
            )));
        }
        let servers: Self =
            serde_json::from_str(json).map_err(|e| OwnServersError::Invalid(e.to_string()))?;
        let problems = servers.problems();
        if problems.is_empty() {
            Ok(servers)
        } else {
            Err(OwnServersError::Refused(problems.join("; ")))
        }
    }

    /// The file written back, indented for a person who opens it.
    pub fn to_json(&self) -> Result<String, OwnServersError> {
        serde_json::to_string_pretty(self).map_err(|e| OwnServersError::Invalid(e.to_string()))
    }

    pub fn server(&self, id: &ProviderId) -> Option<&OwnServer> {
        self.servers.iter().find(|s| &s.id == id)
    }

    /// A new server's id: `own-` and eight random hexadecimal digits, none of those taken.
    pub fn new_id(&self) -> ProviderId {
        loop {
            let id = ProviderId::new(format!("{ID_PREFIX}{}", kalamu_core::ids::random_hex8()));
            if self.server(&id).is_none() {
                return id;
            }
        }
    }

    /// Every problem found in the list, in words a researcher can read; empty when there is
    /// none. Ids of the right form, unique and never a bundled provider's; names given and
    /// unique; addresses `http://` or `https://` with a host and no user name; 1 to
    /// [`MOST_AT_ONCE`] requests at once; each server's models named once, with an answer
    /// shorter than what they read, prices not negative, and a trial day written
    /// `YYYY-MM-DD`.
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let bundled: BTreeSet<String> = Catalog::read(crate::BUNDLED)
            .map(|c| c.providers.into_iter().map(|p| p.id.0).collect())
            .unwrap_or_default();
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for server in &self.servers {
            let id = server.id.as_str();
            let name = server.name.trim();
            let at = if name.is_empty() {
                format!("the server {id}")
            } else {
                format!("the server “{name}”")
            };
            if !is_own_id(id) || bundled.contains(id) {
                problems.push(format!("{at} has an id that is not own- and 8 hex digits"));
            }
            if !ids.insert(id) {
                problems.push(format!("{at} is listed twice under the id {id}"));
            }
            if name.is_empty() {
                problems.push(format!("{at} has no name"));
            } else if !names.insert(name.to_lowercase()) {
                problems.push(format!("two servers are called “{name}”"));
            }
            if let Err(why) = address_problem(&server.address) {
                problems.push(format!("{at}: {why}"));
            }
            if !(1..=MOST_AT_ONCE).contains(&server.at_once) {
                problems.push(format!(
                    "{at}: requests at the same time must be from 1 to {MOST_AT_ONCE}"
                ));
            }
            let mut models = BTreeSet::new();
            for model in &server.models {
                let id = model.id.trim();
                let of = format!("{at}, model “{id}”");
                if id.is_empty() {
                    problems.push(format!("{at} has a model without a name"));
                } else if !models.insert(id) {
                    problems.push(format!("{of} is listed twice"));
                }
                if model.max_output == 0 {
                    problems.push(format!("{of}: the longest answer is 0 tokens"));
                }
                if model.context <= model.max_output {
                    problems.push(format!(
                        "{of}: it must read more tokens than its longest answer"
                    ));
                }
                for price in [model.input_per_mtok, model.output_per_mtok] {
                    if price.is_some_and(|p| !p.is_finite() || p < 0.0) {
                        problems.push(format!("{of}: a price is negative or not a number"));
                    }
                }
                if model
                    .tried_at
                    .as_deref()
                    .is_some_and(|d| !checks::is_day(d))
                {
                    problems.push(format!("{of}: the day of its trial is not YYYY-MM-DD"));
                }
            }
        }
        problems
    }
}

/// What is wrong with a server's address, if anything: only `http://` and `https://`, a host,
/// no user name, no space or control character.
fn address_problem(address: &str) -> Result<(), String> {
    if address
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err("the address holds a space or a character addresses cannot".into());
    }
    let lower = address.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("http://")
        .or_else(|| lower.strip_prefix("https://"))
    else {
        return Err("the address must start with http:// or https://".into());
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return Err("the address must not hold a user name".into());
    }
    if kalamu_core::address::host_of(address).is_empty() {
        return Err("the address names no server".into());
    }
    Ok(())
}

impl OwnServer {
    /// The server as a catalogue's provider: its name, no key page, and where it is.
    pub fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            key_url: String::new(),
            key_prefix: None,
            local: false,
            translations: Default::default(),
            own: Some(OwnInfo {
                address: self.address.clone(),
                host: kalamu_core::address::host_of(&self.address),
                this_computer: kalamu_core::address::on_this_computer(&self.address),
                key: self.key,
                dialect: self.dialect,
            }),
        }
    }
}

impl OwnModel {
    /// The model as a catalogue's model of `server`, offered for what it did in its trial:
    /// reading pages (a picture, one a call) and filling an analysis's form (text). `None`
    /// for a model that did neither, or whose limits leave it nothing to read.
    pub fn model_info(&self, server: &ProviderId) -> Option<ModelInfo> {
        let mut tasks = Vec::new();
        let mut inputs = Vec::new();
        if self.reads_pages {
            tasks.push(Task::TextExtraction);
        }
        if self.fills_forms {
            // Contents, corrections and translations are made of analyses, as for every model
            // (`Task::request_task`): a table of contents needs the pictures too.
            tasks.push(Task::Analysis);
            inputs.push("text/plain".to_string());
        }
        if self.reads_pages {
            inputs.extend(PICTURES.iter().map(|m| m.to_string()));
        }
        let reads = self.context.checked_sub(self.max_output).filter(|t| *t > 0);
        if tasks.is_empty() || self.max_output == 0 || reads.is_none() {
            return None;
        }
        Some(ModelInfo {
            provider: server.clone(),
            id: self.id.clone(),
            name: self.id.clone(),
            // The interface words a server's models itself, from the provider's `own`.
            description: String::new(),
            summary: None,
            tasks,
            inputs,
            languages: LanguageSupport::Any("any".into()),
            status: ModelStatus::Stable,
            tier: Tier::Balanced,
            recommended_for: Vec::new(),
            provider_default_for: Vec::new(),
            accepts_prompt: true,
            speakers: false,
            takes_language: true,
            detects_language: true,
            language_hints: Default::default(),
            two_letter_codes: false,
            layout: false,
            vocabulary: false,
            translates: false,
            costly: false,
            refusal_fallback: false,
            successor: None,
            retired: false,
            pricing: Pricing {
                input_per_mtok: self.input_per_mtok,
                output_per_mtok: self.output_per_mtok,
                ..Pricing::default()
            },
            limits: Limits {
                max_request_mb: None,
                max_pages: Some(1),
                max_audio_minutes: None,
                max_output_tokens: Some(self.max_output),
                max_input_tokens: reads,
            },
            reasoning: None,
            page_thinking: None,
            translations: Default::default(),
        })
    }
}

impl Catalog {
    /// This catalogue with the researcher's servers added: each server a provider, each model
    /// that did something in its trial a model. What a catalogue held of servers before is
    /// replaced, so merging again, or merging into a refreshed list, gives the same result.
    /// A server whose id is not of the `own-` form or is already a listed provider's is left
    /// out, and so is a second entry under an id or a model's name already added. The result
    /// keeps the catalogue's own invariants but is never checked as a downloaded list is
    /// (its providers have no key page), so it must never be stored as one.
    pub fn with_servers(&self, servers: &OwnServers) -> Catalog {
        let mut merged = self.clone();
        let earlier: BTreeSet<ProviderId> = merged
            .providers
            .iter()
            .filter(|p| p.own.is_some())
            .map(|p| p.id.clone())
            .collect();
        merged.providers.retain(|p| p.own.is_none());
        merged.models.retain(|m| !earlier.contains(&m.provider));
        for server in &servers.servers {
            if !is_own_id(server.id.as_str()) || merged.provider(&server.id).is_some() {
                continue;
            }
            merged.providers.push(server.provider_info());
            let mut added = BTreeSet::new();
            for model in &server.models {
                if !added.insert(model.id.as_str()) {
                    continue;
                }
                merged.models.extend(model.model_info(&server.id));
            }
        }
        merged
    }
}

#[cfg(test)]
mod tests;
