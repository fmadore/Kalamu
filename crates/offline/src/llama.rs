//! llama.cpp's multimodal command-line program (`llama-mtmd-cli`), shipped next to the app and
//! run once per page as a separate program, never linked in: whisper.cpp already links one copy
//! of ggml into the app, and llama.cpp brings its own, of another version, whose symbols would
//! clash with it (docs/decisions/2026-11-06-ocr-vision-models.md). A separate program also gives
//! its memory back after each page and cannot take the app down if a page crashes it.
//!
//! Beside each build sits its text program (`llama-completion`, with the same suffix), which the
//! offline language models read texts with: the multimodal program answers once only about an
//! image, and takes its request on the command line, which Windows limits to about 32,000
//! characters; the text program reads the instructions, the document and the answer's schema
//! from files (docs/decisions/2026-11-07-offline-language-models.md).
//!
//! The program is built for the same processors as Whisper (`.cargo/config.toml`), on the
//! processor on Windows and on the graphics processor with Metal on a Mac, by
//! `.github/workflows/llama.yml`, and pinned in `scripts/llama.json`. On Windows a second build,
//! `llama-mtmd-cli-vulkan`, uses the graphics processor through Vulkan; it is run only when it
//! lists a device (`device.rs`), since it needs the Vulkan loader graphics drivers install.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use kalamu_core::{ProviderError, StopSignal};
use kalamu_media::program::{self, Ended, Limits, Unfinished};

use crate::diagnostics;

/// The program's name, without the system's executable suffix.
pub const PROGRAM: &str = "llama-mtmd-cli";

/// The text program's name; each build's has the same suffix as its multimodal program.
pub const TEXT_PROGRAM: &str = "llama-completion";

/// The Windows build that uses the graphics processor through Vulkan.
pub const VULKAN_PROGRAM: &str = "llama-mtmd-cli-vulkan";

/// The Windows ARM64 build for Snapdragon's Adreno through OpenCL.
pub const OPENCL_PROGRAM: &str = "llama-mtmd-cli-opencl";

/// How long the program may take to list its devices: a graphics driver that hangs must not
/// keep the OCR models from running on the processor.
const LIST_DEVICES_LIMIT: Duration = Duration::from_secs(30);

/// How long the program may take to say which build it is: it loads no model to do so.
const VERSION_LIMIT: Duration = Duration::from_secs(30);

/// How long a run may write nothing on either pipe before it is taken to be hung and ended, as
/// a graphics driver that stopped answering leaves it: holding the host's turn, every later
/// page and request would wait behind it until the researcher cancelled.
///
/// A silence, not a bound on the whole run, because the program is rarely silent for long
/// while it works: it writes a line for each step of loading (with a dot for each percent of
/// the model read), and the answer token by token on its standard output. What it does in
/// silence is a single step of computation: encoding the page image (49 s and 73 s on an Intel
/// laptop's processor, 42 to 44 s on a Snapdragon's, more on CI's small runners, which take two
/// to three minutes for a whole page), reading one batch of a text's tokens (up to 2,048 for
/// the language models, a few minutes on a weak processor) or the image's (one batch of up to
/// 4,096), and, with the model's file mapped rather than read, the first touch of five
/// gigabytes on a slow disk. None of these is known to pass three or four minutes on a
/// computer the models run on at all; ten leave room for a computer two or three times slower
/// than the slowest measured, and a hang is still ended rather than waited on for ever. No
/// overall bound: the output limit (`-n`) already bounds how much is written, and a slow
/// computer that keeps writing is working (docs/decisions/2026-11-10-offline-robustness.md).
pub(crate) const SILENCE: Duration = Duration::from_secs(600);

/// The most of the program's diagnostics kept, from the end: enough for the error it ends with
/// and the timing it prints.
const STDERR_KEPT: usize = 64 * 1024;

/// Windows' `CREATE_NO_WINDOW`: a console program started from the app must not flash a
/// console window.
#[cfg(windows)]
fn quiet(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000)
}

#[cfg(not(windows))]
fn quiet(command: &mut Command) -> &mut Command {
    command
}

/// The bundled program, once found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Llama {
    program: PathBuf,
    /// The same build's text program, when it was shipped.
    text: Option<PathBuf>,
    /// Set for each run: where the OpenCL build keeps the kernels it compiles.
    env: Vec<(String, PathBuf)>,
}

/// Why a run gave no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The job was paused or cancelled: the program was ended.
    Stopped,
    /// It wrote nothing for this long ([`SILENCE`]) and was ended: a driver or the program
    /// hung.
    Hung(Duration),
    /// It could not start or ended with an error, and why, as the researcher reads it.
    Failed(String),
    /// This copy of the app does not carry the program asked for.
    Missing(String),
}

impl From<RunError> for ProviderError {
    fn from(e: RunError) -> Self {
        match e {
            RunError::Stopped => Self::Interrupted,
            RunError::Hung(after) => Self::Other(format!(
                "The offline model's engine made no progress for {} minutes and was stopped",
                after.as_secs() / 60
            )),
            RunError::Failed(reason) => Self::Other(reason),
            RunError::Missing(reason) => Self::Unsupported(reason),
        }
    }
}

/// What one run wrote.
#[derive(Debug)]
pub struct Run {
    /// The model's answer, as the program prints it.
    pub text: String,
    /// The tokens the model wrote, from the timing the program prints last; `None` when it
    /// printed none.
    pub generated: Option<u32>,
    /// The tokens the model read (the instructions, the text or the image), from the same
    /// timing; `None` when it printed none.
    pub read: Option<u32>,
    pub elapsed: Duration,
}

/// A device the program can run a model on besides the processor, as `--list-devices` names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// The program's name for it, given back with `--device`: `Vulkan0`.
    pub id: String,
    /// What the driver calls it: `NVIDIA GeForce RTX 4060 Laptop GPU`.
    pub name: String,
    /// Its memory in MiB, 0 when the driver reports none.
    pub memory_mib: u64,
}

impl Llama {
    /// The processor build: looks in `KALAMU_LLAMA_DIR`, then in `dirs` (the app's own folder),
    /// then, in development builds only, on `PATH`.
    pub fn locate(dirs: &[PathBuf]) -> Option<Self> {
        Self::locate_program(PROGRAM, dirs)
    }

    /// The Vulkan build, looked for in the same places; Windows only.
    pub fn locate_vulkan(dirs: &[PathBuf]) -> Option<Self> {
        if cfg!(windows) {
            Self::locate_program(VULKAN_PROGRAM, dirs)
        } else {
            None
        }
    }

    /// The OpenCL build, looked for in the same places; Windows ARM64 only.
    pub fn locate_opencl(dirs: &[PathBuf]) -> Option<Self> {
        if cfg!(all(windows, target_arch = "aarch64")) {
            Self::locate_program(OPENCL_PROGRAM, dirs)
        } else {
            None
        }
    }

    /// The same program with `key` set to `value` for every run.
    #[must_use]
    pub fn with_env(mut self, key: &str, value: PathBuf) -> Self {
        self.env.push((key.to_owned(), value));
        self
    }

    /// Whether this build also has its text program.
    pub fn reads_text(&self) -> bool {
        self.text.is_some()
    }

    fn command(&self) -> Command {
        self.command_for(&self.program)
    }

    fn command_for(&self, program: &Path) -> Command {
        let mut command = Command::new(program);
        command.envs(self.env.iter().map(|(k, v)| (k, v)));
        quiet(&mut command);
        command
    }

    fn locate_program(name: &str, dirs: &[PathBuf]) -> Option<Self> {
        let mut candidates: Vec<PathBuf> = std::env::var_os("KALAMU_LLAMA_DIR")
            .map(PathBuf::from)
            .into_iter()
            .chain(dirs.iter().cloned())
            .collect();
        if cfg!(debug_assertions)
            && let Some(path) = std::env::var_os("PATH")
        {
            candidates.extend(std::env::split_paths(&path));
        }
        let exe = std::env::consts::EXE_SUFFIX;
        candidates.into_iter().find_map(|dir| {
            let program = dir.join(format!("{name}{exe}"));
            let text = dir.join(format!("{}{exe}", name.replacen(PROGRAM, TEXT_PROGRAM, 1)));
            program.is_file().then(|| Self {
                program,
                text: text.is_file().then_some(text),
                env: Vec::new(),
            })
        })
    }

    #[cfg(test)]
    pub(crate) fn at(program: PathBuf) -> Self {
        Self {
            program,
            text: None,
            env: Vec::new(),
        }
    }

    /// The program's build, as it reports it (`version: 12110 (8c5a6f1)`), for provenance.
    /// Given [`VERSION_LIMIT`]: it is asked from an export, which must not wait on a program
    /// that hangs.
    pub fn version(&self) -> Option<String> {
        let mut command = self.command();
        command.arg("--version");
        let limits = Limits {
            overall: Some(VERSION_LIMIT),
            stderr_kept: STDERR_KEPT,
            ..Limits::silence(VERSION_LIMIT)
        };
        let output = program::run_with(command, &StopSignal::default(), limits).ok()?;
        let text = String::from_utf8_lossy(&output.stderr).into_owned()
            + &String::from_utf8_lossy(&output.stdout);
        text.lines()
            .find_map(|line| line.trim().strip_prefix("version:"))
            .map(|v| v.trim().to_owned())
    }

    /// The devices besides the processor this build can run on: none for the processor build,
    /// the graphics processors its Vulkan driver offers for the Vulkan build. An error when the
    /// program does not start (no Vulkan loader on this system), fails or takes too long.
    pub fn devices(&self) -> Result<Vec<Device>, String> {
        let mut command = self.command();
        command.arg("--list-devices");
        let limits = Limits {
            overall: Some(LIST_DEVICES_LIMIT),
            stderr_kept: STDERR_KEPT,
            ..Limits::silence(LIST_DEVICES_LIMIT)
        };
        let listed = match program::run_with(command, &StopSignal::default(), limits) {
            Ok(listed) => listed,
            Err(Unfinished {
                ended: Ended::NotStarted(e),
                ..
            }) => return Err(format!("could not start: {e}")),
            Err(Unfinished { ended, stderr }) => {
                diagnostics::warn(&self.program, "listing its devices", &stderr);
                return Err(match ended {
                    Ended::Silent(_) | Ended::TooLong(_) => {
                        "took too long to list its devices".to_owned()
                    }
                    other => format!("{other:?}"),
                });
            }
        };
        if !listed.status.success() {
            diagnostics::warn(&self.program, "listing its devices", &listed.stderr);
            let stderr = String::from_utf8_lossy(&listed.stderr);
            return Err(format!("{}: {}", listed.status, last_error(&stderr)));
        }
        Ok(parse_devices(&String::from_utf8_lossy(&listed.stdout)))
    }

    /// Runs the program with `args` until it ends, until `stop` is raised, when it is ended at
    /// once and the page counts as interrupted, or until it has written nothing for
    /// [`SILENCE`]. Nothing is kept from a run that did not end on its own.
    pub fn run(&self, args: &[String], stop: &StopSignal) -> Result<Run, RunError> {
        self.run_program(&self.program, args, stop)
    }

    /// Runs the text program the same way.
    pub fn run_text(&self, args: &[String], stop: &StopSignal) -> Result<Run, RunError> {
        let text = self.text.as_ref().ok_or_else(|| {
            RunError::Missing(
                "This copy of Kalamu has no engine for offline language models to read texts with"
                    .into(),
            )
        })?;
        self.run_program(text, args, stop)
    }

    fn run_program(
        &self,
        program: &Path,
        args: &[String],
        stop: &StopSignal,
    ) -> Result<Run, RunError> {
        let started = Instant::now();
        let mut command = self.command_for(program);
        command.args(args);
        let limits = Limits {
            stderr_kept: STDERR_KEPT,
            ..Limits::silence(SILENCE)
        };
        let finished = program::run_with(command, stop, limits).map_err(|unfinished| {
            let Unfinished { ended, stderr } = unfinished;
            match ended {
                Ended::Stopped => RunError::Stopped,
                Ended::NotStarted(e) => {
                    RunError::Failed(format!("The offline model's engine could not start: {e}"))
                }
                Ended::Silent(after) | Ended::TooLong(after) => {
                    diagnostics::warn(program, "a run that made no progress", &stderr);
                    RunError::Hung(after)
                }
                Ended::Lost(e) => {
                    diagnostics::warn(program, "a run it lost track of", &stderr);
                    RunError::Failed(format!(
                        "The offline model's engine stopped responding: {e}"
                    ))
                }
            }
        })?;
        let text = String::from_utf8_lossy(&finished.stdout).into_owned();
        let diagnostics = String::from_utf8_lossy(&finished.stderr).into_owned();
        if !finished.status.success() {
            diagnostics::warn(program, "a failed run", &finished.stderr);
            return Err(RunError::Failed(format!(
                "The offline model's engine failed: {}",
                last_error(&diagnostics)
            )));
        }
        Ok(Run {
            text,
            generated: generated_tokens(&diagnostics),
            read: prompt_tokens(&diagnostics),
            elapsed: started.elapsed(),
        })
    }
}

/// The last line that reads as an error, or the last line at all.
fn last_error(diagnostics: &str) -> String {
    let lines: Vec<&str> = diagnostics
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    lines
        .iter()
        .rev()
        .find(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("error") || l.contains("failed") || l.contains("too long")
        })
        .or(lines.last())
        .map_or_else(|| "no message".to_owned(), |l| (*l).to_owned())
}

/// The devices `--list-devices` prints after `Available devices:`, one a line:
/// `  Vulkan0: NVIDIA GeForce RTX 3060 (12288 MiB, 11873 MiB free)`, or `  (none)`. A name may
/// hold brackets of its own (`Intel(R) Arc(TM) Graphics`), so the memory is read from the end.
fn parse_devices(listed: &str) -> Vec<Device> {
    listed
        .lines()
        .skip_while(|line| !line.trim_start().starts_with("Available devices"))
        .skip(1)
        .filter_map(|line| {
            let (id, rest) = line.trim().split_once(": ")?;
            if id.is_empty() || id.contains(' ') {
                return None;
            }
            let (name, memory) = match rest.rsplit_once(" (") {
                Some((name, memory)) if memory.contains("MiB") => (name, memory),
                _ => (rest, ""),
            };
            let memory_mib = memory
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            Some(Device {
                id: id.to_owned(),
                name: name.trim().to_owned(),
                memory_mib,
            })
        })
        .collect()
}

/// The tokens written, from llama.cpp's closing timing line,
/// `llama_perf_context_print:        eval time =  … ms /   812 runs   (…)`: one run per token
/// after the first, which the prompt's evaluation gives.
fn generated_tokens(diagnostics: &str) -> Option<u32> {
    diagnostics.lines().rev().find_map(|line| {
        let rest = line.split_once("llama_perf_context_print:")?.1.trim_start();
        let rest = rest.strip_prefix("eval time")?;
        let runs = rest.split_once('/')?.1.split_whitespace().next()?;
        runs.parse::<u32>().ok().map(|n| n + 1)
    })
}

/// The names of the folders a run's files are written in, inside the scratch folder: only
/// folders with these names are swept from it.
const SCRATCH_PREFIXES: [&str; 2] = ["kalamu-text-", "kalamu-page-"];

/// A folder of its own for one run's files, in `root` (the app's scratch folder, or the
/// system's temporary folder when there is none), deleted when dropped.
fn scratch_dir(root: &Path, prefix: &str) -> std::io::Result<tempfile::TempDir> {
    let builder = {
        let mut builder = tempfile::Builder::new();
        builder.prefix(prefix);
        builder
    };
    if root.as_os_str().is_empty() {
        builder.tempdir()
    } else {
        std::fs::create_dir_all(root)?;
        builder.tempdir_in(root)
    }
}

/// Deletes what runs left in the scratch folder when the app was killed or crashed before it
/// could delete their files: the page images and a text's instructions and contents, the
/// researcher's material. Only the folders runs make are touched, so a wrong root deletes
/// nothing else. Called once, before any run.
pub(crate) fn sweep_scratch(root: &Path) {
    if root.as_os_str().is_empty() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let mut swept = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let ours = SCRATCH_PREFIXES
            .iter()
            .any(|prefix| name.to_string_lossy().starts_with(prefix));
        if ours
            && entry.file_type().is_ok_and(|t| t.is_dir())
            && std::fs::remove_dir_all(entry.path()).is_ok()
        {
            swept += 1;
        }
    }
    if swept > 0 {
        tracing::info!(swept, "files of interrupted offline runs deleted");
    }
}

/// Files written where the program can read them (a text's instructions, the text, the answer's
/// schema), deleted afterwards.
pub struct Scratch {
    dir: tempfile::TempDir,
}

impl Scratch {
    /// A folder for one request's files in `root`, the app's scratch folder.
    pub fn new(root: &Path) -> Result<Self, ProviderError> {
        let dir = scratch_dir(root, SCRATCH_PREFIXES[0])
            .map_err(|e| ProviderError::Other(format!("Could not write the request: {e}")))?;
        Ok(Self { dir })
    }

    /// Writes `bytes` as `name` and returns its path.
    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, ProviderError> {
        let path = self.dir.path().join(name);
        std::fs::write(&path, bytes)
            .map_err(|e| ProviderError::Other(format!("Could not write the request: {e}")))?;
        Ok(path)
    }
}

/// The tokens read, from `llama_perf_context_print: prompt eval time = … ms /  1290 tokens (…)`.
fn prompt_tokens(diagnostics: &str) -> Option<u32> {
    diagnostics.lines().rev().find_map(|line| {
        let rest = line.split_once("llama_perf_context_print:")?.1.trim_start();
        let rest = rest.strip_prefix("prompt eval time")?;
        rest.split_once('/')?
            .1
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

/// A page written where the program can read it, deleted afterwards.
pub struct PageFile {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl PageFile {
    /// Writes the page in a folder of its own in `root`, the app's scratch folder.
    pub fn write(root: &Path, bytes: &[u8], extension: &str) -> Result<Self, ProviderError> {
        let dir = scratch_dir(root, SCRATCH_PREFIXES[1])
            .map_err(|e| ProviderError::Other(format!("Could not write the page image: {e}")))?;
        let path = dir.path().join(format!("page.{extension}"));
        std::fs::write(&path, bytes)
            .map_err(|e| ProviderError::Other(format!("Could not write the page image: {e}")))?;
        Ok(Self { _dir: dir, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The page as the model should see it: no larger than `max_side` on its longest side, as PNG,
/// or the original bytes when they are small enough already.
pub fn page_image(
    bytes: &[u8],
    mime: &str,
    max_side: u32,
) -> Result<(Vec<u8>, &'static str), ProviderError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| ProviderError::InvalidRequest(format!("The page image is unreadable: {e}")))?;
    if image.width().max(image.height()) <= max_side {
        let extension = if mime == "image/png" { "png" } else { "jpg" };
        return Ok((bytes.to_vec(), extension));
    }
    let resized = image.resize(max_side, max_side, image::imageops::FilterType::Lanczos3);
    let mut png = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| ProviderError::Other(format!("Could not prepare the page image: {e}")))?;
    Ok((png.into_inner(), "png"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn the_tokens_written_come_from_the_closing_timing() {
        let stderr = "load time = 812.4 ms\n\
llama_perf_context_print:        load time =     812.40 ms\n\
llama_perf_context_print: prompt eval time =   10432.11 ms /  1290 tokens (    8.09 ms per token)\n\
llama_perf_context_print:        eval time =   20514.55 ms /   811 runs   (   25.30 ms per token)\n\
llama_perf_context_print:       total time =   31120.02 ms /  2101 tokens\n";
        assert_eq!(generated_tokens(stderr), Some(812));
        assert_eq!(prompt_tokens(stderr), Some(1290));
        assert_eq!(generated_tokens("nothing here"), None);
    }

    #[test]
    fn devices_are_read_from_the_listing() {
        let listed = "ggml_vulkan: Found 2 Vulkan devices:\n\
Available devices:\n\
  Vulkan0: NVIDIA GeForce RTX 4060 Laptop GPU (8188 MiB, 7900 MiB free)\n\
  Vulkan1: Intel(R) Arc(TM) Graphics (16384 MiB, 16000 MiB free)\n";
        assert_eq!(
            parse_devices(listed),
            [
                Device {
                    id: "Vulkan0".into(),
                    name: "NVIDIA GeForce RTX 4060 Laptop GPU".into(),
                    memory_mib: 8188,
                },
                Device {
                    id: "Vulkan1".into(),
                    name: "Intel(R) Arc(TM) Graphics".into(),
                    memory_mib: 16384,
                },
            ]
        );
        assert!(parse_devices("Available devices:\n  (none)\n").is_empty());
        assert!(parse_devices("").is_empty());
    }

    #[test]
    fn a_failure_reports_its_last_error_line() {
        let stderr = "loading model\nclip_init: failed to load image encoder\nmain: exiting\n";
        assert_eq!(
            last_error(stderr),
            "clip_init: failed to load image encoder"
        );
        assert_eq!(last_error(""), "no message");
        let long = "load
main: prompt is too long (40210 tokens, max 32764)
exiting
";
        assert_eq!(
            last_error(long),
            "main: prompt is too long (40210 tokens, max 32764)"
        );
    }

    #[test]
    fn the_text_program_is_found_beside_its_build() {
        let dir = tempfile::tempdir().unwrap();
        let exe = std::env::consts::EXE_SUFFIX;
        std::fs::write(dir.path().join(format!("llama-mtmd-cli-vulkan{exe}")), b"").unwrap();
        let without = Llama::locate_program(VULKAN_PROGRAM, &[dir.path().to_owned()]).unwrap();
        assert!(!without.reads_text());
        std::fs::write(
            dir.path().join(format!("llama-completion-vulkan{exe}")),
            b"",
        )
        .unwrap();
        let with = Llama::locate_program(VULKAN_PROGRAM, &[dir.path().to_owned()]).unwrap();
        assert!(with.reads_text());
        assert_eq!(
            with.text.unwrap(),
            dir.path().join(format!("llama-completion-vulkan{exe}"))
        );
    }

    #[test]
    fn run_files_go_in_the_scratch_folder_and_what_a_crash_left_is_swept() {
        let root = tempfile::tempdir().unwrap();
        let scratch = root.path().join("offline-scratch");
        let page = PageFile::write(&scratch, b"png", "png").unwrap();
        assert!(page.path().starts_with(&scratch));
        let request = Scratch::new(&scratch).unwrap();
        let text = request.write("request.txt", b"a text").unwrap();
        assert!(text.starts_with(&scratch));
        // As a kill leaves them: never dropped.
        std::mem::forget(page);
        std::mem::forget(request);
        // Something else in the folder is not the sweep's to delete.
        std::fs::write(scratch.join("notes.txt"), b"kept").unwrap();
        std::fs::create_dir(scratch.join("kept")).unwrap();
        assert_eq!(std::fs::read_dir(&scratch).unwrap().count(), 4);
        sweep_scratch(&scratch);
        let mut left: Vec<String> = std::fs::read_dir(&scratch)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, ["kept", "notes.txt"]);
        // No root: the system's temporary folder, never swept.
        sweep_scratch(Path::new(""));
        let page = PageFile::write(Path::new(""), b"png", "png").unwrap();
        assert!(page.path().starts_with(std::env::temp_dir()));
    }

    #[test]
    fn a_program_that_does_not_start_fails_its_run() {
        let llama = Llama::at(PathBuf::from("no-such-llama-program"));
        let ran = llama.run(&[], &StopSignal::default());
        assert!(
            matches!(&ran, Err(RunError::Failed(m)) if m.contains("could not start")),
            "{ran:?}"
        );
        assert!(matches!(
            llama.run_text(&[], &StopSignal::default()),
            Err(RunError::Missing(_))
        ));
        assert_eq!(llama.version(), None);
        assert!(llama.devices().unwrap_err().starts_with("could not start"));
    }

    #[test]
    fn a_missing_program_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Llama::locate(&[dir.path().to_owned()]).filter(|l| l.program.starts_with(dir.path())),
            None
        );
    }
}
