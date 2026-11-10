//! What the offline engines' programs wrote on their standard error, for the log when one fails
//! or hangs: the last lines, at `warn`, which the app's default filter keeps.
//!
//! llama.cpp's programs and the Whisper helper write the answer on standard output, which is
//! never logged, and on standard error what they are doing: loading (`llama_model_loader: …`,
//! `load_tensors: …`, `clip_model_loader: …`), the device (`ggml_vulkan: 0 = …`), each step
//! (`encoding image slice...`, `image slice encoded in 4102 ms`), the timings and errors
//! (`main: prompt is too long (40210 tokens, max 32764)`). Neither echoes the request there
//! unless asked to (llama.cpp's `--verbose-prompt`, which Kalamu does not pass, and its text
//! program's echo, which `--no-display-prompt` turns off; whisper.cpp's printing is turned off),
//! so the page's, the text's or the recording's words are not expected there; but a future
//! build or a debug switch could change that, so only lines shaped like a diagnostic are kept:
//! a function's or a program's name, then a colon (a name with an underscore, a digit, a hyphen
//! or a path in it, or `main`, `error`, `warning`), or one of the image steps above. A line of
//! prose, a chat template's example and anything else are left out, and every line is cut
//! short.

use std::path::Path;

/// The most lines kept, from the end.
const LINES: usize = 20;
/// The most bytes kept in all.
const BYTES: usize = 4096;
/// The most characters kept of one line.
const LINE_CHARS: usize = 240;

/// The multimodal program's steps on a page, which carry no name: where a hung run stopped.
const STEPS: [&str; 5] = [
    "encoding image",
    "image slice encoded",
    "decoding image batch",
    "image decoded",
    "terminate called",
];

/// Whether `line` reads as a diagnostic: `name: …`, as above, or one of [`STEPS`].
fn diagnostic(line: &str) -> bool {
    if STEPS.iter().any(|step| line.starts_with(step)) {
        return true;
    }
    // A source file's path on Windows starts with its drive: `D:\a\llama\ggml.c:1234: …`.
    let bytes = line.as_bytes();
    let path = bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/');
    let line = if path { &line[2..] } else { line };
    let Some((head, _)) = line.split_once(':') else {
        return false;
    };
    let named = !head.is_empty()
        && head.len() <= 64
        && head
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '\\'));
    let marked = head
        .chars()
        .any(|c| c.is_ascii_digit() || matches!(c, '_' | '-' | '/' | '\\'))
        || matches!(head, "main" | "error" | "warning");
    named && marked
}

/// The last diagnostic lines of `stderr`, at most [`LINES`] of them and [`BYTES`] in all, one
/// a line, each cut to [`LINE_CHARS`].
pub(crate) fn tail(stderr: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    let mut bytes = 0;
    for line in stderr.lines().rev() {
        let line = line.trim();
        if !diagnostic(line) {
            continue;
        }
        let line: String = line.chars().take(LINE_CHARS).collect();
        if kept.len() == LINES || bytes + line.len() + 1 > BYTES {
            break;
        }
        bytes += line.len() + 1;
        kept.push(line);
    }
    kept.reverse();
    kept.join("\n")
}

/// Logs the end of what `program` wrote on its standard error `during` a run that failed;
/// nothing when it wrote nothing that reads as a diagnostic (a graphics build on a PC without
/// the Vulkan loader ends at once, silent, at every launch). The failure itself reaches the
/// log or the job through its error.
pub(crate) fn warn(program: &Path, during: &str, stderr: &[u8]) {
    let diagnostics = tail(&String::from_utf8_lossy(stderr));
    if diagnostics.is_empty() {
        return;
    }
    let program = program
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    tracing::warn!(%program, during, %diagnostics, "an offline engine's program failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_are_kept_and_prose_left_out() {
        let stderr = "\
llama_model_loader: loaded meta data with 40 key-value pairs
print_info: file type   = Q8_0
ggml_vulkan: 0 = NVIDIA GeForce RTX 3060 (NVIDIA) | uma: 0 | fp16: 1
Note: the minister arrived in Porto-Novo on Tuesday.
<start_of_turn>user
Hello<end_of_turn>
encoding image slice...
image slice encoded in 4102 ms
D:\\a\\llama\\ggml\\src\\ggml.c:1234: GGML_ASSERT(n > 0) failed
main: prompt is too long (40210 tokens, max 32764)
terminate called after throwing an instance of 'std::runtime_error'
";
        assert_eq!(
            tail(stderr),
            "\
llama_model_loader: loaded meta data with 40 key-value pairs
print_info: file type   = Q8_0
ggml_vulkan: 0 = NVIDIA GeForce RTX 3060 (NVIDIA) | uma: 0 | fp16: 1
encoding image slice...
image slice encoded in 4102 ms
D:\\a\\llama\\ggml\\src\\ggml.c:1234: GGML_ASSERT(n > 0) failed
main: prompt is too long (40210 tokens, max 32764)
terminate called after throwing an instance of 'std::runtime_error'"
        );
        assert_eq!(tail(""), "");
    }

    #[test]
    fn only_the_last_lines_are_kept_within_bounds() {
        let many: String = (0..100)
            .map(|i| format!("load_tensors: layer {i}\n"))
            .collect();
        let kept = tail(&many);
        assert_eq!(kept.lines().count(), LINES);
        assert!(kept.ends_with("load_tensors: layer 99"));
        let long = format!("llama_model_loader: {}\n", "x".repeat(10_000)).repeat(40);
        let kept = tail(&long);
        assert!(kept.len() <= BYTES);
        assert!(kept.lines().all(|l| l.chars().count() <= LINE_CHARS));
    }
}
