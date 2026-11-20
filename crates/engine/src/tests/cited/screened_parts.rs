//! A long document read from scanned pages, screened against a question by a model that cannot
//! read it at once (#51): read in parts of whole passages as a long interview is, each part told
//! of passages, not of words said; brought together by the step whose instructions say how a
//! choice is answered for the whole text and which receives each quotation with its page; the
//! passages of the whole answer still on their pages; the cost of the parts said before Start;
//! and a part not read named by its pages.

use kalamu_core::analysis::cited::{self as cite, windows};
use kalamu_core::{PieceRef, PieceStatus};

use super::*;

/// The pretend model reads 250 tokens, 1,000 characters by the catalogue's measure.
const TOKENS: u32 = 250;
const QUESTION: &str = "Les commerçantes du marché et la municipalité";

/// Four pages of eight paragraphs, each with words found nowhere else; the market women are on
/// page 3 only.
fn pages() -> Vec<String> {
    (1..=4)
        .map(|page| {
            (1..=8)
                .map(|n| {
                    if page == 3 && n == 4 {
                        "Les commerçantes du marché demandent un abri à la mairie.".to_string()
                    } else {
                        format!("Page {page}, alinéa {n} : le conseil parle du budget.")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .collect()
}

/// A scan of four pages read by the fake model, each page corrected by the researcher, frozen
/// as *Use this text* freezes it.
async fn read_pages(f: &Interviews) -> SourceFile {
    let path = f.dir.path().join("conseil-1958.pdf");
    std::fs::write(&path, kalamu_media::sample_pdf(&[(595.0, 842.0); 4], false))
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
    for (index, text) in pages().into_iter().enumerate() {
        f.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: job.sources[0].id,
                index: index as u32,
                text: Some(text),
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

fn screening_setup() -> SetupRequest {
    setup(
        "interviewer",
        "interviewer-1",
        Some("analysis.screening"),
        Some(QUESTION),
    )
}

fn screening(f: &Interviews, source: SourceFile) -> Job {
    f.engine
        .create_job(NewJob {
            name: "Tri".into(),
            task: Task::Analysis,
            sources: vec![source],
            setups: vec![screening_setup()],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt, ExportFormat::Csv],
                destination: f.folder(),
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"))
}

/// The parts the screening reads the text in, as the engine cuts it.
fn planned(f: &Interviews, source: &SourceFile) -> Vec<(u32, u32)> {
    let text = f
        .engine
        .quotable_text(source)
        .unwrap_or_else(|e| panic!("{e}"));
    windows::plan(&text.sizes(), &windows::TurnWindows::new(TOKENS * 4))
        .unwrap_or_else(|e| panic!("{e:?}"))
        .unwrap_or_else(|| panic!("the text fits"))
}

/// What every part answers: partly relevant, quoting the market women, found on page 3 of the
/// whole text whichever part says it.
fn part_answer() -> serde_json::Value {
    serde_json::json!({
        "relevance": "Partly relevant",
        "reason": "Une partie.",
        "passages": [{"quotes": [{"turn": 20,
            "words": "Les commerçantes du marché demandent un abri à la mairie."}]}],
    })
}

fn together_answer() -> serde_json::Value {
    serde_json::json!({
        "relevance": "Relevant",
        "reason": "Les commerçantes demandent un abri à la mairie.",
        "passages": [{"quotes": ["Q1"]}],
    })
}

fn whole(f: &Interviews, job: &Job) -> crate::PieceView {
    f.engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_default()
        .into_iter()
        .find(|v| v.piece == PieceRef::Whole)
        .unwrap_or_else(|| panic!("no whole piece"))
}

#[tokio::test]
async fn a_long_document_is_screened_in_parts_and_its_passages_keep_their_pages() {
    let f = interviews_reading(TOKENS);
    let source = read_pages(&f).await;
    let parts = planned(&f, &source);
    assert!(parts.len() >= 2, "{parts:?}");
    let mut script = vec![part_answer(); parts.len()];
    script.push(together_answer());
    f.model.script(&script);

    // Priced before Start: every part, then the step after them; nothing too long.
    let estimate = f.engine.estimate(&EstimateRequest {
        task: Task::Analysis,
        sources: vec![source.clone()],
        setups: vec![screening_setup()],
        sample_pages: None,
    });
    assert!(estimate.rows[0].too_long.is_empty());
    assert_eq!(estimate.rows[0].parts, Some(parts.len() as u32));
    assert!(estimate.rows[1].merge);
    assert!(estimate.total_usd.is_some_and(|usd| usd > 0.0));

    let job = screening(&f, source);
    // The step's own instructions speak of passages as well as turns, and the screening
    // prompt says how the parts' relevance is brought together.
    let step = &job.setups[0]
        .merge
        .as_ref()
        .unwrap_or_else(|| panic!("a step after the parts"))
        .prompt
        .text;
    assert!(step.contains("a passage in a document"), "{step}");
    assert!(
        step.contains("**Relevant** if any part read is Relevant"),
        "{step}"
    );
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), parts.len() + 1);

    // Each part was told of passages, never of words said, with the question.
    let sent = f.model.sent();
    for (_, request) in &sent[..parts.len()] {
        assert!(request.contains(cite::REQUEST_PASSAGES), "{request}");
        assert!(!request.contains("what was said"), "{request}");
        assert!(request.contains(QUESTION), "{request}");
    }
    for (first, last) in &parts {
        assert!(
            sent[..parts.len()]
                .iter()
                .any(|(_, r)| r.contains(&format!("passages {first} to {last}"))),
            "no part of passages {first} to {last}"
        );
    }
    // The step received the quotation once, with its page.
    let (findings, request) = &sent[parts.len()];
    let findings: serde_json::Value =
        serde_json::from_str(findings).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(findings["quotations"]["Q1"]["page"], "3");
    assert!(findings["quotations"]["Q2"].is_null(), "{findings}");
    assert!(findings["quotations"]["Q1"]["speaker"].is_null());
    assert!(request.contains(QUESTION), "{request}");

    // The whole document's answer: relevant, its passage still the text's, on page 3.
    let view = whole(&f, &job);
    let relevance = view
        .ai_fields
        .iter()
        .find(|a| a.key == "relevance")
        .map(|a| a.value.joined());
    assert_eq!(relevance.as_deref(), Some("Relevant"));
    let passages = cited(&view.ai_fields, "passages");
    assert_eq!(passages.entries.len(), 1);
    let evidence = &passages.entries[0].evidence[0];
    assert_eq!(evidence.page, Some(PieceRef::page(3)));
    assert_eq!(
        evidence.quote,
        "Les commerçantes du marché demandent un abri à la mairie."
    );
    // The files write the whole answer only.
    let txt = f
        .engine
        .outputs(job.id)
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.ends_with(".txt") && !p.ends_with("conseil-1958.txt"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    assert!(txt.contains("Les commerçantes demandent un abri"), "{txt}");
    assert!(!txt.contains("Une partie."), "{txt}");
}

#[tokio::test]
async fn a_part_of_a_document_not_read_is_named_by_its_pages() {
    let f = interviews_reading(TOKENS);
    let source = read_pages(&f).await;
    let parts = planned(&f, &source);
    let (first, last) = *parts.last().unwrap_or_else(|| panic!("parts"));
    // The last part fails: it alone holds the last paragraph of page 4.
    *f.model.fail.lock().unwrap_or_else(|p| p.into_inner()) = Some("Page 4, alinéa 8".into());
    let mut script = vec![part_answer(); parts.len() - 1];
    script.push(together_answer());
    f.model.script(&script);
    let job = screening(&f, source.clone());
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    assert_eq!(whole(&f, &job).status, PieceStatus::Done);

    // The page each passage lies on, as the review names a part not read.
    let read = f
        .engine
        .cited_source(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read.pages.len(), read.turns.len());
    assert_eq!(read.pages.first(), Some(&Some(1)));
    assert_eq!(read.pages.last(), Some(&Some(4)));
    let (from, to) = (
        read.pages[first as usize - 1].unwrap_or_default(),
        read.pages[last as usize - 1].unwrap_or_default(),
    );
    let named = if from == to {
        format!("(page {from})")
    } else {
        format!("(pages {from}–{to})")
    };
    let txt = f
        .engine
        .outputs(job.id)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.ends_with(".txt"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<String>();
    assert!(
        txt.contains(&format!("[Not read: part {} {named}", parts.len())),
        "{txt}"
    );
}
