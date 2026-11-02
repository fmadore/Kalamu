//! Merges proposed by a model (#48). The rows of one field are sent together, or in large
//! groups when they are too many for one request, each a text the engine sends as any analysis, so caching, resuming, cost and provenance are those
//! of every call. The answers are read back here against the rows each group held: a reference
//! to no row is counted and dropped, rows the researcher kept apart are never offered together,
//! and nothing is applied until the researcher accepts it, through the same merge as by hand.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use kalamu_core::dictionary::proposal;
use kalamu_core::{
    CacheKey, CollectionId, DictionaryError, DocumentId, Entry, JobId, JobStatus, KeptApart,
    PieceRef, ProjectId, ProposalGroup, ProposalRequest, StoreError, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::inventory::holder;
use crate::{Dictionary, DictionaryRow, Item, folded};

/// What one request may hold, in tokens of input: two to three thousand rows. Every model
/// offered for analyses reads far more, which leaves room for the instructions and the answer.
pub const INPUT_TOKENS: usize = 100_000;
/// Documents named for each row, as context.
const TITLES: usize = 3;

/// The groups a field's rows are sent in, ready to be written as a job's sources.
pub struct Prepared {
    pub namespace: String,
    /// What the researcher calls the field.
    pub label: String,
    pub rows: u32,
    pub groups: Vec<PreparedGroup>,
}

pub struct PreparedGroup {
    pub file_name: String,
    pub text: String,
    /// Each row's forms, in the order numbered in the text.
    pub rows: Vec<Vec<String>>,
}

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

/// Roughly what a line costs in tokens: a token taken as three characters, which errs on the
/// long side for names in any script, and a few for its reference.
fn tokens(line: &str) -> usize {
    line.chars().count() / 3 + 4
}

/// Groups of consecutive rows, each within `budget` tokens: every row is in one at least, and
/// each group shares a tenth of its rows with the next, so neighbours across a boundary are
/// compared. One group when everything fits.
pub(crate) fn windows(costs: &[usize], budget: usize) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    loop {
        let (mut end, mut sum) = (start, 0);
        while end < costs.len() && (end == start || sum + costs[end] <= budget) {
            sum += costs[end];
            end += 1;
        }
        out.push(start..end);
        if end >= costs.len() {
            return out;
        }
        start = (end - (end - start) / 10).max(start + 1);
    }
}

/// A row's line after its reference: its label, its other spellings and the documents naming
/// it.
fn row_line(
    row: &DictionaryRow,
    titles: &dyn Fn(&DictionaryRow) -> (Vec<String>, usize),
) -> String {
    let mut line = row.label.clone();
    let others: Vec<&str> = row
        .forms
        .iter()
        .map(|f| f.form.as_str())
        .filter(|f| *f != row.label)
        .collect();
    if !others.is_empty() {
        line.push_str(&format!(" | also written: {}", others.join("; ")));
    }
    let (named, count) = titles(row);
    if count > 0 {
        let more = count.saturating_sub(named.len());
        let quoted: Vec<String> = named.iter().map(|t| format!("\"{t}\"")).collect();
        line.push_str(&format!(
            " | named in {count} document{}: {}{}",
            if count == 1 { "" } else { "s" },
            quoted.join("; "),
            if more > 0 {
                format!(" and {more} more")
            } else {
                String::new()
            }
        ));
    }
    line
}

/// A group's text: the field, each row numbered with its line, and the pairs kept apart.
fn group_text(field: &str, rows: &[&DictionaryRow], lines: &[String]) -> String {
    let mut text = format!("Field: {field}\n\nRows:\n");
    let number: HashMap<&str, usize> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| (r.key.as_str(), i + 1))
        .collect();
    for (i, line) in lines.iter().enumerate() {
        text.push_str(&format!("R{}. {line}\n", i + 1));
    }
    let mut apart = BTreeSet::new();
    for row in rows {
        let Some(&a) = number.get(row.key.as_str()) else {
            continue;
        };
        for other in &row.apart {
            if let Some(&b) = number.get(other.as_str()) {
                apart.insert((a.min(b), a.max(b)));
            }
        }
    }
    if !apart.is_empty() {
        let pairs: Vec<String> = apart
            .iter()
            .map(|(a, b)| format!("R{a} and R{b}"))
            .collect();
        text.push_str(&format!(
            "\nKept apart by the researcher: {}.\n",
            pairs.join("; ")
        ));
    }
    text
}

impl Dictionary {
    /// One field's rows, in the project or a collection, as a model compares them: all in one
    /// text when they fit in [`INPUT_TOKENS`], as they nearly always do; otherwise in groups in
    /// alphabetical order with capitals and accents set aside, so variants that sort together
    /// are compared, neighbouring groups sharing a tenth of their rows.
    pub fn prepare_proposals(
        &self,
        project: ProjectId,
        scope: Option<CollectionId>,
        namespace: &str,
    ) -> Result<Prepared, DictionaryError> {
        let view = self.view(project, scope)?;
        let label = view
            .namespaces
            .iter()
            .find(|n| n.key == namespace)
            .map(|n| n.label.clone())
            .ok_or(DictionaryError::NothingToCompare)?;
        let mut rows: Vec<&DictionaryRow> = view
            .rows
            .iter()
            .filter(|r| r.namespace == namespace && !r.unseen)
            .collect();
        if rows.len() < 2 {
            return Err(DictionaryError::NothingToCompare);
        }
        rows.sort_by_cached_key(|r| (folded(&r.label), r.label.clone()));

        let gathered = self.gather(project, scope)?;
        let mut naming: HashMap<&str, BTreeSet<DocumentId>> = HashMap::new();
        for reading in &gathered.readings {
            for form in reading.fields.get(namespace).into_iter().flatten() {
                naming.entry(form).or_default().insert(reading.document);
            }
        }
        let titles = |row: &DictionaryRow| {
            let documents: BTreeSet<DocumentId> = row
                .forms
                .iter()
                .flat_map(|f| naming.get(f.form.as_str()).into_iter().flatten().copied())
                .collect();
            let mut named: Vec<String> = documents
                .iter()
                .filter_map(|d| gathered.titles.get(d).cloned())
                .collect();
            named.sort();
            named.truncate(TITLES);
            (named, documents.len())
        };

        let lines: Vec<String> = rows.iter().map(|r| row_line(r, &titles)).collect();
        let costs: Vec<usize> = lines.iter().map(|l| tokens(l)).collect();
        let ranges = windows(&costs, INPUT_TOKENS);
        let count = ranges.len();
        let groups = ranges
            .into_iter()
            .enumerate()
            .map(|(i, range)| {
                let slice = &rows[range.clone()];
                PreparedGroup {
                    file_name: if count == 1 {
                        format!("{namespace}.txt")
                    } else {
                        format!("{namespace}-{}-of-{count}.txt", i + 1)
                    },
                    text: group_text(&label, slice, &lines[range]),
                    rows: slice
                        .iter()
                        .map(|r| r.forms.iter().map(|f| f.form.clone()).collect())
                        .collect(),
                }
            })
            .collect();
        Ok(Prepared {
            namespace: namespace.to_string(),
            label,
            rows: rows.len() as u32,
            groups,
        })
    }

    /// Keeps a request once its job is made, with the groups as sent (by the SHA-256 of their
    /// text, which is the job's source).
    pub fn record_request(
        &self,
        project: ProjectId,
        scope: Option<CollectionId>,
        prepared: &Prepared,
        job_id: JobId,
        sha256: &[String],
    ) -> Result<ProposalRequest, DictionaryError> {
        self.check_project(project)?;
        let request = ProposalRequest {
            id: ProposalRequest::new_id(),
            project_id: project,
            namespace: prepared.namespace.clone(),
            collection: scope,
            job_id,
            groups: prepared
                .groups
                .iter()
                .zip(sha256)
                .map(|(g, sha)| ProposalGroup {
                    sha256: sha.clone(),
                    rows: g.rows.clone(),
                })
                .collect(),
            rows: prepared.rows,
            created_at: Timestamp::now(),
        };
        self.store.put_request(&request)?;
        Ok(request)
    }

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
                    let corrected = corrections
                        .iter()
                        .find(|c| {
                            c.setup_id == setup.id
                                && c.source_id == source.id
                                && c.index == piece.index
                        })
                        .and_then(|c| c.fields.clone());
                    let answers = match (corrected, &piece.cache_key) {
                        (Some(answers), _) => answers,
                        (None, Some(key)) => {
                            match self.jobs.cached_result(&CacheKey(key.clone()))? {
                                Some(result) => result.output.fields,
                                None => continue,
                            }
                        }
                        (None, None) => continue,
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
