//! A passage corrected from the review: the request carries the passage between its context
//! and the correction preset with the passage's own request; the call is recorded before and
//! after, cached, cancellable; a refused answer is billed and not offered; and only accepting
//! writes the researcher's correction, the model's output left as it was, with provenance.

use std::sync::atomic::AtomicBool;

use kalamu_core::passage_correction::PassageError;
use kalamu_core::{Correction, PassageAsk, ProposalNote, ProposalStatus};

use super::*;

/// Corrects passages: answers the passage's corrected text, or what it is told to, and records
/// each request.
struct Corrector {
    calls: AtomicUsize,
    /// What each request sent: the instructions, the request and the text.
    sent: Mutex<Vec<(String, String, String)>>,
    /// The answer to give; the passage with "fideles" corrected when empty.
    answer: Mutex<String>,
    /// Waits for the stop signal before answering.
    slow: AtomicBool,
}

impl Corrector {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            sent: Mutex::new(Vec::new()),
            answer: Mutex::new(String::new()),
            slow: AtomicBool::new(false),
        })
    }
}

#[async_trait]
impl Provider for Corrector {
    fn id(&self) -> ProviderId {
        ProviderId::new("corrector")
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextCorrection => &["text/plain"],
            _ => &[],
        }
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.task, Task::TextCorrection);
        assert_eq!(request.input.mime, "text/plain");
        let text = String::from_utf8_lossy(&request.input.bytes).into_owned();
        self.sent.lock().unwrap_or_else(|p| p.into_inner()).push((
            request.prompt.to_string(),
            request.user_prompt.to_string(),
            text.clone(),
        ));
        if self.slow.load(Ordering::SeqCst) {
            while !request.stop.is_stopped() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let given = self
            .answer
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        let answer = if given.is_empty() {
            let passage = text
                .split("<passage>\n")
                .nth(1)
                .and_then(|t| t.split("\n</passage>").next())
                .unwrap_or_default();
            passage.replace("fideles", "fidèles")
        } else {
            given
        };
        let fields = vec![FieldAnswer {
            key: kalamu_core::correction::TEXT.into(),
            value: FieldValue::Text(answer),
            reason: None,
        }];
        Ok(ProcessOutput {
            text: String::new(),
            fields,
            usage: Usage {
                input_tokens: 400,
                output_tokens: 40,
                ..Usage::default()
            },
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["corrector-model".into()])
    }
}

struct Setting {
    engine: Arc<Engine>,
    store: Arc<SqliteStore>,
    corrector: Arc<Corrector>,
    job: Job,
    _dir: tempfile::TempDir,
}

const TEXT: &str = "Le Gouverneur général a reçu ce matin deux cents fideles de la ville.";

/// A text extraction of one page, read and corrected by hand to `TEXT`.
async fn setting() -> Setting {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let fake = Fake::new();
    let corrector = Corrector::new();
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![fake.clone(), corrector.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            write_delay: Duration::from_secs(3600),
            ..EngineConfig::default()
        },
    );
    let f = Fixture {
        engine: engine.clone(),
        fake,
        store: store.clone(),
        dir,
    };
    let job = f.new_job(vec![f.pdf("bulletin.pdf", 1)], &["m"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
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
    Setting {
        engine,
        store,
        corrector,
        job,
        _dir: f.dir,
    }
}

impl Setting {
    fn ask(&self, id: &str) -> PassageAsk {
        let at = TEXT.find("deux").unwrap_or(0);
        let end = at + "deux cents fideles".len();
        PassageAsk {
            id: id.into(),
            job_id: self.job.id,
            setup_id: self.job.setups[0].id,
            source_id: self.job.sources[0].id,
            index: 0,
            before: TEXT[..at].into(),
            passage: TEXT[at..end].into(),
            after: TEXT[end..].into(),
            at: TEXT[..at].encode_utf16().count() as u32,
            provider: ProviderId::new("corrector"),
            model: "corrector-model".into(),
            preset_id: None,
        }
    }

    fn calls(&self) -> usize {
        self.corrector.calls.load(Ordering::SeqCst)
    }

    fn shown(&self) -> Option<String> {
        self.store
            .corrections(self.job.id)
            .unwrap_or_default()
            .into_iter()
            .next()
            .and_then(|c| c.text)
    }

    fn model_text(&self) -> String {
        let piece = self
            .store
            .pieces(self.job.id)
            .unwrap_or_default()
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("a piece"));
        self.store
            .finished_result(&piece)
            .ok()
            .flatten()
            .map(|r| r.output.text)
            .unwrap_or_default()
    }
}

#[tokio::test]
async fn the_request_carries_the_passage_between_its_context_and_the_preset() {
    let s = setting().await;
    let proposal = s
        .engine
        .ask_passage(s.ask("p1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(proposal.status, ProposalStatus::Answered);
    assert_eq!(proposal.answer.as_deref(), Some("deux cents fidèles"));
    let sent = s
        .corrector
        .sent
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    let (prompt, request, text) = &sent[0];
    // The correction preset, keeping the lines of the passage, with its own request after.
    assert!(
        prompt.contains("correcting OCR") || prompt.contains("OCR"),
        "{prompt}"
    );
    assert!(prompt.contains("Keep the lines of the source"), "{prompt}");
    assert!(request.starts_with("Correct the machine-read text above"));
    assert!(request.ends_with(kalamu_core::passage_correction::REQUEST));
    assert_eq!(
        text,
        &kalamu_core::passage_correction::input(
            "Le Gouverneur général a reçu ce matin ",
            "deux cents fideles",
            " de la ville."
        )
    );
    // The prompt as sent is kept with the proposal, and what it cost.
    assert_eq!(&proposal.prompt.user_prompt, request);
    assert!(!proposal.cached);
}

#[tokio::test]
async fn a_passage_is_recorded_before_and_after_and_asked_once() {
    let s = setting().await;
    let spent_before = s.store.spent(s.job.id).unwrap_or_default();
    let first = s
        .engine
        .ask_passage(s.ask("p1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.calls(), 1);
    assert_eq!(
        s.store.passage("p1").map(|p| p.status).ok(),
        Some(ProposalStatus::Answered)
    );
    // Its cost (the catalogue does not price this model) is counted with the job's.
    assert_eq!(first.cost_usd, None);
    assert!(s.store.spent(s.job.id).unwrap_or_default() >= spent_before);
    // The same passage asked again: the stored answer, for nothing.
    let again = s
        .engine
        .ask_passage(s.ask("p2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.calls(), 1);
    assert!(again.cached);
    assert_eq!(again.answer, first.answer);
    assert_eq!(again.cache_key, first.cache_key);
    let estimate = s
        .engine
        .passage_estimate(&s.ask("p3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(estimate.cached);
    assert!(!estimate.offline);
    // Another prompt is another key.
    let mut other = s.ask("p4");
    other.before = other.before[3..].to_string();
    let other = s
        .engine
        .ask_passage(other)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(other.cache_key, first.cache_key);
    assert_eq!(s.calls(), 2);
}

#[tokio::test]
async fn nothing_changes_the_text_until_the_proposal_is_accepted() {
    let s = setting().await;
    let model = s.model_text();
    s.engine
        .ask_passage(s.ask("p1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    let before = s
        .engine
        .accept_passage("p1", "deux cents fidèles".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        s.shown().as_deref(),
        Some("Le Gouverneur général a reçu ce matin deux cents fidèles de la ville.")
    );
    // Left reviewed as it was; the model's own text untouched.
    let kept = s.store.corrections(s.job.id).unwrap_or_default();
    assert!(kept[0].reviewed);
    assert_eq!(s.model_text(), model);
    assert_eq!(before.text.as_deref(), Some(TEXT));
    // Accepted once only.
    assert!(matches!(
        s.engine.accept_passage("p1", "x".into()),
        Err(EngineError::Passage(PassageError::NotWaiting))
    ));
    // Provenance names the model that proposed it and that the researcher accepted it.
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let passages = &doc.provenance.passages;
    assert_eq!(passages.len(), 1);
    assert_eq!(passages[0].model, "corrector-model");
    assert_eq!(passages[0].piece, 1);
    assert!(!passages[0].edited);
    assert!(
        passages[0]
            .prompt
            .user_prompt
            .ends_with(kalamu_core::passage_correction::REQUEST)
    );
    // Undone: the correction as it stood, and no longer in provenance.
    s.engine
        .undo_passage("p1", before)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    assert_eq!(
        s.store.passage("p1").map(|p| p.status).ok(),
        Some(ProposalStatus::Undone)
    );
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(doc.provenance.passages.is_empty());
}

#[tokio::test]
async fn an_edited_proposal_is_accepted_as_edited_and_a_discarded_one_changes_nothing() {
    let s = setting().await;
    s.engine
        .ask_passage(s.ask("p1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    s.engine
        .discard_passage("p1")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    assert_eq!(
        s.store.passage("p1").map(|p| p.status).ok(),
        Some(ProposalStatus::Discarded)
    );
    s.engine
        .ask_passage(s.ask("p2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    s.engine
        .accept_passage("p2", "deux cents fidèles musulmans".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        s.shown()
            .is_some_and(|t| t.contains("deux cents fidèles musulmans de la ville"))
    );
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(doc.provenance.passages[0].edited);
}

#[tokio::test]
async fn a_passage_whose_text_changed_meanwhile_is_not_applied() {
    let s = setting().await;
    s.engine
        .ask_passage(s.ask("p1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    // The researcher corrected the same words by hand while the model answered.
    let typed = TEXT.replace("fideles", "fidèles");
    s.engine
        .save_correction(Correction {
            job_id: s.job.id,
            setup_id: s.job.setups[0].id,
            source_id: s.job.sources[0].id,
            index: 0,
            text: Some(typed.clone()),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        s.engine.accept_passage("p1", "deux cents fidèles".into()),
        Err(EngineError::Passage(PassageError::Changed))
    ));
    assert_eq!(s.shown(), Some(typed));
    // Asking about it again is refused before anything is sent.
    assert!(matches!(
        s.engine.ask_passage(s.ask("p2")).await,
        Err(EngineError::Passage(PassageError::Changed))
    ));
    assert_eq!(s.calls(), 1);
}

#[tokio::test]
async fn an_answer_that_is_not_a_correction_of_the_passage_is_refused_and_billed() {
    let s = setting().await;
    *s.corrector.answer.lock().unwrap_or_else(|p| p.into_inner()) =
        "I cannot help with that request, as the text appears to be an excerpt of a colonial \
         bulletin that I would need more context to correct properly."
            .into();
    let proposal = s
        .engine
        .ask_passage(s.ask("p1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(proposal.status, ProposalStatus::Failed);
    assert!(proposal.answer.is_none());
    assert!(proposal.error.is_some_and(|e| e.contains("too different")));
    // Billed, and not kept: asking again calls the model again.
    assert_eq!(s.store.charges(s.job.id).unwrap_or_default().len(), 1);
    *s.corrector.answer.lock().unwrap_or_else(|p| p.into_inner()) =
        "Here is the corrected passage:\ndeux cents fidèles".into();
    let proposal = s
        .engine
        .ask_passage(s.ask("p2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.calls(), 2);
    assert_eq!(proposal.answer.as_deref(), Some("deux cents fidèles"));
    assert_eq!(proposal.notes, vec![ProposalNote::CommentaryRemoved]);
}

#[tokio::test]
async fn a_passage_can_stop_being_waited_for_and_its_answer_is_kept() {
    let s = setting().await;
    s.corrector.slow.store(true, Ordering::SeqCst);
    let engine = s.engine.clone();
    let ask = s.ask("p1");
    let asking = tokio::spawn(async move { engine.ask_passage(ask).await });
    // Waits until the call is under way, then cancels it.
    for _ in 0..200 {
        if s.calls() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    s.engine.cancel_passage("p1");
    let proposal = asking
        .await
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
    assert_eq!(
        s.store.passage("p1").map(|p| p.status).ok(),
        Some(ProposalStatus::Cancelled)
    );
    assert!(s.engine.accept_passage("p1", "x".into()).is_err());
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    // The answer that came after is stored: asked again, it costs nothing.
    let mut stored = false;
    for _ in 0..200 {
        stored = s
            .engine
            .passage_estimate(&s.ask("p2"))
            .is_ok_and(|e| e.cached);
        if stored {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(stored);
    s.corrector.slow.store(false, Ordering::SeqCst);
    let again = s
        .engine
        .ask_passage(s.ask("p2"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(again.cached);
    assert_eq!(s.calls(), 1);
}

#[tokio::test]
async fn a_passage_too_short_too_long_or_of_an_analysis_is_refused_before_it_is_sent() {
    let s = setting().await;
    let mut short = s.ask("p1");
    short.passage = "d,".into();
    assert!(matches!(
        s.engine.ask_passage(short).await,
        Err(EngineError::Passage(PassageError::TooShort))
    ));
    let mut long = s.ask("p2");
    long.passage = "a".repeat(kalamu_core::passage_correction::MAX_CHARS + 1);
    assert!(matches!(
        s.engine.ask_passage(long).await,
        Err(EngineError::Passage(PassageError::TooLong(_)))
    ));
    // A provider that cannot correct text is not asked.
    let mut other = s.ask("p3");
    other.provider = ProviderId::new("fake");
    other.model = "m".into();
    assert!(matches!(
        s.engine.ask_passage(other).await,
        Err(EngineError::UnsupportedModel(_))
    ));
    assert_eq!(s.calls(), 0);
}
