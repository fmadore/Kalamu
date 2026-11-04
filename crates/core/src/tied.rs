//! The results tied to a project's documents: each set-up of each job read from a document's
//! source, and which of them the researcher preferred. Worked out here once for what reads a
//! project's results (export profiles, the review queue, search and the dictionary), so that
//! each reads the jobs once, skips a job deleted since it was tied, and returns any other
//! failure of the job store rather than counting it as nothing read.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    DocumentId, DocumentLink, Job, JobId, PreferredResult, SetupId, SourceId, StoreError, Task,
};

/// One result of a document: a set-up of a job over the document's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TiedResult {
    pub document_id: DocumentId,
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub task: Task,
    /// The researcher chose it as the document's result for its task.
    pub preferred: bool,
}

/// Every result tied to some of a project's documents, with the jobs they belong to.
#[derive(Debug, Clone, Default)]
pub struct TiedResults {
    jobs: HashMap<JobId, Job>,
    results: Vec<TiedResult>,
    preferences: Vec<PreferredResult>,
}

impl TiedResults {
    /// Reads, through `load`, each job tied by `links` to a document `keep` accepts, once. A
    /// job deleted since it was tied is left out; any other failure is returned. The results
    /// are in the order of the links, then of each job's set-ups.
    pub fn read(
        links: &[DocumentLink],
        preferred: &[PreferredResult],
        keep: impl Fn(DocumentId) -> bool,
        mut load: impl FnMut(JobId) -> Result<Job, StoreError>,
    ) -> Result<Self, StoreError> {
        let mut jobs: HashMap<JobId, Option<Job>> = HashMap::new();
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        for link in links.iter().filter(|l| keep(l.document_id)) {
            let job = match jobs.entry(link.job_id) {
                Entry::Occupied(known) => known.into_mut(),
                Entry::Vacant(slot) => slot.insert(match load(link.job_id) {
                    Ok(job) => Some(job),
                    Err(StoreError::NotFound) => None,
                    Err(e) => return Err(e),
                }),
            };
            let Some(job) = job.as_ref() else {
                continue;
            };
            for setup in &job.setups {
                if !seen.insert((link.document_id, job.id, setup.id, link.source_id)) {
                    continue;
                }
                results.push(TiedResult {
                    document_id: link.document_id,
                    job_id: job.id,
                    setup_id: setup.id,
                    source_id: link.source_id,
                    task: job.task,
                    preferred: preferred.iter().any(|p| {
                        p.document_id == link.document_id
                            && p.task == job.task
                            && p.job_id == job.id
                            && p.setup_id == setup.id
                    }),
                });
            }
        }
        Ok(Self {
            jobs: jobs
                .into_iter()
                .filter_map(|(id, job)| Some((id, job?)))
                .collect(),
            results,
            preferences: preferred.to_vec(),
        })
    }

    pub fn job(&self, id: JobId) -> Option<&Job> {
        self.jobs.get(&id)
    }

    /// The jobs read, by id, so that what depends on their order is the same each time.
    pub fn jobs(&self) -> Vec<&Job> {
        let mut jobs: Vec<&Job> = self.jobs.values().collect();
        jobs.sort_by_key(|j| j.id);
        jobs
    }

    /// Every result, in the order of the links.
    pub fn all(&self) -> &[TiedResult] {
        &self.results
    }

    /// A document's results for a task.
    pub fn of(&self, document: DocumentId, task: Task) -> impl Iterator<Item = &TiedResult> {
        self.results
            .iter()
            .filter(move |r| r.document_id == document && r.task == task)
    }

    /// The tasks a document has results of, in the order tasks are listed.
    pub fn tasks(&self, document: DocumentId) -> Vec<Task> {
        let mut tasks: Vec<Task> = self
            .results
            .iter()
            .filter(|r| r.document_id == document)
            .map(|r| r.task)
            .collect();
        tasks.sort_by_key(|t| Task::ALL.iter().position(|x| x == t));
        tasks.dedup();
        tasks
    }

    /// The choice the researcher made for a document's task, even if its job or set-up has gone
    /// since.
    pub fn preference(&self, document: DocumentId, task: Task) -> Option<&PreferredResult> {
        self.preferences
            .iter()
            .find(|p| p.document_id == document && p.task == task)
    }

    /// The result the researcher preferred for a document's task, while it still exists.
    pub fn preferred(&self, document: DocumentId, task: Task) -> Option<&TiedResult> {
        self.of(document, task).find(|r| r.preferred)
    }

    /// Every document's results, task by task, the documents in id order and the tasks in the
    /// order tasks are listed.
    pub fn by_document_and_task(&self) -> BTreeMap<(DocumentId, usize), Vec<&TiedResult>> {
        let mut out: BTreeMap<(DocumentId, usize), Vec<&TiedResult>> = BTreeMap::new();
        for r in &self.results {
            let order = Task::ALL.iter().position(|t| *t == r.task).unwrap_or(0);
            out.entry((r.document_id, order)).or_default().push(r);
        }
        out
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{
        JobSettings, JobStatus, OutputSettings, PromptSnapshot, ProviderId, Setup, SetupOptions,
        Timestamp,
    };

    fn setup() -> Setup {
        Setup {
            id: SetupId::new(),
            label: "a".into(),
            provider: ProviderId::new("gemini"),
            model: "gemini-flash".into(),
            prompt: PromptSnapshot {
                preset_id: None,
                preset_name: "Printed".into(),
                template: String::new(),
                text: String::new(),
                user_prompt: String::new(),
                schema: None,
            },
            options: SetupOptions::default(),
            merge: None,
            pages_cap: None,
        }
    }

    fn job(task: Task, setups: usize) -> Job {
        Job {
            id: JobId::new(),
            name: "job".into(),
            task,
            created_at: Timestamp::now(),
            sources: Vec::new(),
            setups: (0..setups).map(|_| setup()).collect(),
            settings: JobSettings::default(),
            output: OutputSettings::default(),
            status: JobStatus::Completed,
        }
    }

    fn link(document: DocumentId, job: &Job) -> DocumentLink {
        DocumentLink {
            document_id: document,
            job_id: job.id,
            source_id: SourceId::new(),
            linked_at: Timestamp::now(),
        }
    }

    #[test]
    fn reads_each_job_once_and_skips_deleted_ones() {
        let (a, b) = (DocumentId::new(), DocumentId::new());
        let compared = job(Task::TextExtraction, 2);
        let gone = job(Task::Analysis, 1);
        let links = [link(a, &compared), link(b, &compared), link(a, &gone)];
        let preferred = [PreferredResult {
            document_id: a,
            task: Task::TextExtraction,
            job_id: compared.id,
            setup_id: compared.setups[1].id,
        }];
        let mut loads = 0;
        let tied = TiedResults::read(
            &links,
            &preferred,
            |_| true,
            |id| {
                loads += 1;
                if id == compared.id {
                    Ok(compared.clone())
                } else {
                    Err(StoreError::NotFound)
                }
            },
        )
        .unwrap();
        assert_eq!(loads, 2);
        assert_eq!(tied.all().len(), 4);
        assert_eq!(tied.tasks(a), [Task::TextExtraction]);
        let chosen = tied.preferred(a, Task::TextExtraction).unwrap();
        assert_eq!(chosen.setup_id, compared.setups[1].id);
        assert!(tied.preferred(b, Task::TextExtraction).is_none());
        assert!(tied.preference(a, Task::TextExtraction).is_some());
        assert_eq!(tied.by_document_and_task().len(), 2);
    }

    #[test]
    fn keeps_only_the_documents_asked_for_and_returns_other_failures() {
        let (a, b) = (DocumentId::new(), DocumentId::new());
        let j = job(Task::Analysis, 1);
        let links = [link(a, &j), link(b, &j)];
        let tied = TiedResults::read(&links, &[], |d| d == b, |_| Ok(j.clone())).unwrap();
        assert!(tied.all().iter().all(|r| r.document_id == b));
        let failed = TiedResults::read(
            &links,
            &[],
            |_| true,
            |_| Err(StoreError::Database("locked".into())),
        );
        assert!(failed.is_err());
    }
}
