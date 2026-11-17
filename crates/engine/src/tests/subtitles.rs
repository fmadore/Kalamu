//! A transcript translated line by line (#167): its lines frozen with its text, sent as
//! numbered units without times or speakers, answered and checked by number, and written as
//! subtitles in the language translated into with the times and speakers of the lines they
//! translate; resumed without sending a passage again; refused, never shifted, when an answer or
//! a correction does not keep the lines.

use kalamu_core::{
    Correction, JobSettings, LanguageChoice, LanguageCode, PieceRef, Segment, aligned,
};

use super::*;

/// Translates whatever it is sent: numbered lines into the same numbers, each prefixed `EN:`,
/// or a passage of text as one field. Can be told to leave out a line for some answers.
struct Translator {
    calls: AtomicUsize,
    /// What each call read, and the request that came with it.
    sent: Mutex<Vec<(String, String)>>,
    /// Answers still to give with the last line left out.
    broken: AtomicUsize,
}

impl Translator {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            sent: Mutex::new(Vec::new()),
            broken: AtomicUsize::new(0),
        })
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn sent(&self) -> Vec<(String, String)> {
        self.sent.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

fn answer(key: &str, value: FieldValue) -> FieldAnswer {
    FieldAnswer {
        key: key.into(),
        value,
        reason: None,
    }
}

#[async_trait]
impl Provider for Translator {
    fn id(&self) -> ProviderId {
        ProviderId::new("translator")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["text/plain"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let read = String::from_utf8_lossy(&request.input.bytes).into_owned();
        self.sent
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push((read.clone(), request.user_prompt.to_string()));
        let schema = request
            .analysis
            .unwrap_or_else(|| panic!("a translation answers in fields"));
        let fields = if schema.fields[0].key == aligned::LINES {
            let mut entries: Vec<Vec<FieldAnswer>> = read
                .lines()
                .filter_map(|line| {
                    let (number, said) = line.strip_prefix('[')?.split_once("] ")?;
                    Some(vec![
                        answer(aligned::ID, FieldValue::Text(number.into())),
                        answer(aligned::TEXT, FieldValue::Text(format!("EN: {said}"))),
                    ])
                })
                .collect();
            let broken = self
                .broken
                .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
                .is_ok();
            if broken {
                entries.pop();
            }
            vec![answer(aligned::LINES, FieldValue::Entries(entries))]
        } else {
            vec![answer(
                "text",
                FieldValue::Text(format!("EN: {}", read.trim())),
            )]
        };
        Ok(ProcessOutput {
            text: schema.render(&fields),
            fields,
            usage: Usage {
                input_tokens: 400,
                output_tokens: 80,
                ..Usage::default()
            },
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

struct Subtitles {
    engine: Arc<Engine>,
    translator: Arc<Translator>,
    dir: tempfile::TempDir,
}

fn subtitles() -> Subtitles {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let translator = Translator::new();
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store,
        vec![Fake::new(), translator.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 2,
            max_attempts: 3,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    Subtitles {
        engine,
        translator,
        dir,
    }
}

impl Subtitles {
    fn folder(&self) -> OutputDestination {
        OutputDestination::Folder(self.dir.path().display().to_string())
    }

    /// A video transcribed with times in its text: `[00:00:05] Speaker 1: Bonjour.` and
    /// `[00:01:10] Speaker 2: Merci.`
    async fn transcript(&self) -> Job {
        let job = self
            .engine
            .create_job(NewJob {
                name: "Video".into(),
                task: Task::AudioTranscription,
                sources: self
                    .engine
                    .inspect_links(&["https://youtu.be/dQw4w9WgXcQ"])
                    .into_iter()
                    .flatten()
                    .collect(),
                setups: vec![SetupRequest {
                    provider: ProviderId::new("fake"),
                    model: "fake-model".into(),
                    preset_id: None,
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings: Default::default(),
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt],
                    destination: self.folder(),
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(self.run(job.id).await, JobStatus::Completed);
        job
    }

    fn translation(&self, sources: Vec<SourceFile>) -> Job {
        self.engine
            .create_job(NewJob {
                name: "Translation".into(),
                task: Task::Translation,
                sources,
                setups: vec![SetupRequest {
                    provider: ProviderId::new("translator"),
                    model: "translator-1".into(),
                    preset_id: Some("translation.research".into()),
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings: JobSettings {
                    languages: LanguageChoice::Explicit(vec![LanguageCode::new("en")]),
                    ..Default::default()
                },
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt, ExportFormat::Srt, ExportFormat::Vtt],
                    destination: self.folder(),
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"))
    }

    async fn run(&self, id: JobId) -> JobStatus {
        let mut events = self.engine.subscribe();
        self.engine
            .start(id)
            .unwrap_or_else(|e| panic!("start: {e}"));
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

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(name)).unwrap_or_default()
    }

    fn correct(&self, job: &Job, index: u32, text: &str) {
        self.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: job.sources[0].id,
                index,
                text: Some(text.into()),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn document(&self, job: &Job) -> kalamu_core::ExportDocument {
        let job = self
            .engine
            .store
            .job(job.id)
            .unwrap_or_else(|e| panic!("{e}"));
        self.engine
            .export_document(&job, job.setups[0].id, job.sources[0].id)
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

fn subtitle(format: ExportFormat, doc: &kalamu_core::ExportDocument) -> Result<String, String> {
    kalamu_export::exporter(format)
        .ok_or_else(|| "no exporter".to_string())?
        .export(doc)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn a_translated_transcript_is_written_as_subtitles_with_the_times_of_its_lines() {
    let t = subtitles();
    let transcript = t.transcript().await;
    let frozen = t
        .engine
        .snapshot(transcript.id, transcript.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let from = frozen[0]
        .derived_from
        .clone()
        .unwrap_or_else(|| panic!("lineage"));
    assert!(from.timed, "a transcript with times keeps its lines");
    let lines = t
        .engine
        .store
        .snapshot(&from.snapshot_id)
        .unwrap_or_else(|e| panic!("{e}"))
        .lines;
    assert_eq!(
        lines
            .iter()
            .map(|l| (l.start_ms, l.speaker.as_deref()))
            .collect::<Vec<_>>(),
        vec![(5_000, Some("Speaker 1")), (70_000, Some("Speaker 2"))]
    );

    let job = t.translation(frozen);
    assert_eq!(
        job.output.formats,
        vec![ExportFormat::Txt, ExportFormat::Srt, ExportFormat::Vtt]
    );
    assert_eq!(t.run(job.id).await, JobStatus::Completed);

    // One passage, its lines numbered: neither times nor speakers reach the model.
    let sent = t.translator.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, "[1] Bonjour.\n[2] Merci.");
    assert!(sent[0].1.ends_with(aligned::REQUEST), "{}", sent[0].1);

    let srt = t.read("youtube-dQw4w9WgXcQ.translated.en.srt");
    assert!(
        srt.starts_with("1\n00:00:05,000 --> 00:00:11,000\nSpeaker 1: EN: Bonjour.\n"),
        "{srt}"
    );
    assert!(srt.contains("00:01:10,000 --> "), "{srt}");
    assert!(srt.contains("Speaker 2: EN: Merci."), "{srt}");
    let vtt = t.read("youtube-dQw4w9WgXcQ.translated.en.vtt");
    assert!(vtt.contains("<v Speaker 2>EN: Merci."), "{vtt}");
    let txt = t.read("youtube-dQw4w9WgXcQ.translated.en.txt");
    assert!(txt.contains("[00:01:10] Speaker 2: EN: Merci."), "{txt}");

    // The review shows the lines with their times beside their translations, one paragraph
    // each.
    let views = t
        .engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        views[0].source_text.as_deref(),
        Some("[00:00:05] Speaker 1: Bonjour.\n\n[00:01:10] Speaker 2: Merci.")
    );
    assert_eq!(
        views[0].ai_text.as_deref(),
        Some("EN: Bonjour.\n\nEN: Merci.")
    );

    // A correction that keeps one paragraph per line keeps the times, with its words.
    t.correct(&job, 0, "Good morning.\n\nThank you.");
    let doc = t.document(&job);
    assert!(doc.provenance.lines_changed.is_empty());
    let srt = subtitle(ExportFormat::Srt, &doc).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        srt.contains("00:01:10,000 --> 00:01:14,000\nSpeaker 2: Thank you."),
        "{srt}"
    );

    // One that joins the two lines loses them: subtitles are refused with the reason, and
    // provenance says which passage; its text is written as corrected.
    t.correct(&job, 0, "Good morning, and thank you.");
    let doc = t.document(&job);
    assert_eq!(doc.provenance.lines_changed, vec![1]);
    assert!(doc.parts[0].segments.is_empty());
    let refused = subtitle(ExportFormat::Vtt, &doc).err().unwrap_or_default();
    assert!(
        refused.contains("passage 1 no longer has one paragraph per line"),
        "{refused}"
    );
    let txt = subtitle(ExportFormat::Txt, &doc).unwrap_or_else(|e| panic!("{e}"));
    assert!(txt.contains("Good morning, and thank you."), "{txt}");

    // The translation's own text, frozen, keeps the lines for a next language.
    t.correct(&job, 0, "Good morning.\n\nThank you.");
    let next = t
        .engine
        .snapshot(job.id, job.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(next[0].derived_from.as_ref().is_some_and(|d| d.timed));
}

#[tokio::test]
async fn a_text_without_times_is_translated_as_before_and_offers_no_subtitles() {
    let t = subtitles();
    let path = t.dir.path().join("lettre.txt");
    std::fs::write(&path, "[00:00:05] Speaker 1: Bonjour.\n").unwrap_or_else(|e| panic!("{e}"));
    let text = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let job = t.translation(vec![text]);
    assert_eq!(
        job.output.formats,
        vec![ExportFormat::Txt],
        "no times, no subtitles"
    );
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    let sent = t.translator.sent();
    assert_eq!(
        sent[0].0.trim(),
        "[00:00:05] Speaker 1: Bonjour.",
        "sent as it was"
    );
    assert!(!sent[0].1.contains(aligned::REQUEST));
    assert!(!t.dir.path().join("lettre.translated.en.srt").exists());
    let views = t
        .engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(views[0].segments.is_empty());
}

#[tokio::test]
async fn an_answer_that_loses_a_line_is_asked_again_and_a_passage_that_never_keeps_them_fails() {
    let t = subtitles();
    let transcript = t.transcript().await;
    let frozen = t
        .engine
        .snapshot(transcript.id, transcript.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));

    // Once: asked again, and the second answer is the one kept.
    t.translator.broken.store(1, Ordering::SeqCst);
    let job = t.translation(frozen.clone());
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    assert_eq!(t.translator.calls(), 2);
    let doc = t.document(&job);
    assert_eq!(doc.parts[0].segments.len(), 2);
    // The answer refused was billed: its usage counts in the job's cost.
    assert_eq!(
        t.engine
            .store
            .source_charges(job.id, job.setups[0].id, job.sources[0].id)
            .unwrap_or_default()
            .len(),
        1
    );

    // Every time: the passage fails with the reason, and no subtitle has a shifted time.
    t.translator.broken.store(10, Ordering::SeqCst);
    let mut other = frozen;
    other[0].id = SourceId::new();
    let job = t
        .engine
        .create_job(NewJob {
            name: "Translation".into(),
            task: Task::Translation,
            sources: other,
            setups: vec![SetupRequest {
                provider: ProviderId::new("translator"),
                model: "translator-2".into(),
                preset_id: Some("translation.research".into()),
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: JobSettings {
                languages: LanguageChoice::Explicit(vec![LanguageCode::new("en")]),
                ..Default::default()
            },
            output: OutputSettings {
                formats: vec![ExportFormat::Srt],
                destination: t.folder(),
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(t.run(job.id).await, JobStatus::CompletedWithErrors);
    let pieces = t.engine.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces[0].status, PieceStatus::Failed);
    let error = pieces[0].error.clone().unwrap_or_default();
    assert!(error.contains("did not give back its lines"), "{error}");
    assert!(error.contains("1 of 2 lines"), "{error}");
}

#[tokio::test]
async fn a_stopped_translation_of_a_transcript_resumes_without_sending_a_passage_again() {
    let t = subtitles();
    let transcript = t.transcript().await;
    // Corrected into a long interview, times kept in the text: two passages of lines.
    let long: Vec<String> = (0..150u64)
        .map(|i| {
            format!(
                "[{}] Speaker {}: Ligne {i}.",
                kalamu_core::result::clock(i * 4_000),
                i % 2 + 1
            )
        })
        .collect();
    t.correct(&transcript, 0, &long.join("\n"));
    let frozen = t
        .engine
        .snapshot(transcript.id, transcript.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(frozen[0].derived_from.as_ref().is_some_and(|d| d.timed));
    let job = t.translation(frozen);
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    assert_eq!(t.translator.calls(), 2, "two passages");
    let pieces = t.engine.store.pieces(job.id).unwrap_or_default();
    assert_eq!(
        pieces.iter().map(|p| p.piece.clone()).collect::<Vec<_>>(),
        vec![
            PieceRef::Passage { number: 1 },
            PieceRef::Passage { number: 2 }
        ]
    );
    assert!(t.translator.sent()[1].0.starts_with("[121] Ligne 120."));

    // Crashed with the second passage in flight: resuming sends nothing again.
    let mut piece = pieces[1].clone();
    piece.status = PieceStatus::Running;
    t.engine
        .store
        .upsert_piece(&piece)
        .unwrap_or_else(|e| panic!("{e}"));
    t.engine
        .store
        .set_job_status(job.id, JobStatus::Running)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(t.engine.recover().unwrap_or_default(), vec![job.id]);
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        t.translator.calls(),
        2,
        "the passage in flight came from the cache"
    );

    let srt = t.read("youtube-dQw4w9WgXcQ.translated.en.srt");
    assert!(srt.contains("150\n00:09:56,000 --> "), "{srt}");
    assert!(srt.contains("Speaker 2: EN: Ligne 149."), "{srt}");
    let lines: Vec<Segment> = t
        .document(&job)
        .parts
        .into_iter()
        .flat_map(|p| p.segments)
        .collect();
    assert_eq!(lines.len(), 150);
    assert!(lines.windows(2).all(|w| w[0].start_ms < w[1].start_ms));
}
