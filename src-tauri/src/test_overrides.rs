//! What the test build changes, and nothing else does: the app driven in CI against the stand-in
//! provider (`scripts/app-driven/stand-in.mjs`, `docs/decisions/2026-11-25-app-driven-in-ci.md`). Compiled only
//! with the `test-overrides` feature, which no release build enables, and refused in any build
//! that is not a debug build; `tests/test_build.rs` checks that no workflow or default turns it on.
//!
//! Five things:
//!
//! - [`DATA`]: the data folder, so a test run never reads or writes a researcher's own;
//! - [`PROVIDERS`]: an address every cloud provider is reached under (`<address>/<provider>`),
//!   handed to the registry as [`kalamu_providers::Addresses`];
//! - the keys: the keychain is never asked or written in this build. Each cloud provider holds
//!   [`KEY`] in memory when [`PROVIDERS`] is set, and none otherwise; a key saved in Settings is
//!   kept in memory until the app ends;
//! - the native dialogs ([`dialogs`]), answered from a queue in the data folder, since no
//!   webview driver reaches a system window. The single-instance plugin is left out too, so a
//!   test build runs beside an installed Kalamu instead of focusing it;
//! - [`DEVTOOLS_PORT`]: the port the window's WebView2 opens the DevTools protocol on, which the
//!   driver connects to. The app asks for it itself: WebView2's own variable
//!   (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`) is not read on GitHub's runners, whose browser
//!   process starts without what it names.

#[cfg(not(debug_assertions))]
compile_error!(
    "the `test-overrides` feature is for debug test builds only: a release build must never carry it"
);

use std::collections::HashMap;
use std::path::PathBuf;

use kalamu_core::ProviderId;
use kalamu_providers::Addresses;

/// The data folder of a test run: an absolute path, made if needed.
pub const DATA: &str = "KALAMU_TEST_DATA";
/// The stand-in provider's address (`http://127.0.0.1:<port>`), under which each cloud provider
/// is reached at `/<provider id>`.
pub const PROVIDERS: &str = "KALAMU_TEST_PROVIDERS";
/// The key every cloud provider holds when [`PROVIDERS`] is set: the stand-in accepts it, and
/// no real service would.
pub const KEY: &str = "stand-in-key";

/// The port WebView2 opens the DevTools protocol on, for the driver (Windows).
pub const DEVTOOLS_PORT: &str = "KALAMU_TEST_DEVTOOLS_PORT";

/// The arguments the window's WebView2 is started with when [`DEVTOOLS_PORT`] names a port:
/// the ones Tauri gives by default, which arguments of the app's own replace, then the port.
pub fn browser_arguments() -> Option<String> {
    let port: u16 = std::env::var(DEVTOOLS_PORT).ok()?.trim().parse().ok()?;
    Some(format!(
        "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --remote-debugging-port={port}"
    ))
}

/// The data folder [`DATA`] names, if it names an absolute one.
pub fn data_folder() -> Option<PathBuf> {
    let folder = PathBuf::from(std::env::var_os(DATA)?);
    if !folder.is_absolute() {
        eprintln!("{DATA} is not an absolute path; the usual data folder is used");
        return None;
    }
    Some(folder)
}

/// The stand-in's address, without a trailing slash, if [`PROVIDERS`] is set.
fn stand_in() -> Option<String> {
    let url = std::env::var(PROVIDERS).ok()?;
    let url = url.trim().trim_end_matches('/');
    (!url.is_empty()).then(|| url.to_string())
}

/// The cloud providers' ids, as the registry lists them.
fn cloud_providers() -> Vec<ProviderId> {
    kalamu_providers::registry(reqwest::Client::new())
        .iter()
        .map(|p| p.id())
        .collect()
}

/// Where each cloud provider is reached: under the stand-in when [`PROVIDERS`] is set, else at
/// its own service.
pub fn addresses() -> Addresses {
    let Some(url) = stand_in() else {
        return Addresses::default();
    };
    cloud_providers()
        .into_iter()
        .fold(Addresses::default(), |addresses, id| {
            let base = format!("{url}/{}", id.as_str());
            addresses.with(id.as_str(), base)
        })
}

/// The file in the test data folder whose answers the native dialogs give, in order: a JSON array
/// the driver writes before the action that asks (see [`dialogs`]).
pub const DIALOG_ANSWERS: &str = "dialog-answers.json";

/// The dialog plugin of the test build, under the same name and commands as Tauri's
/// (`plugin:dialog|open`, `save`, `message`), so the interface and the capability are unchanged:
/// each call takes the next answer from [`DIALOG_ANSWERS`] instead of opening a native window no
/// webview driver can reach. A file picker's answer is a path or a list of paths, a save
/// dialog's a path, and a question's `true` for its OK button or `false` for its other; null is
/// a dialog cancelled. A call with no answer queued is refused, failing the flow where it asked.
pub fn dialogs<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("dialog")
        .invoke_handler(tauri::generate_handler![open, save, message])
        .build()
}

/// The next queued answer, taken out of the file.
fn next_answer(command: &str) -> Result<serde_json::Value, String> {
    let folder = data_folder().ok_or_else(|| format!("{DATA} is not set"))?;
    let path = folder.join(DIALOG_ANSWERS);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| "[]".into());
    let mut queue: Vec<serde_json::Value> =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if queue.is_empty() {
        tracing::warn!(command, "test build: a dialog asked with no answer queued");
        return Err(format!("no answer queued for the dialog's {command}"));
    }
    let answer = queue.remove(0);
    let rest = serde_json::to_string(&queue).map_err(|e| e.to_string())?;
    std::fs::write(&path, rest).map_err(|e| format!("{}: {e}", path.display()))?;
    tracing::info!(command, %answer, "test build: a dialog answered from the queue");
    Ok(answer)
}

#[tauri::command]
fn open() -> Result<serde_json::Value, String> {
    next_answer("open")
}

#[tauri::command]
fn save() -> Result<serde_json::Value, String> {
    next_answer("save")
}

/// A message or a question: the label of the button the queued answer stands for, as the
/// plugin returns it (`ask` compares it with its OK label).
#[tauri::command]
fn message(buttons: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let answer = next_answer("message")?;
    let Some(yes) = answer.as_bool() else {
        return Ok(answer);
    };
    let label = match buttons.as_ref() {
        Some(serde_json::Value::String(kind)) => match (kind.as_str(), yes) {
            ("YesNo" | "YesNoCancel", true) => "Yes",
            ("YesNo" | "YesNoCancel", false) => "No",
            ("OkCancel", false) => "Cancel",
            _ => "Ok",
        }
        .to_string(),
        Some(custom) => {
            // `{ ok, cancel }` or `{ yes, no, cancel }`, as the plugin's JavaScript sends them.
            let pick = |keys: &[&str]| {
                keys.iter()
                    .find_map(|k| custom.get(k).and_then(serde_json::Value::as_str))
                    .map(str::to_string)
            };
            if yes {
                pick(&["ok", "yes"]).unwrap_or_else(|| "Ok".into())
            } else {
                pick(&["cancel", "no"]).unwrap_or_else(|| "Cancel".into())
            }
        }
        None => if yes { "Ok" } else { "Cancel" }.to_string(),
    };
    Ok(serde_json::Value::String(label))
}

/// The keys held in memory from the start: [`KEY`] for each cloud provider when the stand-in is
/// used, and for every other provider none, so the keychain is never asked.
pub fn keys() -> HashMap<ProviderId, Option<String>> {
    let key = stand_in().map(|_| KEY.to_string());
    cloud_providers()
        .into_iter()
        .map(|id| (id, key.clone()))
        .collect()
}
