//! The Vulkan helper (`helpers/whisper-vulkan`), seen from Whisper: asked once a session which
//! graphics processor it would use, started when a piece is transcribed there, kept with the
//! model while it is loaded. Every way it can fail — it cannot start (no Vulkan loader on the
//! PC), finds no usable graphics processor, stops answering, dies — comes back as an error; the
//! caller then transcribes on the processor. Ported from Live Translation's `whisper/gpu.rs`.

use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use kalamu_core::StopSignal;

use super::engine::{self, Decode, Devices, RawSegment, Request, Response};
use crate::models::ModelLease;

/// How long the helper may take to list the graphics processors: a hanging driver must not hold
/// anything up.
const LIST_TIMEOUT: Duration = Duration::from_secs(30);
/// Loading includes building the device's compute pipelines, which a first run on a slow driver
/// can take a while over.
const LOAD_TIMEOUT: Duration = Duration::from_secs(180);
/// A two-minute window takes seconds on a graphics processor; one that takes ten minutes has
/// hung.
const DECODE_TIMEOUT: Duration = Duration::from_secs(600);
const SHORT_TIMEOUT: Duration = Duration::from_secs(30);
/// How often a wait for an answer looks at the job's stop signal.
const POLL: Duration = Duration::from_millis(100);

/// Why the helper did not answer.
#[derive(Debug)]
pub(super) enum Failure {
    /// The job was paused or cancelled: the helper was stopped for it.
    Stopped,
    /// The helper failed; the piece goes to the processor.
    Failed(String),
}

impl Failure {
    fn failed(message: impl std::fmt::Display) -> Self {
        Self::Failed(message.to_string())
    }
}

fn command(program: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: the helper is a console program; no console may flash up.
        command.creation_flags(0x0800_0000);
    }
    command
}

/// The graphics processor the helper would use, asked with `--list-devices`; `None` when it
/// cannot start, finds none it can use, or does not answer within [`LIST_TIMEOUT`].
pub(super) fn find_device(program: &Path) -> Option<String> {
    let mut child = command(program).arg("--list-devices").spawn().ok()?;
    let stdout = child.stdout.take()?;
    let (sender, answer) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(stdout).read_line(&mut line);
        let _ = sender.send(line);
    });
    let line = answer.recv_timeout(LIST_TIMEOUT).ok();
    let _ = child.kill();
    let _ = child.wait();
    let found: Devices = serde_json::from_str(line?.trim()).ok()?;
    tracing::info!(devices = ?found.devices, chosen = ?found.chosen, "graphics processors for Whisper");
    found.chosen
}

pub(super) struct Worker {
    child: Child,
    stdin: ChildStdin,
    /// What the helper answers, read on a thread of its own so every wait can time out.
    answers: Receiver<io::Result<Response>>,
    /// The graphics processor, as its driver names it.
    pub device: String,
    /// The model's text context, for the prompt's budget.
    pub text_context: i32,
}

impl Worker {
    /// Start `program` and load the leased model on the graphics processor it chooses.
    pub fn start(program: &Path, lease: &ModelLease) -> Result<Self, Failure> {
        let mut child = command(program)
            .spawn()
            .map_err(|e| Failure::failed(format!("Could not start {}: {e}", program.display())))?;
        let pipes = (child.stdin.take(), child.stdout.take(), child.stderr.take());
        let (Some(stdin), Some(stdout), Some(stderr)) = pipes else {
            let _ = child.kill();
            return Err(Failure::failed("The helper started without its pipes"));
        };
        let (sender, answers) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                match engine::receive::<Response>(&mut stdout) {
                    Ok(Some(answer)) => {
                        if sender.send(Ok(answer)).is_err() {
                            return;
                        }
                    }
                    Ok(None) => return,
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        return;
                    }
                }
            }
        });
        // whisper.cpp and ggml write what they are doing to standard error: kept in the log.
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                tracing::debug!(target: "whisper_vulkan", "{line}");
            }
        });
        let mut worker = Self {
            child,
            stdin,
            answers,
            device: String::new(),
            text_context: 0,
        };
        let load = Request::Load {
            path: lease.path().to_path_buf(),
            bytes: lease.model.bytes,
            sha256: lease.model.sha256.to_owned(),
        };
        match worker.ask(&load, None, LOAD_TIMEOUT, None)? {
            Response::Loaded {
                device,
                text_context,
            } => {
                worker.device = device;
                worker.text_context = text_context;
                Ok(worker)
            }
            Response::Failed { message } => Err(Failure::Failed(message)),
            other => Err(Failure::failed(format!("Unexpected answer: {other:?}"))),
        }
    }

    /// Send `request` (and `samples`) and wait for its answer. The helper is stopped when the
    /// wait times out or `stop` fires, so it can never answer late.
    fn ask(
        &mut self,
        request: &Request,
        samples: Option<&[f32]>,
        timeout: Duration,
        stop: Option<&StopSignal>,
    ) -> Result<Response, Failure> {
        let sent = engine::send(&mut self.stdin, request).and_then(|()| match samples {
            Some(samples) => {
                engine::write_frame(&mut self.stdin, &engine::samples_to_bytes(samples))
            }
            None => Ok(()),
        });
        if let Err(error) = sent {
            return Err(Failure::failed(format!(
                "The helper stopped ({}): {error}",
                self.status()
            )));
        }
        let deadline = Instant::now() + timeout;
        loop {
            if stop.is_some_and(StopSignal::is_stopped) {
                self.stop();
                return Err(Failure::Stopped);
            }
            match self.answers.recv_timeout(POLL) {
                Ok(answer) => return answer.map_err(Failure::failed),
                Err(RecvTimeoutError::Timeout) if Instant::now() < deadline => {}
                Err(RecvTimeoutError::Timeout) => {
                    self.stop();
                    return Err(Failure::failed("The helper stopped answering"));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(Failure::failed(format!(
                        "The helper stopped ({})",
                        self.status()
                    )));
                }
            }
        }
    }

    /// `text` as the model's tokens; `None` when the tokenizer refuses it or the helper fails.
    pub fn tokenize(&mut self, text: &str) -> Option<Vec<i32>> {
        let request = Request::Tokenize { text: text.into() };
        match self.ask(&request, None, SHORT_TIMEOUT, None) {
            Ok(Response::Tokens { tokens }) => tokens,
            _ => None,
        }
    }

    /// The language spoken in `samples`, as whisper.cpp's id.
    pub fn detect(
        &mut self,
        samples: &[f32],
        threads: i32,
        stop: &StopSignal,
    ) -> Result<Option<i32>, Failure> {
        let request = Request::Detect { threads };
        match self.ask(&request, Some(samples), SHORT_TIMEOUT, Some(stop))? {
            Response::Detected { language } => Ok(language),
            Response::Failed { message } => Err(Failure::Failed(message)),
            other => Err(Failure::failed(format!("Unexpected answer: {other:?}"))),
        }
    }

    /// Decode one window.
    pub fn decode(
        &mut self,
        decode: &Decode,
        samples: &[f32],
        stop: &StopSignal,
    ) -> Result<Vec<RawSegment>, Failure> {
        let request = Request::Decode(decode.clone());
        match self.ask(&request, Some(samples), DECODE_TIMEOUT, Some(stop))? {
            Response::Decoded { segments } => Ok(segments),
            Response::Failed { message } => Err(Failure::Failed(message)),
            other => Err(Failure::failed(format!("Unexpected answer: {other:?}"))),
        }
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// How the helper ended, for the log, once it has.
    fn status(&mut self) -> String {
        match self.child.try_wait() {
            Ok(Some(status)) => status.to_string(),
            _ => "still running".into(),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}
