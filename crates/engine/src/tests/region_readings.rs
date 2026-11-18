//! A region of a page read again from the review: the picture cut from the page is sent alone as
//! a text extraction, recorded before and after, cached, cancellable, billed when refused, for a
//! job of any task and for a model on this computer; refusals of the picture reach the
//! interface with their own codes. Accepting is in [`accepting`].

use std::sync::atomic::AtomicBool;

use kalamu_core::{
    Correction, ProposalStatus, ReadingTarget, RegionAsk, RegionDraft, RegionReadingError,
};

use super::*;

mod accepting;
mod refusals;

/// Reads pictures: answers what it is told to, and records what each request sent.
struct Reader {
    id: &'static str,
    /// Runs on this computer: no key, no price.
    local: bool,
    calls: AtomicUsize,
    /// What each request sent: the task, the picture's type and size, the request.
    sent: Mutex<Vec<(Task, String, usize, String)>>,
    answer: Mutex<String>,
    /// Waits for the stop signal before answering.
    slow: AtomicBool,
    /// Refuses, billed for what it read.
    refuse: AtomicBool,
}

impl Reader {
    fn new(id: &'static str, local: bool) -> Arc<Self> {
        Arc::new(Self {
            id,
            local,
            calls: AtomicUsize::new(0),
            sent: Mutex::new(Vec::new()),
            answer: Mutex::new("Note en marge : vu le 3 mai 1956".into()),
            slow: AtomicBool::new(false),
            refuse: AtomicBool::new(false),
        })
    }
}

#[async_trait]
impl Provider for Reader {
    fn id(&self) -> ProviderId {
        ProviderId::new(self.id)
    }

    fn needs_key(&self) -> bool {
        !self.local
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["image/png", "image/jpeg"],
            _ => &[],
        }
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.sent.lock().unwrap_or_else(|p| p.into_inner()).push((
            request.task,
            request.input.mime.clone(),
            request.input.bytes.len(),
            request.user_prompt.to_string(),
        ));
        if self.slow.load(Ordering::SeqCst) {
            while !request.stop.is_stopped() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let usage = Usage {
            input_tokens: 300,
            output_tokens: 20,
            ..Usage::default()
        };
        if self.refuse.load(Ordering::SeqCst) {
            return Err(ProviderError::Blocked {
                reason: "declined".into(),
                usage: Some(usage),
            });
        }
        let text = self
            .answer
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        Ok(ProcessOutput {
            text: format!("  {text}\n"),
            usage,
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["reader-model".into()])
    }
}

struct Setting {
    engine: Arc<Engine>,
    store: Arc<SqliteStore>,
    reader: Arc<Reader>,
    local: Arc<Reader>,
    job: Job,
    /// A region the researcher drew on page 1, with no text yet.
    drawn: String,
    _dir: tempfile::TempDir,
}

const TEXT: &str = "Le Gouverneur général a reçu ce matin deux cents fideles de la ville.";

fn engine_with(
    store: Arc<SqliteStore>,
    providers: Vec<Arc<dyn Provider>>,
    dir: &Path,
) -> Arc<Engine> {
    Engine::new(
        store,
        providers,
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.join("listening"),
            snapshots: dir.join("snapshots"),
            write_delay: Duration::from_secs(3600),
            ..EngineConfig::default()
        },
    )
}

/// A job of `task` over a one-page PDF, run by the fake provider; for a text extraction, its
/// page corrected by hand to `TEXT`; a region drawn on its page.
async fn setting_for(task: Task) -> Setting {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let fake = Fake::new();
    let reader = Reader::new("reader", false);
    let local = Reader::new("tesseract-like", true);
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = engine_with(
        store.clone(),
        vec![fake.clone(), reader.clone(), local.clone()],
        dir.path(),
    );
    let f = Fixture {
        engine: engine.clone(),
        fake,
        store: store.clone(),
        dir,
    };
    let job = f.new_job_for(task, vec![f.pdf("bulletin.pdf", 1)], &["m"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    if task == Task::TextExtraction {
        engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: job.sources[0].id,
                index: 0,
                text: Some(TEXT.into()),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
    let drawn = engine
        .save_region(RegionDraft {
            id: None,
            job_id: job.id,
            source_id: job.sources[0].id,
            page: 1,
            half: None,
            x: 0.1,
            y: 0.1,
            width: 0.4,
            height: 0.1,
            text: String::new(),
            adjusts: None,
        })
        .unwrap_or_else(|e| panic!("{e}"))
        .id;
    Setting {
        engine,
        store,
        reader,
        local,
        job,
        drawn,
        _dir: f.dir,
    }
}

async fn setting() -> Setting {
    setting_for(Task::TextExtraction).await
}

impl Setting {
    /// Reading the drawn region again, as its text.
    fn ask(&self, id: &str) -> RegionAsk {
        RegionAsk {
            id: id.into(),
            job_id: self.job.id,
            source_id: self.job.sources[0].id,
            page: 1,
            half: None,
            x: 0.1,
            y: 0.1,
            width: 0.4,
            height: 0.1,
            region_id: self.drawn.clone(),
            target: ReadingTarget::Region,
            provider: ProviderId::new("reader"),
            model: "reader-model".into(),
            preset_id: None,
        }
    }

    /// Reading an engine's region whose words stand at "deux cents fideles" in the text shown.
    fn ask_text(&self, id: &str) -> RegionAsk {
        let at = TEXT.find("deux").unwrap_or(0);
        let start = TEXT[..at].encode_utf16().count() as u32;
        let was = "deux cents fideles";
        RegionAsk {
            region_id: "r1_l2".into(),
            target: ReadingTarget::Text {
                setup_id: self.job.setups[0].id,
                index: 0,
                start,
                end: start + was.encode_utf16().count() as u32,
                was: was.into(),
            },
            ..self.ask(id)
        }
    }

    fn calls(&self) -> usize {
        self.reader.calls.load(Ordering::SeqCst)
    }

    fn shown(&self) -> Option<String> {
        self.store
            .corrections(self.job.id)
            .unwrap_or_default()
            .into_iter()
            .next()
            .and_then(|c| c.text)
    }

    fn drawn_text(&self) -> String {
        self.store
            .region(&self.drawn)
            .map(|r| r.text)
            .unwrap_or_default()
    }
}

#[tokio::test]
async fn the_picture_of_the_region_is_sent_alone_as_a_text_extraction() {
    let s = setting().await;
    let estimate = s
        .engine
        .region_reading_estimate(&s.ask("estimate"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!estimate.cached);
    assert!(!estimate.offline);
    // The catalogue does not price this model: no price, rather than a guess.
    assert_eq!(estimate.usd, None);
    let reading = s
        .engine
        .ask_region_reading(s.ask("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reading.status, ProposalStatus::Answered);
    // The answer as a text extraction stores it: trimmed.
    assert_eq!(
        reading.answer.as_deref(),
        Some("Note en marge : vu le 3 mai 1956")
    );
    let sent = s
        .reader
        .sent
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    let (task, mime, size, request) = &sent[0];
    assert_eq!(*task, Task::TextExtraction);
    assert_eq!(mime, "image/png");
    assert!(*size > 0);
    // The text extraction's default preset, as sent, kept with the reading.
    assert_eq!(&reading.prompt.user_prompt, request);
    assert!(reading.prompt.preset_id.is_some());
    // Nothing is written until the reading is accepted.
    assert!(s.drawn_text().is_empty());
    assert_eq!(s.shown().as_deref(), Some(TEXT));
}

#[tokio::test]
async fn a_reading_is_recorded_before_and_after_and_asked_once() {
    let s = setting().await;
    let first = s
        .engine
        .ask_region_reading(s.ask("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.calls(), 1);
    assert_eq!(
        s.store.region_reading("g1").map(|r| r.status).ok(),
        Some(ProposalStatus::Answered)
    );
    assert!(!first.cache_key.is_empty());
    // The same picture read the same way again, under another id: the stored answer, free.
    let again = s
        .engine
        .ask_region_reading(s.ask("g2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.calls(), 1);
    assert!(again.cached);
    assert_eq!(again.cost_usd, None);
    assert_eq!(again.answer, first.answer);
    assert_eq!(again.cache_key, first.cache_key);
    let estimate = s
        .engine
        .region_reading_estimate(&s.ask("g3"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(estimate.cached);
    // Another picture is another key. (The sample page is blank, so a box of the same size
    // elsewhere on it is the same picture, rightly answered from the cache.)
    let other = RegionAsk {
        height: 0.2,
        ..s.ask("g4")
    };
    let other = s
        .engine
        .ask_region_reading(other)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(other.cache_key, first.cache_key);
    assert_eq!(s.calls(), 2);
    let listed = s
        .store
        .source_region_readings(s.job.id, s.job.sources[0].id)
        .unwrap_or_default();
    assert_eq!(listed.len(), 3);
}

#[tokio::test]
async fn a_reading_can_stop_being_waited_for_and_its_answer_is_kept() {
    let s = setting().await;
    s.reader.slow.store(true, Ordering::SeqCst);
    let engine = s.engine.clone();
    let ask = s.ask("g1");
    let asking = tokio::spawn(async move { engine.ask_region_reading(ask).await });
    for _ in 0..300 {
        if s.calls() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    s.engine.cancel_region_reading("g1");
    let reading = asking
        .await
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reading.status, ProposalStatus::Cancelled);
    assert_eq!(
        s.store.region_reading("g1").map(|r| r.status).ok(),
        Some(ProposalStatus::Cancelled)
    );
    assert!(s.engine.accept_region_reading("g1", "x".into()).is_err());
    // The answer that came after is stored: asked again, it costs nothing.
    let mut stored = false;
    for _ in 0..200 {
        stored = s
            .engine
            .region_reading_estimate(&s.ask("g2"))
            .await
            .is_ok_and(|e| e.cached);
        if stored {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(stored);
    s.reader.slow.store(false, Ordering::SeqCst);
    let again = s
        .engine
        .ask_region_reading(s.ask("g2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(again.cached);
    assert_eq!(s.calls(), 1);
}

#[tokio::test]
async fn a_refused_reading_is_billed_and_not_kept_and_an_empty_one_is_not_offered() {
    let s = setting().await;
    s.reader.refuse.store(true, Ordering::SeqCst);
    let reading = s
        .engine
        .ask_region_reading(s.ask("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reading.status, ProposalStatus::Failed);
    assert!(reading.error.is_some_and(|e| e.contains("declined")));
    // Billed to the job, charged to the page's piece; nothing kept, so asking again calls.
    assert_eq!(s.store.charges(s.job.id).unwrap_or_default().len(), 1);
    s.reader.refuse.store(false, Ordering::SeqCst);
    *s.reader.answer.lock().unwrap_or_else(|p| p.into_inner()) = "[BLANK PAGE]".into();
    let blank = s
        .engine
        .ask_region_reading(s.ask("g2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.calls(), 2);
    assert_eq!(blank.status, ProposalStatus::Failed);
    assert_eq!(
        blank.error.as_deref(),
        Some(RegionReadingError::Empty.to_string().as_str())
    );
    assert!(s.engine.accept_region_reading("g2", "x".into()).is_err());
}

#[tokio::test]
async fn a_job_of_another_task_reads_a_region_of_its_source() {
    let s = setting_for(Task::Analysis).await;
    let reading = s
        .engine
        .ask_region_reading(s.ask("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reading.status, ProposalStatus::Answered);
    assert_eq!(
        s.reader.sent.lock().unwrap_or_else(|p| p.into_inner())[0].0,
        Task::TextExtraction
    );
    s.engine
        .accept_region_reading("g1", "Note en marge".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.drawn_text(), "Note en marge");
}

#[tokio::test]
async fn a_model_on_this_computer_reads_a_region_without_a_key_or_a_price() {
    let s = setting().await;
    let ask = RegionAsk {
        provider: ProviderId::new("tesseract-like"),
        ..s.ask("g1")
    };
    let estimate = s
        .engine
        .region_reading_estimate(&ask)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(estimate.offline);
    assert_eq!(estimate.usd, None);
    let reading = s
        .engine
        .ask_region_reading(ask)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reading.status, ProposalStatus::Answered);
    assert_eq!(reading.cost_usd, None);
    assert_eq!(s.local.calls.load(Ordering::SeqCst), 1);
    assert_eq!(s.calls(), 0);
}
