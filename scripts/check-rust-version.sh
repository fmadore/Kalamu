#!/usr/bin/env bash
# `rust-version` in the workspace Cargo.toml must name the toolchain rust-toolchain.toml pins:
# every build compiles with that one, so it is the oldest Rust the code is known to build with,
# and Clippy reads `rust-version` to decide which lints apply. Dependabot moves the toolchain
# alone, so this fails its pull request until `rust-version` follows. Run by CI's `scripts` job,
# and locally from anywhere in the repository:
#
#   bash scripts/check-rust-version.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

# The first `rust-version =` under [workspace.package], and the first `channel =` under
# [toolchain]; values in double quotes, as both files write them.
cargo=$(awk -F'"' '/^\[/ { in_section = /^\[workspace\.package\]/ } in_section && /^rust-version[[:space:]]*=/ { print $2; exit }' "$root/Cargo.toml")
toolchain=$(awk -F'"' '/^\[/ { in_section = /^\[toolchain\]/ } in_section && /^channel[[:space:]]*=/ { print $2; exit }' "$root/rust-toolchain.toml")

if [ -z "$cargo" ] || [ -z "$toolchain" ]; then
  message="Could not read the versions: Cargo.toml's [workspace.package] rust-version is \"$cargo\", rust-toolchain.toml's [toolchain] channel \"$toolchain\"."
elif [ "$cargo" != "$toolchain" ]; then
  message="rust-version in Cargo.toml is $cargo but rust-toolchain.toml pins $toolchain: set rust-version to $toolchain."
else
  echo "rust-version and the pinned toolchain agree: $cargo."
  exit 0
fi

if [ "${GITHUB_ACTIONS:-}" = true ]; then
  echo "::error file=Cargo.toml::$message"
else
  echo "$message" >&2
fi
exit 1
