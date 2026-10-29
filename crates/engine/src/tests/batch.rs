//! Economy mode against a fake batch service: batches are sent, waited for and collected,
//! survive the app closing, and are cancelled with their job.

use std::sync::atomic::AtomicBool;

use kalamu_core::{BatchContext, BatchLimits, BatchPhase, BatchProgress, JobSettings, Pace};

use super::*;

/// A batch as the fake service holds it: its id, and each item's id and page number.
type Sent = (String, Vec<(String, u32)>);

/// A provider with a batch service: answers each page with its number, after a number of polls.
#[derive(Default)]
struct Batchy {
    sent: Mutex<Vec<Sent>>,
    calls: AtomicUsize,
    /// Polls answered "still working" before a batch ends.
    working_polls: AtomicUsize,
    expire: AtomicBool,
    cancelled: Mutex<Vec<String>>,
    /// The effort its requests carry, which an update of the app may change mid-batch.
    effort: Mutex<&'static str>,
}

impl Batchy {
    fn submitted(&self) -> usize {
        self.sent.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    fn pages_sent(&self) -> Vec<u32> {
        let sent = self.sent.lock().unwrap_or_else(|p| p.into_inner());
        let mut pages: Vec<u32> = sent
            .iter()
            .flat_map(|(_, items)| items.iter().map(|(_, n)| *n))
            .collect();
        pages.sort_unstable();
        pages
    }
}

#[async_trait]
impl Provider for Batchy {
    fn id(&self) -> ProviderId {
        ProviderId::new("batchy")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf"]
    }

    fn request_settings(&self, _task: Task, _model: &str) -> Vec<kalamu_core::RequestSetting> {
        let effort = *self.effort.lock().unwrap_or_else(|p| p.into_inner());
        vec![kalamu_core::RequestSetting::new("reasoning_effort", effort)]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        _request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ProcessOutput {
            text: "one by one".into(),
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }

    fn batches(&self, _task: Task, _model: &str) -> bool {
        true
    }

    fn batch_limits(&self) -> BatchLimits {
        BatchLimits {
            max_items: 2,
            ..BatchLimits::default()
        }
    }

    async fn submit_batch(
        &self,
        _key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let items = requests
            .iter()
            .map(|(id, r)| match r.input.piece {
                kalamu_core::PieceRef::Page { number, .. } => (id.clone(), number),
                _ => (id.clone(), 0),
            })
            .collect();
        let mut sent = self.sent.lock().unwrap_or_else(|p| p.into_inner());
        let remote = format!("remote-{}", sent.len());
        sent.push((remote.clone(), items));
        Ok(remote)
    }

    async fn poll_batch(
        &self,
        _key: &ApiKey,
        batch_id: &str,
        _context: BatchContext<'_>,
        ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        if self.expire.load(Ordering::SeqCst) {
            return Ok(BatchProgress::Abandoned("the batch expired".into()));
        }
        if self.working_polls.load(Ordering::SeqCst) > 0 {
            self.working_polls.fetch_sub(1, Ordering::SeqCst);
            return Ok(BatchProgress::Working {
                finished: 0,
                total: ids.len() as u32,
            });
        }
        let sent = self.sent.lock().unwrap_or_else(|p| p.into_inner());
        let items = sent
            .iter()
            .find(|(remote, _)| remote == batch_id)
            .map(|(_, items)| items.clone())
            .unwrap_or_default();
        Ok(BatchProgress::Ended(
            items
                .into_iter()
                .map(|(id, number)| {
                    (
                        id,
                        Ok(ProcessOutput {
                            text: format!("batched page {number}"),
                            usage: Usage {
                                input_tokens: 1000,
                                output_tokens: 100,
                                ..Usage::default()
                            },
                            ..ProcessOutput::default()
                        }),
                    )
                })
                .collect(),
        ))
    }

    async fn cancel_batch(&self, _key: &ApiKey, batch_id: &str) -> Result<(), ProviderError> {
        self.cancelled
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(batch_id.to_string());
        Ok(())
    }
}

fn engine(store: Arc<SqliteStore>, batchy: Arc<Batchy>) -> Arc<Engine> {
    Engine::new(
        store,
        vec![batchy],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            batch_poll: Duration::from_millis(10),
            ..EngineConfig::default()
        },
    )
}

struct Economy {
    f: Fixture,
    batchy: Arc<Batchy>,
}

fn economy() -> Economy {
    let mut f = fixture();
    let batchy = Arc::new(Batchy::default());
    f.engine = engine(f.store.clone(), batchy.clone());
    Economy { f, batchy }
}

impl Economy {
    fn job(&self, pages: usize, pace: Pace) -> Job {
        let source = self.f.pdf("register.pdf", pages);
        self.f
            .engine
            .create_job(NewJob {
                name: "Register".into(),
                task: Task::TextExtraction,
                sources: vec![source],
                setups: vec![SetupRequest {
                    provider: ProviderId::new("batchy"),
                    model: "batchy-model".into(),
                    preset_id: None,
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings: JobSettings {
                    pace,
                    ..JobSettings::default()
                },
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt],
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("create job: {e}"))
    }

    /// Starts a job and returns, still listening, once the provider holds `batches` batches.
    async fn start_until_waiting(
        &self,
        id: JobId,
        batches: usize,
    ) -> tokio::sync::broadcast::Receiver<ProgressEvent> {
        let mut events = self.f.engine.subscribe();
        self.f.engine.start(id).unwrap_or_else(|e| panic!("{e}"));
        loop {
            match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
                Ok(Ok(ProgressEvent::BatchWaiting { .. })) => {
                    let waiting = self
                        .f
                        .store
                        .batches(id)
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|b| b.phase == BatchPhase::Waiting)
                        .count();
                    if waiting >= batches {
                        return events;
                    }
                }
                Ok(Ok(ProgressEvent::JobFinished { status, .. })) => {
                    panic!("finished before waiting: {status:?}")
                }
                Ok(Ok(_)) => {}
                other => panic!("no BatchWaiting event: {other:?}"),
            }
        }
    }

    async fn finish(
        &self,
        id: JobId,
        mut events: tokio::sync::broadcast::Receiver<ProgressEvent>,
    ) -> JobStatus {
        loop {
            match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
                Ok(Ok(ProgressEvent::JobFinished { job_id, status })) if job_id == id => {
                    return status;
                }
                Ok(Ok(_)) => {}
                other => panic!("no JobFinished event: {other:?}"),
            }
        }
    }
}

#[tokio::test]
async fn an_economy_job_sends_its_pages_in_batches_and_collects_them() {
    let e = economy();
    e.batchy.working_polls.store(2, Ordering::SeqCst);
    let job = e.job(3, Pace::Economy);
    assert_eq!(e.f.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        e.batchy.calls.load(Ordering::SeqCst),
        0,
        "nothing sent one by one"
    );
    assert_eq!(
        e.batchy.submitted(),
        2,
        "two pages a batch: three pages make two"
    );
    assert_eq!(e.batchy.pages_sent(), vec![1, 2, 3]);

    let txt = std::fs::read_to_string(e.f.dir.path().join("register.txt")).unwrap_or_default();
    assert!(txt.contains("batched page 3"), "{txt}");
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 3);
    let key = CacheKey(pieces[0].cache_key.clone().unwrap_or_default());
    let result = e.f.store.cached_result(&key).ok().flatten();
    assert!(
        result.is_some_and(|r| r.output.usage.batch),
        "billed as a batch"
    );
    let batches = e.f.store.batches(job.id).unwrap_or_default();
    assert!(batches.iter().all(|b| b.phase == BatchPhase::Collected));
}

#[tokio::test]
async fn a_fast_job_calls_one_by_one_even_where_batches_exist() {
    let e = economy();
    let job = e.job(2, Pace::Fast);
    assert_eq!(e.f.run(job.id).await, JobStatus::Completed);
    assert_eq!(e.batchy.calls.load(Ordering::SeqCst), 2);
    assert_eq!(e.batchy.submitted(), 0);
}

#[tokio::test]
async fn closing_the_app_mid_batch_collects_the_answers_on_the_next_launch() {
    let e = economy();
    e.batchy.working_polls.store(usize::MAX, Ordering::SeqCst);
    *e.batchy.effort.lock().unwrap_or_else(|p| p.into_inner()) = "low";
    let job = e.job(3, Pace::Economy);
    let events = e.start_until_waiting(job.id, 2).await;
    // The provider holds the batches while the app is closed: the job is stopped as a pause
    // would, and its database is left as a crash leaves it.
    e.f.engine.pause(job.id);
    assert_eq!(e.finish(job.id, events).await, JobStatus::Paused);
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Submitted), 3);
    e.f.store
        .set_job_status(job.id, JobStatus::Running)
        .unwrap_or_else(|e| panic!("{e}"));

    // A batch recorded but never confirmed as sent: its piece (page 3) must be sent again.
    let third =
        e.f.store
            .batches(job.id)
            .unwrap_or_default()
            .into_iter()
            .find(|b| b.items.iter().any(|i| i.index == 2))
            .unwrap_or_else(|| panic!("no batch holds page 3"));
    let mut orphan = third.clone();
    orphan.id = "never-sent".into();
    orphan.phase = BatchPhase::Sending;
    orphan.remote_id = None;
    let mut lost = third;
    lost.phase = BatchPhase::Abandoned;
    e.f.store.put_batch(&lost).unwrap_or_else(|e| panic!("{e}"));
    e.f.store
        .put_batch(&orphan)
        .unwrap_or_else(|e| panic!("{e}"));

    // The next launch, by an updated app that sends another effort: a new engine on the same
    // database and the same provider account.
    e.batchy.working_polls.store(0, Ordering::SeqCst);
    *e.batchy.effort.lock().unwrap_or_else(|p| p.into_inner()) = "high";
    let relaunched = engine(e.f.store.clone(), e.batchy.clone());
    relaunched.recover().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        relaunched.jobs_waiting_for_batches().unwrap_or_default(),
        vec![job.id]
    );
    // At launch the waiting batch is only collected: the unsent page waits for the researcher.
    let events = relaunched.subscribe();
    relaunched
        .collect_waiting(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.finish(job.id, events).await, JobStatus::Interrupted);
    assert_eq!(e.batchy.submitted(), 2, "nothing sent at launch");
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 2);
    assert_eq!(count(&pieces, PieceStatus::Pending), 1);

    // Resumed, the job sends what is left.
    let events = relaunched.subscribe();
    relaunched.start(job.id).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.finish(job.id, events).await, JobStatus::Completed);
    assert_eq!(
        e.batchy.submitted(),
        3,
        "the waiting batch is collected, not sent again; only the unsent piece is"
    );
    assert_eq!(e.batchy.pages_sent(), vec![1, 2, 3, 3]);
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 3);

    // Each result keeps the effort it was sent with: pages 1 and 2 before the update, page 3,
    // sent again, after it.
    let mut efforts: Vec<(u32, String)> = pieces
        .iter()
        .map(|p| {
            let key = CacheKey(p.cache_key.clone().unwrap_or_default());
            let result =
                e.f.store
                    .cached_result(&key)
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| panic!("no result for piece {}", p.index));
            (p.index, result.request[0].value.clone())
        })
        .collect();
    efforts.sort();
    assert_eq!(
        efforts,
        vec![(0, "low".into()), (1, "low".into()), (2, "high".into())]
    );
}

#[tokio::test]
async fn cancelling_a_job_cancels_its_batches_at_the_provider() {
    let e = economy();
    e.batchy.working_polls.store(usize::MAX, Ordering::SeqCst);
    let job = e.job(2, Pace::Economy);
    let events = e.start_until_waiting(job.id, 1).await;
    e.f.engine.cancel(job.id);
    assert_eq!(e.finish(job.id, events).await, JobStatus::Cancelled);
    assert_eq!(
        *e.batchy.cancelled.lock().unwrap_or_else(|p| p.into_inner()),
        vec!["remote-0".to_string()]
    );
    let batches = e.f.store.batches(job.id).unwrap_or_default();
    assert!(batches.iter().all(|b| b.phase == BatchPhase::Abandoned));
}

#[tokio::test]
async fn an_expired_batch_fails_its_pages_for_a_later_retry() {
    let e = economy();
    e.batchy.expire.store(true, Ordering::SeqCst);
    let job = e.job(2, Pace::Economy);
    assert_eq!(e.f.run(job.id).await, JobStatus::CompletedWithErrors);
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Failed), 2);
    assert!(pieces[0].error.as_deref() == Some("the batch expired"));

    // Retried, the pages go in a new batch.
    e.batchy.expire.store(false, Ordering::SeqCst);
    let mut events = e.f.engine.subscribe();
    e.f.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    while !matches!(events.recv().await, Ok(ProgressEvent::JobFinished { .. })) {}
    assert_eq!(e.batchy.submitted(), 2);
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 2);
}
