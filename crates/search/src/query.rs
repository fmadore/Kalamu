//! A search: the results in scope chosen, the index brought up to date with them, the passages
//! holding the words found, and every match placed in the text shown.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use kalamu_core::contents;
use kalamu_core::{
    Document, DocumentId, Field, FieldKind, Job, JobId, Passage, PieceRef, ProjectId, SetupId,
    SourceId, Task,
};

use crate::fold::{context, find, folded, searchable};
use crate::passages::searched;
use crate::{
    Coverage, DocumentHits, FieldRef, Hit, MAX_LISTED, ResultHits, ResultPolicy, ReviewFilter,
    Search, SearchError, SearchRequest, SearchView,
};

/// A result chosen for the search: a set-up of a job over a document's source.
struct Chosen {
    document: DocumentId,
    job_id: JobId,
    setup_id: SetupId,
    source_id: SourceId,
    preferred: bool,
}

/// Whether a task's results are searched.
fn wanted_task(task: Task, request: &SearchRequest) -> bool {
    (task.yields_text() || request.generated) && request.task.is_none_or(|t| t == task)
}

/// Every field an analysis or a table of contents asks for, entries' fields included, by key.
fn field_labels(fields: &[Field], out: &mut HashMap<String, String>) {
    for f in fields {
        out.entry(f.key.clone()).or_insert_with(|| f.label.clone());
        if let FieldKind::Entries { fields } = &f.kind {
            field_labels(fields, out);
        }
    }
}

fn labels(job: &Job, setup: SetupId) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Some(schema) = job
        .setups
        .iter()
        .find(|s| s.id == setup)
        .and_then(|s| s.prompt.schema.as_ref())
    else {
        return out;
    };
    if job.task == Task::Contents {
        field_labels(&contents::merge_schema(schema).fields, &mut out);
    } else {
        field_labels(&schema.fields, &mut out);
    }
    out
}

/// A result with its matches, before they are listed.
type Listed = (ResultHits, Vec<Found>);
/// The results of each document with matches, by job and set-up.
type Grouped = BTreeMap<(JobId, SetupId), Listed>;

/// A match before it is listed: the words around it are cut from its text only if it is.
struct Found {
    hit: Hit,
    text: Rc<str>,
    span: (usize, usize),
}

impl Found {
    fn listed(mut self) -> Hit {
        let (before, words, after) = context(&self.text, self.span);
        self.hit.before = before;
        self.hit.words = words;
        self.hit.after = after;
        self.hit
    }

    fn copy(&self) -> Self {
        Self {
            hit: self.hit.clone(),
            text: self.text.clone(),
            span: self.span,
        }
    }
}

/// The matches in one passage, numbered within its piece as the review numbers them.
fn hits(passage: &Passage, needle: &[char], labels: &HashMap<String, String>) -> Vec<Found> {
    let start_ms = match passage.piece {
        PieceRef::Segment { start_ms, .. } => Some(start_ms),
        _ => None,
    };
    let mut out = Vec::new();
    let mut nth = 0;
    for part in &passage.parts {
        let found = find(&part.text, needle);
        // A recording's match is tied to its line when the lines hold as many matches as the
        // text: the n-th match is then in the line holding the n-th match.
        let lines: Vec<u64> = part
            .lines
            .iter()
            .flat_map(|(at, text)| find(text, needle).into_iter().map(move |_| *at))
            .collect();
        let timed = start_ms.is_some() && !lines.is_empty() && lines.len() == found.len();
        let text: Rc<str> = Rc::from(part.text.as_str());
        for (i, m) in found.into_iter().enumerate() {
            let hit = Hit {
                index: passage.key.index,
                piece: passage.piece.clone(),
                nth,
                field: part.field.as_ref().map(|key| FieldRef {
                    key: key.clone(),
                    label: labels.get(key).cloned().unwrap_or_else(|| key.clone()),
                }),
                entry: part.entry.clone(),
                reason: part.reason,
                before: String::new(),
                words: String::new(),
                after: String::new(),
                at_ms: timed.then(|| start_ms.unwrap_or_default() + lines[i]),
                reviewed: passage.reviewed,
                corrected: passage.corrected,
            };
            out.push(Found {
                hit,
                text: text.clone(),
                span: m,
            });
            nth += 1;
        }
    }
    out
}

/// Titles as a reader sorts them: capitals and accents set aside.
fn by_title(a: &str, b: &str) -> Ordering {
    folded(a).cmp(&folded(b)).then_with(|| a.cmp(b))
}

impl Search {
    /// Every place the project's documents hold `request.query`, in the text as the researcher
    /// corrected it. `stop` is asked between steps: true when a newer search has begun.
    pub fn search(
        &self,
        project: ProjectId,
        request: &SearchRequest,
        stop: &dyn Fn() -> bool,
    ) -> Result<SearchView, SearchError> {
        self.check_project(project)?;
        if !searchable(&request.query) {
            return Err(SearchError::TooShort);
        }
        let members = self.projects.members(project)?;
        let documents: Vec<Document> = self
            .projects
            .documents(project)?
            .into_iter()
            .filter(|d| {
                request
                    .collection
                    .is_none_or(|c| members.contains(&(c, d.id)))
            })
            .filter(|d| request.tag.as_deref().is_none_or(|t| d.has_tag(t)))
            .collect();
        let titles: HashMap<DocumentId, &str> =
            documents.iter().map(|d| (d.id, d.title.as_str())).collect();
        let preferred = self.projects.preferred(project)?;
        let mut coverage = Coverage {
            documents: documents.len() as u32,
            ..Coverage::default()
        };

        // The results of each document, by task, and the preferred one where it is chosen.
        let mut jobs: HashMap<JobId, Option<Job>> = HashMap::new();
        let mut candidates: BTreeMap<(DocumentId, &'static str), Vec<Chosen>> = BTreeMap::new();
        for link in self.projects.links(project)? {
            if !titles.contains_key(&link.document_id) {
                continue;
            }
            let job = jobs
                .entry(link.job_id)
                .or_insert_with(|| self.jobs.job(link.job_id).ok());
            let Some(job) = job.as_ref() else { continue };
            if !wanted_task(job.task, request) {
                continue;
            }
            for setup in &job.setups {
                candidates
                    .entry((link.document_id, job.task.as_str()))
                    .or_default()
                    .push(Chosen {
                        document: link.document_id,
                        job_id: job.id,
                        setup_id: setup.id,
                        source_id: link.source_id,
                        preferred: preferred.iter().any(|p| {
                            p.document_id == link.document_id
                                && p.task == job.task
                                && p.job_id == job.id
                                && p.setup_id == setup.id
                        }),
                    });
            }
        }
        let mut chosen = Vec::new();
        let mut without_preference = HashSet::new();
        for ((document, _), results) in candidates {
            let has_preferred = results.iter().any(|c| c.preferred);
            if request.results == ResultPolicy::Preferred && has_preferred {
                chosen.extend(results.into_iter().filter(|c| c.preferred));
            } else {
                if !has_preferred && results.len() > 1 {
                    without_preference.insert(document);
                }
                chosen.extend(results);
            }
        }
        coverage.without_preference = without_preference.len() as u32;

        // The index brought up to date with every result chosen, job by job.
        let mut wanted: HashMap<JobId, HashSet<(SetupId, SourceId)>> = HashMap::new();
        for c in &chosen {
            wanted
                .entry(c.job_id)
                .or_default()
                .insert((c.setup_id, c.source_id));
        }
        let mut read: HashSet<(JobId, SetupId, SourceId)> = HashSet::new();
        for (job_id, results) in &wanted {
            if stop() {
                return Err(SearchError::Stopped);
            }
            let Some(Some(job)) = jobs.get(job_id) else {
                continue;
            };
            for piece in self.refresh(job, results, stop)? {
                if !results.contains(&(piece.setup_id, piece.source_id))
                    || !searched(job.task, &piece.piece)
                {
                    continue;
                }
                if piece.status.is_finished() {
                    coverage.pieces += 1;
                    read.insert((piece.job_id, piece.setup_id, piece.source_id));
                } else {
                    coverage.pieces_unread += 1;
                }
            }
        }
        let searched_documents: HashSet<DocumentId> = chosen
            .iter()
            .filter(|c| read.contains(&(c.job_id, c.setup_id, c.source_id)))
            .map(|c| c.document)
            .collect();
        coverage.searched = searched_documents.len() as u32;
        coverage.unread = coverage.documents - coverage.searched;

        // The passages holding the words, each placed in the results that read it.
        let needle: Vec<char> = folded(&request.query).chars().collect();
        let needle_text: String = needle.iter().collect();
        let mut by_result: HashMap<(JobId, SetupId, SourceId), Vec<&Chosen>> = HashMap::new();
        for c in &chosen {
            by_result
                .entry((c.job_id, c.setup_id, c.source_id))
                .or_default()
                .push(c);
        }
        let mut grouped: HashMap<DocumentId, Grouped> = HashMap::new();
        let mut label_cache: HashMap<(JobId, SetupId), HashMap<String, String>> = HashMap::new();
        for passage in self.index.find(&needle_text)? {
            let k = passage.key;
            let Some(results) = by_result.get(&(k.job_id, k.setup_id, k.source_id)) else {
                continue;
            };
            let reviewed_ok = match request.review {
                ReviewFilter::Any => true,
                ReviewFilter::Reviewed => passage.reviewed,
                ReviewFilter::NotReviewed => !passage.reviewed,
            };
            let Some(Some(job)) = jobs.get(&k.job_id) else {
                continue;
            };
            if !reviewed_ok {
                continue;
            }
            let labels = label_cache
                .entry((k.job_id, k.setup_id))
                .or_insert_with(|| labels(job, k.setup_id));
            let found = hits(&passage, &needle, labels);
            if found.is_empty() {
                continue;
            }
            for c in results {
                let setup = job.setups.iter().find(|s| s.id == k.setup_id);
                grouped
                    .entry(c.document)
                    .or_default()
                    .entry((k.job_id, k.setup_id))
                    .or_insert_with(|| {
                        let result = ResultHits {
                            job_id: k.job_id,
                            job_name: job.name.clone(),
                            setup_id: k.setup_id,
                            setup_label: setup.map(|s| s.label.clone()).unwrap_or_default(),
                            source_id: k.source_id,
                            task: job.task,
                            preferred: c.preferred,
                            generated: !job.task.yields_text(),
                            stale: false,
                            hits: Vec::new(),
                        };
                        (result, Vec::new())
                    })
                    .1
                    .extend(found.iter().map(Found::copy));
            }
        }

        // In title order, each document's results by task and when their job was made, each
        // result's matches in reading order; the first matches listed, all counted.
        let order = |task: Task| Task::ALL.iter().position(|t| *t == task);
        let created = |id: JobId| {
            jobs.get(&id)
                .and_then(Option::as_ref)
                .map(|j| j.created_at.0)
        };
        let title = |id: &DocumentId| titles.get(id).copied().unwrap_or("");
        let mut documents: Vec<(DocumentId, Vec<Listed>)> = grouped
            .into_iter()
            .map(|(document_id, results)| {
                let mut results: Vec<_> = results.into_values().collect();
                results
                    .sort_by_key(|(r, _)| (order(r.task), created(r.job_id), r.job_id, r.setup_id));
                for (_, found) in &mut results {
                    found.sort_by_key(|f| (f.hit.index, f.hit.nth));
                }
                (document_id, results)
            })
            .collect();
        documents.sort_by(|a, b| by_title(title(&a.0), title(&b.0)));
        let mut matches = 0usize;
        let mut documents_hit = Vec::new();
        for (document_id, results) in documents {
            let mut listed = Vec::new();
            for (mut result, found) in results {
                let room = MAX_LISTED.saturating_sub(matches);
                matches += found.len();
                result.hits = found.into_iter().take(room).map(Found::listed).collect();
                if !result.hits.is_empty() {
                    listed.push(result);
                }
            }
            if !listed.is_empty() {
                documents_hit.push(DocumentHits {
                    document_id,
                    title: title(&document_id).to_string(),
                    results: listed,
                });
            }
        }
        Ok(SearchView {
            documents: documents_hit,
            matches: matches as u32,
            listed: matches.min(MAX_LISTED) as u32,
            coverage,
        })
    }
}
