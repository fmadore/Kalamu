//! Runs real jobs against the live provider APIs, with the keys saved in the app's Settings
//! (Windows Credential Manager), to check every task end to end before a release. Never run in
//! CI: it spends money and needs keys.
//!
//!   cargo run -p kalamu --example live -- <samples> <out> [job name filter]
//!
//! `<samples>` holds `newspaper.jpg`, `letter.pdf`, `interview.wav` and `article.txt`; each job
//! writes its outputs to its own folder in `<out>` and the run ends with a table of pieces.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use kalamu_catalog::Catalog;
use kalamu_core::{
    ApiKey, ExportFormat, JobSettings, JobStatus, LineBreaks, OutputDestination, OutputSettings,
    PieceStatus, ProgressEvent, ProviderId, SetupOptions, Task,
};
use kalamu_engine::{Engine, EngineConfig, KeySource, NewJob, SetupRequest};
use kalamu_prompts::PromptLibrary;
use kalamu_store::SqliteStore;

/// The same entries the app writes (`src/secrets.rs`).
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

    let engine = engine(&out)?;
    let mut total = 0.0;
    // LIVE_SKIP=gemini,openai leaves out providers without credit.
    let skip = std::env::var("LIVE_SKIP").unwrap_or_default();
    let skip: Vec<&str> = skip.split(',').filter(|s| !s.is_empty()).collect();
    for mut plan in plans().into_iter().filter(|p| p.name.contains(&filter)) {
        plan.setups.retain(|s| !skip.contains(&s.provider.as_str()));
        if plan.setups.is_empty() {
            continue;
        }
        total += run(&engine, &samples, &out, plan).await?;
    }
    println!("\nSpent about ${total:.4} in all.");
    Ok(())
}

fn engine(out: &Path) -> anyhow::Result<Arc<Engine>> {
    let data = out.join("data");
    std::fs::create_dir_all(&data)?;
    let store = SqliteStore::open(&data.join("jobs.sqlite"))?;
    let prompts = PromptLibrary::open(&data.join("presets"))?;
    // The installed app's FFmpeg, or KALAMU_FFMPEG_DIR.
    let installed = std::env::var_os("LOCALAPPDATA")
        .map(|d| PathBuf::from(d).join("Kalamu"))
        .into_iter()
        .collect::<Vec<_>>();
    let ffmpeg = kalamu_media::Ffmpeg::locate(&installed);
    println!(
        "FFmpeg: {}",
        if ffmpeg.is_some() { "found" } else { "missing" }
    );
    Ok(Engine::new(
        Arc::new(store),
        kalamu_providers::registry(kalamu_providers::client()),
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
    ))
}

/// Runs one job to the end and prints each piece; returns what it cost.
async fn run(engine: &Arc<Engine>, samples: &Path, out: &Path, plan: Plan) -> anyhow::Result<f64> {
    println!("\n== {} ==", plan.name);
    let paths: Vec<PathBuf> = plan.files.iter().map(|f| samples.join(f)).collect();
    let mut sources = Vec::new();
    for inspected in engine.inspect(&paths) {
        let (source, warnings) = inspected?;
        for w in warnings {
            println!("  warning: {w:?}");
        }
        sources.push(source);
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
        println!(
            "  {:<28} {:<16} #{} {:?}{}",
            setup.map_or("?", |s| s.label.as_str()),
            source.map_or("?", |s| s.file_name.as_str()),
            piece.index,
            piece.status,
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
