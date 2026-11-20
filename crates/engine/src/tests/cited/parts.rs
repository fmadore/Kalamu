//! A transcript too long for its model read in parts of whole turns, each part sent with the
//! whole text's turn numbers, then brought together by one more call that refers to the parts'
//! quotations by number: every quotation still the transcript's own words, a number not known
//! left out; a part that failed named, the others brought together, and the part read again
//! alone; a run stopped part-way repeating no part read; the cost of every part and of the step
//! after them said before Start; and a transcript that fits read whole under the key it always
//! had.

use kalamu_core::analysis::cited::windows;
use kalamu_core::{PieceRef, PieceStatus};

use super::*;

/// The pretend model reads 250 tokens, 1,000 characters by the catalogue's measure.
const TOKENS: u32 = 250;

/// Thirty paragraphs, each its own turn, each with words found nowhere else.
fn long_text() -> String {
    (1..=30)
        .map(|n| format!("Paragraphe numéro {n:02} : le port de Dakar et ses quais."))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// What every part answers: the same two quotations, of turns 3 and 25, whichever part it is,
/// both found in the whole text.
fn part_answer() -> serde_json::Value {
    serde_json::json!({
        "summary": "Une partie.",
        "themes": [{"name": "Le port", "description": "Les quais.", "origin": "The interview",
                    "quotes": [{"turn": 3, "words": "Paragraphe numéro 03"},
                               {"turn": 25, "words": "Paragraphe numéro 25"},
                               {"turn": 4, "words": "jamais dit"}]}],
        "guide_themes_not_found": [], "keywords": ["Dakar"],
        "quotations": [{"why_it_matters": "Le début.",
                        "quotes": [{"turn": 3, "words": "Paragraphe numéro 03"}]}],
    })
}

/// What brings the parts together: one theme backed by the two quotations, by number, with
/// words written in place of a number and a number past the last, both to be left out.
fn together_answer() -> serde_json::Value {
    serde_json::json!({
        "summary": "Tout l'entretien.",
        "themes": [{"name": "Le port de Dakar", "description": "Du début à la fin.", "origin": "The interview",
                    "quotes": ["Q2", "Q1", "Paragraphe numéro 99 inventé", "Q40"]}],
        "guide_themes_not_found": [], "keywords": ["Dakar", "port"],
        "quotations": [{"why_it_matters": "Le début.", "quotes": ["Q1"]}],
    })
}

/// The parts the job's set-up reads the text in, as the engine cuts it.
fn planned(f: &Interviews, source: &SourceFile) -> Vec<(u32, u32)> {
    let text = f
        .engine
        .quotable_text(source)
        .unwrap_or_else(|e| panic!("{e}"));
    windows::plan(&text.sizes(), &windows::TurnWindows::new(TOKENS * 4))
        .unwrap_or_else(|e| panic!("{e:?}"))
        .unwrap_or_else(|| panic!("the text fits"))
}

/// The answers that stand for the whole text: the step's, after the parts.
fn whole(f: &Interviews, job: &Job) -> crate::PieceView {
    f.engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_default()
        .into_iter()
        .find(|v| v.piece == PieceRef::Whole)
        .unwrap_or_else(|| panic!("no whole piece"))
}

#[tokio::test]
async fn a_transcript_too_long_is_read_in_parts_and_brought_together_by_number() {
    let f = interviews_reading(TOKENS);
    let source = f.text_file("long.txt", &long_text());
    let parts = planned(&f, &source);
    assert!(parts.len() >= 3, "{parts:?}");
    let mut script = vec![part_answer(); parts.len()];
    script.push(together_answer());
    f.model.script(&script);

    // Priced before Start: every part, then the step after them.
    let estimate = f.engine.estimate(&EstimateRequest {
        task: Task::Analysis,
        sources: vec![source.clone()],
        setups: vec![setup(
            "interviewer",
            "interviewer-1",
            Some("analysis.interview"),
            None,
        )],
        sample_pages: None,
    });
    assert_eq!(estimate.rows[0].too_long, None);
    assert_eq!(estimate.rows[0].parts, Some(parts.len() as u32));
    assert!(estimate.rows[1].merge);
    assert!(estimate.total_usd.is_some_and(|usd| usd > 0.0));

    let job = f
        .analysis(vec![source.clone()], Some("Le port"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), parts.len() + 1);

    // Each part was sent its own turns, numbered as in the whole text, and told it is a part.
    let sent = f.model.sent();
    let mut read: Vec<&(String, String)> = sent[..parts.len()].iter().collect();
    read.sort_by_key(|(text, _)| text.clone());
    for (first, last) in &parts {
        let opening = format!("[{first}] Paragraphe numéro {first:02}");
        let (text, request) = read
            .iter()
            .find(|(t, _)| t.starts_with(&opening))
            .unwrap_or_else(|| panic!("no part opens with turn {first}"));
        assert_eq!(text.lines().count() as u32, last - first + 1);
        assert!(text.ends_with(&format!(
            "[{last}] Paragraphe numéro {last:02} : le port de Dakar et ses quais."
        )));
        assert!(
            request.contains(&format!("turns {first} to {last}")),
            "{request}"
        );
        assert!(request.contains("Le port"), "{request}");
    }
    // The step after them was sent the parts' quotations, each once, by number.
    let (findings, request) = &sent[parts.len()];
    let findings: serde_json::Value =
        serde_json::from_str(findings).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        findings["parts"].as_array().map(Vec::len),
        Some(parts.len())
    );
    assert_eq!(
        findings["quotations"]["Q1"]["words"],
        "Paragraphe numéro 03"
    );
    assert_eq!(findings["quotations"]["Q2"]["turn"], 25);
    assert!(findings["quotations"]["Q3"].is_null(), "{findings}");
    assert!(
        request.contains(kalamu_prompts::CONSOLIDATION_REQUEST),
        "{request}"
    );
    assert!(request.contains("Le port"), "{request}");

    // The quotations are put back from their numbers: the transcript's own words and turns.
    let view = whole(&f, &job);
    let themes = cited(&view.ai_fields, "themes");
    assert_eq!(themes.entries.len(), 1);
    let quotes: Vec<(&str, u32)> = themes.entries[0]
        .evidence
        .iter()
        .map(|e| (e.quote.as_str(), e.turn))
        .collect();
    assert_eq!(
        quotes,
        [("Paragraphe numéro 25", 25), ("Paragraphe numéro 03", 3)]
    );
    let text = std::fs::read_to_string(&source.path).unwrap_or_default();
    for e in &themes.entries[0].evidence {
        let at: String = text
            .chars()
            .skip(e.anchor.start as usize)
            .take((e.anchor.end - e.anchor.start) as usize)
            .collect();
        assert_eq!(at, e.quote);
    }
    // Each part's "jamais dit" left out, and the step's two that are no quotation.
    assert_eq!(themes.left_out, parts.len() as u32 + 2);

    // The files write the whole answer, and the record names the parts read.
    let outputs = f.engine.outputs(job.id).unwrap_or_default();
    let txt = outputs
        .iter()
        .find(|p| p.ends_with(".txt"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    assert!(txt.contains("Tout l'entretien."), "{txt}");
    assert!(!txt.contains("Une partie."), "{txt}");
    let provenance = f.provenance(&job);
    let request = provenance["merge"]["request"].to_string();
    assert!(request.contains("turns_read"), "{provenance}");
    assert!(!request.contains("turns_not_read"), "{provenance}");
}

#[tokio::test]
async fn a_part_that_failed_is_named_and_read_again_alone() {
    let f = interviews_reading(TOKENS);
    let source = f.text_file("long.txt", &long_text());
    let parts = planned(&f, &source);
    // The last part fails; it holds turn 30 and no other part does.
    let (first, last) = *parts.last().unwrap_or_else(|| panic!("parts"));
    *f.model.fail.lock().unwrap_or_else(|p| p.into_inner()) = Some("numéro 30".into());
    let mut script = vec![part_answer(); parts.len() - 1];
    script.push(together_answer());
    f.model.script(&script);
    let job = f
        .analysis(vec![source.clone()], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);

    // The others were brought together, told which turns no part read.
    let calls = f.model.calls();
    let (findings, _) = f.model.sent().last().cloned().unwrap_or_default();
    assert!(
        findings.contains(&format!("\"{first}-{last}\"")) && findings.contains("turns_not_read"),
        "{findings}"
    );
    let view = whole(&f, &job);
    assert_eq!(view.status, PieceStatus::Done);
    assert!(!cited(&view.ai_fields, "themes").entries.is_empty());
    let failed: Vec<PieceRef> = f
        .engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_default()
        .into_iter()
        .filter(|v| v.status == PieceStatus::Failed)
        .map(|v| v.piece)
        .collect();
    assert_eq!(
        failed,
        [PieceRef::Turns {
            number: parts.len() as u32,
            first,
            last
        }]
    );
    // The files say which part was not read, and the record which turns.
    let txt = f
        .engine
        .outputs(job.id)
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.ends_with(".txt"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    assert!(
        txt.contains(&format!(
            "[Not read: part {} (turns {first}–{last})",
            parts.len()
        )),
        "{txt}"
    );
    assert!(
        f.provenance(&job)["merge"]["request"]
            .to_string()
            .contains("turns_not_read")
    );

    // Read again, the part alone is sent, and the parts brought together again.
    *f.model.fail.lock().unwrap_or_else(|p| p.into_inner()) = None;
    f.model.script(&[part_answer(), together_answer()]);
    let mut events = f.engine.subscribe();
    f.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    loop {
        match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { job_id, status })) if job_id == job.id => {
                assert_eq!(status, JobStatus::Completed);
                break;
            }
            Ok(Ok(_)) => {}
            other => panic!("no JobFinished event: {other:?}"),
        }
    }
    assert_eq!(f.model.calls(), calls + 2);
    let sent = f.model.sent();
    assert!(sent[calls].0.contains("numéro 30"));
    assert!(!sent[calls + 1].0.contains("turns_not_read"));
}

#[tokio::test]
async fn a_run_stopped_part_way_repeats_no_part_read() {
    let f = interviews_reading(TOKENS);
    let source = f.text_file("long.txt", &long_text());
    let parts = planned(&f, &source);
    let mut script = vec![part_answer(); parts.len()];
    script.push(together_answer());
    f.model.script(&script);
    // The part holding turn 30 is stopped the first time it is sent.
    *f.model.interrupt.lock().unwrap_or_else(|p| p.into_inner()) = Some("numéro 30".into());
    let job = f
        .analysis(vec![source], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Paused);
    assert_eq!(f.model.calls(), parts.len());
    assert_eq!(whole(&f, &job).status, PieceStatus::Pending);

    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    // The part stopped sent again, then the step after them: no part read is sent twice.
    assert_eq!(f.model.calls(), parts.len() + 2);
    let sent = f.model.sent();
    for (first, _) in &parts {
        let opening = format!("[{first}] ");
        let times = sent.iter().filter(|(t, _)| t.starts_with(&opening)).count();
        let expected = if *first == parts.last().map_or(0, |p| p.0) {
            2
        } else {
            1
        };
        assert_eq!(times, expected, "part from turn {first}");
    }
    assert_eq!(whole(&f, &job).status, PieceStatus::Done);
}

/// A transcript that fits is read whole, in one call, under the key it had before texts were
/// read in parts: the job's set-up now says how a long one would be read, which no key holds.
#[tokio::test]
async fn a_transcript_that_fits_is_read_whole_under_the_key_it_always_had() {
    let f = interviews_reading(TOKENS);
    let source = f.text_file(
        "court.txt",
        "Nous partions tôt\nau marché.\n\nLe soir, on comptait.\n",
    );
    f.model.script(&[serde_json::json!({
        "summary": "s", "themes": [], "guide_themes_not_found": [], "keywords": [],
        "quotations": [{"why_it_matters": "w",
                        "quotes": [{"turn": 1, "words": "Nous partions tôt au marché."}]}],
    })]);
    let job = f
        .analysis(vec![source.clone()], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(job.setups[0].windows.is_some() && job.setups[0].merge.is_some());
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 1);
    let pieces = f
        .engine
        .store
        .source_pieces(job.id, job.setups[0].id, job.sources[0].id)
        .unwrap_or_default();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].piece, PieceRef::Whole);
    // The key a job made before parts existed gives this piece: its set-up as it was then.
    let mut before = job.setups[0].clone();
    before.windows = None;
    before.merge = None;
    let text = f
        .engine
        .quotable_text(&source)
        .unwrap_or_else(|e| panic!("{e}"));
    let (content, at) = source.cache_identity(&PieceRef::Whole);
    let mut fixes = crate::output::preprocessing(&source);
    fixes.push(text.fix().to_string());
    let key = kalamu_core::CacheKey::compute(
        Task::Analysis,
        content,
        &at,
        &before,
        &job.settings.languages,
        &fixes,
    );
    assert_eq!(pieces[0].cache_key.as_deref(), Some(key.as_str()));
    // A record of a short interview names no step after its parts.
    assert!(f.provenance(&job)["merge"].is_null());
}
