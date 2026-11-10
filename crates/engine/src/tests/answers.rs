//! A name an analysis listed, renamed or taken out in every set-up at once, and put back.

use super::*;

/// An analysis of one document by two models, each listing the place "Bamako".
async fn two_readings(f: &Fixture) -> (Job, SourceFile) {
    let source = f.pdf("bulletin.pdf", 1);
    let setup = |model: &str| SetupRequest {
        provider: ProviderId::new("fake"),
        model: model.into(),
        preset_id: Some("analysis.entities".into()),
        label: None,
        options: Default::default(),
        merge: None,
    };
    let job = f
        .engine
        .create_job(NewJob {
            name: "Places".into(),
            task: Task::Analysis,
            sources: vec![source.clone()],
            setups: vec![setup("fake-model"), setup("other-model")],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Csv],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    (job, source)
}

/// Each set-up's place and review mark, in the job's order of set-ups: the store lists pieces
/// by set-up id, which is random.
fn places(f: &Fixture, job: &Job, source: &SourceFile) -> Vec<(Option<FieldValue>, bool)> {
    let mut pieces = f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default();
    pieces.sort_by_key(|v| job.setups.iter().position(|s| s.id == v.setup_id));
    pieces
        .iter()
        .map(|v| {
            let place = v
                .corrected_fields
                .as_ref()
                .and_then(|fields| fields.iter().find(|a| a.key == "places"))
                .map(|a| a.value.clone());
            (place, v.reviewed)
        })
        .collect()
}

#[tokio::test]
async fn a_name_is_renamed_in_every_set_up_and_put_back_as_it_was() {
    let f = fixture();
    let (job, source) = two_readings(&f).await;
    // The first set-up was already marked as reviewed; it stays so.
    f.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: source.id,
            index: 0,
            text: None,
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let before = f
        .engine
        .rename_listed(
            job.id,
            source.id,
            0,
            "places",
            "bamako",
            Some("Bamako (Mali)"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(before.len(), 2, "both set-ups listed the name");
    let renamed = Some(FieldValue::List(vec!["Bamako (Mali)".into()]));
    assert_eq!(
        places(&f, &job, &source),
        vec![(renamed.clone(), true), (renamed, false)]
    );
    assert_eq!(f.engine.write_outdated().ok(), Some(0));
    // Each set-up's table is written again with the name.
    let tables: Vec<String> = std::fs::read_dir(f.dir.path())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "csv"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect();
    assert_eq!(tables.len(), 2);
    assert!(
        tables.iter().all(|t| t.contains("Bamako (Mali)")),
        "{tables:?}"
    );

    // A name no set-up holds changes nothing.
    let none = f
        .engine
        .rename_listed(job.id, source.id, 0, "places", "Ségou", None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(none.is_empty());

    f.engine
        .save_corrections(before)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        places(&f, &job, &source),
        vec![(None, true), (None, false)],
        "each set-up as it was, the model's answers under no correction"
    );
}

#[tokio::test]
async fn a_name_taken_out_leaves_the_rest_of_each_answer() {
    let f = fixture();
    let (job, source) = two_readings(&f).await;
    f.engine
        .rename_listed(job.id, source.id, 0, "places", "Bamako", None)
        .unwrap_or_else(|e| panic!("{e}"));
    let empty = Some(FieldValue::List(Vec::new()));
    assert_eq!(
        places(&f, &job, &source),
        vec![(empty.clone(), false), (empty, false)]
    );
    let view = &f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default()[0];
    assert_eq!(
        view.corrected_fields.as_ref().map(Vec::len),
        Some(view.ai_fields.len()),
        "every other field is kept"
    );
}
