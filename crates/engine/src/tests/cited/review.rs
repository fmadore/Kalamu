//! The researcher's corrections of an interview analysis: an entry's field typed anew, an
//! entry or one of its quotations removed, the model's answer unchanged and what was left out
//! still said; a quotation the model did not find refused.

use kalamu_core::analysis::cited::CitedEdit;

use super::*;

#[tokio::test]
async fn the_researcher_removes_entries_and_quotations_but_never_adds_one() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[answer()]);
    let job = f
        .analysis(vec![source], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let (setup, source) = (job.setups[0].id, job.sources[0].id);
    let edit =
        |key: &str, change: CitedEdit| f.engine.edit_cited(job.id, setup, source, 0, key, &change);
    edit(
        "themes",
        CitedEdit::Answer {
            entry: 1,
            key: "name".into(),
            value: "La devise".into(),
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    edit("themes", CitedEdit::RemoveEntry { entry: 0 }).unwrap_or_else(|e| panic!("{e}"));
    edit(
        "quotations",
        CitedEdit::RemoveQuotation {
            entry: 0,
            quotation: 0,
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let corrected = f.fields(&job, 0);
    let themes = cited(&corrected, "themes");
    assert_eq!(themes.entries.len(), 1);
    assert_eq!(
        themes.entries[0].answers[0].value,
        FieldValue::Text("La devise".into())
    );
    assert_eq!(themes.left_out, 1, "what was left out stays said");
    assert!(
        cited(&corrected, "quotations").entries[0]
            .evidence
            .is_empty()
    );
    // The model's answer is unchanged.
    let model = f.engine.source_pieces(job.id, source).unwrap_or_default()[0]
        .ai_fields
        .clone();
    assert_eq!(cited(&model, "themes").entries.len(), 2);
    assert!(matches!(
        edit("themes", CitedEdit::RemoveEntry { entry: 4 }),
        Err(EngineError::Cited(_))
    ));

    // A quotation the model did not find cannot be written through a correction.
    let mut forged = corrected.clone();
    if let Some(FieldValue::Cited(value)) = forged.get_mut(1).map(|a| &mut a.value) {
        value.entries[0].evidence[0].quote = "nous étions des pionnières".into();
    }
    let refused = f.engine.save_correction(Correction {
        job_id: job.id,
        setup_id: setup,
        source_id: source,
        index: 0,
        text: None,
        fields: Some(forged),
        reviewed: true,
        updated_at: Timestamp::now(),
    });
    assert_eq!(refused.err().map(|e| e.code()), Some("cited.invented"));
    assert_eq!(cited(&f.fields(&job, 0), "themes"), themes);
}
