//! The inventory: every form the project's analyses listed, field by field, with the documents
//! that name it and the entries the researcher made of them.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use kalamu_core::analysis::latest_labels;
use kalamu_core::{
    CollectionId, Correction, DictionaryError, DocumentId, Entry, EntryId, Field, FieldKind,
    FieldValue, JobId, KeptApart, PieceState, ProjectId, SetupId, SourceId, Task, TiedResults,
    Timestamp, dictionary::form_of,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Dictionary, folded, word_order_key};

/// A list field of the project's analyses: its names are an inventory of their own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Namespace {
    /// The field's key (`people`), shared by every analysis that asks for it.
    pub key: String,
    /// What the researcher calls it, from the most recent analysis that asks for it.
    pub label: String,
    pub rows: u32,
    /// Rows the researcher has not reviewed.
    pub to_review: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DictionaryView {
    pub namespaces: Vec<Namespace>,
    pub rows: Vec<DictionaryRow>,
    /// Documents in scope with at least one analysis read.
    pub documents_read: u32,
    /// Documents in scope.
    pub documents: u32,
}

/// An entry, or a form no entry holds, with what supports it in the scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DictionaryRow {
    /// Stable within a view: the entry's id, or `form:` and the form.
    pub key: String,
    pub entry: Option<EntryId>,
    pub namespace: String,
    pub label: String,
    pub forms: Vec<FormView>,
    /// Documents in scope where any of its forms was listed.
    pub documents: u32,
    /// Over those documents, the results that listed one of its forms, of the results that
    /// read that field: two models listing a name is agreement, not a second mention.
    pub listed: u32,
    pub read: u32,
    pub reviewed: bool,
    /// An entry none of whose forms is listed any longer: its evidence is gone.
    pub unseen: bool,
    /// Rows the researcher kept it apart from, by key.
    pub apart: Vec<String>,
    /// Rows whose forms read the same with capitals, accents, punctuation or word order set
    /// aside: possibly the same, for the researcher to decide.
    pub similar: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FormView {
    pub form: String,
    pub documents: u32,
}

/// One result that listed some of a row's forms, to open in the review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Evidence {
    pub document_id: DocumentId,
    pub document: String,
    pub job_id: JobId,
    pub job_name: String,
    pub source_id: SourceId,
    pub setup_label: String,
    pub created_at: Timestamp,
    /// The forms it listed, as written.
    pub forms: Vec<String>,
}

/// Results, each a set-up of a job.
type Results = BTreeSet<(JobId, SetupId)>;

/// What one result (a set-up of a job) read from one document, field by field.
pub(crate) struct Reading {
    pub document: DocumentId,
    pub job_id: JobId,
    pub job_name: String,
    pub source_id: SourceId,
    pub setup_id: SetupId,
    pub setup_label: String,
    pub created_at: Timestamp,
    /// The list fields it answered, each with its forms (possibly none).
    pub fields: BTreeMap<String, BTreeSet<String>>,
}

pub(crate) struct Gathered {
    /// Field keys with their labels, the latest analysis's label kept.
    pub namespaces: BTreeMap<String, (Timestamp, String)>,
    pub readings: Vec<Reading>,
    pub titles: HashMap<DocumentId, String>,
    pub documents: u32,
}

impl Dictionary {
    /// Every list answer of the analyses tied to the project's documents (or a collection's),
    /// through the researcher's corrections where there are some.
    pub(crate) fn gather(
        &self,
        project: ProjectId,
        scope: Option<CollectionId>,
    ) -> Result<Gathered, DictionaryError> {
        self.check_project(project)?;
        let members = self.projects.members(project)?;
        let documents: Vec<_> = self
            .projects
            .documents(project)?
            .into_iter()
            .filter(|d| scope.is_none_or(|c| members.contains(&(c, d.id))))
            .collect();
        let titles: HashMap<DocumentId, String> =
            documents.iter().map(|d| (d.id, d.title.clone())).collect();
        let tied = TiedResults::read(
            &self.projects.links(project)?,
            &[],
            |d| titles.contains_key(&d),
            |id| self.jobs.job(id),
        )?;
        let is_list = |f: &Field| matches!(f.kind, FieldKind::List { .. });
        let mut gathered = Gathered {
            namespaces: latest_labels(tied.jobs(), is_list),
            readings: Vec::new(),
            documents: documents.len() as u32,
            titles: HashMap::new(),
        };
        // Each analysis's pieces and corrections, read once however many documents it read.
        let mut read: HashMap<JobId, (Vec<PieceState>, Vec<Correction>)> = HashMap::new();
        for result in tied.all().iter().filter(|r| r.task == Task::Analysis) {
            let Some(job) = tied.job(result.job_id) else {
                continue;
            };
            let Some(setup) = job.setups.iter().find(|s| s.id == result.setup_id) else {
                continue;
            };
            let Some(schema) = &setup.prompt.schema else {
                continue;
            };
            let lists: Vec<&str> = schema
                .fields
                .iter()
                .filter(|f| is_list(f))
                .map(|f| f.key.as_str())
                .collect();
            if lists.is_empty() {
                continue;
            }
            let (pieces, corrections) = match read.entry(job.id) {
                std::collections::hash_map::Entry::Occupied(known) => known.into_mut(),
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert((self.jobs.pieces(job.id)?, self.jobs.corrections(job.id)?))
                }
            };
            let mut fields: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for piece in pieces.iter().filter(|p| {
                p.setup_id == setup.id && p.source_id == result.source_id && p.status.is_finished()
            }) {
                let Some(answers) = self.jobs.answers(piece, corrections)? else {
                    continue;
                };
                for answer in answers.iter().filter(|a| lists.contains(&a.key.as_str())) {
                    let forms = fields.entry(answer.key.clone()).or_default();
                    if let FieldValue::List(items) = &answer.value {
                        forms.extend(items.iter().filter_map(|i| form_of(i)));
                    }
                }
            }
            if !fields.is_empty() {
                gathered.readings.push(Reading {
                    document: result.document_id,
                    job_id: job.id,
                    job_name: job.name.clone(),
                    source_id: result.source_id,
                    setup_id: setup.id,
                    setup_label: setup.label.clone(),
                    created_at: job.created_at,
                    fields,
                });
            }
        }
        gathered.titles = titles;
        Ok(gathered)
    }

    /// The dictionary over the whole project, or a collection: every entry, and every form
    /// listed that no entry holds. Entries whose forms are no longer listed anywhere stay, marked
    /// unseen; in a collection, only what its documents name is shown.
    pub fn view(
        &self,
        project: ProjectId,
        scope: Option<CollectionId>,
    ) -> Result<DictionaryView, DictionaryError> {
        let gathered = self.gather(project, scope)?;
        let entries = self.store.entries(project)?;
        let apart = self.store.apart(project)?;

        // Per namespace and form: the documents naming it, and per document the results that
        // listed it; per namespace and document, the results that read the field.
        let mut named: BTreeMap<(String, String), BTreeMap<DocumentId, Results>> = BTreeMap::new();
        let mut read: HashMap<(String, DocumentId), Results> = HashMap::new();
        for r in &gathered.readings {
            for (key, forms) in &r.fields {
                read.entry((key.clone(), r.document))
                    .or_default()
                    .insert((r.job_id, r.setup_id));
                for form in forms {
                    named
                        .entry((key.clone(), form.clone()))
                        .or_default()
                        .entry(r.document)
                        .or_default()
                        .insert((r.job_id, r.setup_id));
                }
            }
        }

        let row_of = |namespace: &str, forms: &[String]| {
            let mut documents: BTreeMap<DocumentId, Results> = BTreeMap::new();
            let views = forms
                .iter()
                .map(|form| {
                    let by = named.get(&(namespace.to_string(), form.clone()));
                    for (doc, results) in by.into_iter().flatten() {
                        documents.entry(*doc).or_default().extend(results);
                    }
                    FormView {
                        form: form.clone(),
                        documents: by.map_or(0, |d| d.len() as u32),
                    }
                })
                .collect::<Vec<_>>();
            let listed = documents.values().map(|r| r.len() as u32).sum();
            let read_by = documents
                .keys()
                .map(|d| {
                    read.get(&(namespace.to_string(), *d))
                        .map_or(0, |r| r.len() as u32)
                })
                .sum();
            (views, documents.len() as u32, listed, read_by)
        };

        let mut rows = Vec::new();
        let mut held: BTreeSet<(String, String)> = BTreeSet::new();
        for entry in &entries {
            for form in &entry.forms {
                held.insert((entry.namespace.clone(), form.clone()));
            }
            let (forms, documents, listed, read_by) = row_of(&entry.namespace, &entry.forms);
            if scope.is_some() && documents == 0 {
                continue;
            }
            rows.push(DictionaryRow {
                key: entry.id.to_string(),
                entry: Some(entry.id),
                namespace: entry.namespace.clone(),
                label: entry.label.clone(),
                forms,
                documents,
                listed,
                read: read_by,
                reviewed: entry.reviewed,
                unseen: documents == 0,
                apart: kept_from(&apart, entry.id),
                similar: Vec::new(),
            });
        }
        for (namespace, form) in named.keys() {
            if held.contains(&(namespace.clone(), form.clone())) {
                continue;
            }
            let (forms, documents, listed, read_by) = row_of(namespace, std::slice::from_ref(form));
            rows.push(DictionaryRow {
                key: format!("form:{form}"),
                entry: None,
                namespace: namespace.clone(),
                label: form.clone(),
                forms,
                documents,
                listed,
                read: read_by,
                reviewed: false,
                unseen: false,
                apart: Vec::new(),
                similar: Vec::new(),
            });
        }
        mark_similar(&mut rows);
        rows.sort_by(|a, b| {
            a.namespace.cmp(&b.namespace).then_with(|| {
                a.label
                    .to_lowercase()
                    .cmp(&b.label.to_lowercase())
                    .then_with(|| a.label.cmp(&b.label))
            })
        });

        let mut namespaces: BTreeMap<String, Namespace> = BTreeMap::new();
        for key in gathered
            .namespaces
            .keys()
            .chain(entries.iter().map(|e| &e.namespace))
        {
            namespaces.entry(key.clone()).or_insert_with(|| Namespace {
                key: key.clone(),
                label: gathered
                    .namespaces
                    .get(key)
                    .map_or_else(|| key.clone(), |(_, label)| label.clone()),
                rows: 0,
                to_review: 0,
            });
        }
        for row in &rows {
            if let Some(ns) = namespaces.get_mut(&row.namespace) {
                ns.rows += 1;
                ns.to_review += u32::from(!row.reviewed);
            }
        }
        let read_documents: BTreeSet<DocumentId> =
            gathered.readings.iter().map(|r| r.document).collect();
        Ok(DictionaryView {
            namespaces: namespaces.into_values().collect(),
            rows,
            documents_read: read_documents.len() as u32,
            documents: gathered.documents,
        })
    }

    /// The results that listed any of these forms of a field, newest first, to open beside the
    /// document.
    pub fn evidence(
        &self,
        project: ProjectId,
        namespace: &str,
        forms: &[String],
    ) -> Result<Vec<Evidence>, DictionaryError> {
        let gathered = self.gather(project, None)?;
        let mut out: Vec<Evidence> = gathered
            .readings
            .iter()
            .filter_map(|r| {
                let listed: Vec<String> = r
                    .fields
                    .get(namespace)?
                    .iter()
                    .filter(|f| forms.contains(f))
                    .cloned()
                    .collect();
                (!listed.is_empty()).then(|| Evidence {
                    document_id: r.document,
                    document: gathered
                        .titles
                        .get(&r.document)
                        .cloned()
                        .unwrap_or_default(),
                    job_id: r.job_id,
                    job_name: r.job_name.clone(),
                    source_id: r.source_id,
                    setup_label: r.setup_label.clone(),
                    created_at: r.created_at,
                    forms: listed,
                })
            })
            .collect();
        out.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.document.cmp(&b.document))
        });
        Ok(out)
    }
}

/// The keys of the entries an entry was kept apart from.
fn kept_from(apart: &[KeptApart], id: EntryId) -> Vec<String> {
    apart
        .iter()
        .filter(|p| p.holds(id))
        .map(|p| if p.a == id { p.b } else { p.a }.to_string())
        .collect()
}

/// Ties rows of a namespace whose forms share a folded or word-order key, unless the researcher
/// kept them apart.
fn mark_similar(rows: &mut [DictionaryRow]) {
    let mut by_key: HashMap<(String, String), BTreeSet<usize>> = HashMap::new();
    for (i, row) in rows.iter().enumerate() {
        for form in &row.forms {
            let keys = std::iter::once(folded(&form.form)).chain(word_order_key(&form.form));
            for key in keys.filter(|k| !k.is_empty()) {
                by_key
                    .entry((row.namespace.clone(), key))
                    .or_default()
                    .insert(i);
            }
        }
    }
    let mut similar: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); rows.len()];
    for group in by_key.values().filter(|g| g.len() > 1) {
        for &i in group {
            similar[i].extend(group.iter().filter(|&&j| j != i));
        }
    }
    let keys: Vec<String> = rows.iter().map(|r| r.key.clone()).collect();
    for (i, row) in rows.iter_mut().enumerate() {
        row.similar = similar[i]
            .iter()
            .map(|&j| keys[j].clone())
            .filter(|k| !row.apart.contains(k))
            .collect();
    }
}

/// The entry holding a form of a namespace, if any.
pub(crate) fn holder<'a>(entries: &'a [Entry], namespace: &str, form: &str) -> Option<&'a Entry> {
    entries
        .iter()
        .find(|e| e.namespace == namespace && e.forms.iter().any(|f| f == form))
}
