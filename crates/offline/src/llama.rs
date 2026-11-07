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

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use kalamu_core::{ProviderError, StopSignal};

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

/// How often a running page checks whether the job was paused or cancelled.
const POLL: Duration = Duration::from_millis(100);

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
    pub fn version(&self) -> Option<String> {
        let output = self
            .command()
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .ok()?;
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
        let mut child = self
            .command()
            .arg("--list-devices")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not start: {e}"))?;
        let stdout = drain(child.stdout.take(), STDERR_KEPT);
        let stderr = drain(child.stderr.take(), STDERR_KEPT);
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() < LIST_DEVICES_LIMIT => std::thread::sleep(POLL),
                Ok(None) => {
                    end(&mut child);
                    return Err("took too long to list its devices".into());
                }
                Err(e) => {
                    end(&mut child);
                    return Err(e.to_string());
                }
            }
        };
        let listed = String::from_utf8_lossy(&stdout.join().unwrap_or_default()).into_owned();
        if !status.success() {
            let diagnostics =
                String::from_utf8_lossy(&stderr.join().unwrap_or_default()).into_owned();
            return Err(format!("{status}: {}", last_error(&diagnostics)));
        }
        Ok(parse_devices(&listed))
    }

    /// Runs the program with `args` until it ends, or until `stop` is raised, when it is ended
    /// at once and the page counts as interrupted: nothing was kept from it.
    pub fn run(&self, args: &[String], stop: &StopSignal) -> Result<Run, ProviderError> {
        self.run_program(&self.program, args, stop)
    }

    /// Runs the text program the same way.
    pub fn run_text(&self, args: &[String], stop: &StopSignal) -> Result<Run, ProviderError> {
        let text = self.text.as_ref().ok_or_else(|| {
            ProviderError::Unsupported(
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
    ) -> Result<Run, ProviderError> {
        let started = Instant::now();
        let mut child = self
            .command_for(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ProviderError::Other(format!("The offline model's engine could not start: {e}"))
            })?;
        // Both pipes are read while the program runs, so neither fills and stalls it.
        let stdout = drain(child.stdout.take(), usize::MAX);
        let stderr = drain(child.stderr.take(), STDERR_KEPT);
        let status = loop {
            if stop.is_stopped() {
                end(&mut child);
                return Err(ProviderError::Interrupted);
            }
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => std::thread::sleep(POLL),
                Err(e) => {
                    end(&mut child);
                    return Err(ProviderError::Other(format!(
                        "The offline model's engine stopped responding: {e}"
                    )));
                }
            }
        };
        let text = String::from_utf8_lossy(&stdout.join().unwrap_or_default()).into_owned();
        let diagnostics = String::from_utf8_lossy(&stderr.join().unwrap_or_default()).into_owned();
        if !status.success() {
            return Err(ProviderError::Other(format!(
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

fn end(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Reads a pipe to its end on a thread of its own, keeping at most the last `keep` bytes.
fn drain<R: Read + Send + 'static>(
    pipe: Option<R>,
    keep: usize,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let Some(mut pipe) = pipe else {
            return kept;
        };
        let mut buffer = [0u8; 8192];
        while let Ok(n) = pipe.read(&mut buffer) {
            if n == 0 {
                break;
            }
            kept.extend_from_slice(&buffer[..n]);
            if kept.len() > keep {
                kept.drain(..kept.len() - keep);
            }
        }
        kept
    })
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

/// Files written where the program can read them (a text's instructions, the text, the answer's
/// schema), deleted afterwards.
pub struct Scratch {
    dir: tempfile::TempDir,
}

impl Scratch {
    pub fn new() -> Result<Self, ProviderError> {
        let dir = tempfile::Builder::new()
            .prefix("kalamu-text-")
            .tempdir()
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
    pub fn write(bytes: &[u8], extension: &str) -> Result<Self, ProviderError> {
        let dir = tempfile::Builder::new()
            .prefix("kalamu-page-")
            .tempdir()
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
    fn a_missing_program_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Llama::locate(&[dir.path().to_owned()]).filter(|l| l.program.starts_with(dir.path())),
            None
        );
    }
}
