//! What every program shipped next to the app shares: where it is looked for, and how it is
//! started without a console window. FFmpeg (`ffmpeg.rs`) and the offline engines' llama.cpp
//! programs and Whisper helper (`offline`) use both.

use std::path::PathBuf;
use std::process::Command;

/// Windows' `CREATE_NO_WINDOW`: a console program started from the app must not flash a
/// console window.
#[cfg(windows)]
pub fn quiet(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000)
}

/// Windows' `CREATE_NO_WINDOW`: a console program started from the app must not flash a
/// console window. Nothing to do elsewhere.
#[cfg(not(windows))]
pub fn quiet(command: &mut Command) -> &mut Command {
    command
}

/// The folders a bundled program is looked for in, in order: in development builds only, first
/// the folder the environment variable `var` names; then `dirs` (the app's own folder); in
/// development builds only, last every folder on `PATH`. A release build runs only the programs
/// it ships with, whatever the environment says.
pub fn folders(var: &str, dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = std::env::var_os(var)
        .filter(|_| cfg!(debug_assertions))
        .map(PathBuf::from)
        .into_iter()
        .chain(dirs.iter().cloned())
        .collect();
    if cfg!(debug_assertions)
        && let Some(path) = std::env::var_os("PATH")
    {
        candidates.extend(std::env::split_paths(&path));
    }
    candidates
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn the_app_folders_come_after_the_variable_and_before_path() {
        let dirs = [PathBuf::from("first"), PathBuf::from("second")];
        let found = folders("KALAMU_NO_SUCH_VARIABLE_FOR_TESTS", &dirs);
        assert_eq!(found[..2], dirs);
        if cfg!(debug_assertions) {
            let on_path = std::env::var_os("PATH").map_or(0, |p| std::env::split_paths(&p).count());
            assert_eq!(found.len(), 2 + on_path);
        } else {
            assert_eq!(found.len(), 2);
        }
    }
}
