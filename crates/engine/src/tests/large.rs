//! A large job timed (#143): 5,000 one-page sources read by two set-ups, every page stored with
//! the size of a real page's answer, a few corrections and refused answers, and each source a
//! document in two projects. Ignored by default; run it with
//! `cargo test -p kalamu-engine --release large_job -- --ignored --nocapture`, and with
//! `KALAMU_LARGE_SOURCES=2000` before it for the smaller job the earlier rounds measured.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use kalamu_core::{
    Charge, Document, DocumentId, DocumentLink, ExportError, PieceRef, PieceState, Project,
    ProjectId, ProjectStore, Setup, StoredResult, TextRegion,
};

use super::*;

/// The issue's 5,000 sources, unless `KALAMU_LARGE_SOURCES` says otherwise.
fn sources_count() -> usize {
    std::env::var("KALAMU_LARGE_SOURCES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(5_000)
}

/// Time spent in each step of writing files, with how often and how many bytes.
#[derive(Default)]
struct Steps(BTreeMap<String, (f64, usize, usize)>);

impl Steps {
    fn add(&mut self, step: impl Into<String>, start: Instant, bytes: usize) {
        let entry = self.0.entry(step.into()).or_default();
        entry.0 += ms(start);
        entry.1 += 1;
        entry.2 += bytes;
    }

    fn print(&self, per: usize) {
        let total: f64 = self.0.values().map(|v| v.0).sum();
        for (step, (time, count, bytes)) in &self.0 {
            eprintln!(
                "  {step:<28} {:>8.3} ms a source  {:>5.1} %  ({count} times, {} bytes each)",
                time / per as f64,
                100.0 * time / total,
                bytes / (*count).max(1)
            );
        }
        eprintln!("  {:<28} {:>8.3} ms a source", "all", total / per as f64);
    }
}

/// What `write_files` does for one source under one set-up, step by step, each timed.
fn write_split(
    f: &Fixture,
    job: &Job,
    names: &dyn Fn(&SourceFile, &str) -> (PathBuf, String),
    source: &SourceFile,
    setup: &Setup,
    steps: &mut Steps,
) {
    let start = Instant::now();
    let pieces = f.store.source_pieces(job.id, setup.id, source.id);
    let marks = f.store.outdated_files(Some(job.id));
    assert!(pieces.is_ok() && marks.is_ok());
    steps.add("store: pieces and marks", start, 0);

    let start = Instant::now();
    let document = f
        .engine
        .export_document(job, setup.id, source.id)
        .unwrap_or_else(|e| panic!("{e}"));
    steps.add("read: export_document", start, 0);

    let start = Instant::now();
    let (dir, base) = names(&document.source, &setup.label);
    steps.add("names: output base", start, 0);

    let start = Instant::now();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    steps.add("disk: create_dir_all", start, 0);

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for format in &job.output.formats {
        let Some(exporter) = kalamu_export::exporter(*format) else {
            continue;
        };
        if !exporter.supports(job.task) {
            continue;
        }
        let start = Instant::now();
        let bytes = match exporter.export(&document) {
            Ok(bytes) => bytes,
            Err(ExportError::Unsupported(_)) => continue,
            Err(e) => panic!("{e}"),
        };
        steps.add(
            format!("render: {}", format.extension()),
            start,
            bytes.len(),
        );
        files.push((format.extension().to_string(), bytes));
    }
    let start = Instant::now();
    let provenance = serde_json::to_vec_pretty(&document.provenance).unwrap_or_default();
    steps.add("render: provenance.json", start, provenance.len());
    files.push(("provenance.json".into(), provenance));

    for (ext, bytes) in files {
        let path = dir.join(format!("{base}.{ext}"));
        let tmp = path.with_extension(format!(
            "{}.pending",
            path.extension().and_then(|e| e.to_str()).unwrap_or("out")
        ));
        let start = Instant::now();
        std::fs::write(&tmp, &bytes).unwrap_or_else(|e| panic!("{e}"));
        steps.add(format!("disk: write {ext}.pending"), start, bytes.len());
        let start = Instant::now();
        std::fs::rename(&tmp, &path).unwrap_or_else(|e| panic!("{e}"));
        steps.add(format!("disk: rename {ext}"), start, 0);
    }
}

/// What writing a file costs on this disk, apart from what is in it: small files and larger
/// ones, written new or over themselves, straight or through a temporary file renamed.
fn disk_probe(root: &Path) {
    let dir = root.join("probe");
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    let n = 500;
    let probe = |label: &str, size: usize, atomic: bool, prefix: &str| {
        let bytes = vec![b'x'; size];
        let start = Instant::now();
        for i in 0..n {
            let path = dir.join(format!("{prefix}-{i}.txt"));
            if atomic {
                crate::output::write_atomic(&path, &bytes).unwrap_or_else(|e| panic!("{e}"));
            } else {
                std::fs::write(&path, &bytes).unwrap_or_else(|e| panic!("{e}"));
            }
        }
        eprintln!("  {label:<44} {:.3} ms a file", ms(start) / n as f64);
    };
    probe("1 KB, new, written straight", 1_000, false, "a");
    probe("1 KB, over itself, written straight", 1_000, false, "a");
    probe("1 KB, new, temporary file renamed", 1_000, true, "b");
    probe(
        "1 KB, over itself, temporary file renamed",
        1_000,
        true,
        "b",
    );
    probe("100 KB, new, written straight", 100_000, false, "c");
    probe("100 KB, new, temporary file renamed", 100_000, true, "d");
    let start = Instant::now();
    for i in 0..n {
        let _ = dir.join(format!("a-{i}.txt")).is_file();
    }
    eprintln!(
        "  {:<44} {:.3} ms a file",
        "is_file on an existing file",
        ms(start) / n as f64
    );
}

/// The engine over a database file, as the app opens it, in a temporary folder.
fn on_disk() -> Fixture {
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let store = Arc::new(
        SqliteStore::open(&dir.path().join("kalamu.db")).unwrap_or_else(|e| panic!("store: {e}")),
    );
    let fake = Fake::new();
    let engine = Engine::new(
        store.clone(),
        vec![fake.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    Fixture {
        engine,
        fake,
        store,
        dir,
    }
}

/// A page's answer of about 9 KB, as an OCR model's with its blocks placed.
fn answer(model: &str, source: usize) -> ProcessOutput {
    let line = format!("{model} reads line of source {source}, with the words a page holds. ");
    let regions = (0..40)
        .map(|i| TextRegion::block(0.05, 0.02 * i as f32, 0.9, 0.02, line.repeat(2)))
        .collect();
    ProcessOutput {
        text: line.repeat(40),
        usage: Usage {
            input_tokens: 1500,
            output_tokens: 900,
            ..Usage::default()
        },
        model_version: Some(format!("{model}-001")),
        regions,
        truncated: source.is_multiple_of(97),
        ..ProcessOutput::default()
    }
}

/// Milliseconds since `start`.
fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[tokio::test]
#[ignore = "a timing, not a check: run by hand before and after a change to exports"]
async fn large_job() {
    let count = sources_count();
    let f = on_disk();
    let built = Instant::now();
    let template = f.pdf("template.pdf", 1);
    let bytes = std::fs::read(&template.path).unwrap_or_default();
    let sources: Vec<SourceFile> = (0..count)
        .map(|i| {
            let name = format!("page-{i:04}.pdf");
            let path = f.dir.path().join(&name);
            std::fs::write(&path, &bytes).unwrap_or_else(|e| panic!("{e}"));
            SourceFile {
                id: SourceId::new(),
                path: path.display().to_string(),
                file_name: name,
                sha256: format!("{i:064x}"),
                ..template.clone()
            }
        })
        .collect();
    let mut job = f.new_job(sources, &["flash", "pro"]);
    // The fake model places no text in the catalogue, so the searchable copy is asked for here,
    // as a model that places it would have it.
    job.output.formats.push(ExportFormat::SearchablePdf);
    f.store.delete_job(job.id).unwrap_or_else(|e| panic!("{e}"));
    f.store.create_job(&job).unwrap_or_else(|e| panic!("{e}"));

    let mut pieces = Vec::new();
    for (i, source) in job.sources.iter().enumerate() {
        for setup in &job.setups {
            let key = format!("{}-{i}", setup.model);
            f.store
                .put_result(&StoredResult {
                    cache_key: key.clone(),
                    provider: ProviderId::new("fake"),
                    model: setup.model.clone(),
                    output: answer(&setup.model, i),
                    request: Vec::new(),
                    cost_usd: Some(0.001),
                    duration_ms: 1200,
                    created_at: Timestamp::now(),
                })
                .unwrap_or_else(|e| panic!("{e}"));
            pieces.push(PieceState {
                job_id: job.id,
                setup_id: setup.id,
                source_id: source.id,
                index: 0,
                piece: PieceRef::Page {
                    number: 1,
                    half: None,
                },
                status: kalamu_core::PieceStatus::Done,
                cache_key: Some(key),
                error: None,
                attempts: 1,
                updated_at: Timestamp::now(),
            });
        }
    }
    f.store
        .upsert_pieces(&pieces)
        .unwrap_or_else(|e| panic!("{e}"));
    for (i, source) in job.sources.iter().enumerate().step_by(100) {
        f.store
            .put_correction(&Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: source.id,
                index: 0,
                text: Some(format!("corrected page of source {i}")),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
        f.store
            .add_charge(&Charge {
                job_id: job.id,
                setup_id: job.setups[1].id,
                source_id: source.id,
                index: 0,
                usage: Usage::default(),
                cost_usd: Some(0.002),
                created_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
    // Every source a document in two projects, as a researcher who sorts a fonds twice.
    for p in 0..2 {
        let project = Project {
            id: ProjectId::new(),
            name: format!("Project {p}"),
            description: String::new(),
            archived: false,
            source_defaults: Default::default(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        f.store
            .put_project(&project)
            .unwrap_or_else(|e| panic!("{e}"));
        for source in &job.sources {
            let document = Document {
                id: DocumentId::new(),
                project_id: project.id,
                title: source.file_name.clone(),
                source: source.clone(),
                created_at: Timestamp::now(),
                tags: Vec::new(),
                source_description: Default::default(),
            };
            f.store
                .put_document(&document)
                .unwrap_or_else(|e| panic!("{e}"));
            f.store
                .put_link(&DocumentLink {
                    document_id: document.id,
                    job_id: job.id,
                    source_id: source.id,
                    linked_at: Timestamp::now(),
                })
                .unwrap_or_else(|e| panic!("{e}"));
        }
    }
    f.engine
        .set_source_documents(Arc::new(kalamu_projects::Projects::new(
            f.store.clone(),
            f.store.clone(),
        )));
    eprintln!(
        "large job: {count} sources x {} set-ups built in {:.0} ms",
        job.setups.len(),
        ms(built)
    );

    let middle = &job.sources[count / 2];
    let setup = job.setups[0].id;
    let start = Instant::now();
    let runs = 20;
    for _ in 0..runs {
        f.engine
            .export_document(&job, setup, middle.id)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    eprintln!(
        "export_document of one source (read, not written): {:.2} ms",
        ms(start) / runs as f64
    );

    let start = Instant::now();
    for _ in 0..runs {
        f.engine
            .export(job.id, setup, middle.id)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    eprintln!("export of one source: {:.2} ms", ms(start) / runs as f64);
    // Of which reading the job's record, which `export` does first and a run does once.
    let start = Instant::now();
    for _ in 0..runs {
        f.store.job(job.id).unwrap_or_else(|e| panic!("{e}"));
    }
    eprintln!(
        "  of which the job's record read: {:.2} ms",
        ms(start) / runs as f64
    );

    let setups: Vec<&Setup> = job.setups.iter().collect();
    let start = Instant::now();
    for source in &job.sources {
        let failed = f
            .engine
            .export_finished(&job, source.id, &setups)
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(!failed);
    }
    eprintln!(
        "export_finished over the job ({count} sources): {:.0} ms",
        ms(start)
    );

    // The names alone, for every source under every set-up, as the job's writes ask for them.
    let start = Instant::now();
    for source in &job.sources {
        for setup in &job.setups {
            std::hint::black_box(crate::output::output_base(&job, source, &setup.label));
        }
    }
    eprintln!("  of which the output names: {:.0} ms", ms(start));

    // Writing a source's files, step by step, for one source in ten (over the files the pass
    // above wrote: each is replaced, as a later export or a resumed job replaces it).
    let mut steps = Steps::default();
    let names = |source: &SourceFile, label: &str| crate::output::output_base(&job, source, label);
    let sampled: Vec<&SourceFile> = job.sources.iter().step_by(10).collect();
    for source in &sampled {
        for setup in &job.setups {
            write_split(&f, &job, &names, source, setup, &mut steps);
        }
    }
    eprintln!(
        "a source's files written, step by step (one source in ten, {} set-ups):",
        job.setups.len()
    );
    steps.print(sampled.len());
    eprintln!("writing a file on this disk:");
    disk_probe(f.dir.path());

    let start = Instant::now();
    let runs = 10;
    for _ in 0..runs {
        let jobs = f.store.list_jobs().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(jobs[0].pieces_total as usize, count * 2);
    }
    eprintln!("list_jobs: {:.2} ms", ms(start) / runs as f64);

    let start = Instant::now();
    for _ in 0..runs {
        f.store.spent(job.id).unwrap_or_else(|e| panic!("{e}"));
    }
    eprintln!("spent: {:.2} ms", ms(start) / runs as f64);

    // What a running job's page reads twice a second: every piece (as it did until the
    // fourth round, and as counting them in the engine did until the third round of large
    // jobs), or the counts of the files it shows (the first two hundred), made by the store.
    let start = Instant::now();
    let mut sent = 0;
    for _ in 0..runs {
        let pieces = f.store.pieces(job.id).unwrap_or_else(|e| panic!("{e}"));
        sent = serde_json::to_vec(&pieces).map(|v| v.len()).unwrap_or(0);
    }
    eprintln!(
        "job page, every piece: {:.2} ms, {sent} bytes of JSON",
        ms(start) / runs as f64
    );
    let shown: Vec<SourceId> = job.sources.iter().take(200).map(|s| s.id).collect();
    let start = Instant::now();
    for _ in 0..runs {
        let progress = f
            .engine
            .job_progress(job.id, &shown)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(progress.files.len(), 400);
        sent = serde_json::to_vec(&progress).map(|v| v.len()).unwrap_or(0);
    }
    eprintln!(
        "job page, counts of 200 files: {:.2} ms, {sent} bytes of JSON",
        ms(start) / runs as f64
    );

    let start = Instant::now();
    for i in 0..runs {
        f.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: setup,
                source_id: middle.id,
                index: 0,
                text: Some(format!("corrected again, {i}")),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
    eprintln!("save_correction: {:.2} ms", ms(start) / runs as f64);

    f.store.delete_job(job.id).unwrap_or_else(|e| panic!("{e}"));
    let start = Instant::now();
    let pruned = f.engine.prune_results().unwrap_or_else(|e| panic!("{e}"));
    eprintln!(
        "prune_results after deleting the job: {:.0} ms, {} results, {} bytes",
        ms(start),
        pruned.results,
        pruned.bytes
    );
}

/// Corrections in a long document: each save only stores and marks, and every output of the
/// source is written again once afterwards, the searchable copy from the whole original.
#[tokio::test]
#[ignore = "a timing, not a check: run by hand before and after a change to exports"]
async fn long_document_correction() {
    const PAGES: u32 = 300;
    let f = on_disk();
    let source = f.pdf("register.pdf", PAGES as usize);
    let mut job = f.new_job(vec![source], &["flash"]);
    job.output.formats.push(ExportFormat::SearchablePdf);
    f.store.delete_job(job.id).unwrap_or_else(|e| panic!("{e}"));
    f.store.create_job(&job).unwrap_or_else(|e| panic!("{e}"));
    let (setup, source) = (job.setups[0].id, job.sources[0].id);
    let mut pieces = Vec::new();
    for index in 0..PAGES {
        let key = format!("register-{index}");
        f.store
            .put_result(&StoredResult {
                cache_key: key.clone(),
                provider: ProviderId::new("fake"),
                model: "flash".into(),
                output: answer("flash", index as usize),
                request: Vec::new(),
                cost_usd: Some(0.001),
                duration_ms: 1200,
                created_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
        pieces.push(PieceState {
            job_id: job.id,
            setup_id: setup,
            source_id: source,
            index,
            piece: PieceRef::page(index + 1),
            status: kalamu_core::PieceStatus::Done,
            cache_key: Some(key),
            error: None,
            attempts: 1,
            updated_at: Timestamp::now(),
        });
    }
    f.store
        .upsert_pieces(&pieces)
        .unwrap_or_else(|e| panic!("{e}"));
    let runs = 5;
    let start = Instant::now();
    for i in 0..runs {
        f.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: setup,
                source_id: source,
                index: i,
                text: Some(format!("corrected page {i}")),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }
    eprintln!(
        "save_correction in a {PAGES}-page document: {:.2} ms",
        ms(start) / f64::from(runs)
    );
    let start = Instant::now();
    assert_eq!(f.engine.write_outdated().ok(), Some(0));
    eprintln!("its files written afterwards, once: {:.1} ms", ms(start));
}

/// Pruning the cache keeps every answer a job still refers to: a job read again after another
/// with the same files was deleted reuses the answers, and a job that keeps them never pays
/// again.
#[tokio::test]
async fn pruning_keeps_what_a_job_still_reads() {
    let f = fixture();
    let source = f.pdf("bulletin.pdf", 2);
    let first = f.new_job(vec![source.clone()], &["fake-model"]);
    assert_eq!(f.run(first.id).await, JobStatus::Completed);
    let second = f.new_job(vec![source], &["fake-model"]);
    assert_eq!(f.run(second.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 2, "the second job read the first's answers");

    f.engine
        .delete_job(first.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let pruned = f.engine.prune_results().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pruned.results, 0, "the second job still reads them");
    f.engine
        .export(second.id, second.setups[0].id, second.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let txt = std::fs::read_to_string(f.dir.path().join("bulletin.txt")).unwrap_or_default();
    assert!(txt.contains("fake-model page 2"), "{txt}");

    f.engine
        .delete_job(second.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let pruned = f.engine.prune_results().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pruned.results, 2);
    let third = f.new_job(vec![f.pdf("bulletin.pdf", 2)], &["fake-model"]);
    assert_eq!(f.run(third.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 4, "answers pruned are asked for again");
}
