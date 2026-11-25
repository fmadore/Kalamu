//! A launch that cannot finish: the data folder, the job database or the saved libraries could
//! not be opened, or the window could not be made (Tauri itself, or its webview, failing). The interface does not exist yet, so the researcher is told in a native
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
    /// The window or its webview could not be made, or Tauri itself could not start: WebView2
    /// missing or damaged on Windows, more rarely the system's WebKit on macOS.
    Window,
}

/// What draws the window's contents, named in the message when it cannot be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "each system's build names only its own webview (`platform::WEBVIEW`)"
)]
pub(crate) enum Webview {
    /// Microsoft Edge WebView2, which Windows 11 includes and the installer sets up on Windows 10.
    WebView2,
    /// The system's own WebKit, part of macOS.
    WebKit,
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

/// What to do when the window cannot be made, for each system's webview.
fn window_advice(webview: Webview, english: bool) -> &'static str {
    match (webview, english) {
        (Webview::WebView2, true) => {
            "Kalamu could not open its window. Windows draws it with Microsoft Edge WebView2, \
             which may be missing or damaged: install the WebView2 Runtime from Microsoft's \
             website, or repair it from Installed apps in Settings, then open Kalamu again."
        }
        (Webview::WebView2, false) => {
            "Kalamu n’a pas pu ouvrir sa fenêtre. Windows l’affiche avec Microsoft Edge \
             WebView2, qui manque peut-être ou est endommagé\u{a0}: installez WebView2 Runtime \
             depuis le site de Microsoft, ou réparez-le depuis Applications installées dans \
             Paramètres, puis rouvrez Kalamu."
        }
        (Webview::WebKit, true) => {
            "Kalamu could not open its window, which macOS draws with its own WebKit. Restart \
             the Mac, or update macOS, then open Kalamu again."
        }
        (Webview::WebKit, false) => {
            "Kalamu n’a pas pu ouvrir sa fenêtre, que macOS affiche avec son propre WebKit. \
             Redémarrez le Mac, ou mettez macOS à jour, puis rouvrez Kalamu."
        }
    }
}

/// The message's title and text.
fn message(cause: Cause, detail: &str, logs: Option<&Path>, system: &[String]) -> (String, String) {
    message_for(cause, crate::platform::WEBVIEW, detail, logs, system)
}

/// The message's title and text, for the system whose webview is `webview`.
fn message_for(
    cause: Cause,
    webview: Webview,
    detail: &str,
    logs: Option<&Path>,
    system: &[String],
) -> (String, String) {
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
        (Cause::Window, english) => window_advice(webview, english),
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

/// Asks the researcher what to do with a job database that cannot be read, in a native
/// message as [`tell`] does: open the newest copy in its place, or, without one, start with
/// an empty job list; either way the file is set aside in `folder`, never deleted. Or quit,
/// which changes nothing.
pub(crate) fn ask_damaged(
    damage: &kalamu_store::protect::Damage,
    folder: &Path,
    logs: Option<&Path>,
) -> kalamu_store::protect::Choice {
    use kalamu_store::protect::Choice;
    let system: Vec<String> = sys_locale::get_locales().collect();
    let asked = damaged_message(damage, folder, logs, &system);
    let answer = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(&asked.title)
        .set_description(&asked.text)
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            asked.go.clone(),
            asked.quit.clone(),
        ))
        .show();
    let went = match answer {
        rfd::MessageDialogResult::Ok | rfd::MessageDialogResult::Yes => true,
        rfd::MessageDialogResult::Custom(label) => label == asked.go,
        _ => false,
    };
    match (went, damage.latest.is_some()) {
        (false, _) => Choice::Quit,
        (true, true) => Choice::UseCopy,
        (true, false) => Choice::StartEmpty,
    }
}

/// The question about a damaged database: its title and text, and its two buttons.
struct Asked {
    title: String,
    text: String,
    /// The button that sets the file aside and goes on.
    go: String,
    quit: String,
}

fn damaged_message(
    damage: &kalamu_store::protect::Damage,
    folder: &Path,
    logs: Option<&Path>,
    system: &[String],
) -> Asked {
    let english = language(system) == Language::English;
    let folder = folder.display();
    let (title, what, go) = match (&damage.latest, english) {
        (Some(copy), true) => (
            "Kalamu’s job database cannot be read",
            format!(
                "Kalamu’s job database (jobs.sqlite) is damaged and cannot be read. Kalamu can \
                 put it aside, without deleting it, and open in its place the copy it made on \
                 {}. Work done since that copy is not in it.\n\nThe damaged file will be kept \
                 in {folder}",
                day(copy.made, true)
            ),
            "Open the copy",
        ),
        (Some(copy), false) => (
            "La base de données de Kalamu ne peut pas être lue",
            format!(
                "La base de données des traitements de Kalamu (jobs.sqlite) est endommagée et \
                 ne peut pas être lue. Kalamu peut la mettre de côté, sans l’effacer, et ouvrir \
                 à sa place la copie faite le {}. Le travail fait depuis cette copie n’y est \
                 pas.\n\nLe fichier endommagé sera gardé dans {folder}",
                day(copy.made, false)
            ),
            "Ouvrir la copie",
        ),
        (None, true) => (
            "Kalamu’s job database cannot be read",
            format!(
                "Kalamu’s job database (jobs.sqlite) is damaged and cannot be read, and there \
                 is no copy of it yet. Kalamu can put it aside, without deleting it, and start \
                 with an empty list of jobs.\n\nThe damaged file will be kept in {folder}"
            ),
            "Start with an empty list",
        ),
        (None, false) => (
            "La base de données de Kalamu ne peut pas être lue",
            format!(
                "La base de données des traitements de Kalamu (jobs.sqlite) est endommagée et \
                 ne peut pas être lue, et il n’en existe encore aucune copie. Kalamu peut la \
                 mettre de côté, sans l’effacer, et démarrer avec une liste de traitements \
                 vide.\n\nLe fichier endommagé sera gardé dans {folder}"
            ),
            "Démarrer avec une liste vide",
        ),
    };
    let (details, quit) = if english {
        ("Details:", "Quit")
    } else {
        ("Détails\u{a0}:", "Quitter")
    };
    let mut text = format!("{what}\n\n{details} {}", damage.reason);
    if let Some(logs) = logs {
        let logs = crate::platform::shown_path(logs);
        text.push_str("\n\n");
        text.push_str(&if english {
            format!("Kalamu’s logs are in {}", logs.display())
        } else {
            format!("Les journaux de Kalamu sont dans {}", logs.display())
        });
    }
    Asked {
        title: title.to_owned(),
        text,
        go: go.to_owned(),
        quit: quit.to_owned(),
    }
}

/// A copy's day, in UTC, in words: `24 November 2026`, `24 novembre 2026`.
fn day(made: kalamu_core::Timestamp, english: bool) -> String {
    const EN: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const FR: [&str; 12] = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let date = made.utc_date();
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(d)) = (parts.next(), parts.next(), parts.next()) else {
        return date;
    };
    let month = month.parse::<usize>().ok().and_then(|m| {
        (if english { EN } else { FR })
            .get(m.wrapping_sub(1))
            .copied()
    });
    match (month, d.parse::<u32>()) {
        (Some(month), Ok(d)) => format!("{d} {month} {year}"),
        _ => date,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn a_damaged_database_s_question_offers_the_copy_or_an_empty_start() {
        use kalamu_store::protect::{Copy, CopyKind, Damage};
        let copy = Copy {
            name: "jobs-daily-2026-11-24-120000.sqlite".into(),
            kind: CopyKind::Daily,
            made: kalamu_core::Timestamp(1_795_521_600_000),
            bytes: 10,
        };
        let damage = Damage {
            reason: "file is not a database".into(),
            latest: Some(copy),
        };
        let folder = Path::new("backups-folder");
        let asked = damaged_message(
            &damage,
            folder,
            Some(Path::new("logs-folder")),
            &tags(&["en-GB"]),
        );
        assert_eq!(asked.go, "Open the copy");
        assert_eq!(asked.quit, "Quit");
        assert!(asked.text.contains("24 November 2026"), "{}", asked.text);
        assert!(asked.text.contains("backups-folder"), "{}", asked.text);
        assert!(asked.text.contains("without deleting it"), "{}", asked.text);
        assert!(
            asked.text.contains("Details: file is not a database"),
            "{}",
            asked.text
        );
        assert!(asked.text.contains("logs-folder"), "{}", asked.text);

        let none = Damage {
            latest: None,
            ..damage
        };
        let asked = damaged_message(&none, folder, None, &tags(&["fr-FR"]));
        assert_eq!(asked.go, "Démarrer avec une liste vide");
        assert_eq!(asked.quit, "Quitter");
        assert!(asked.text.contains("sans l’effacer"), "{}", asked.text);
        assert!(!asked.text.contains("journaux"), "{}", asked.text);
        assert_eq!(
            day(kalamu_core::Timestamp(1_795_521_600_000), false),
            "24 novembre 2026"
        );
    }

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
    fn a_window_that_cannot_be_made_names_each_system_s_webview() {
        let logs = Path::new("logs-folder");
        let (_, text) = message_for(
            Cause::Window,
            Webview::WebView2,
            "WebView2 not found",
            Some(logs),
            &tags(&["en-US"]),
        );
        assert!(text.contains("Microsoft Edge WebView2"), "{text}");
        assert!(text.contains("Installed apps"), "{text}");
        assert!(text.contains("Details: WebView2 not found"), "{text}");
        assert!(text.contains("logs-folder"), "{text}");

        let (title, text) = message_for(
            Cause::Window,
            Webview::WebKit,
            "no webview",
            None,
            &tags(&["fr-FR"]),
        );
        assert_eq!(title, "Kalamu n’a pas pu démarrer");
        assert!(text.contains("WebKit"), "{text}");
        assert!(text.contains("macOS"), "{text}");
        assert!(!text.contains("WebView2"), "{text}");

        // The message this system shows is its own webview's.
        let (_, here) = message(Cause::Window, "x", None, &tags(&["en-US"]));
        let mine = if cfg!(windows) { "WebView2" } else { "WebKit" };
        assert!(here.contains(mine), "{here}");
    }

    #[test]
    fn the_logs_folder_is_known_before_tauri_starts() {
        use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
        let root = PathBuf::from("Kalamu Data");
        assert_eq!(
            crate::platform::log_folder(&AppDirectoriesOverride::Root(root.clone())),
            Some(root.join("logs"))
        );
        let log = PathBuf::from("Logs").join("Kalamu");
        let dirs = AppDirectoriesOverride::Directories(AppDirectoryOverrides {
            config: None,
            data: None,
            local_data: None,
            cache: None,
            log: Some(log.clone()),
        });
        assert_eq!(crate::platform::log_folder(&dirs), Some(log));
        // A folder Tauri would still resolve (no home known) cannot be used before it starts.
        let unresolved = AppDirectoriesOverride::Root(PathBuf::from("$LOCALDATA").join("Kalamu"));
        assert_eq!(crate::platform::log_folder(&unresolved), None);
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
