//! Failures nobody expected in the interface (a screen that threw as it was drawn, an error or a
//! rejected promise no code caught), written to the log so that a bug report has something to
//! quote. The log's rule holds: no document text, no keys, nothing a researcher typed or a
//! model answered. What the interface sends is cut down here, whatever it sent: the error's
//! name, its message with every quoted part taken out, and where in Kalamu's code it happened.
//! A burst is bounded, so a screen that fails on every frame cannot fill the disk.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use ts_rs::TS;

/// How the failure reached the interface's net.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InterfaceErrorKind {
    /// Thrown and not caught (the window's `error` event).
    Error,
    /// A promise rejected with nobody to handle it (`unhandledrejection`).
    Rejection,
    /// A screen failed as it was drawn, caught by its boundary.
    Screen,
}

impl InterfaceErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Rejection => "rejection",
            Self::Screen => "screen",
        }
    }
}

/// A failure as the interface reports it. Every field is cut down before it is written.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct InterfaceError {
    pub kind: InterfaceErrorKind,
    /// The screen on view (`review`, `settings`…), as the interface names its views.
    pub screen: String,
    /// The error's name (`TypeError`), or what was thrown when it was not an error.
    pub name: String,
    pub message: String,
    /// The error's stack as the webview writes it: only its frames' functions and places in
    /// the code are kept.
    pub stack: String,
}

/// Failures written in a minute; past them, only their number is, once the minute is over.
const PER_WINDOW: u32 = 10;
const WINDOW: Duration = Duration::from_secs(60);
const NAME_MAX: usize = 64;
const SCREEN_MAX: usize = 32;
const MESSAGE_MAX: usize = 300;
const FRAMES_MAX: usize = 12;
const FUNCTION_MAX: usize = 80;
const PLACE_MAX: usize = 160;

static LIMIT: Mutex<Limiter> = Mutex::new(Limiter::new());

/// Writes a failure of the interface to the log, cut down to what may be written there; past
/// ten in a minute, only how many were left out.
#[tauri::command]
pub fn log_interface_error(error: InterfaceError) {
    let admitted = LIMIT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .admit(Instant::now());
    let Admit::Log { dropped_before } = admitted else {
        return;
    };
    if dropped_before > 0 {
        tracing::warn!(
            dropped = dropped_before,
            "interface failures left out of the log in the last minute"
        );
    }
    let entry = Entry::from(&error);
    tracing::error!(
        kind = error.kind.as_str(),
        screen = %entry.screen,
        name = %entry.name,
        message = %entry.message,
        stack = %entry.stack,
        "unexpected failure in the interface"
    );
}

/// What is written of a failure.
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    screen: String,
    name: String,
    message: String,
    stack: String,
}

impl From<&InterfaceError> for Entry {
    fn from(e: &InterfaceError) -> Self {
        Self {
            screen: word(&e.screen, SCREEN_MAX, |c| c.is_ascii_alphanumeric(), "-"),
            name: word(
                &e.name,
                NAME_MAX,
                |c| c.is_ascii_alphanumeric() || "_$.".contains(c),
                "Error",
            ),
            message: message(&e.message),
            stack: stack(&e.stack),
        }
    }
}

/// The characters `keep` allows, at most `max` of them, or `empty` when none is left.
fn word(text: &str, max: usize, keep: impl Fn(char) -> bool, empty: &str) -> String {
    let kept: String = text.chars().filter(|c| keep(*c)).take(max).collect();
    if kept.is_empty() {
        empty.to_string()
    } else {
        kept
    }
}

/// The opening quotation marks a message may hold, each with its closing one.
const QUOTES: [(char, char); 6] = [
    ('"', '"'),
    ('\'', '\''),
    ('`', '`'),
    ('“', '”'),
    ('‘', '’'),
    ('«', '»'),
];

/// A message with what it quotes taken out (a value, a fragment of text read: `Unexpected token
/// 'a', "abc" is not valid JSON` keeps only its words), on one line, cut short.
fn message(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let closing = QUOTES
            .iter()
            .find(|(open, _)| *open == c)
            .map(|(_, close)| *close);
        if let Some(close) = closing
            && let Some(end) = chars[i + 1..].iter().position(|d| *d == close)
        {
            out.push_str("\"…\"");
            i += end + 2;
            continue;
        }
        out.push(if c.is_control() { ' ' } else { c });
        i += 1;
    }
    let one_line = out.split_whitespace().collect::<Vec<_>>().join(" ");
    cut(&one_line, MESSAGE_MAX)
}

/// At most `max` characters, an ellipsis marking what was cut.
fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut kept: String = text.chars().take(max - 1).collect();
    kept.push('…');
    kept
}

/// The stack's frames, each as its function and its place in the code (`load
/// _app/immutable/chunks/x.js:1:2345`), joined on one line. Lines that are not frames (the
/// message the V8 engine repeats first) are left out.
fn stack(text: &str) -> String {
    text.lines()
        .filter_map(frame)
        .take(FRAMES_MAX)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// One frame, written by Chromium (`at load (http://host/x.js:1:2)`) or WebKit
/// (`load@http://host/x.js:1:2`), as its function and the file's path with line and column;
/// none for a line with no address in it.
fn frame(line: &str) -> Option<String> {
    let line = line.trim().trim_end_matches(')');
    let scheme = line.find("://")?;
    let start = line[..scheme]
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '+' || c == '.'))
        .map_or(0, |i| i + 1);
    let (function, address) = line.split_at(start);
    let (rest, column) = address.rsplit_once(':')?;
    let (url, row) = rest.rsplit_once(':')?;
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if !digits(row) || !digits(column) {
        return None;
    }
    // The path on the server, without the address it is served from or what follows `?`.
    let after_scheme = &url[url.find("://")? + 3..];
    let path = after_scheme
        .find('/')
        .map_or("", |i| &after_scheme[i + 1..]);
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let path: String = path
        .chars()
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .collect();
    let path = if path.chars().count() > PLACE_MAX {
        // The end names the file: the start is dropped.
        let skip = path.chars().count() - PLACE_MAX;
        format!("…{}", path.chars().skip(skip + 1).collect::<String>())
    } else {
        path
    };
    // Chromium's words before the function, not part of its name.
    let function = function
        .split_whitespace()
        .filter(|w| *w != "at" && *w != "async")
        .collect::<String>();
    let function: String = function
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "_$.<>".contains(*c))
        .take(FUNCTION_MAX)
        .collect();
    Some(if function.is_empty() {
        format!("{path}:{row}:{column}")
    } else {
        format!("{function} {path}:{row}:{column}")
    })
}

/// Whether a failure is written: at most [`PER_WINDOW`] a minute, those left out counted.
#[derive(Debug)]
struct Limiter {
    start: Option<Instant>,
    logged: u32,
    dropped: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum Admit {
    /// Written, after a line saying how many the last minute left out.
    Log {
        dropped_before: u32,
    },
    Drop,
}

impl Limiter {
    const fn new() -> Self {
        Self {
            start: None,
            logged: 0,
            dropped: 0,
        }
    }

    fn admit(&mut self, now: Instant) -> Admit {
        match self.start {
            Some(start) if now.saturating_duration_since(start) < WINDOW => {
                if self.logged < PER_WINDOW {
                    self.logged += 1;
                    Admit::Log { dropped_before: 0 }
                } else {
                    self.dropped += 1;
                    Admit::Drop
                }
            }
            _ => {
                self.start = Some(now);
                self.logged = 1;
                Admit::Log {
                    dropped_before: std::mem::take(&mut self.dropped),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(name: &str, message: &str, stack: &str) -> InterfaceError {
        InterfaceError {
            kind: InterfaceErrorKind::Error,
            screen: "review".into(),
            name: name.into(),
            message: message.into(),
            stack: stack.into(),
        }
    }

    #[test]
    fn what_a_message_quotes_is_left_out() {
        assert_eq!(
            message("Unexpected token 'L', \"Le Gouverneur général\" is not valid JSON"),
            "Unexpected token \"…\", \"…\" is not valid JSON"
        );
        assert_eq!(
            message("JSON Parse error: Unexpected identifier “Le”"),
            "JSON Parse error: Unexpected identifier \"…\""
        );
        assert_eq!(
            message("Cannot read properties of undefined (reading 'length')"),
            "Cannot read properties of undefined (reading \"…\")"
        );
        // A quotation mark never closed is kept as it is.
        assert_eq!(message("can't continue"), "can't continue");
    }

    #[test]
    fn a_message_is_one_line_and_cut_short() {
        assert_eq!(message("first\nsecond\r\n\tthird"), "first second third");
        let long = message(&"word ".repeat(200));
        assert_eq!(long.chars().count(), MESSAGE_MAX);
        assert!(long.ends_with('…'));
    }

    #[test]
    fn a_stack_keeps_its_functions_and_places_in_the_code() {
        let chromium = "TypeError: Cannot read properties of null (reading 'x')\n    at load (http://tauri.localhost/_app/immutable/chunks/Review.js?v=3:12:345)\n    at async Promise.all (index 0)\n    at http://tauri.localhost/_app/immutable/entry/app.js:1:20";
        assert_eq!(
            stack(chromium),
            "load _app/immutable/chunks/Review.js:12:345 | _app/immutable/entry/app.js:1:20"
        );
        let webkit = "load@tauri://localhost/_app/immutable/chunks/Review.js:12:345\n@tauri://localhost/_app/x.js:3:4\n[native code]";
        assert_eq!(
            stack(webkit),
            "load _app/immutable/chunks/Review.js:12:345 | _app/x.js:3:4"
        );
        // Nothing but frames: text where a frame would be is not one.
        assert_eq!(stack("Le Gouverneur général\nat nothing"), "");
        let many = "f@http://h/a.js:1:1\n".repeat(40);
        assert_eq!(stack(&many).split(" | ").count(), FRAMES_MAX);
    }

    #[test]
    fn names_and_screens_keep_only_what_code_names_hold() {
        let entry = Entry::from(&InterfaceError {
            screen: "review\nforged line".into(),
            ..report("Type Error<script>", "", "")
        });
        assert_eq!(entry.screen, "reviewforgedline");
        assert_eq!(entry.name, "TypeErrorscript");
        assert_eq!(Entry::from(&report("", "", "")).name, "Error");
    }

    #[test]
    fn a_burst_is_written_ten_a_minute_the_rest_counted() {
        let mut limit = Limiter::new();
        let start = Instant::now();
        for _ in 0..PER_WINDOW {
            assert_eq!(limit.admit(start), Admit::Log { dropped_before: 0 });
        }
        for _ in 0..5 {
            assert_eq!(limit.admit(start + Duration::from_secs(30)), Admit::Drop);
        }
        assert_eq!(
            limit.admit(start + WINDOW),
            Admit::Log { dropped_before: 5 }
        );
        assert_eq!(
            limit.admit(start + WINDOW + Duration::from_secs(1)),
            Admit::Log { dropped_before: 0 }
        );
    }
}
