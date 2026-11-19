//! An interview analysis whose fields cite the transcript: the frozen text sent with its turns
//! numbered, each quotation the model gives looked for in the text and kept with the turn's
//! speaker and times (never the model's), one not found left out and counted, an answer whose
//! every quotation is missing refused, charged and asked again; resumed without sending a piece
//! twice, cached for the same question and not for another; a plain text read in paragraphs, a
//! scan refused; and the researcher's corrections kept to quotations the model found.

use kalamu_core::Snapshot;

use super::*;

mod fixture;
use fixture::*;

#[tokio::test]
async fn quotations_are_anchored_in_the_frozen_transcript_with_its_speakers_and_times() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[answer()]);
    let job = f
        .analysis(vec![source.clone()], Some("Le travail des femmes au port"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 1);

    // The text went with its turns numbered, and the request with the question.
    let (read, request) = f.model.sent().remove(0);
    let lines: Vec<&str> = read.lines().collect();
    assert_eq!(
        lines,
        [
            "[1] [00:00:05] Interviewer: Comment avez-vous commencé au port ?",
            "[2] [00:00:12] Awa: J’ai commencé en 1998. Au début, on ne savait pas.",
            "[3] [00:01:10] Interviewer: Et ensuite ?",
            "[4] [00:01:15] Awa: Ensuite, «\u{a0}on ne savait pas\u{a0}» est devenu notre devise.",
        ]
    );
    assert!(
        request.contains("Le travail des femmes au port"),
        "{request}"
    );
    assert!(
        request.contains(kalamu_core::analysis::cited::REQUEST),
        "{request}"
    );

    let fields = f.fields(&job, 0);
    let themes = cited(&fields, "themes");
    assert_eq!(themes.entries.len(), 2);
    let debut = &themes.entries[0].evidence;
    assert_eq!(debut.len(), 1);
    // Found in turn 2, though the model named turn 1 and said the interviewer said it.
    assert_eq!(debut[0].turn, 2);
    assert_eq!(debut[0].speaker.as_deref(), Some("Awa"));
    assert_eq!(debut[0].start_ms, Some(12_000));
    assert_eq!(debut[0].end_ms, Some(70_000));
    // The transcript's own apostrophe and final mark.
    assert_eq!(debut[0].quote, "J’ai commencé en 1998.");
    assert_eq!(themes.left_out, 1);
    // The phrase said twice: where the model named it.
    let devise = &themes.entries[1].evidence[0];
    assert_eq!((devise.turn, devise.start_ms), (4, Some(75_000)));
    let snapshot: Snapshot = f
        .engine
        .store
        .snapshot(
            &source
                .derived_from
                .as_ref()
                .map(|d| d.snapshot_id.clone())
                .unwrap_or_default(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let text = std::fs::read_to_string(&source.path).unwrap_or_default();
    for evidence in themes.entries.iter().flat_map(|e| &e.evidence) {
        assert_eq!(
            kalamu_core::snapshot::check(&text, &snapshot.parts, &evidence.anchor, &evidence.quote),
            Ok(())
        );
    }

    // Provenance says how many were checked and left out, and the snapshot read.
    let provenance = f.provenance(&job);
    assert_eq!(
        provenance["cited"],
        serde_json::json!([
            {"part": 1, "field": "themes", "checked": 3, "left_out": 1},
            {"part": 1, "field": "quotations", "checked": 1, "left_out": 0},
        ])
    );
    assert_eq!(
        provenance["source"]["derived_from"]["snapshot_id"],
        serde_json::json!(snapshot.id)
    );

    // The review can show them in place.
    let read = f
        .engine
        .cited_source(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read.snapshot_id.as_deref(), Some(snapshot.id.as_str()));
    assert_eq!(read.turns, snapshot.turns);
    assert_eq!(read.text, text);
}

#[tokio::test]
async fn the_same_question_costs_nothing_twice_and_another_is_another_analysis() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[answer()]);
    let first = f
        .analysis(vec![source.clone()], Some("Le travail au port"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(first.id).await, JobStatus::Completed);
    let again = f
        .analysis(vec![source.clone()], Some("Le travail au port"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(again.id).await, JobStatus::Completed);
    assert_eq!(
        f.model.calls(),
        1,
        "the same question is read from the cache"
    );
    let pieces = f.engine.store.pieces(again.id).unwrap_or_default();
    assert_eq!(pieces[0].status, PieceStatus::Cached);
    assert_eq!(f.fields(&again, 0), f.fields(&first, 0));
    let other = f
        .analysis(vec![source], Some("La famille"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(other.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 2, "another question is asked");
}

#[tokio::test]
async fn an_answer_whose_every_quotation_is_missing_is_refused_charged_and_asked_again() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[invented(), answer()]);
    let job = f
        .analysis(vec![source], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 2);
    // The same request both times.
    let sent = f.model.sent();
    assert_eq!(sent[0], sent[1]);
    // The refused answer is charged to the piece.
    let charges = f.engine.store.charges(job.id).unwrap_or_default();
    assert_eq!(charges.len(), 1);
    assert_eq!(charges[0].usage.output_tokens, 300);
    assert_eq!(f.provenance(&job)["refused"]["answers"], 1);
    assert_eq!(cited(&f.fields(&job, 0), "themes").entries.len(), 2);

    // An answer that never finds a quotation fails the piece with the reason.
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[invented()]);
    let job = f
        .analysis(vec![source], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    assert_eq!(f.model.calls(), 3);
    let pieces = f.engine.store.pieces(job.id).unwrap_or_default();
    assert!(
        pieces[0]
            .error
            .as_deref()
            .is_some_and(|e| e.contains("could be found in the text")),
        "{:?}",
        pieces[0].error
    );
}

#[tokio::test]
async fn a_job_stopped_part_way_resumes_without_sending_a_piece_twice() {
    let f = interviews();
    let first = f.text_file("premier.txt", "J'ai commencé en 1998.\n\nAu port.");
    let second = f.text_file("second.txt", "Au début, on ne savait pas.\n\nENSUITE.");
    f.model.script(&[serde_json::json!({
        "summary": "s",
        "themes": [{"name": "n", "description": "d",
                    "quotes": [{"turn": 1, "words": "en 1998"}, {"turn": 1, "words": "on ne savait pas"}]}],
        "keywords": [],
        "quotations": [],
    })]);
    *f.model.interrupt.lock().unwrap_or_else(|p| p.into_inner()) = Some("ENSUITE".into());
    let job = f
        .analysis(vec![first, second], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Paused);
    assert_eq!(f.model.calls(), 2);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 3, "only the piece stopped is sent again");
    // Each text's own quotation found, the other's left out.
    let (one, two) = (
        cited(&f.fields(&job, 0), "themes"),
        cited(&f.fields(&job, 1), "themes"),
    );
    assert_eq!(one.entries[0].evidence[0].quote, "en 1998");
    assert_eq!(two.entries[0].evidence[0].quote, "on ne savait pas");
    assert_eq!((one.left_out, two.left_out), (1, 1));
}

#[tokio::test]
async fn a_plain_text_is_read_in_paragraphs_and_a_scan_is_refused() {
    let f = interviews();
    let notes = f.text_file(
        "notes.txt",
        "Nous partions tôt\nau marché.\n\nLe soir, on comptait.\n",
    );
    f.model.script(&[serde_json::json!({
        "summary": "s",
        "themes": [],
        "keywords": [],
        "quotations": [{"why_it_matters": "w",
                        "quotes": [{"turn": 1, "words": "Nous partions tôt au marché."}]}],
    })]);
    let job = f
        .analysis(vec![notes], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        f.model.sent()[0].0,
        "[1] Nous partions tôt au marché.\n[2] Le soir, on comptait."
    );
    let quotations = cited(&f.fields(&job, 0), "quotations");
    let evidence = &quotations.entries[0].evidence[0];
    assert_eq!(evidence.quote, "Nous partions tôt\nau marché.");
    assert_eq!(
        (evidence.turn, &evidence.speaker, evidence.start_ms),
        (1, &None, None)
    );
    let read = f
        .engine
        .cited_source(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!((read.snapshot_id, read.turns.len()), (None, 2));

    // A scan has no words to quote: refused before anything is sent.
    let pdf = {
        let path = f.dir.path().join("scan.pdf");
        std::fs::write(&path, kalamu_media::sample_pdf(&[(595.0, 842.0)], false))
            .unwrap_or_else(|e| panic!("{e}"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"))
    };
    let refused = f.analysis(vec![pdf], None);
    assert!(
        matches!(&refused, Err(EngineError::CitedNeedsText(name)) if name == "scan.pdf"),
        "{refused:?}"
    );
    assert_eq!(
        refused.err().map(|e| e.code()),
        Some("job.cited_needs_text")
    );
    assert_eq!(f.model.calls(), 1);
}

mod research;
mod review;
