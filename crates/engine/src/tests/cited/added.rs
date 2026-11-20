//! A quotation the researcher adds from a passage selected in the frozen transcript: found in
//! the text, its speaker and time the turn's, kept in their corrections with the model's answer
//! unchanged, put back as it stood (the toast's Undo) and refused once altered, and written in
//! the quotations table as theirs.

use kalamu_core::analysis::cited::CitedEdit;

use super::*;

/// Where `words` stand in the text, in characters.
fn span(text: &str, words: &str) -> (u32, u32) {
    let at = text.find(words).unwrap_or_else(|| panic!("{words}"));
    let start = text[..at].chars().count() as u32;
    (start, start + words.chars().count() as u32)
}

#[tokio::test]
async fn a_passage_selected_in_the_transcript_is_added_as_the_researchers_quotation() {
    let f = interviews();
    let frozen = f.frozen().await;
    f.model.script(&[answer()]);
    let mut job = f
        .analysis(vec![frozen.clone()], None)
        .unwrap_or_else(|e| panic!("{e}"));
    job.output.formats = vec![ExportFormat::Csv];
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let (setup, source) = (job.setups[0].id, job.sources[0].id);
    let text = f
        .engine
        .cited_source(job.id, source)
        .unwrap_or_else(|e| panic!("{e}"))
        .text;
    let (start, end) = span(&text, "Et ensuite ?");
    let before = f
        .engine
        .store
        .source_corrections(job.id, setup, source)
        .unwrap_or_default();
    f.engine
        .edit_cited(
            job.id,
            setup,
            source,
            0,
            "themes",
            &CitedEdit::AddQuotation {
                entry: 1,
                start,
                end,
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let corrected = f.fields(&job, 0);
    let added = cited(&corrected, "themes").entries[1]
        .evidence
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("no quotation added"));
    assert!(added.added);
    assert_eq!(added.quote, "Et ensuite ?");
    // The interviewer's turn, and its time, from the transcript.
    assert_eq!(added.turn, 3);
    assert_eq!(added.speaker.as_deref(), Some("Interviewer"));
    assert_eq!(added.start_ms, Some(70_000));
    // The model's answer is unchanged.
    let model = f.engine.source_pieces(job.id, source).unwrap_or_default()[0]
        .ai_fields
        .clone();
    assert!(
        cited(&model, "themes")
            .entries
            .iter()
            .flat_map(|e| &e.evidence)
            .all(|e| !e.added)
    );

    // Written as theirs in the quotations table.
    let path = f
        .engine
        .write_outputs(&job, setup, source)
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.ends_with(".quotations.csv"))
        .unwrap_or_default();
    let rows = std::fs::read_to_string(path).unwrap_or_default();
    assert!(rows.contains("Et ensuite ?,3,researcher"), "{rows}");

    // Saved again as it stands (a toast's Undo puts a correction back so), it is kept; altered,
    // or moved across two turns, it is refused.
    let now = f
        .engine
        .store
        .source_corrections(job.id, setup, source)
        .unwrap_or_default();
    f.engine
        .save_correction(now[0].clone())
        .unwrap_or_else(|e| panic!("{e}"));
    let mut altered = now[0].clone();
    if let Some(fields) = altered.fields.as_mut()
        && let Some(FieldValue::Cited(value)) = fields
            .iter_mut()
            .find(|a| a.key == "themes")
            .map(|a| &mut a.value)
        && let Some(e) = value.entries[1].evidence.last_mut()
    {
        e.quote = "Et après ?".into();
    }
    assert!(matches!(
        f.engine.save_correction(altered),
        Err(EngineError::Cited(_))
    ));
    let (_, across) = span(&text, "Ensuite, «");
    assert!(matches!(
        f.engine.edit_cited(
            job.id,
            setup,
            source,
            0,
            "themes",
            &CitedEdit::AddQuotation {
                entry: 0,
                start,
                end: across,
            },
        ),
        Err(EngineError::Cited(_))
    ));
    assert!(before.is_empty());
}
