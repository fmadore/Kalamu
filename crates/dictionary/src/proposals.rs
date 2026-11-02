//! Merges proposed by a model (#48). The rows of one field are sent together, or in large
//! groups when they are too many for one request, each a text the engine sends as any analysis, so caching, resuming, cost and provenance are those
//! of every call. The answers are read back here against the rows each group held: a reference
//! to no row is counted and dropped, rows the researcher kept apart are never offered together,
//! and nothing is applied until the researcher accepts it, through the same merge as by hand.

use std::collections::{BTreeMap, HashSet};

use kalamu_core::dictionary::proposal;
use kalamu_core::{
    DictionaryError, Entry, JobId, JobStatus, KeptApart, PieceRef, ProjectId, StoreError, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::inventory::holder;
use crate::{Dictionary, Item};

/// A request and how far its job has come.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestView {
    pub id: String,
    pub namespace: String,
    pub job_id: JobId,
    pub job_name: String,
    pub status: JobStatus,
    pub created_at: Timestamp,
    pub rows: u32,
    pub groups: u32,
    /// Groups the model has answered.
    pub answered: u32,
    /// Proposals still open from this request.
    pub proposals: u32,
    /// References in the answers to no row that was sent: dropped.
    pub unknown: u32,
}

/// A group of rows a model proposed as one, as the rows stand now.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Proposal {
    /// The forms it would join, sorted: the same proposal whichever request made it.
    pub key: String,
    pub namespace: String,
    pub request_id: String,
    pub job_id: JobId,
    pub label: String,
    pub reason: String,
    /// `certain`, `likely` or `possible`, in the model's words: not a probability.
    pub certainty: String,
    pub members: Vec<ProposalMember>,
    /// Turned down by the researcher; kept so it can be taken back.
    pub dismissed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProposalMember {
    pub item: Item,
    /// The row's key in the dictionary view.
    pub key: String,
    pub label: String,
    pub forms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProposalsView {
    pub requests: Vec<RequestView>,
    pub proposals: Vec<Proposal>,
}

/// A row as it stands now: an entry, or a form no entry holds.
fn current(entries: &[Entry], namespace: &str, form: &str) -> ProposalMember {
    match holder(entries, namespace, form) {
        Some(entry) => ProposalMember {
            item: Item::Entry { id: entry.id },
            key: entry.id.to_string(),
            label: entry.label.clone(),
            forms: entry.forms.clone(),
        },
        None => ProposalMember {
            item: Item::Form {
                form: form.to_string(),
            },
            key: format!("form:{form}"),
            label: form.to_string(),
            forms: vec![form.to_string()],
        },
    }
}

impl Dictionary {
    /// Every request whose job still exists, newest first, and the proposals their answers
    /// make of the rows as they stand now. A proposal whose rows are already one entry, or two
    /// of which the researcher kept apart, is not offered; the same proposal from two requests
    /// is offered once, from the newest.
    pub fn proposals(&self, project: ProjectId) -> Result<ProposalsView, DictionaryError> {
        self.check_project(project)?;
        let entries = self.store.entries(project)?;
        let apart = self.store.apart(project)?;
        let dismissed: HashSet<(String, String)> =
            self.store.dismissed(project)?.into_iter().collect();
        let mut requests = self.store.requests(project)?;
        requests.reverse();

        let mut views = Vec::new();
        let mut proposals: BTreeMap<String, Proposal> = BTreeMap::new();
        for request in &requests {
            let job = match self.jobs.job(request.job_id) {
                Ok(job) => job,
                Err(StoreError::NotFound) => continue,
                Err(e) => return Err(e.into()),
            };
            let pieces = self.jobs.pieces(job.id)?;
            let corrections = self.jobs.corrections(job.id)?;
            let mut view = RequestView {
                id: request.id.clone(),
                namespace: request.namespace.clone(),
                job_id: job.id,
                job_name: job.name.clone(),
                status: job.status,
                created_at: request.created_at,
                rows: request.rows,
                groups: request.groups.len() as u32,
                answered: 0,
                proposals: 0,
                unknown: 0,
            };
            for source in &job.sources {
                let Some(group) = request.groups.iter().find(|g| g.sha256 == source.sha256) else {
                    continue;
                };
                let mut answered = false;
                for setup in &job.setups {
                    let Some(piece) = pieces.iter().find(|p| {
                        p.setup_id == setup.id
                            && p.source_id == source.id
                            && p.piece == PieceRef::Whole
                            && p.status.is_finished()
                    }) else {
                        continue;
                    };
                    let Some(answers) = self.jobs.answers(piece, &corrections)? else {
                        continue;
                    };
                    answered = true;
                    for answer in proposal::groups(&answers) {
                        let mut members: BTreeMap<String, ProposalMember> = BTreeMap::new();
                        for reference in &answer.rows {
                            let Some(forms) =
                                proposal::row_number(reference).and_then(|n| group.rows.get(n - 1))
                            else {
                                view.unknown += 1;
                                continue;
                            };
                            for form in forms {
                                let member = current(&entries, &request.namespace, form);
                                members.entry(member.key.clone()).or_insert(member);
                            }
                        }
                        if members.len() < 2 {
                            continue;
                        }
                        let ids: Vec<_> = members
                            .values()
                            .filter_map(|m| match m.item {
                                Item::Entry { id } => Some(id),
                                Item::Form { .. } => None,
                            })
                            .collect();
                        let kept_apart = ids.iter().enumerate().any(|(i, a)| {
                            ids[i + 1..]
                                .iter()
                                .any(|b| apart.contains(&KeptApart::new(*a, *b)))
                        });
                        if kept_apart {
                            continue;
                        }
                        let mut forms: Vec<&str> = members
                            .values()
                            .flat_map(|m| m.forms.iter().map(String::as_str))
                            .collect();
                        forms.sort_unstable();
                        let key = forms.join("\u{1f}");
                        if proposals.contains_key(&key) {
                            continue;
                        }
                        let turned_down =
                            dismissed.contains(&(request.namespace.clone(), key.clone()));
                        if !turned_down {
                            view.proposals += 1;
                        }
                        proposals.insert(
                            key.clone(),
                            Proposal {
                                key,
                                namespace: request.namespace.clone(),
                                request_id: request.id.clone(),
                                job_id: job.id,
                                label: answer.label.clone(),
                                reason: answer.reason.clone(),
                                certainty: answer.certainty.clone(),
                                members: members.into_values().collect(),
                                dismissed: turned_down,
                            },
                        );
                    }
                }
                view.answered += u32::from(answered);
            }
            views.push(view);
        }
        let rank = |c: &str| {
            proposal::CERTAINTIES
                .iter()
                .position(|k| *k == c)
                .unwrap_or(proposal::CERTAINTIES.len())
        };
        let mut proposals: Vec<Proposal> = proposals.into_values().collect();
        proposals.sort_by(|a, b| {
            a.namespace
                .cmp(&b.namespace)
                .then_with(|| rank(&a.certainty).cmp(&rank(&b.certainty)))
                .then_with(|| a.label.to_lowercase().cmp(&b.label.to_lowercase()))
        });
        Ok(ProposalsView {
            requests: views,
            proposals,
        })
    }

    /// Turns a proposal down, or takes that back; it stays listed as turned down.
    pub fn dismiss(
        &self,
        project: ProjectId,
        namespace: &str,
        key: &str,
        dismissed: bool,
    ) -> Result<(), DictionaryError> {
        self.check_project(project)?;
        if dismissed {
            self.store.put_dismissed(project, namespace, key)?;
        } else {
            self.store.remove_dismissed(project, namespace, key)?;
        }
        Ok(())
    }

    /// Forgets a request, leaving its job (and what it cost) in Activity.
    pub fn forget_request(&self, project: ProjectId, id: &str) -> Result<(), DictionaryError> {
        self.check_project(project)?;
        if !self.store.requests(project)?.iter().any(|r| r.id == id) {
            return Err(DictionaryError::NotFound);
        }
        self.store.delete_request(id)?;
        Ok(())
    }
}
