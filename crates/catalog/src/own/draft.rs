//! What Settings (*Other servers*) sends and receives while a server is added or changed: the
//! server found at an address, a model's trial, and the draft saved. A draft is applied to the
//! list by [`OwnServers::apply`], which takes each model's abilities from its trial (one made in
//! this session, at this address, or the one stored for it), never from the draft: the
//! researcher cannot tick an ability.

use std::collections::BTreeMap;

use kalamu_core::address::{checked_address, host_of, on_this_computer};
use kalamu_core::{Dialect, ProviderId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::problems::ServerProblem;
use super::{OwnModel, OwnServer, OwnServers};

/// A server found at an address, as Settings shows it before anything is saved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FoundServer {
    pub dialect: Dialect,
    /// The address to keep, which the draft saved carries: as typed and checked, or with `/v1`
    /// where an OpenAI-style server answered only there.
    pub address: String,
    /// Ollama's version; none for an OpenAI-style server.
    pub version: Option<String>,
    /// The address's host, in lower case, as consent names it.
    pub host: String,
    /// The address is this computer: nothing leaves it, no consent is asked.
    pub this_computer: bool,
    pub models: Vec<FoundServerModel>,
}

/// A model of a server found, with what the server says of it (`None` where it does not say)
/// and the limits Kalamu fills.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FoundServerModel {
    /// The server's own name for the model (`gemma3:12b`).
    pub id: String,
    /// Reads pictures, as the server says; its trial decides in the end.
    pub vision: Option<bool>,
    pub audio: Option<bool>,
    /// The context the model was trained for, or the one the server gives it, in tokens.
    pub trained_context: Option<u32>,
    /// Where it runs when not on this server (a model Ollama runs at ollama.com): shown, and
    /// never ticked.
    pub elsewhere: Option<String>,
    #[ts(type = "number | null")]
    pub size_bytes: Option<u64>,
    /// The digest the server gives for the model, passed back with its trial.
    pub digest: Option<String>,
    /// *Reads up to*, as Kalamu fills it, or as the researcher set it for a model already added.
    pub context: u32,
    /// *Longest answer*, likewise.
    pub max_output: u32,
    /// Already one of the server's models (the server edited, or a saved one at this address).
    pub added: bool,
}

impl FoundServer {
    /// A server found at `address`, with where the address leads.
    pub fn new(
        dialect: Dialect,
        address: String,
        version: Option<String>,
        models: Vec<FoundServerModel>,
    ) -> Self {
        Self {
            dialect,
            host: host_of(&address),
            this_computer: on_this_computer(&address),
            address,
            version,
            models,
        }
    }
}

/// The server a model is tried on, as the draft stands: it need not be saved yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TrialServer {
    /// The server edited, if it is saved: its key is used when none is typed.
    #[serde(default)]
    pub id: Option<ProviderId>,
    pub name: String,
    /// [`FoundServer::address`].
    pub address: String,
    pub dialect: Dialect,
    /// *This server asks for a key*.
    #[serde(default)]
    pub asks_key: bool,
}

/// The model tried, as found and as the draft sets it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TrialModel {
    pub id: String,
    /// *Reads up to*, the context the model is run with during its trial.
    pub context: u32,
    /// [`FoundServerModel::vision`]: where the server says no, the picture is not sent.
    #[serde(default)]
    pub vision: Option<bool>,
    /// [`FoundServerModel::digest`], kept with the trial.
    #[serde(default)]
    pub digest: Option<String>,
}

/// What a model did in its trial, for Settings: *Reads pages* and *Fills an analysis's form*.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelTrial {
    pub reads_pages: bool,
    pub fills_forms: bool,
    /// When it did not answer at all: the failure's id (`provider.invalid_request`,
    /// `provider.transient`…, as for every provider)…
    pub failure: Option<String>,
    /// …and the server's own words, shortened.
    pub words: Option<String>,
    /// What went wrong in the picture or the form, in English, from the trial.
    pub note: Option<String>,
    /// The day of the trial, `YYYY-MM-DD`.
    pub tried_at: String,
}

/// A trial as the list keeps it until the draft is saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnTrial {
    pub reads_pages: bool,
    pub fills_forms: bool,
    pub digest: Option<String>,
    pub tried_at: String,
}

/// The trials made in this session, by the address tried and the model's name. Only these, and
/// the trials a saved server's models already hold, give a model its abilities.
#[derive(Debug, Clone, Default)]
pub struct Trials {
    made: BTreeMap<(String, String), OwnTrial>,
}

impl Trials {
    /// Keeps a trial of `model` at `address`, replacing an earlier one. An address
    /// `checked_address` refuses keeps nothing.
    pub fn record(&mut self, address: &str, model: &str, trial: OwnTrial) {
        if let Ok(address) = checked_address(address) {
            self.made.insert((address, model.trim().to_string()), trial);
        }
    }

    pub fn get(&self, address: &str, model: &str) -> Option<&OwnTrial> {
        let address = checked_address(address).ok()?;
        self.made.get(&(address, model.trim().to_string()))
    }
}

/// A server as Settings saves it: new when `id` is none, else the server of that id changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ServerDraft {
    #[serde(default)]
    pub id: Option<ProviderId>,
    pub name: String,
    /// [`FoundServer::address`].
    pub address: String,
    pub dialect: Dialect,
    /// *This server asks for a key*: off, any key saved for it is removed.
    #[serde(default)]
    pub asks_key: bool,
    /// *Requests at the same time*.
    pub at_once: u32,
    /// The models ticked, each tried.
    pub models: Vec<ModelDraft>,
}

/// A model as Settings saves it: what the researcher sets under *More choices*. What it does
/// comes from its trial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelDraft {
    pub id: String,
    pub context: u32,
    pub max_output: u32,
    #[serde(default)]
    pub input_per_mtok: Option<f64>,
    #[serde(default)]
    pub output_per_mtok: Option<f64>,
}

/// Why a draft was not saved, each with a stable id and the values it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftError {
    /// The server changed is no longer in the list.
    NotFound { id: String },
    /// A model without a trial at this address.
    NotTried { model: String },
    /// A model whose trial showed it neither reads pages nor fills a form: it is not listed.
    DidNothing { model: String },
    /// The list, with the draft in it, would have this problem.
    Problem(ServerProblem),
}

impl DraftError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "server.not_found",
            Self::NotTried { .. } => "server.not_tried",
            Self::DidNothing { .. } => "server.did_nothing",
            Self::Problem(problem) => problem.code(),
        }
    }

    pub fn params(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::NotFound { id } => vec![("id", id.clone())],
            Self::NotTried { model } | Self::DidNothing { model } => {
                vec![("model", model.clone())]
            }
            Self::Problem(problem) => problem.params(),
        }
    }
}

impl std::fmt::Display for DraftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { id } => write!(f, "the server {id} is no longer in the list"),
            Self::NotTried { model } => write!(f, "“{model}” has not been tried at this address"),
            Self::DidNothing { model } => write!(
                f,
                "“{model}” neither read the picture nor filled the form in its trial"
            ),
            Self::Problem(problem) => problem.fmt(f),
        }
    }
}

impl std::error::Error for DraftError {}

/// A draft applied: the server as saved, and as it stood before (none for a new server).
#[derive(Debug, Clone, PartialEq)]
pub struct Saved {
    pub server: OwnServer,
    pub previous: Option<OwnServer>,
}

impl Saved {
    /// The server existed at another address: consent to send to it is asked again.
    pub fn address_changed(&self) -> bool {
        self.previous
            .as_ref()
            .is_some_and(|p| p.address != self.server.address)
    }

    /// Whether saving would change what running work sends to, given the models of this server
    /// it uses (`in_use`, from the engine): another address or way of speaking, or a model in
    /// use no longer listed. A rename, new limits or prices, or a model added change nothing a
    /// job relies on. A new server is used by nothing.
    pub fn changes_under(&self, in_use: &[String]) -> bool {
        let Some(previous) = &self.previous else {
            return false;
        };
        !in_use.is_empty()
            && (previous.address != self.server.address
                || previous.dialect != self.server.dialect
                || in_use
                    .iter()
                    .any(|m| !self.server.models.iter().any(|o| &o.id == m)))
    }
}

impl OwnModel {
    /// A model as its trial left it on `today` (`YYYY-MM-DD`), with the limits it is run with
    /// and no price.
    pub fn tried(
        id: &str,
        reads_pages: bool,
        fills_forms: bool,
        context: u32,
        max_output: u32,
        digest: Option<String>,
        today: &str,
    ) -> Self {
        Self {
            id: id.trim().to_string(),
            reads_pages,
            fills_forms,
            context,
            max_output,
            input_per_mtok: None,
            output_per_mtok: None,
            tried_at: Some(today.to_string()),
            digest,
        }
    }

    fn trial(&self) -> Option<OwnTrial> {
        Some(OwnTrial {
            reads_pages: self.reads_pages,
            fills_forms: self.fills_forms,
            digest: self.digest.clone(),
            tried_at: self.tried_at.clone()?,
        })
    }
}

impl OwnServers {
    /// `found`, with each model already added marked and shown with the limits the researcher
    /// set: the models of the server `editing`, else of a saved server at the same address.
    pub fn found(&self, editing: Option<&ProviderId>, mut found: FoundServer) -> FoundServer {
        let server = match editing {
            Some(id) => self.server(id),
            None => self.servers.iter().find(|s| s.address == found.address),
        };
        let Some(server) = server else {
            return found;
        };
        for model in &mut found.models {
            if let Some(added) = server.models.iter().find(|m| m.id == model.id) {
                model.added = true;
                model.context = added.context;
                model.max_output = added.max_output;
            }
        }
        found
    }

    /// Saves `draft`: a new server under a new id, or the server of its id changed. Each model
    /// takes what it does from a trial at the draft's address, made in this session
    /// (`trials`) or, for a model the server already had at the same address and in the same
    /// way of speaking, the one stored; the key is saved as `asks_key` says, the shell keeping
    /// the keychain in step. Refused whole, the list unchanged, for the first problem found.
    pub fn apply(&mut self, draft: ServerDraft, trials: &Trials) -> Result<Saved, DraftError> {
        let label = || {
            let name = draft.name.trim();
            match (&draft.id, name.is_empty()) {
                (Some(id), true) => id.to_string(),
                _ => name.to_string(),
            }
        };
        let address = checked_address(&draft.address).map_err(|reason| {
            DraftError::Problem(ServerProblem::Address {
                server: label(),
                reason,
            })
        })?;
        let previous = match &draft.id {
            Some(id) => Some(
                self.server(id)
                    .cloned()
                    .ok_or_else(|| DraftError::NotFound { id: id.to_string() })?,
            ),
            None => None,
        };
        let same_place = previous
            .as_ref()
            .is_some_and(|p| p.address == address && p.dialect == draft.dialect);
        let mut models = Vec::new();
        for model in &draft.models {
            let id = model.id.trim();
            let stored = previous
                .as_ref()
                .filter(|_| same_place)
                .and_then(|p| p.models.iter().find(|m| m.id == id))
                .and_then(OwnModel::trial);
            let trial = trials.get(&address, id).cloned().or(stored);
            let Some(trial) = trial else {
                return Err(DraftError::NotTried {
                    model: id.to_string(),
                });
            };
            if !trial.reads_pages && !trial.fills_forms {
                return Err(DraftError::DidNothing {
                    model: id.to_string(),
                });
            }
            // A model ticked twice is left to the list's checks, which name it.
            models.push(OwnModel {
                input_per_mtok: model.input_per_mtok,
                output_per_mtok: model.output_per_mtok,
                ..OwnModel::tried(
                    id,
                    trial.reads_pages,
                    trial.fills_forms,
                    model.context,
                    model.max_output,
                    trial.digest,
                    &trial.tried_at,
                )
            });
        }
        let server = OwnServer {
            id: previous
                .as_ref()
                .map_or_else(|| self.new_id(), |p| p.id.clone()),
            name: draft.name.trim().to_string(),
            address,
            dialect: draft.dialect,
            key: draft.asks_key,
            at_once: draft.at_once,
            models,
        };
        let mut next = self.clone();
        match next.servers.iter_mut().find(|s| s.id == server.id) {
            Some(slot) => *slot = server.clone(),
            None => next.servers.push(server.clone()),
        }
        if let Some(problem) = next.problems().into_iter().next() {
            return Err(DraftError::Problem(problem));
        }
        *self = next;
        Ok(Saved { server, previous })
    }

    /// Removes the server of `id`, and gives it back; its jobs and results stay.
    pub fn remove(&mut self, id: &ProviderId) -> Option<OwnServer> {
        let at = self.servers.iter().position(|s| &s.id == id)?;
        Some(self.servers.remove(at))
    }
}

#[cfg(test)]
#[path = "draft_tests.rs"]
mod tests;
