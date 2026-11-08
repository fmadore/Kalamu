//! The crate table in `AGENTS.md`, checked: each crate depends only on the internal crates the
//! table lets it use, so dependencies keep pointing inwards.
//!
//! It lives here because every crate depends on `core`, which is also the cheapest to build, so
//! the check runs in every `cargo test`. Dev-dependencies are not checked: a crate's tests may
//! use a real store or exporter as a fixture without the crate itself depending on it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

const CORE: &str = "kalamu-core";
const CATALOG: &str = "kalamu-catalog";
const PROMPTS: &str = "kalamu-prompts";
const MEDIA: &str = "kalamu-media";
const PROVIDERS: &str = "kalamu-providers";
const OFFLINE: &str = "kalamu-offline";
const STORE: &str = "kalamu-store";
const PROJECTS: &str = "kalamu-projects";
const DICTIONARY: &str = "kalamu-dictionary";
const SEARCH: &str = "kalamu-search";
const EXPORT: &str = "kalamu-export";
const ENGINE: &str = "kalamu-engine";

/// The engine may use every library crate; the app, everything.
const LIBRARIES: &[&str] = &[
    CORE, CATALOG, PROMPTS, MEDIA, PROVIDERS, OFFLINE, STORE, PROJECTS, DICTIONARY, SEARCH, EXPORT,
];

/// The internal crates each workspace member may depend on, as the table in `AGENTS.md` says.
fn allowed(member: &str) -> Option<Vec<&'static str>> {
    Some(match member {
        CORE => vec![],
        CATALOG | PROMPTS | MEDIA | PROVIDERS | STORE | PROJECTS | DICTIONARY | SEARCH | EXPORT => {
            vec![CORE]
        }
        OFFLINE => vec![CORE, MEDIA],
        ENGINE => LIBRARIES.to_vec(),
        "kalamu" => [LIBRARIES, &[ENGINE]].concat(),
        // A program beside the crates: it includes `offline`'s engine file by path instead.
        "whisper-vulkan" => vec![],
        _ => return None,
    })
}

#[test]
fn dependencies_point_inwards() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let packages = metadata["packages"].as_array().unwrap();
    let members: BTreeSet<&str> = packages
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();

    let mut wrong = Vec::new();
    for package in packages {
        let name = package["name"].as_str().unwrap();
        let Some(allowed) = allowed(name) else {
            wrong.push(format!(
                "{name} is not in the crate table: add it to AGENTS.md and here"
            ));
            continue;
        };
        for dependency in package["dependencies"].as_array().unwrap() {
            let dependency_name = dependency["name"].as_str().unwrap();
            let is_dev = dependency["kind"].as_str() == Some("dev");
            if members.contains(dependency_name) && !is_dev && !allowed.contains(&dependency_name) {
                wrong.push(format!("{name} may not depend on {dependency_name}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
