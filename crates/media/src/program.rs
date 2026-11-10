//! A separate program run to its end, unless the job it works for is paused or cancelled, or it
//! stops making progress: then it is ended and reaped, and nothing it wrote is kept.
//!
//! Both pipes are read on threads of their own while the program runs, so neither fills and
//! stalls it, and every read counts as a sign of life. The offline engines' llama.cpp runner
//! does the same with a stop signal alone (`offline/src/llama.rs`); this one adds the watchdog,
//! and is the one to share when that runner gains one.

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

/// Why a program gave no result.
#[derive(Debug)]
pub enum Ended {
    /// It could not be started.
    NotStarted(std::io::Error),
    /// The stop signal was raised; the program was ended.
    Stopped,
    /// It wrote nothing on either pipe for this long; the program was ended.
    Silent(Duration),
    /// The system could not say whether it was still running; the program was ended.
    Lost(std::io::Error),
}

/// Runs `command` with no input until it ends, `stop` is raised, or it writes nothing on
/// either pipe for `silence`.
pub fn run(mut command: Command, stop: &StopSignal, silence: Duration) -> Result<Finished, Ended> {
    let started = Instant::now();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Ended::NotStarted)?;
    // Milliseconds since `started` of the last read on either pipe.
    let heard = Arc::new(AtomicU64::new(0));
    let stdout = drain(child.stdout.take(), heard.clone(), started);
    let stderr = drain(child.stderr.take(), heard.clone(), started);
    let status = loop {
        if stop.is_stopped() {
            end(&mut child);
            return Err(Ended::Stopped);
        }
        let quiet_for = started
            .elapsed()
            .saturating_sub(Duration::from_millis(heard.load(Ordering::Relaxed)));
        if quiet_for >= silence {
            end(&mut child);
            return Err(Ended::Silent(silence));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(POLL.min(silence)),
            Err(e) => {
                end(&mut child);
                return Err(Ended::Lost(e));
            }
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

/// Reads a pipe to its end on a thread of its own, noting when it last heard from it.
fn drain<R: Read + Send + 'static>(
    pipe: Option<R>,
    heard: Arc<AtomicU64>,
    started: Instant,
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
}
