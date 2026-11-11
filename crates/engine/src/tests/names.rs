//! The names a job's files take, fixed: sources of the same name told apart in the job's order
//! (case ignored), items cut from a document (#50) written beside it, videos named by their
//! title, comparisons by set-up and translations by language. Names worked out for the whole job
//! at once must be those worked out for one source, and a job resumed half-way names its files
//! as one run through does.

use std::path::Path;

use kalamu_core::{DerivedFrom, LanguageChoice, LanguageCode};

use super::*;

/// Each source's files under each set-up, as `folder/base` from the temporary folder.
fn names(f: &Fixture, job: &Job) -> Vec<String> {
    let mut out = Vec::new();
    for source in &job.sources {
        for setup in &job.setups {
            let (dir, base) = crate::output::output_base(job, source, &setup.label);
            let folder = dir
                .strip_prefix(f.dir.path())
                .map(|d| d.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| dir.display().to_string());
            out.push(format!("{folder}/{base}"));
        }
    }
    out
}

/// A source at `folder/name` in the temporary folder, as the template with another name.
fn at(f: &Fixture, template: &SourceFile, folder: &str, name: &str) -> SourceFile {
    SourceFile {
        id: SourceId::new(),
        path: f.dir.path().join(folder).join(name).display().to_string(),
        file_name: name.into(),
        ..template.clone()
    }
}

/// Sources whose names meet: the same stem in one folder, another folder, another case; an item
/// cut from the first `Scan.pdf`; and two videos of nearly the same title.
fn meeting(f: &Fixture) -> Vec<SourceFile> {
    let template = f.pdf("template.pdf", 1);
    let first = at(f, &template, "janvier", "Scan.pdf");
    let item = SourceFile {
        derived_from: Some(DerivedFrom {
            snapshot_id: "snapshot".into(),
            job_id: JobId::new(),
            setup_id: SetupId::new(),
            source_id: first.id,
            original_path: first.path.clone(),
            unit: None,
        }),
        ..at(f, &template, "snapshots", "Scan.pdf")
    };
    let video = |title: &str| SourceFile {
        id: SourceId::new(),
        path: "https://www.youtube.com/watch?v=abc".into(),
        file_name: title.into(),
        kind: InputKind::YouTube,
        ..template.clone()
    };
    vec![
        first,
        at(f, &template, "février", "scan.pdf"),
        at(f, &template, "janvier", "Scan.tif"),
        at(f, &template, "janvier", "lettre.v2.pdf"),
        item,
        at(f, &template, "mars", "SCAN.PDF"),
        video("Conférence v1.2"),
        video("conférence V1.2"),
    ]
}

/// A job of these sources and models, set up in memory: names are worked out from the job alone,
/// so it need not be one the engine would start (videos beside their sources, a recording's
/// task over PDFs).
fn job_of(
    f: &Fixture,
    task: Task,
    models: &[&str],
    sources: Vec<SourceFile>,
    into: Option<&str>,
) -> Job {
    let mut job = f.new_job(sources[..1].to_vec(), models);
    job.task = task;
    job.sources = sources;
    if let Some(folder) = into {
        job.output.destination =
            OutputDestination::Folder(f.dir.path().join(folder).display().to_string());
    }
    job
}

/// The first sources of [`meeting`].
fn first(f: &Fixture, n: usize) -> Vec<SourceFile> {
    let mut sources = meeting(f);
    sources.truncate(n);
    sources
}

#[test]
fn output_names_are_fixed() {
    let f = fixture();
    let extraction = Task::TextExtraction;

    // Beside each source: only those in one folder meet, the item beside its original.
    let job = job_of(&f, extraction, &["fake-model"], first(&f, 6), None);
    assert_eq!(
        names(&f, &job),
        [
            "janvier/Scan",
            "février/scan",
            "janvier/Scan (2)",
            "janvier/lettre.v2",
            "janvier/Scan (3)",
            "mars/SCAN",
        ]
    );

    // Into one folder: every `scan` meets, whatever its case or folder; a video keeps its dots.
    let job = job_of(&f, extraction, &["fake-model"], meeting(&f), Some("out"));
    assert_eq!(
        names(&f, &job),
        [
            "out/Scan",
            "out/scan (2)",
            "out/Scan (3)",
            "out/lettre.v2",
            "out/Scan (4)",
            "out/SCAN (5)",
            "out/Conférence v1.2",
            "out/conférence V1.2 (2)",
        ]
    );

    // A comparison translated: the number, then the set-up, then the language.
    let mut job = job_of(
        &f,
        Task::Translation,
        &["flash", "pro"],
        first(&f, 3),
        Some("out"),
    );
    job.settings.languages = LanguageChoice::Explicit(vec![LanguageCode::new("zh-Hans")]);
    assert_eq!(
        names(&f, &job),
        [
            "out/Scan.flash.translated.zh-hans",
            "out/Scan.pro.translated.zh-hans",
            "out/scan (2).flash.translated.zh-hans",
            "out/scan (2).pro.translated.zh-hans",
            "out/Scan (3).flash.translated.zh-hans",
            "out/Scan (3).pro.translated.zh-hans",
        ]
    );

    // Each task's own ending, after the number.
    for (task, ending) in [
        (Task::Analysis, ".analysis"),
        (Task::Contents, ".contents"),
        (Task::TextCorrection, ".corrected"),
        (Task::Translation, ".translated"),
        (Task::AudioTranscription, ""),
    ] {
        let job = job_of(&f, task, &["fake-model"], first(&f, 2), Some("out"));
        assert_eq!(
            names(&f, &job),
            [format!("out/Scan{ending}"), format!("out/scan (2){ending}")],
            "{task:?}"
        );
    }
}

/// The files in a folder, by name.
fn listed(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[tokio::test]
async fn a_job_resumed_half_way_names_its_files_as_one_run_through() {
    let f = fixture();
    let mut sources = Vec::new();
    for (folder, name) in [
        ("janvier", "Scan.pdf"),
        ("février", "scan.pdf"),
        ("mars", "SCAN.pdf"),
        ("avril", "lettre.pdf"),
    ] {
        let dir = f.dir.path().join(folder);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        let path = dir.join(name);
        let bytes = kalamu_media::sample_pdf(&[(595.0, 842.0)], false);
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{e}"));
        sources.push(kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}")));
    }
    let job_into = |out: &Path| {
        f.engine
            .create_job(NewJob {
                name: "Scans".into(),
                task: Task::TextExtraction,
                sources: sources.clone(),
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
                    formats: vec![ExportFormat::Txt, ExportFormat::Json],
                    destination: OutputDestination::Folder(out.display().to_string()),
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"))
    };

    let once = f.dir.path().join("once");
    let job = job_into(&once);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let expected: Vec<String> = ["SCAN (3)", "Scan", "lettre", "scan (2)"]
        .iter()
        .flat_map(|base| ["json", "provenance.json", "txt"].map(|ext| format!("{base}.{ext}")))
        .collect();
    let mut sorted = expected.clone();
    sorted.sort();
    assert_eq!(listed(&once), sorted);

    // A second job stopped after its first two sources, as by a crash: their files written,
    // the later sources' pages in flight. Its files go, and the resumed run writes them all.
    let resumed = f.dir.path().join("resumed");
    let job = job_into(&resumed);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    std::fs::remove_dir_all(&resumed).unwrap_or_else(|e| panic!("{e}"));
    for mut piece in f.store.pieces(job.id).unwrap_or_default() {
        if piece.source_id == job.sources[2].id || piece.source_id == job.sources[3].id {
            piece.status = PieceStatus::Running;
            f.store
                .upsert_piece(&piece)
                .unwrap_or_else(|e| panic!("{e}"));
        }
    }
    f.store
        .set_job_status(job.id, JobStatus::Running)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.engine.recover().unwrap_or_default(), vec![job.id]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(listed(&resumed), sorted);
}
