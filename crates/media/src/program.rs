//! A separate program run to its end, unless the job it works for is paused or cancelled, or it
//! stops making progress: then it is ended and reaped, and nothing it wrote is kept.
//!
//! Both pipes are read on threads of their own while the program runs, so neither fills and
//! stalls it, and every read counts as a sign of life. FFmpeg's runs (`ffmpeg.rs`) and the
//! offline engines' llama.cpp programs (`offline/src/llama.rs`) both run here, each with the
//! limits its program's way of working allows ([`Limits`]).

use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use kalamu_core::StopSignal;

/// How often the stop signal and the watchdog are looked at: a pause ends the program within a
/// tenth of a second of being asked.
const POLL: Duration = Duration::from_millis(100);

/// What a program wrote, once it ended on its own.
#[derive(Debug)]
pub struct Finished {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// What a run is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// How long it may write nothing on either pipe.
    pub silence: Duration,
    /// How long it may run in all, `None` for no bound: for a program whose whole run is short
    /// and known (a version, a list of devices), never for one whose work has no fixed length.
    pub overall: Option<Duration>,
    /// The most of its standard error kept, from the end; its standard output is kept whole.
    pub stderr_kept: usize,
}

impl Limits {
    /// A silence limit alone, everything it writes kept.
    pub fn silence(silence: Duration) -> Self {
        Self {
            silence,
            overall: None,
            stderr_kept: usize::MAX,
        }
    }
}

/// A program that gave no result, with the end of what it wrote on its standard error before
/// it was ended (empty when it did not start), for the log.
#[derive(Debug)]
pub struct Unfinished {
    pub ended: Ended,
    pub stderr: Vec<u8>,
}

/// Why a program gave no result.
#[derive(Debug)]
pub enum Ended {
    /// It could not be started.
    NotStarted(std::io::Error),
    /// The stop signal was raised; the program was ended.
    Stopped,
    /// It wrote nothing on either pipe for this long; the program was ended.
    Silent(Duration),
    /// It ran for this long in all, its [`Limits::overall`]; the program was ended.
    TooLong(Duration),
    /// The system could not say whether it was still running; the program was ended.
    Lost(std::io::Error),
}

/// Runs `command` with no input until it ends, `stop` is raised, or it writes nothing on
/// either pipe for `silence`.
pub fn run(command: Command, stop: &StopSignal, silence: Duration) -> Result<Finished, Ended> {
    run_with(command, stop, Limits::silence(silence)).map_err(|unfinished| unfinished.ended)
}

/// [`run`] within `limits`, giving what the program wrote on its standard error when it is
/// ended without a result.
pub fn run_with(
    mut command: Command,
    stop: &StopSignal,
    limits: Limits,
) -> Result<Finished, Unfinished> {
    let started = Instant::now();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Unfinished {
            ended: Ended::NotStarted(e),
            stderr: Vec::new(),
        })?;
    // Milliseconds since `started` of the last read on either pipe.
    let heard = Arc::new(AtomicU64::new(0));
    let stdout = drain(child.stdout.take(), heard.clone(), started, usize::MAX);
    let stderr = drain(
        child.stderr.take(),
        heard.clone(),
        started,
        limits.stderr_kept,
    );
    let ended = |child: &mut Child, ended: Ended, stderr: JoinHandle<Vec<u8>>| {
        end(child);
        Err(Unfinished {
            ended,
            stderr: stderr.join().unwrap_or_default(),
        })
    };
    let poll = POLL.min(limits.silence);
    let status = loop {
        if stop.is_stopped() {
            return ended(&mut child, Ended::Stopped, stderr);
        }
        let quiet_for = started
            .elapsed()
            .saturating_sub(Duration::from_millis(heard.load(Ordering::Relaxed)));
        if quiet_for >= limits.silence {
            return ended(&mut child, Ended::Silent(limits.silence), stderr);
        }
        if let Some(overall) = limits.overall
            && started.elapsed() >= overall
        {
            return ended(&mut child, Ended::TooLong(overall), stderr);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(poll),
            Err(e) => return ended(&mut child, Ended::Lost(e), stderr),
        }
    };
    Ok(Finished {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

/// Ends the program and waits for it, so it leaves no zombie behind. The pipes close with it,
/// which ends their threads.
fn end(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Reads a pipe to its end on a thread of its own, noting when it last heard from it and
/// keeping at most the last `keep` bytes.
fn drain<R: Read + Send + 'static>(
    pipe: Option<R>,
    heard: Arc<AtomicU64>,
    started: Instant,
    keep: usize,
) -> JoinHandle<Vec<u8>> {
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
            let now = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            heard.fetch_max(now, Ordering::Relaxed);
            kept.extend_from_slice(&buffer[..n]);
            if kept.len() > keep {
                kept.drain(..kept.len() - keep);
            }
        }
        kept
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Set in the environment of this test program started again as a fake program: the
    /// test named below then behaves as the program, and the others are filtered out.
    const ROLE: &str = "KALAMU_FAKE_PROGRAM";

    /// Not a test of its own: the body of the fake program, run only when [`ROLE`] is set. It
    /// is this test executable started again, so it runs the same on Windows and macOS with
    /// nothing written to disk.
    #[test]
    fn fake_program() {
        match std::env::var(ROLE).as_deref() {
            // Writes a line every 50 ms for a minute: alive, and slow.
            Ok("slow") => {
                for _ in 0..1200 {
                    eprintln!("still working");
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            // Says nothing for a minute, as FFmpeg does reading from a share that went away.
            Ok("silent") => std::thread::sleep(Duration::from_secs(60)),
            Ok("quick") => println!("done"),
            // Fills its standard error with numbered lines, then fails.
            Ok("talkative") => {
                for i in 0..20_000 {
                    eprintln!("line {i}");
                }
                std::process::exit(3);
            }
            _ => {}
        }
    }

    fn fake(role: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "program::tests::fake_program", "--nocapture"])
            .args(["--test-threads", "1"])
            .env(ROLE, role);
        command
    }

    #[test]
    fn a_program_that_ends_gives_what_it_wrote() {
        let finished = run(
            fake("quick"),
            &StopSignal::default(),
            Duration::from_secs(30),
        )
        .unwrap();
        assert!(finished.status.success());
        assert!(String::from_utf8_lossy(&finished.stdout).contains("done"));
    }

    #[test]
    fn a_stop_ends_a_program_still_working_within_a_second() {
        let stop = StopSignal::default();
        let raise = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            raise.stop();
        });
        let started = Instant::now();
        let ended = run(fake("slow"), &stop, Duration::from_secs(30));
        assert!(matches!(ended, Err(Ended::Stopped)), "{ended:?}");
        // Half a second before the stop, and at most a second after it.
        assert!(
            started.elapsed() < Duration::from_millis(1_500),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn a_program_that_says_nothing_is_ended_by_the_watchdog() {
        let started = Instant::now();
        let ended = run(
            fake("silent"),
            &StopSignal::default(),
            Duration::from_secs(2),
        );
        assert!(matches!(ended, Err(Ended::Silent(_))), "{ended:?}");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn a_program_that_keeps_writing_is_not_cut_short() {
        // Its lines come every 50 ms: a watchdog of a second never fires, however long it
        // runs; the stop ends it.
        let stop = StopSignal::default();
        let raise = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(3));
            raise.stop();
        });
        let ended = run(fake("slow"), &stop, Duration::from_secs(1));
        assert!(matches!(ended, Err(Ended::Stopped)), "{ended:?}");
    }

    #[test]
    fn an_overall_limit_ends_a_program_that_keeps_writing() {
        let started = Instant::now();
        let ended = run_with(
            fake("slow"),
            &StopSignal::default(),
            Limits {
                silence: Duration::from_secs(30),
                overall: Some(Duration::from_secs(1)),
                stderr_kept: 1024,
            },
        )
        .unwrap_err();
        assert!(matches!(ended.ended, Ended::TooLong(_)), "{ended:?}");
        // What it wrote before it was ended, for the log, within the bound.
        let stderr = String::from_utf8_lossy(&ended.stderr);
        assert!(stderr.contains("still working"), "{stderr}");
        assert!(ended.stderr.len() <= 1024);
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn only_the_end_of_standard_error_is_kept() {
        let finished = run_with(
            fake("talkative"),
            &StopSignal::default(),
            Limits {
                stderr_kept: 4096,
                ..Limits::silence(Duration::from_secs(30))
            },
        )
        .unwrap();
        assert_eq!(finished.status.code(), Some(3));
        assert!(finished.stderr.len() <= 4096);
        let stderr = String::from_utf8_lossy(&finished.stderr);
        assert!(stderr.trim_end().ends_with("line 19999"), "{stderr}");
        assert!(!stderr.contains("line 0\n"));
    }
}
