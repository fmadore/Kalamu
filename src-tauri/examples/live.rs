//! Runs real jobs against the live provider APIs, with the keys saved in the app's Settings
//! (Credential Manager on Windows, the Keychain on macOS), to check every task end to end before a release. Never run in
//! CI: it spends money and needs keys.
//!
//!   cargo run -p kalamu --example live -- <samples> <out> [job name filter]
//!
//! `<samples>` holds `newspaper.jpg`, `letter.pdf`, `interview.wav` and `article.txt`; each job
//! writes its outputs to its own folder in `<out>` and the run ends with a table of pieces.
//! `LIVE_SKIP=gemini,openai` leaves out providers; `LIVE_PACE=economy` sends every job through
//! the providers' batch services. `LIVE_TESSDATA=<folder>` adds the offline job, read with
//! Tesseract from the `fra` and `eng` language data in that folder; it needs no key, costs
//! nothing and can run with the network off.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use kalamu_catalog::Catalog;
use kalamu_core::{
    ApiKey, ExportFormat, JobSettings, JobStatus, JobStore, LanguageChoice, LanguageCode,
    LineBreaks, OutputDestination, OutputSettings, Pace, PieceStatus, ProgressEvent, ProviderId,
    SetupOptions, Task,
};
use kalamu_engine::{Engine, EngineConfig, EstimateRequest, KeySource, NewJob, SetupRequest};
use kalamu_prompts::PromptLibrary;
use kalamu_store::SqliteStore;

/// The same entries the app writes (`src/secrets.rs`); keys still under the Scriptorium entry
/// are moved there the first time the app reads them.
struct CredentialKeys;

impl KeySource for CredentialKeys {
    fn key(&self, provider: &ProviderId) -> Option<ApiKey> {
        keyring::Entry::new(
            "io.github.fmadore.kalamu",
            &format!("{}-api-key", provider.as_str()),
        )
        .and_then(|e| e.get_password())
        .ok()
        .filter(|k| !k.trim().is_empty())
        .map(ApiKey::new)
    }
}

struct Plan {
    name: &'static str,
    task: Task,
    files: &'static [&'static str],
    setups: Vec<SetupRequest>,
    settings: JobSettings,
    formats: Vec<ExportFormat>,
}

fn setup(provider: &str, model: &str, preset: Option<&str>, options: SetupOptions) -> SetupRequest {
    SetupRequest {
        provider: ProviderId::new(provider),
        model: model.into(),
        preset_id: preset.map(Into::into),
        label: None,
        options,
        merge: None,
    }
}

fn plans() -> Vec<Plan> {
    let plain = SetupOptions::default;
    let interview = || SetupOptions {
        diarize: true,
        timestamps: true,
        speakers: Some(2),
        ..SetupOptions::default()
    };
    let ocr = |preset: Option<&'static str>| {
        vec![
            setup("gemini", "gemini-3.8-flash", preset, plain()),
            setup("openai", "gpt-6-luna", preset, plain()),
            setup("anthropic", "claude-haiku-5-5", preset, plain()),
            setup("mistral", "mistral-ocr-4-1", None, plain()),
        ]
    };
    let analysis = |preset: &'static str| {
        vec![
            setup("gemini", "gemini-3.8-flash", Some(preset), plain()),
            setup("openai", "gpt-6-luna", Some(preset), plain()),
            setup("anthropic", "claude-haiku-5-5", Some(preset), plain()),
            setup("mistral", "mistral-small-2603", Some(preset), plain()),
        ]
    };
    vec![
        // Offline: the first pages of a multi-page PDF and an image, French and English.
        Plan {
            name: "offline-ocr",
            task: Task::TextExtraction,
            files: &["newspaper.jpg", "letter.pdf"],
            setups: vec![setup("tesseract", "tesseract-5", None, plain())],
            settings: JobSettings {
                languages: LanguageChoice::Explicit(vec![
                    LanguageCode::new("fr"),
                    LanguageCode::new("en"),
                ]),
                sample_pages: Some(3),
                ..JobSettings::default()
            },
            formats: vec![
                ExportFormat::Txt,
                ExportFormat::Json,
                ExportFormat::SearchablePdf,
                ExportFormat::Alto,
            ],
        },
        Plan {
            name: "ocr-as-on-page",
            task: Task::TextExtraction,
            files: &["newspaper.jpg", "letter.pdf"],
            setups: ocr(None),
            settings: JobSettings {
                line_breaks: Some(LineBreaks::AsOnPage),
                ..JobSettings::default()
            },
            formats: vec![
                ExportFormat::Txt,
                ExportFormat::Json,
                ExportFormat::Docx,
                ExportFormat::SearchablePdf,
            ],
        },
        Plan {
            name: "ocr-running-text",
            task: Task::TextExtraction,
            files: &["newspaper.jpg"],
            setups: vec![
                setup(
                    "gemini",
                    "gemini-3.8-flash",
                    Some("text_extraction.newspaper"),
                    plain(),
                ),
                setup(
                    "anthropic",
                    "claude-haiku-5-5",
                    Some("text_extraction.newspaper"),
                    plain(),
                ),
            ],
            settings: JobSettings {
                line_breaks: Some(LineBreaks::RunningText),
                ..JobSettings::default()
            },
            formats: vec![ExportFormat::Txt],
        },
        Plan {
            name: "audio",
            task: Task::AudioTranscription,
            files: &["interview.wav"],
            setups: vec![
                setup("gemini", "gemini-3.8-flash", None, interview()),
                setup("openai", "gpt-4o-transcribe-diarize", None, interview()),
                setup("mistral", "voxtral-mini-transcribe-3-0", None, interview()),
            ],
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt, ExportFormat::Srt, ExportFormat::Json],
        },
        Plan {
            name: "analysis-entities",
            task: Task::Analysis,
            files: &["article.txt"],
            setups: analysis("analysis.entities"),
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        },
        Plan {
            name: "analysis-tone",
            task: Task::Analysis,
            files: &["article.txt"],
            setups: analysis("analysis.tone"),
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        },
        Plan {
            name: "analysis-summary",
            task: Task::Analysis,
            files: &["article.txt"],
            setups: analysis("analysis.summary_research"),
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt, ExportFormat::Json],
        },
        // Three texts with the same long instructions, for what the providers' prompt caches
        // save; best run with LIVE_PACE=economy. It also needs `article2.txt` and `article3.txt`.
        Plan {
            name: "analysis-batch",
            task: Task::Analysis,
            files: &["article.txt", "article2.txt", "article3.txt"],
            setups: analysis("analysis.entities"),
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Json],
        },
        Plan {
            name: "audio-transcribe",
            task: Task::AudioTranscription,
            files: &["interview.wav"],
            setups: vec![setup("gemini", "gemini-3.5-transcribe", None, interview())],
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt, ExportFormat::Srt, ExportFormat::Json],
        },
        Plan {
            name: "ocr-terra",
            task: Task::TextExtraction,
            files: &["newspaper.jpg", "letter.pdf"],
            setups: vec![setup("openai", "gpt-6.1-terra", None, plain())],
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt],
        },
        Plan {
            name: "analysis-more-models",
            task: Task::Analysis,
            files: &["article.txt"],
            setups: ["gpt-6.1-terra", "mistral-medium-3-5", "mistral-large-4-0"]
                .into_iter()
                .chain(["zai-glm-5-3-flash", "deepseek-v4-1-flash-1017"])
                .map(|model| {
                    let provider = if model.starts_with("gpt") {
                        "openai"
                    } else {
                        "mistral"
                    };
                    setup(provider, model, Some("analysis.entities"), plain())
                })
                .collect(),
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        },
    ]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [samples, out, rest @ ..] = args.as_slice() else {
        anyhow::bail!("usage: live <samples> <out> [job name filter]");
    };
    let (samples, out) = (PathBuf::from(samples), PathBuf::from(out));
    let filter = rest.first().cloned().unwrap_or_default();
    std::fs::create_dir_all(&out)?;

    let (engine, store) = engine(&out)?;
    let mut total = 0.0;
    let skip = std::env::var("LIVE_SKIP").unwrap_or_default();
    let skip: Vec<&str> = skip.split(',').filter(|s| !s.is_empty()).collect();
    let economy = std::env::var("LIVE_PACE").is_ok_and(|p| p == "economy");
    let tessdata = std::env::var_os("LIVE_TESSDATA").is_some();
    for mut plan in plans().into_iter().filter(|p| p.name.contains(&filter)) {
        plan.setups.retain(|s| {
            !skip.contains(&s.provider.as_str()) && (tessdata || s.provider.as_str() != "tesseract")
        });
        if plan.setups.is_empty() {
            continue;
        }
        if economy {
            plan.settings.pace = Pace::Economy;
        }
        total += run(&engine, &store, &samples, &out, plan).await?;
    }
    println!("\nSpent about ${total:.4} in all.");
    Ok(())
}

fn engine(out: &Path) -> anyhow::Result<(Arc<Engine>, Arc<SqliteStore>)> {
    let data = out.join("data");
    std::fs::create_dir_all(&data)?;
    let store = Arc::new(SqliteStore::open(&data.join("jobs.sqlite"))?);
    let prompts = PromptLibrary::open(&data.join("presets"))?;
    // The installed app's FFmpeg (in Kalamu's folder in Windows' local app data, or inside the
    // Mac app in Applications), or KALAMU_FFMPEG_DIR.
    let installed = [
        std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Kalamu")),
        Some(PathBuf::from("/Applications/Kalamu.app/Contents/MacOS")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let ffmpeg = kalamu_media::Ffmpeg::locate(&installed);
    println!(
        "FFmpeg: {}",
        if ffmpeg.is_some() { "found" } else { "missing" }
    );
    let mut providers = kalamu_providers::registry(kalamu_providers::client());
    if let Some(dir) = std::env::var_os("LIVE_TESSDATA") {
        providers.push(Arc::new(kalamu_offline::Tesseract::new(
            kalamu_offline::ModelManager::new(dir.into(), &kalamu_offline::TESSDATA),
        )));
    }
    let engine = Engine::new(
        store.clone(),
        providers,
        Catalog::newest(None),
        prompts,
        Arc::new(CredentialKeys),
        EngineConfig {
            concurrency: 4,
            max_attempts: 2,
            ffmpeg,
            app_version: env!("CARGO_PKG_VERSION").into(),
            ..EngineConfig::default()
        },
    );
    Ok((engine, store))
}

/// Runs one job to the end and prints each piece; returns what it cost.
async fn run(
    engine: &Arc<Engine>,
    store: &SqliteStore,
    samples: &Path,
    out: &Path,
    plan: Plan,
) -> anyhow::Result<f64> {
    println!("\n== {} ({:?}) ==", plan.name, plan.settings.pace);
    let paths: Vec<PathBuf> = plan.files.iter().map(|f| samples.join(f)).collect();
    let mut sources = Vec::new();
    for inspected in engine.inspect(&paths) {
        let (source, warnings) = inspected?;
        for w in warnings {
            println!("  warning: {w:?}");
        }
        sources.push(source);
    }
    // The estimate New job would show, to set against what the providers billed.
    let estimate = engine.estimate(&EstimateRequest {
        task: plan.task,
        sources: sources.clone(),
        setups: plan.setups.clone(),
        sample_pages: plan.settings.sample_pages,
    });
    for row in &estimate.rows {
        println!(
            "  estimate {:<28} {} pages: ${:.4} (economy {})",
            row.model,
            row.pages,
            row.estimate.usd.unwrap_or(0.0),
            row.estimate
                .economy_usd
                .map_or("-".into(), |u| format!("${u:.4}"))
        );
    }
    let folder = out.join(plan.name);
    let job = engine.create_job(NewJob {
        name: plan.name.into(),
        task: plan.task,
        sources,
        setups: plan.setups,
        settings: plan.settings,
        output: OutputSettings {
            formats: plan.formats,
            destination: OutputDestination::Folder(folder.to_string_lossy().into_owned()),
            options: Default::default(),
        },
    })?;
    let mut events = engine.subscribe();
    engine.start(job.id)?;
    let mut spent = 0.0;
    let status = loop {
        match events.recv().await? {
            ProgressEvent::CostUpdated { job_id, spent_usd } if job_id == job.id => {
                spent = spent_usd;
            }
            ProgressEvent::SourceFailed { job_id, error, .. } if job_id == job.id => {
                println!("  source failed: {error}");
            }
            ProgressEvent::JobFinished { job_id, status } if job_id == job.id => break status,
            _ => {}
        }
    };
    for piece in engine.pieces(job.id)? {
        let setup = job.setups.iter().find(|s| s.id == piece.setup_id);
        let source = job.sources.iter().find(|s| s.id == piece.source_id);
        // What the provider reported, and the cost the catalogue puts on it.
        let billed = store.finished_result(&piece)?.map(|r| {
            let u = &r.output.usage;
            format!(
                " in {} (cached {}, written {}) out {} audio {:.0}s pages {} ${:.5}",
                u.input_tokens,
                u.cached_input_tokens,
                u.cache_write_tokens,
                u.output_tokens,
                u.audio_seconds,
                u.pages,
                r.cost_usd.unwrap_or(0.0)
            )
        });
        println!(
            "  {:<28} {:<16} #{} {:?}{}{}",
            setup.map_or("?", |s| s.label.as_str()),
            source.map_or("?", |s| s.file_name.as_str()),
            piece.index,
            piece.status,
            billed.unwrap_or_default(),
            piece
                .error
                .filter(|_| piece.status == PieceStatus::Failed)
                .map(|e| format!(": {e}"))
                .unwrap_or_default()
        );
    }
    println!(
        "  {status:?}, ${spent:.4}{}",
        if status == JobStatus::Completed {
            ""
        } else {
            "  <-- check"
        }
    );
    Ok(spent)
}
