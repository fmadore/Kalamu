//! User preferences, stored by Rust in the app data folder (not the WebView's `localStorage`),
//! so the engine and the interface read the same values.

use std::path::{Path, PathBuf};

use kalamu_core::{ExportFormat, ExportOptions, LanguageCode, OutputDestination, ProviderId};
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
    /// The interface language: `system` (the first of the system's languages Kalamu has, else
    /// English), `en` or `fr`.
    pub language: String,
    /// Check the repository for a newer model catalogue at startup.
    pub refresh_catalog: bool,
    /// Languages starred in New job, listed first.
    pub favourite_languages: Vec<LanguageCode>,
    /// The folder last chosen for results, offered first when a job needs one.
    pub last_folder: Option<String>,
    /// The cloud providers the researcher agreed to send their files to, asked once each before
    /// the first job, run or proposal sent to it (App Review Guideline 5.1.2(i), and plain
    /// courtesy on Windows too). Only ever added to.
    pub ai_consent: Vec<ProviderId>,
    /// Offline OCR models read on the graphics processor when Kalamu finds one it can use
    /// (Windows, through Vulkan); off, they read on the processor.
    pub ocr_graphics_processor: bool,
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
            language: "system".into(),
            refresh_catalog: true,
            favourite_languages: Vec::new(),
            last_folder: None,
            ai_consent: Vec::new(),
            ocr_graphics_processor: true,
        }
    }
}

impl Settings {
    /// The saved settings, or the defaults when there are none. A file that cannot be read or
    /// understood does not block the launch, and is not overwritten at the next save either: it
    /// is moved aside ([`set_aside_name`]), and where it went is returned for the interface to
    /// say once.
    pub fn load(path: &Path) -> (Self, Option<PathBuf>) {
        let failure = match std::fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(settings) => return (settings, None),
                Err(e) => e.to_string(),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Self::default(), None),
            Err(e) => e.to_string(),
        };
        let aside = set_aside_name(path, |p| p.symlink_metadata().is_ok());
        match std::fs::rename(path, &aside) {
            Ok(()) => {
                tracing::warn!(
                    error = %failure,
                    "the settings could not be read: the defaults are in use, the file is kept as {}",
                    aside.display()
                );
                (Self::default(), Some(aside))
            }
            Err(e) => {
                tracing::warn!(
                    error = %failure,
                    "the settings could not be read, nor moved aside ({e}): the defaults are in use"
                );
                (Self::default(), None)
            }
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        write_json(path, self)
    }
}

/// Where a settings file that cannot be read is moved: `settings.json.bad` beside it, or, when an
/// earlier one is there (`taken`), the first free of `settings.json.2.bad`,
/// `settings.json.3.bad`…, so no earlier file is lost either.
fn set_aside_name(path: &Path, taken: impl Fn(&Path) -> bool) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "settings.json".into(), |n| n.to_string_lossy());
    (1..=999)
        .map(|n| match n {
            1 => path.with_file_name(format!("{name}.bad")),
            n => path.with_file_name(format!("{name}.{n}.bad")),
        })
        .find(|candidate| !taken(candidate))
        .unwrap_or_else(|| path.with_file_name(format!("{name}.bad")))
}

/// Writes `value` as JSON whole through a temporary file beside `path`, so a crash never leaves
/// half a file.
pub(crate) fn write_json(path: &Path, value: &impl Serialize) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
    let tmp = path.with_extension("json.pending");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn saved_settings_are_read_and_missing_ones_are_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        assert_eq!(Settings::load(&path), (Settings::default(), None));
        let settings = Settings {
            concurrency: 5,
            ..Settings::default()
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), (settings, None));
    }

    /// An unreadable file is moved aside rather than overwritten at the next save, and an
    /// earlier one moved aside is kept too.
    #[test]
    fn an_unreadable_file_is_kept_beside_without_replacing_an_earlier_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ not json").unwrap();
        let (settings, aside) = Settings::load(&path);
        assert_eq!(settings, Settings::default());
        assert_eq!(aside, Some(dir.path().join("settings.json.bad")));
        assert!(!path.exists());

        std::fs::write(&path, r#"{ "concurrency": "many" }"#).unwrap();
        let (_, aside) = Settings::load(&path);
        assert_eq!(aside, Some(dir.path().join("settings.json.2.bad")));
        assert_eq!(
            std::fs::read_to_string(dir.path().join("settings.json.bad")).unwrap(),
            "{ not json"
        );
        // Nothing is left in its place: the next launch starts from the defaults without a word.
        assert_eq!(Settings::load(&path), (Settings::default(), None));
    }

    #[test]
    fn the_name_set_aside_is_the_first_free_one() {
        let path = Path::new("data").join("settings.json");
        let taken = [
            Path::new("data").join("settings.json.bad"),
            Path::new("data").join("settings.json.2.bad"),
        ];
        assert_eq!(
            set_aside_name(&path, |_| false),
            Path::new("data").join("settings.json.bad")
        );
        assert_eq!(
            set_aside_name(&path, |p| taken.iter().any(|t| t == p)),
            Path::new("data").join("settings.json.3.bad")
        );
    }
}
