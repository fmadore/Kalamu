//! The requests that ask a model for merges: a field's rows, each with its spellings, the
//! documents naming it and the pairs kept apart, in one text or in overlapping groups of
//! consecutive rows when they are too many for one request. Each group is a source of an
//! analysis job the engine runs as any other.

use std::collections::{BTreeSet, HashMap};

use kalamu_core::{
    CollectionId, DictionaryError, DocumentId, JobId, ProjectId, ProposalGroup, ProposalRequest,
    Timestamp,
};

use crate::{Dictionary, DictionaryRow, folded};

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
}
