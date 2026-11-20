//! Documents read against a research question with the default screening prompt (#51), over
//! text read from scanned pages and corrected: each page's paragraphs numbered for the model,
//! the question in the request, a passage found word for word kept with its page and no speaker,
//! one found nowhere left out; a document that does not speak of the question kept with no
//! passage; an answer whose every passage is invented refused and asked again. The prompt is
//! the shipped one, run by the same engine as the interview's: nothing knows it by name.

use kalamu_core::PieceRef;

use super::*;

const QUESTION: &str = "Les commerçantes du marché et la municipalité";

/// A scan of two pages read by the fake model, each page corrected by the researcher, frozen
/// as *Use this text* freezes it.
async fn read_pages(f: &Interviews) -> SourceFile {
    let path = f.dir.path().join("conseil-1957.pdf");
    std::fs::write(
        &path,
        kalamu_media::sample_pdf(&[(595.0, 842.0), (595.0, 842.0)], false),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let scan = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let job = f
        .engine
        .create_job(NewJob {
            name: "Conseil".into(),
            task: Task::TextExtraction,
            sources: vec![scan],
            setups: vec![setup("fake", "fake-model", None, None)],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt],
                destination: f.folder(),
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let pages = [
        "Le conseil municipal s’est réuni le 3 mars.\n\nIl a voté le budget du marché.",
        "Les commerçantes du marché demandent un abri contre la pluie.",
    ];
    for (index, text) in pages.iter().enumerate() {
        f.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: job.sources[0].id,
                index: index as u32,
                text: Some((*text).into()),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
    let mut frozen = f
        .engine
        .snapshot(job.id, job.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    frozen.remove(0)
}

fn screening(f: &Interviews, sources: Vec<SourceFile>) -> Job {
    f.engine
        .create_job(NewJob {
            name: "Tri".into(),
            task: Task::Analysis,
            sources,
            setups: vec![setup(
                "interviewer",
                "interviewer-1",
                Some("analysis.screening"),
                Some(QUESTION),
            )],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt, ExportFormat::Csv],
                destination: f.folder(),
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"))
}

fn relevance(fields: &[FieldAnswer]) -> String {
    fields
        .iter()
        .find(|a| a.key == "relevance")
        .map(|a| a.value.joined())
        .unwrap_or_default()
}

#[tokio::test]
async fn a_passage_of_scanned_pages_is_kept_with_its_page_and_one_found_nowhere_is_left_out() {
    let f = interviews();
    let pages = read_pages(&f).await;
    f.model.script(&[serde_json::json!({
        "relevance": "Relevant",
        "reason": "Les commerçantes demandent un abri à la municipalité.",
        "passages": [
            {"quotes": [{"turn": 3, "words": "Les commerçantes du marché demandent un abri contre la pluie."}]},
            {"quotes": [{"turn": 1, "words": "Les femmes du marché ont manifesté."}]},
        ],
    })]);
    let job = screening(&f, vec![pages]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 1);

    // Each page's paragraphs numbered in turn, the page markers left out; the question asked.
    let (read, request) = f.model.sent().remove(0);
    assert_eq!(
        read.lines().collect::<Vec<_>>(),
        [
            "[1] Le conseil municipal s’est réuni le 3 mars.",
            "[2] Il a voté le budget du marché.",
            "[3] Les commerçantes du marché demandent un abri contre la pluie.",
        ]
    );
    assert!(request.contains(QUESTION), "{request}");
    assert!(
        request.contains("speaks of the researcher's question"),
        "{request}"
    );

    let fields = f.fields(&job, 0);
    assert_eq!(relevance(&fields), "Relevant");
    let passages = cited(&fields, "passages");
    assert_eq!(
        (passages.entries.len(), passages.left_out, passages.dropped),
        (1, 1, 1)
    );
    let evidence = &passages.entries[0].evidence[0];
    assert_eq!(evidence.page, Some(PieceRef::page(2)));
    assert_eq!(
        (evidence.turn, &evidence.speaker, evidence.start_ms),
        (3, &None, None)
    );
    assert_eq!(
        evidence.quote,
        "Les commerçantes du marché demandent un abri contre la pluie."
    );
}

#[tokio::test]
async fn a_document_that_does_not_speak_of_the_question_is_kept_with_no_passage() {
    let f = interviews();
    let pages = read_pages(&f).await;
    f.model.script(&[serde_json::json!({
        "relevance": "Not relevant",
        "reason": "Le procès-verbal ne parle que du budget.",
        "passages": [],
    })]);
    let job = screening(&f, vec![pages]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 1, "no passage is not a broken answer");
    let fields = f.fields(&job, 0);
    assert_eq!(relevance(&fields), "Not relevant");
    assert_eq!(cited(&fields, "passages"), Default::default());
}

#[tokio::test]
async fn an_answer_whose_every_passage_is_invented_is_refused_and_asked_again() {
    let f = interviews();
    let pages = read_pages(&f).await;
    f.model.script(&[
        serde_json::json!({
            "relevance": "Relevant",
            "reason": "r",
            "passages": [{"quotes": [{"turn": 2, "words": "Les femmes du marché ont manifesté."}]}],
        }),
        serde_json::json!({
            "relevance": "Partly relevant",
            "reason": "Le marché est nommé, pas les commerçantes.",
            "passages": [{"quotes": [{"turn": 2, "words": "Il a voté le budget du marché."}]}],
        }),
    ]);
    let job = screening(&f, vec![pages]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 2);
    assert_eq!(f.provenance(&job)["refused"]["answers"], 1);
    let fields = f.fields(&job, 0);
    assert_eq!(relevance(&fields), "Partly relevant");
    let passages = cited(&fields, "passages");
    assert_eq!(
        passages.entries[0].evidence[0].page,
        Some(PieceRef::page(1))
    );
}
