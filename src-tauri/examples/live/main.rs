//! Runs real jobs against the live provider APIs, to check every provider end to end: by hand
//! before a release, and once a week from the *Live check* workflow (`.github/workflows/live.yml`).
//! It spends money and needs keys, so it never runs in CI's tests (only its own tests do, against
//! local stand-ins).
//!
//!   cargo run -p kalamu --example live -- <samples> <out> [step name filter]
//!
//! Sets (`LIVE_SET`):
//! - `weekly`: per provider, one page read, one short recording, one analysis, one analysis sent
//!   as a batch and collected, and one embedding, where the provider offers each, each on the
//!   cheapest model the catalogue offers for it; `<samples>` is the repository's `samples/`.
//! - `roadmap`: the roadmap's *Live runs still to do* that a run can make, on `samples/`.
//! - `release` (the default): every task end to end before a release; `<samples>` holds
//!   `newspaper.jpg`, `letter.pdf`, `interview.wav`, `article.txt` and the others `plans.rs`
//!   names. A job whose files are missing is left out.
//!
//! Keys: `KALAMU_<PROVIDER>_API_KEY` (`KALAMU_GEMINI_API_KEY`, `KALAMU_OPENAI_API_KEY`,
//! `KALAMU_ANTHROPIC_API_KEY`, `KALAMU_MISTRAL_API_KEY`) when set, else the keys saved in the app's
//! Settings (Credential Manager on Windows, the Keychain on macOS) unless `LIVE_KEYCHAIN=off`. A
//! provider without a key is left out. No key is ever printed or written.
//!
//! `LIVE_CEILING=<US dollars>`: the most the run may spend at the catalogue's prices (see
//! `ceiling.rs` for how it is kept); a run with nobody at the terminal refuses to start without
//! one, and one at a terminal asks first. `LIVE_PLAN=1` prints the steps, their models and their
//! estimates, and stops: no key is read, nothing is sent.
//!
//! Also: `LIVE_SKIP=gemini,openai` leaves out providers; `LIVE_PACE=economy` sends every job of
//! the `release` and `roadmap` sets through the providers' batch services; `LIVE_BATCH_WAIT=<min>`
//! is the longest a batch is waited for before it is cancelled at the provider (60 in the weekly
//! set, a day otherwise); `LIVE_FIXTURES=<folder>` keeps the providers' error answers there
//! (`proxy.rs`); `LIVE_SUMMARY=<file>` appends the run's table to that file (the workflow's job
//! summary), which is also written to `<out>/summary.md`. `LIVE_TESSDATA=<folder>` adds the
//! offline Tesseract jobs (the `fra` and `eng` data), `LIVE_WHISPER=<folder>` offline Whisper
//! (the app's Whisper models, and FFmpeg); both need no key and cost nothing.

mod ceiling;
mod keys;
mod plans;
mod proxy;
mod report;
mod run;
mod steps;

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use kalamu_catalog::Catalog;
use kalamu_core::{Embedder, Pace, Provider, ProviderId};
use kalamu_engine::{Engine, EngineConfig, EstimateRequest, KeySource};
use kalamu_prompts::PromptLibrary;
use kalamu_providers::{Anthropic, Gemini, Mistral, OpenAi};
use kalamu_store::SqliteStore;

use crate::keys::Keys;
use crate::proxy::{Fixtures, Relay};
use crate::report::{Row, Totals};
use crate::run::Runner;
use crate::steps::{Step, Work};

/// Each cloud provider's service, as its module's `BASE_URL` names it: its origin, and the path
/// the provider's requests start with. The relay sends `/<id>/…` to `<origin>/…`.
const SERVICES: [(&str, &str, &str); 4] = [
    (
        "gemini",
        "https://generativelanguage.googleapis.com",
        "/v1beta",
    ),
    ("openai", "https://api.openai.com", "/v1"),
    ("anthropic", "https://api.anthropic.com", "/v1"),
    ("mistral", "https://api.mistral.ai", "/v1"),
];

/// The cloud providers and embedders: the app's own registry, or, through the relay, the same
/// providers given its address.
type Cloud = (Vec<Arc<dyn Provider>>, Vec<Arc<dyn Embedder>>);

fn cloud(relay: Option<&Relay>) -> Cloud {
    let client = kalamu_providers::client();
    let Some(relay) = relay else {
        return (
            kalamu_providers::registry(client.clone()),
            kalamu_providers::embedders(client),
        );
    };
    let base = |id: &str| {
        let path = SERVICES
            .iter()
            .find(|(s, _, _)| *s == id)
            .map_or("", |(_, _, path)| path);
        format!("{}/{id}{path}", relay.address)
    };
    let gemini = Arc::new(Gemini::with_base_url(client.clone(), &base("gemini")));
    let openai = Arc::new(OpenAi::with_base_url(client.clone(), &base("openai")));
    let anthropic = Arc::new(Anthropic::with_base_url(client.clone(), &base("anthropic")));
    let mistral = Arc::new(Mistral::with_base_url(client, &base("mistral")));
    (
        vec![gemini.clone(), openai.clone(), anthropic, mistral.clone()],
        vec![gemini, openai, mistral],
    )
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [samples, out, rest @ ..] = args.as_slice() else {
        anyhow::bail!("usage: live <samples> <out> [step name filter]");
    };
    let (samples, out) = (PathBuf::from(samples), PathBuf::from(out));
    let filter = rest.first().cloned().unwrap_or_default();
    let set = env("LIVE_SET").unwrap_or_else(|| "release".into());
    if !["weekly", "roadmap", "release"].contains(&set.as_str()) {
        anyhow::bail!("LIVE_SET is weekly, roadmap or release, not {set:?}");
    }
    let plan_only = env("LIVE_PLAN").is_some_and(|v| v != "0");
    let interactive = std::io::stdin().is_terminal() && std::env::var_os("CI").is_none();
    let ceiling = if plan_only {
        None
    } else {
        ceiling::ceiling(env("LIVE_CEILING").as_deref(), interactive)
            .map_err(|e| anyhow::anyhow!("{e}"))?
    };
    std::fs::create_dir_all(&out)?;

    let skip = env("LIVE_SKIP").unwrap_or_default();
    let skip: Vec<&str> = skip
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let cloud_ids: Vec<ProviderId> = SERVICES
        .iter()
        .map(|(id, _, _)| ProviderId::new(*id))
        .filter(|id| !skip.contains(&id.as_str()))
        .collect();
    // Read once, before anything is sent; never in a plan.
    let keys = Arc::new(if plan_only {
        Keys::default()
    } else {
        let keychain = env("LIVE_KEYCHAIN").is_none_or(|v| v != "off");
        let read: keys::ReadKeychain = &Keys::system_keychain;
        Keys::gather(
            &cloud_ids,
            |n| std::env::var(n).ok(),
            keychain.then_some(read),
        )
    });
    for id in &cloud_ids {
        println!(
            "{:<10} {}",
            id.as_str(),
            match keys.origin(id) {
                Some(origin) => format!("key from the {}", origin.words()),
                None if plan_only => "not asked (plan)".into(),
                None => format!("no key ({} unset): left out", keys::env_name(id)),
            }
        );
    }

    let fixtures = match env("LIVE_FIXTURES") {
        Some(dir) if !plan_only => Some(Fixtures::new(PathBuf::from(dir), keys.redactor())?),
        _ => None,
    };
    let relay = match &fixtures {
        Some(fixtures) => {
            let routes = SERVICES
                .iter()
                .map(|(id, origin, _)| (id.to_string(), origin.to_string()))
                .collect();
            Some(Relay::start(routes, fixtures.clone(), kalamu_providers::client()).await?)
        }
        None => None,
    };
    let (providers, embedders) = cloud(relay.as_ref());
    let (engine, store) = engine(&out, providers, keys.clone())?;

    let tessdata = env("LIVE_TESSDATA").is_some();
    let whisper = env("LIVE_WHISPER").is_some();
    let economy = env("LIVE_PACE").is_some_and(|p| p == "economy");
    let mut steps = match set.as_str() {
        "weekly" => {
            let with_keys: Vec<ProviderId> = cloud_ids
                .iter()
                .filter(|id| plan_only || keys.key(id).is_some())
                .cloned()
                .collect();
            steps::weekly(&engine, &samples, &with_keys)
        }
        other => {
            let mut steps = if other == "roadmap" {
                plans::roadmap(whisper, tessdata)
            } else {
                plans::release(tessdata)
            };
            for step in &mut steps {
                if let Work::Job(plan) = &mut step.work {
                    plan.setups.retain(|s| !skip.contains(&s.provider.as_str()));
                    if economy {
                        plan.settings.pace = Pace::Economy;
                    }
                }
            }
            steps
        }
    };
    steps.retain(|s| s.name.contains(&filter));

    if plan_only {
        print_plan(&engine, &samples, &steps);
        return Ok(());
    }
    if ceiling.is_none() && !confirm_without_ceiling()? {
        anyhow::bail!("stopped: no ceiling");
    }
    let batch_wait = env("LIVE_BATCH_WAIT")
        .and_then(|m| m.parse::<u64>().ok())
        .unwrap_or(if set == "weekly" { 60 } else { 24 * 60 });
    let runner = Runner::new(
        engine.clone(),
        store,
        embedders,
        keys.clone(),
        samples,
        out.clone(),
        Duration::from_secs(batch_wait * 60),
        ceiling,
    );
    let rows = run_all(&runner, steps).await;

    let catalog = engine.catalog();
    let totals = Totals {
        set: set.clone(),
        ceiling,
        spent: runner.spent(),
        passed: runner.ledger_passed(),
        catalogue: format!("{} (revision {})", catalog.updated, catalog.revision),
        date: today(),
    };
    let table = report::markdown(&rows, &totals);
    println!("\n{table}");
    std::fs::write(out.join("summary.md"), &table)?;
    if let Some(summary) = env("LIVE_SUMMARY") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(summary)?;
        file.write_all(table.as_bytes())?;
    }
    if let Some(fixtures) = &fixtures {
        let kept = fixtures.recorded();
        println!("{} error answer(s) kept as fixtures.", kept.len());
    }
    let failed = rows.iter().filter(|r| r.outcome.fails_the_run()).count();
    if failed > 0 {
        anyhow::bail!("{failed} step(s) failed");
    }
    if runner.ledger_passed() {
        anyhow::bail!("what was billed passed the ceiling");
    }
    Ok(())
}

/// Runs every step: the batches started first, side by side, so that their wait overlaps the
/// rest; the rows in the steps' order.
async fn run_all(runner: &Arc<Runner>, steps: Vec<Step>) -> Vec<Row> {
    let mut waiting = Vec::new();
    let mut rows: Vec<(usize, Vec<Row>)> = Vec::new();
    let mut at_once = Vec::new();
    for (at, step) in steps.into_iter().enumerate() {
        if step.is_batch() {
            let runner = runner.clone();
            waiting.push((at, tokio::spawn(async move { runner.step(step).await })));
        } else {
            at_once.push((at, step));
        }
    }
    for (at, step) in at_once {
        rows.push((at, runner.step(step).await));
    }
    for (at, handle) in waiting {
        let got = handle.await.unwrap_or_else(|e| {
            vec![Row::new(
                "batch",
                "?",
                "?",
                report::Outcome::Failed(format!("the step stopped: {e}")),
            )]
        });
        rows.push((at, got));
    }
    rows.sort_by_key(|(at, _)| *at);
    rows.into_iter().flat_map(|(_, r)| r).collect()
}

/// Asks the person at the terminal whether to run with no ceiling.
fn confirm_without_ceiling() -> anyhow::Result<bool> {
    print!("No LIVE_CEILING: nothing caps what this run spends. Type yes to go on: ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(answer.trim().eq_ignore_ascii_case("yes"))
}

/// The steps, their models and their estimates, with no call and no key: what a weekly run
/// costs, by the week and the year.
fn print_plan(engine: &Engine, samples: &Path, steps: &[Step]) {
    let mut total = 0.0;
    let mut unpriced = 0;
    println!("\n| Step | Provider | Model | Estimate |\n| --- | --- | --- | ---: |");
    for step in steps {
        match &step.work {
            Work::Job(plan) => {
                let paths: Vec<PathBuf> = plan.files.iter().map(|f| samples.join(f)).collect();
                let sources: Vec<_> = engine
                    .inspect(&paths)
                    .into_iter()
                    .filter_map(Result::ok)
                    .map(|(s, _)| s)
                    .collect();
                if sources.len() != paths.len() {
                    println!("| {} | – | – | samples missing |", step.name);
                    continue;
                }
                let estimate = engine.estimate(&EstimateRequest {
                    task: plan.task,
                    sources,
                    setups: plan.setups.clone(),
                    sample_pages: plan.settings.sample_pages,
                });
                let economy = plan.settings.pace == Pace::Economy;
                let usd = if economy {
                    estimate.total_economy_usd
                } else {
                    estimate.total_usd
                };
                let models: Vec<&str> = plan.setups.iter().map(|s| s.model.as_str()).collect();
                let providers: Vec<&str> =
                    plan.setups.iter().map(|s| s.provider.as_str()).collect();
                match usd {
                    Some(u) => total += u,
                    None => unpriced += 1,
                }
                println!(
                    "| {} | {} | {}{} | {} |",
                    step.name,
                    providers.join(", "),
                    models.join(", "),
                    if economy { " (batch)" } else { "" },
                    usd.map_or("no price".into(), |u| format!("${u:.5}"))
                );
            }
            Work::Embedding {
                provider,
                model,
                price,
                file,
            } => {
                let chars = std::fs::read_to_string(samples.join(file))
                    .map(|t| t.chars().count().min(steps::EMBEDDING_CHARS))
                    .unwrap_or(0);
                let usd = price.map(|p| {
                    chars as f64 / kalamu_catalog::CHARS_PER_TOKEN as f64 * p / 1_000_000.0
                });
                if let Some(u) = usd {
                    total += u;
                }
                println!(
                    "| {} | {} | {model} | {} |",
                    step.name,
                    provider.as_str(),
                    usd.map_or("no price".into(), |u| format!("${u:.6}"))
                );
            }
            Work::Absent { provider, why } => {
                println!("| {} | {} | – | {why} |", step.name, provider.as_str());
            }
        }
    }
    println!(
        "\nIn all: ${total:.4} a run, ${:.2} a year if run weekly{}.",
        total * 52.0,
        if unpriced > 0 {
            format!(" ({unpriced} step(s) without a price not counted)")
        } else {
            String::new()
        }
    );
}

/// Today's date, `YYYY-MM-DD`, in UTC, from the system clock.
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = (secs / 86_400) as i64;
    // Howard Hinnant's days-to-civil.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn engine(
    out: &Path,
    mut providers: Vec<Arc<dyn Provider>>,
    keys: Arc<Keys>,
) -> anyhow::Result<(Arc<Engine>, Arc<SqliteStore>)> {
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
    if let Some(dir) = std::env::var_os("LIVE_TESSDATA") {
        providers.push(Arc::new(kalamu_offline::Tesseract::new(
            kalamu_offline::ModelManager::new(dir.into(), &kalamu_offline::TESSDATA),
        )));
    }
    if let Some(dir) = std::env::var_os("LIVE_WHISPER") {
        providers.push(Arc::new(kalamu_offline::Whisper::new(
            kalamu_offline::ModelManager::new(dir.into(), &kalamu_offline::WHISPER),
        )));
    }
    let engine = Engine::new(
        store.clone(),
        providers,
        Catalog::newest(None),
        prompts,
        keys,
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

#[cfg(test)]
mod tests;
