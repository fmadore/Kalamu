//! What can be wrong in the list of the researcher's servers, each with a stable id and the
//! values it names, so the interface words it from its translation files; the English, for the
//! logs, when `servers.json` itself is refused.

use std::collections::BTreeSet;
use std::fmt;

use kalamu_core::address::{AddressError, checked_address};

use super::{MOST_AT_ONCE, OwnServers, is_own_id};
use crate::{Catalog, checks};

/// One thing wrong in the list. `server` is how the server is named: its name, or its id when
/// it has none; `model` the model's name on the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerProblem {
    /// An id that is not `own-` and eight hexadecimal digits, or is a bundled provider's.
    BadId {
        server: String,
    },
    SameId {
        server: String,
        id: String,
    },
    NoName {
        server: String,
    },
    SameName {
        name: String,
    },
    /// An address `kalamu_core::address::checked_address` refuses, or one with spaces around
    /// it.
    Address {
        server: String,
        reason: AddressError,
    },
    AtOnce {
        server: String,
    },
    ModelNoName {
        server: String,
    },
    ModelTwice {
        server: String,
        model: String,
    },
    NoAnswer {
        server: String,
        model: String,
    },
    ContextTooSmall {
        server: String,
        model: String,
    },
    Price {
        server: String,
        model: String,
    },
    TrialDay {
        server: String,
        model: String,
    },
}

impl ServerProblem {
    /// The id the interface looks the sentence up by. An address refused has the address's
    /// own id, as when it is typed.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadId { .. } => "server.bad_id",
            Self::SameId { .. } => "server.same_id",
            Self::NoName { .. } => "server.no_name",
            Self::SameName { .. } => "server.same_name",
            Self::Address { reason, .. } => reason.code(),
            Self::AtOnce { .. } => "server.at_once",
            Self::ModelNoName { .. } => "server.model_no_name",
            Self::ModelTwice { .. } => "server.model_twice",
            Self::NoAnswer { .. } => "server.no_answer_room",
            Self::ContextTooSmall { .. } => "server.context_too_small",
            Self::Price { .. } => "server.price",
            Self::TrialDay { .. } => "server.trial_day",
        }
    }

    /// The values the sentence names, by the names the translation files give them.
    pub fn params(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::BadId { server }
            | Self::NoName { server }
            | Self::Address { server, .. }
            | Self::ModelNoName { server } => vec![("server", server.clone())],
            Self::SameId { server, id } => {
                vec![("server", server.clone()), ("id", id.clone())]
            }
            Self::SameName { name } => vec![("name", name.clone())],
            Self::AtOnce { server } => vec![
                ("server", server.clone()),
                ("most", MOST_AT_ONCE.to_string()),
            ],
            Self::ModelTwice { server, model }
            | Self::NoAnswer { server, model }
            | Self::ContextTooSmall { server, model }
            | Self::Price { server, model }
            | Self::TrialDay { server, model } => {
                vec![("server", server.clone()), ("model", model.clone())]
            }
        }
    }
}

impl fmt::Display for ServerProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `server` is the server's name, or its id when it has none.
        match self {
            Self::BadId { server } => {
                write!(
                    f,
                    "the server “{server}” has an id that is not own- and 8 hex digits"
                )
            }
            Self::SameId { server, id } => {
                write!(f, "the server “{server}” is listed twice under the id {id}")
            }
            Self::NoName { server } => write!(f, "the server “{server}” has no name"),
            Self::SameName { name } => write!(f, "two servers are called “{name}”"),
            Self::Address { server, reason } => write!(f, "the server “{server}”: {reason}"),
            Self::AtOnce { server } => write!(
                f,
                "the server “{server}”: requests at the same time must be from 1 to {MOST_AT_ONCE}"
            ),
            Self::ModelNoName { server } => {
                write!(f, "the server “{server}” has a model without a name")
            }
            Self::ModelTwice { server, model } => {
                write!(f, "the server “{server}”, model “{model}” is listed twice")
            }
            Self::NoAnswer { server, model } => {
                write!(
                    f,
                    "the server “{server}”, model “{model}”: the longest answer is 0 tokens"
                )
            }
            Self::ContextTooSmall { server, model } => write!(
                f,
                "the server “{server}”, model “{model}”: it must read more tokens than its longest answer"
            ),
            Self::Price { server, model } => write!(
                f,
                "the server “{server}”, model “{model}”: a price is negative or not a number"
            ),
            Self::TrialDay { server, model } => write!(
                f,
                "the server “{server}”, model “{model}”: the day of its trial is not YYYY-MM-DD"
            ),
        }
    }
}

/// Why `address` is not one a server may have: exactly what `checked_address` refuses, and
/// spaces around it, which it would have trimmed.
pub(super) fn address_problem(address: &str) -> Result<(), AddressError> {
    checked_address(address)?;
    if address.trim() != address {
        return Err(AddressError::NotAnAddress);
    }
    Ok(())
}

impl OwnServers {
    /// Every problem found in the list; empty when there is none. Ids of the right form,
    /// unique and never a bundled provider's; names given and unique; addresses as
    /// `kalamu_core::address::checked_address` takes them; 1 to [`MOST_AT_ONCE`] requests at
    /// once; each server's models named once, with an answer shorter than what they read,
    /// prices not negative, and a trial day written `YYYY-MM-DD`.
    pub fn problems(&self) -> Vec<ServerProblem> {
        let mut problems = Vec::new();
        let bundled: BTreeSet<String> = Catalog::read(crate::BUNDLED)
            .map(|c| c.providers.into_iter().map(|p| p.id.0).collect())
            .unwrap_or_default();
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for server in &self.servers {
            let id = server.id.as_str();
            let name = server.name.trim();
            let label = if name.is_empty() { id } else { name };
            let shown = || label.to_string();
            if !is_own_id(id) || bundled.contains(id) {
                problems.push(ServerProblem::BadId { server: shown() });
            }
            if !ids.insert(id) {
                problems.push(ServerProblem::SameId {
                    server: shown(),
                    id: id.to_string(),
                });
            }
            if name.is_empty() {
                problems.push(ServerProblem::NoName { server: shown() });
            } else if !names.insert(name.to_lowercase()) {
                problems.push(ServerProblem::SameName {
                    name: name.to_string(),
                });
            }
            if let Err(reason) = address_problem(&server.address) {
                problems.push(ServerProblem::Address {
                    server: shown(),
                    reason,
                });
            }
            if !(1..=MOST_AT_ONCE).contains(&server.at_once) {
                problems.push(ServerProblem::AtOnce { server: shown() });
            }
            let mut models = BTreeSet::new();
            for model in &server.models {
                let id = model.id.trim();
                let of = || (shown(), id.to_string());
                if id.is_empty() {
                    problems.push(ServerProblem::ModelNoName { server: shown() });
                } else if !models.insert(id) {
                    let (server, model) = of();
                    problems.push(ServerProblem::ModelTwice { server, model });
                }
                if model.max_output == 0 {
                    let (server, model) = of();
                    problems.push(ServerProblem::NoAnswer { server, model });
                }
                if model.context <= model.max_output {
                    let (server, model) = of();
                    problems.push(ServerProblem::ContextTooSmall { server, model });
                }
                if [model.input_per_mtok, model.output_per_mtok]
                    .iter()
                    .flatten()
                    .any(|p| !p.is_finite() || *p < 0.0)
                {
                    let (server, model) = of();
                    problems.push(ServerProblem::Price { server, model });
                }
                if model
                    .tried_at
                    .as_deref()
                    .is_some_and(|d| !checks::is_day(d))
                {
                    let (server, model) = of();
                    problems.push(ServerProblem::TrialDay { server, model });
                }
            }
        }
        problems
    }
}
