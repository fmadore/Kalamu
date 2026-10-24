//! User preferences, stored by Rust in the app data folder (not the WebView's `localStorage`),
//! so the engine and the interface read the same values.

use std::path::Path;

use scriptorium_core::{ExportFormat, ExportOptions, OutputDestination};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    /// Pages sent at the same time; applies from the next launch.
    pub concurrency: u32,
    pub text_formats: Vec<ExportFormat>,
    pub audio_formats: Vec<ExportFormat>,
    pub destination: OutputDestination,
    pub export_options: ExportOptions,
    /// `system`, `light` or `dark`.
    pub theme: String,
    /// Check the repository for a newer model catalogue at startup.
    pub refresh_catalog: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            concurrency: 2,
            text_formats: vec![ExportFormat::Txt, ExportFormat::Docx, ExportFormat::Json],
            audio_formats: vec![
                ExportFormat::Txt,
                ExportFormat::Docx,
                ExportFormat::Srt,
                ExportFormat::Json,
            ],
            destination: OutputDestination::BesideSources,
            export_options: ExportOptions::default(),
            theme: "system".into(),
            refresh_catalog: true,
        }
    }
}

impl Settings {
    /// Missing or unreadable settings fall back to defaults rather than blocking startup.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension("json.pending");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)
    }
}
