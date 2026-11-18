//! Agreement between set-ups over a whole analysis, read from the store.

use std::time::Instant;

use kalamu_core::{
    AgreementMeasure, AnalysisSchema, Field, Job, PieceRef, PieceState, StoredResult,
};

use super::*;

fn schema() -> AnalysisSchema {
    let field = |key: &str, kind: FieldKind| Field {
        key: key.into(),
        label: key.into(),
        instructions: String::new(),
        kind,
        reason: false,
    };
    AnalysisSchema {
        fields: vec![
            field("summary", FieldKind::Text),
            field(
                "tone",
                FieldKind::Choice {
                    values: vec!["Positive".into(), "Negative".into()],
                },
            ),
            field("places", FieldKind::List { max: None }),
        ],
    }
}

fn answers(tone: &str, places: &[&str]) -> Vec<FieldAnswer> {
    vec![
        FieldAnswer {
            key: "summary".into(),
            value: FieldValue::Text("A meeting.".into()),
            reason: None,
        },
        FieldAnswer {
            key: "tone".into(),
            value: FieldValue::Text(tone.into()),
            reason: None,
        },
        FieldAnswer {
            key: "places".into(),
            value: FieldValue::List(places.iter().map(|p| (*p).to_string()).collect()),
            reason: None,
        },
    ]
}

/// An analysis of `count` documents by two set-ups, stored without running it.
fn analysis(f: &Fixture, count: usize) -> Job {
    let template = f.pdf("template.pdf", 1);
    let sources: Vec<SourceFile> = (0..count)
        .map(|i| SourceFile {
            id: SourceId::new(),
            file_name: format!("doc-{i}.pdf"),
            sha256: format!("{i:064x}"),
            ..template.clone()
        })
        .collect();
    let mut job = f.new_job_for(Task::Analysis, sources, &["flash", "pro"]);
    for setup in &mut job.setups {
        setup.prompt.schema = Some(schema());
    }
    f.store.delete_job(job.id).unwrap_or_else(|e| panic!("{e}"));
    f.store.create_job(&job).unwrap_or_else(|e| panic!("{e}"));
    job
}

/// Stores one set-up's answer to one document, or its failure.
fn answer(f: &Fixture, job: &Job, setup: usize, source: usize, given: Option<Vec<FieldAnswer>>) {
    let setup = &job.setups[setup];
    let source_id = job.sources[source].id;
    let key = format!("{}-{source}", setup.model);
    if let Some(fields) = &given {
        f.store
            .put_result(&StoredResult {
                cache_key: key.clone(),
                provider: ProviderId::new("fake"),
                model: setup.model.clone(),
                output: ProcessOutput {
                    fields: fields.clone(),
                    ..ProcessOutput::default()
                },
                request: Vec::new(),
                cost_usd: None,
                duration_ms: 1,
                created_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
    f.store
        .upsert_piece(&PieceState {
            job_id: job.id,
            setup_id: setup.id,
            source_id,
            index: 0,
            piece: PieceRef::Whole,
            status: if given.is_some() {
                PieceStatus::Done
            } else {
                PieceStatus::Failed
            },
            cache_key: given.is_some().then_some(key),
            error: None,
            attempts: 1,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn agreement_counts_the_models_answers_over_the_documents_both_answered() {
    let f = fixture();
    let job = analysis(&f, 4);
    answer(
        &f,
        &job,
        0,
        0,
        Some(answers("Positive", &["Dakar", "Bamako"])),
    );
    answer(&f, &job, 1, 0, Some(answers("Positive", &["dakar"])));
    answer(&f, &job, 0, 1, Some(answers("Negative", &[])));
    answer(&f, &job, 1, 1, Some(answers("Positive", &[])));
    answer(&f, &job, 0, 2, Some(answers("Negative", &["Ségou"])));
    answer(&f, &job, 1, 2, Some(answers("Negative", &["Segou"])));
    // The second set-up failed the fourth document: it is not counted.
    answer(&f, &job, 0, 3, Some(answers("Positive", &["Kayes"])));
    answer(&f, &job, 1, 3, None);
    // A correction is the researcher's reading: it changes nothing here.
    f.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[1].id,
            source_id: job.sources[1].id,
            index: 0,
            text: None,
            fields: Some(answers("Negative", &["Mopti"])),
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let agreement = f
        .engine
        .job_agreement(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let keys: Vec<&str> = agreement.fields.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["tone", "places"], "free text is not compared");

    let tone = &agreement.fields[0];
    assert_eq!(tone.measure, AgreementMeasure::Choice);
    let pair = &tone.pairs[0];
    assert_eq!(
        (pair.first, pair.second),
        (job.setups[0].id, job.setups[1].id)
    );
    // P/P, N/P, N/N: agreement 2/3. First P1 N2, second P2 N1: chance (2 + 2) / 9, so
    // κ = (6 − 4) / (9 − 4).
    assert_eq!(pair.pieces, 3);
    assert!((pair.agreement.unwrap_or_default() - 2.0 / 3.0).abs() < 1e-9);
    assert!((pair.kappa.unwrap_or_default() - 0.4).abs() < 1e-9);

    // Places: 1/2, both empty 1, Ségou and Segou 1.
    let places = &agreement.fields[1].pairs[0];
    assert_eq!(places.pieces, 3);
    assert!((places.agreement.unwrap_or_default() - 2.5 / 3.0).abs() < 1e-9);
    assert_eq!(places.kappa, None);
}

#[test]
fn a_job_with_one_set_up_or_another_task_has_no_agreement() {
    let f = fixture();
    let one = f.new_job_for(Task::Analysis, vec![f.pdf("a.pdf", 1)], &["flash"]);
    let text = f.new_job(vec![f.pdf("b.pdf", 1)], &["flash", "pro"]);
    for job in [one, text] {
        let agreement = f
            .engine
            .job_agreement(job.id)
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(agreement.fields.is_empty());
    }
}

#[test]
fn an_item_added_by_hand_leaves_the_agreement_as_it_was() {
    let f = fixture();
    let job = analysis(&f, 1);
    answer(
        &f,
        &job,
        0,
        0,
        Some(answers("Positive", &["Dakar", "Bamako"])),
    );
    answer(&f, &job, 1, 0, Some(answers("Positive", &["Dakar"])));
    let before = f
        .engine
        .job_agreement(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let added = f
        .engine
        .add_listed(
            job.id,
            job.sources[0].id,
            0,
            "places",
            &["Bamako; Mopti".into()],
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(added.added, vec!["Bamako", "Mopti"]);
    assert_eq!(added.before.len(), 2);
    // Both corrected answers now list the same places; the models' own answers still differ.
    assert_eq!(
        f.engine
            .job_agreement(job.id)
            .unwrap_or_else(|e| panic!("{e}")),
        before
    );
}

#[test]
fn a_field_of_choices_takes_only_its_choices_as_the_schema_spells_them() {
    let f = fixture();
    let job = analysis(&f, 1);
    let mut job = job;
    for setup in &mut job.setups {
        if let Some(schema) = &mut setup.prompt.schema {
            schema.fields.push(Field {
                key: "themes".into(),
                label: "Themes".into(),
                instructions: String::new(),
                kind: FieldKind::Choices {
                    values: vec!["Education".into(), "Pilgrimage".into()],
                },
                reason: false,
            });
        }
    }
    f.store.delete_job(job.id).unwrap_or_else(|e| panic!("{e}"));
    f.store.create_job(&job).unwrap_or_else(|e| panic!("{e}"));
    answer(&f, &job, 0, 0, Some(answers("Positive", &["Dakar"])));
    answer(&f, &job, 1, 0, None);
    let added = f
        .engine
        .add_listed(
            job.id,
            job.sources[0].id,
            0,
            "themes",
            &["pilgrimage".into(), "Trade".into()],
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(added.added, vec!["Pilgrimage"]);
    assert_eq!(added.refused, vec!["Trade"]);
    assert_eq!(
        added.before.len(),
        1,
        "the set-up that failed has nothing to add to"
    );
    let view = f
        .engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .find(|v| v.setup_id == job.setups[0].id);
    let themes = view
        .and_then(|v| v.corrected_fields)
        .and_then(|fields| fields.into_iter().find(|a| a.key == "themes"))
        .map(|a| a.value);
    assert_eq!(themes, Some(FieldValue::List(vec!["Pilgrimage".into()])));
}

/// 5,000 documents read by two set-ups. Ignored by default; run it with
/// `cargo test -p kalamu-engine --release agreement_of_a_large_job -- --ignored --nocapture`.
#[test]
#[ignore = "a timing, not a check"]
fn agreement_of_a_large_job() {
    let f = fixture();
    let count = 5_000;
    let job = analysis(&f, count);
    let places = ["Dakar", "Bamako", "Ségou", "Kayes", "Mopti"];
    for source in 0..count {
        for setup in 0..2 {
            let tone = if (source + setup) % 3 == 0 {
                "Negative"
            } else {
                "Positive"
            };
            let listed = &places[..(source + setup) % places.len()];
            answer(&f, &job, setup, source, Some(answers(tone, listed)));
        }
    }
    let start = Instant::now();
    let agreement = f
        .engine
        .job_agreement(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    println!(
        "agreement over {count} documents in {:.0} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    assert_eq!(agreement.fields[0].pairs[0].pieces, count as u32);
}
