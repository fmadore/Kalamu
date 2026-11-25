//! The test build's overrides (`--features test-overrides`: its own data folder, the providers
//! at the stand-in's address, keys never in the keychain) must never reach a released build. A
//! release build refuses to compile with them (`src/test_overrides.rs`); these checks fail if a
//! default, a Tauri configuration or any workflow but the one that drives the app turns them on.

use std::path::{Path, PathBuf};

const FEATURE: &str = "test-overrides";
/// The one workflow that builds with the feature: the app driven against the stand-in.
const DRIVING_WORKFLOW: &str = "app-driven.yml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every file under `dir`, recursively.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found
}

/// The `[features]` table of the shell's manifest, as text, to its next table.
fn features_table(manifest: &str) -> &str {
    let start = manifest
        .find("\n[features]")
        .unwrap_or_else(|| panic!("src-tauri/Cargo.toml has no [features]"));
    let rest = &manifest[start + 1..];
    let end = rest[1..].find("\n[").map_or(rest.len(), |i| i + 1);
    &rest[..end]
}

#[test]
fn no_default_or_other_feature_turns_the_overrides_on() {
    let manifest = read(&root().join("src-tauri").join("Cargo.toml"));
    let table = features_table(&manifest);
    assert!(
        table
            .lines()
            .any(|l| l.starts_with(&format!("{FEATURE} = []"))),
        "the feature is declared, enabling nothing else"
    );
    for line in table.lines().filter(|l| !l.trim_start().starts_with('#')) {
        if line.starts_with(FEATURE) {
            continue;
        }
        assert!(
            !line.contains(FEATURE),
            "only the test build names {FEATURE}: {line}"
        );
    }
    // No dependency on the shell turns it on either.
    for crate_manifest in files(&root().join("crates")) {
        if crate_manifest
            .file_name()
            .is_some_and(|n| n == "Cargo.toml")
        {
            assert!(
                !read(&crate_manifest).contains(FEATURE),
                "{crate_manifest:?}"
            );
        }
    }
}

#[test]
fn no_tauri_configuration_builds_with_the_overrides() {
    for config in files(&root().join("src-tauri")) {
        let json = config.extension().is_some_and(|e| e == "json");
        if json && !config.components().any(|c| c.as_os_str() == "gen") {
            assert!(!read(&config).contains(FEATURE), "{config:?}");
        }
    }
}

#[test]
fn only_the_driving_workflow_builds_with_the_overrides_and_only_for_debug() {
    let github = root().join(".github");
    let mut driving = None;
    for file in files(&github) {
        let text = read(&file);
        if !text.contains(FEATURE) {
            continue;
        }
        assert!(
            file.file_name().is_some_and(|n| n == DRIVING_WORKFLOW),
            "{} builds with {FEATURE}: only {DRIVING_WORKFLOW} may",
            file.display()
        );
        driving = Some(text);
    }
    let driving = driving.unwrap_or_else(|| panic!("{DRIVING_WORKFLOW} does not use {FEATURE}"));
    // Its builds are debug builds: never a release profile, never a release's secrets.
    assert!(!driving.contains("--release"), "a release build");
    assert!(
        !driving.contains("secrets."),
        "a secret in the driving workflow"
    );
    for line in driving.lines().filter(|l| l.contains("tauri build")) {
        assert!(line.contains("--debug"), "a release build: {line}");
    }
    // The scripts a release runs never name it.
    for script in files(&root().join("scripts")) {
        let name = script.to_string_lossy().replace('\\', "/");
        if name.contains("/app-driven/") {
            continue;
        }
        assert!(!read(&script).contains(FEATURE), "{name}");
    }
}

#[test]
fn a_release_build_refuses_the_overrides() {
    let module = read(
        &root()
            .join("src-tauri")
            .join("src")
            .join("test_overrides.rs"),
    );
    let guard = module.find("#[cfg(not(debug_assertions))]");
    let error = module.find("compile_error!(");
    assert!(
        matches!((guard, error), (Some(g), Some(e)) if e > g && e - g < 60),
        "test_overrides.rs refuses to compile without debug assertions"
    );
    let lib = read(&root().join("src-tauri").join("src").join("lib.rs"));
    assert!(
        lib.contains("#[cfg(feature = \"test-overrides\")]\nmod test_overrides;"),
        "the module exists only with the feature"
    );
}
