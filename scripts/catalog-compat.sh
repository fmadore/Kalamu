#!/usr/bin/env bash
# Today's model list read by the reader of every release people may still run: each release
# tag's own `kalamu-catalog` (and the `kalamu-core` it reads with), built alone and asked to read
# `crates/catalog/catalog.json` as an installed copy reads it once downloaded from `main`. A
# change that one of them refuses would lock that version out of every later list.
#
# Which releases: every tag `vX.Y.Z` from 1.0.0 on, found in the repository, so a new release is
# checked from the day it is tagged. 1.6.0 is among them: withdrawn from the Mac App Store, it is
# still on the releases page. The 0.x test pre-releases are not.
# (docs/decisions/2026-11-25-model-list-checked.md)
#
# Usage: scripts/catalog-compat.sh [list.json]
# CATALOG_COMPAT_DIR: where the releases' sources and their one build go (target/catalog-compat
# by default; delete it afterwards).
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
list="$(realpath "${1:-$root/crates/catalog/catalog.json}")"
work="${CATALOG_COMPAT_DIR:-$root/target/catalog-compat}"
mkdir -p "$work"
work="$(realpath "$work")"
# One build folder for every release: what two releases share is built once.
export CARGO_TARGET_DIR="$work/target"

python="$(command -v python3 || command -v python)"
expected="$("$python" -c 'import json, sys; print(len(json.load(open(sys.argv[1], encoding="utf-8"))["models"]))' "$list")"

probe='fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    let json = std::fs::read_to_string(&path).unwrap_or_default();
    match kalamu_catalog::Catalog::from_json(&json) {
        Ok(catalog) => println!("{}", catalog.models.len()),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}'

tags="$(git -C "$root" tag --list 'v*' | grep -E '^v[1-9][0-9]*\.[0-9]+\.[0-9]+$' | sort -V)"
if [ -z "$tags" ]; then
  echo "No release tags: fetch them first (actions/checkout with fetch-depth: 0)." >&2
  exit 1
fi

failed=0
for tag in $tags; do
  dir="$work/$tag"
  rm -rf "$dir"
  mkdir -p "$dir"
  # The workspace's manifests and lock, and the two crates the reader needs, nothing more: the
  # catalogue crate builds without the app, its native libraries or the whisper environment.
  git -C "$root" archive "$tag" Cargo.toml Cargo.lock crates/core crates/catalog | tar -x -C "$dir"
  sed -i.orig -e 's|^members = .*|members = ["crates/core", "crates/catalog"]|' \
    -e '/^default-members = /d' "$dir/Cargo.toml"
  mkdir -p "$dir/crates/catalog/examples"
  printf '%s\n' "$probe" > "$dir/crates/catalog/examples/read_today.rs"
  # Not --locked: leaving the other members out drops their packages from the lock; the
  # versions kept are the release's own. The toolchain is today's (rust-toolchain.toml at the
  # root), which builds every release's code.
  manifest=(--manifest-path "$dir/Cargo.toml" -p kalamu-catalog --example read_today)
  if ! (cd "$root" && cargo build --quiet "${manifest[@]}" 2> "$dir/error.txt"); then
    echo "::error::$tag could not be built: $(tail -n 5 "$dir/error.txt" | tr '\n' ' ')"
    failed=1
  elif read="$(cd "$root" && cargo run --quiet "${manifest[@]}" -- "$list" 2> "$dir/error.txt")"; then
    if [ "$read" = "$expected" ]; then
      echo "$tag reads today's list: $read models"
    else
      echo "$tag reads today's list, $((expected - read)) of its $expected models hidden"
    fi
  else
    echo "::error::$tag refuses today's list: $(tail -n 5 "$dir/error.txt" | tr '\n' ' ')"
    failed=1
  fi
done
exit "$failed"
