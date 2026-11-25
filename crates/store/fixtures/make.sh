#!/bin/sh
# Makes the job databases in this folder, each written by an earlier release's own store crate,
# which the store's tests migrate to the current schema and read back (`src/tests/fixtures.rs`).
#
#   sh crates/store/fixtures/make.sh v1.11.0 [more tags…]
#
# Run from the repository's root, in Git Bash on Windows or a shell on macOS, with the Rust that
# `rust-toolchain.toml` names on the PATH. For each tag, the release's `crates/core` and
# `crates/store` are taken out of git into a scratch folder (nothing is checked out here), made
# a workspace of their own, `generator.rs` is appended to the release's store tests, and that
# one test writes `jobs-<version>.sqlite` here. The store builds without the Whisper or
# Tesseract environment. The scratch folder is deleted afterwards; set KALAMU_FIXTURE_WORK to
# keep it elsewhere.
#
# A fixture is added for each release that changed the schema (`src/migrations.rs`) or what a
# stored record holds (the types in `crates/core` the store writes as JSON). Patch releases
# that changed neither need none. Then add its version to `FIXTURES` in `src/tests/fixtures.rs`.
set -eu

root=$(pwd)
here="$root/crates/store/fixtures"
[ -f "$here/generator.rs" ] || { echo "run from the repository's root" >&2; exit 1; }
work=${KALAMU_FIXTURE_WORK:-"$root/target/fixture-work"}
mkdir -p "$work"
# One target folder for every tag, so their dependencies are built once.
export CARGO_TARGET_DIR="$work/target"
# An old release built by a newer Rust: its warnings stay warnings.
export RUSTFLAGS="--cap-lints warn"

for tag in "$@"; do
    version=${tag#v}
    src="$work/kalamu-$version"
    rm -rf "$src"
    mkdir -p "$src"
    git archive "$tag" Cargo.toml Cargo.lock crates/core crates/store \
        | tar -x -C "$src"
    # The two crates alone, as a workspace.
    sed -i.bak \
        -e 's|^members = .*|members = ["crates/core", "crates/store"]|' \
        -e 's|^default-members = .*|default-members = ["crates/core", "crates/store"]|' \
        "$src/Cargo.toml"
    if [ -f "$src/crates/store/src/tests/projects.rs" ]; then
        cat "$here/generator.rs" >> "$src/crates/store/src/tests/projects.rs"
        test=tests::projects::write_fixture
    else
        sed '/PROJECTS-BEGIN/,/PROJECTS-END/d' "$here/generator.rs" \
            >> "$src/crates/store/src/tests.rs"
        test=tests::write_fixture
    fi
    out="$here/jobs-$version.sqlite"
    (cd "$src" && KALAMU_FIXTURE_OUT="$out" cargo test -p kalamu-store --lib \
        -- --ignored --exact "$test")
    [ -f "$out" ] || { echo "no fixture written for $tag" >&2; exit 1; }
    echo "wrote $out"
    rm -rf "$src"
done
if [ -z "${KALAMU_FIXTURE_WORK:-}" ]; then
    rm -rf "$work"
fi
