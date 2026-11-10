//! A launch that cannot finish: the data folder, the job database or the saved libraries could
//! not be opened. The interface does not exist yet, so the researcher is told in a native
//! message, which names the cause and the logs folder, and the app ends.
//!
//! The one place the shell words text itself: every other sentence the researcher reads comes
//! from the interface's translation files, which are not loaded at this point. The few sentences
//! below are kept in each language the interface has, chosen from the system's languages as the
//! interface chooses its own when set to follow the system.

use std::fmt;
use std::path::Path;

/// What stopped the launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cause {
    /// The data folder or the logs folder could not be found or made.
    Folders,
    /// The job database could not be opened: a full disk, a damaged file, no permission.
    Database,
    /// The job database was written by a newer version of Kalamu.
    NewerDatabase,
    /// The saved prompts, pipelines, workflows or export profiles could not be opened.
    Libraries,
}

/// A launch that failed: what stopped it, and the technical detail for the logs and the
/// message's last line.
#[derive(Debug)]
pub(crate) struct StartError {
    pub cause: Cause,
    pub detail: String,
}

impl StartError {
    pub fn new(cause: Cause, detail: impl fmt::Display) -> Self {
        Self {
            cause,
            detail: detail.to_string(),
        }
    }

    /// A failure to open the job database, told apart when a newer version wrote it.
    pub fn database(error: kalamu_core::StoreError) -> Self {
        let cause = match error {
            kalamu_core::StoreError::Newer { .. } => Cause::NewerDatabase,
            _ => Cause::Database,
        };
        Self::new(cause, error)
    }
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.cause, self.detail)
    }
}

impl std::error::Error for StartError {}

/// The languages the message is written in, as the interface's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    French,
}

/// The first of the system's languages (`fr-FR`, `en_US`…) the message is written in, else
/// English: the interface's own rule (`resolveLocale`).
fn language(system: &[String]) -> Language {
    system
        .iter()
        .find_map(|tag| {
            let base = tag.split(['-', '_']).next()?.to_ascii_lowercase();
            match base.as_str() {
                "en" => Some(Language::English),
                "fr" => Some(Language::French),
                _ => None,
            }
        })
        .unwrap_or(Language::English)
}

/// The message's title and text.
fn message(cause: Cause, detail: &str, logs: Option<&Path>, system: &[String]) -> (String, String) {
    let english = language(system) == Language::English;
    let title = if english {
        "Kalamu could not start"
    } else {
        "Kalamu n’a pas pu démarrer"
    };
    let what = match (cause, english) {
        (Cause::Folders, true) => {
            "Kalamu could not find or create its data folder. The disk may be full, or the \
             folder may not allow writing."
        }
        (Cause::Folders, false) => {
            "Kalamu n’a pas pu trouver ou créer son dossier des données. Le disque est peut-être \
             plein, ou le dossier n’autorise pas l’écriture."
        }
        (Cause::Database, true) => {
            "Kalamu could not open its job database (jobs.sqlite). The disk may be full, or the \
             file may be damaged."
        }
        (Cause::Database, false) => {
            "Kalamu n’a pas pu ouvrir sa base de données des traitements (jobs.sqlite). Le \
             disque est peut-être plein, ou le fichier est peut-être endommagé."
        }
        (Cause::NewerDatabase, true) => {
            "Your job database (jobs.sqlite) was last used by a newer version of Kalamu. This \
             version cannot read it safely, so it has left it as it was. Install the newer \
             version again to open your jobs."
        }
        (Cause::NewerDatabase, false) => {
            "Votre base de données des traitements (jobs.sqlite) a été utilisée en dernier par \
             une version plus récente de Kalamu. Cette version ne peut pas la lire sans risque \
             et l’a laissée telle quelle. Installez de nouveau la version plus récente pour \
             retrouver vos traitements."
        }
        (Cause::Libraries, true) => {
            "Kalamu could not open its saved prompts, pipelines, workflows or export profiles. \
             The disk may be full, or a file may be damaged."
        }
        (Cause::Libraries, false) => {
            "Kalamu n’a pas pu ouvrir ses prompts, configurations, chaînes de traitement ou \
             profils d’export enregistrés. Le disque est peut-être plein, ou un fichier est \
             peut-être endommagé."
        }
    };
    let details = if english { "Details" } else { "Détails" };
    // French sets a no-break space before a colon, as the interface does.
    let colon = if english { ":" } else { "\u{a0}:" };
    let mut text = format!("{what}\n\n{details}{colon} {detail}");
    if let Some(logs) = logs {
        let logs = crate::platform::shown_path(logs);
        let line = if english {
            format!("Kalamu’s logs are in {}", logs.display())
        } else {
            format!("Les journaux de Kalamu sont dans {}", logs.display())
        };
        text.push_str("\n\n");
        text.push_str(&line);
    }
    (title.to_owned(), text)
}

/// Tells the researcher why the app cannot start, and waits until they close the message. Runs
/// on the main thread during Tauri's setup, before the interface exists, so it uses the
/// system's own message box (a task dialog on Windows, an alert on macOS) directly rather than
/// the dialog plugin, whose blocking calls wait for the main thread.
pub(crate) fn tell(failure: &StartError, logs: Option<&Path>) {
    let system: Vec<String> = sys_locale::get_locales().collect();
    let (title, text) = message(failure.cause, &failure.detail, logs, &system);
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(title)
        .set_description(text)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|t| (*t).to_owned()).collect()
    }

    #[test]
    fn the_first_system_language_the_message_has_is_used() {
        assert_eq!(language(&tags(&["fr-FR", "en-US"])), Language::French);
        assert_eq!(language(&tags(&["de-DE", "fr_CA"])), Language::French);
        assert_eq!(language(&tags(&["en-GB", "fr-FR"])), Language::English);
        assert_eq!(language(&tags(&["de-DE"])), Language::English);
        assert_eq!(language(&[]), Language::English);
    }

    #[test]
    fn the_message_names_the_cause_the_detail_and_the_logs() {
        let logs = Path::new("logs-folder");
        let (title, text) = message(
            Cause::NewerDatabase,
            "schema 30",
            Some(logs),
            &tags(&["en-US"]),
        );
        assert_eq!(title, "Kalamu could not start");
        assert!(text.contains("newer version of Kalamu"), "{text}");
        assert!(text.contains("Details: schema 30"), "{text}");
        assert!(text.contains("logs-folder"), "{text}");

        let (title, text) = message(Cause::Database, "disk full", None, &tags(&["fr-FR"]));
        assert_eq!(title, "Kalamu n’a pas pu démarrer");
        assert!(text.contains("jobs.sqlite"), "{text}");
        assert!(text.contains("Détails\u{a0}: disk full"), "{text}");
        assert!(!text.contains("journaux"), "{text}");
    }

    #[test]
    fn a_newer_database_is_told_apart() {
        let newer = StartError::database(kalamu_core::StoreError::Newer {
            found: 30,
            known: 29,
        });
        assert_eq!(newer.cause, Cause::NewerDatabase);
        let other = StartError::database(kalamu_core::StoreError::Database("disk I/O".into()));
        assert_eq!(other.cause, Cause::Database);
    }
}
