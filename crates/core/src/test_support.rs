//! Helpers for tests here and in other crates, behind the `test-support` feature.

use std::path::Path;

/// Writes `value` as pretty JSON, ending with a new line, to `path` relative to the repository's
/// root, and only when the content changed, so `cargo test` keeps a file the interface imports
/// current without touching it (and its watchers) on every run.
pub fn write_interface_json(
    path: &str,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    let json = serde_json::to_string_pretty(value)? + "\n";
    if std::fs::read_to_string(&path).ok().as_deref() != Some(json.as_str()) {
        std::fs::write(&path, json)?;
    }
    Ok(())
}

mod fake_embedder;
pub use fake_embedder::{FakeEmbedder, fake_vector};
