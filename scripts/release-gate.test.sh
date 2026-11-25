#!/usr/bin/env bash
# Tests scripts/release-gate.sh against throwaway repositories, one per case, in a temporary
# folder removed afterwards. Run by CI's `scripts` job, and locally (Git Bash on Windows too):
#
#   bash scripts/release-gate.test.sh
set -euo pipefail

gate="$(cd "$(dirname "$0")" && pwd)/release-gate.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

failures=0
cases=0

# A fresh repository with one commit, and a helper to add commits touching a given path.
new_repo() {
  repo="$work/$1"
  git init -q -b main "$repo"
  git -C "$repo" config user.name test
  git -C "$repo" config user.email test@example.invalid
  git -C "$repo" config commit.gpgsign false
  git -C "$repo" config tag.gpgsign false
  git -C "$repo" config core.autocrlf false
  commit_touching src/main.rs "first"
}

commit_touching() {
  mkdir -p "$repo/$(dirname "$1")"
  echo "$2 $RANDOM" >> "$repo/$1"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "$2"
}

tag() {
  git -C "$repo" tag "$1" "${2:-HEAD}"
}

# expect <exit code> <case name> <tag> [text the output must contain]
expect() {
  local want=$1 name=$2 release_tag=$3 needle=${4:-} got out
  cases=$((cases + 1))
  set +e
  out=$(cd "$repo" && env -u GITHUB_ACTIONS -u GITHUB_OUTPUT bash "$gate" "$release_tag" 2>&1)
  got=$?
  set -e
  if [ "$got" != "$want" ]; then
    echo "FAIL $name: exit $got, expected $want"
    echo "$out" | sed 's/^/      /'
    failures=$((failures + 1))
  elif [ -n "$needle" ] && ! grep -qF -- "$needle" <<< "$out"; then
    echo "FAIL $name: the output lacks \"$needle\""
    echo "$out" | sed 's/^/      /'
    failures=$((failures + 1))
  else
    echo "ok   $name"
  fi
}

unset RELEASE_WITHOUT_CANDIDATE

new_repo same-commit
tag v1.2.0-rc1
tag v1.2.0
expect 0 "a candidate on the same commit is allowed" v1.2.0 "on the commit of release candidate v1.2.0-rc1"

new_repo annotated
git -C "$repo" tag -a -m candidate v1.2.0-rc1
git -C "$repo" tag -a -m release v1.2.0
expect 0 "annotated tags are followed to their commit" v1.2.0 "v1.2.0-rc1"

new_repo ancestor-docs
tag v1.2.0-rc1
commit_touching docs/release-1.2.0.md "Release notes for 1.2.0"
commit_touching docs/windows-test-pass.md "Test pass results"
commit_touching README.md "README"
tag v1.2.0
expect 0 "a candidate on an ancestor, documentation only since, is allowed" v1.2.0 "3 commit(s) after"

new_repo ancestor-code
tag v1.2.0-rc1
commit_touching docs/release-1.2.0.md "Release notes for 1.2.0"
commit_touching src/main.rs "A fix after the test pass"
tag v1.2.0
expect 1 "a candidate on an ancestor with code changed since is refused" v1.2.0 "A fix after the test pass"
expect 1 "the refusal names the files outside the documentation" v1.2.0 "src/main.rs"

new_repo ancestor-prompt
tag v1.2.0-rc1
commit_touching crates/prompts/defaults/printed.md "A default prompt reworded"
tag v1.2.0
expect 1 "Markdown compiled into the app is not documentation" v1.2.0 "crates/prompts/defaults/printed.md"

new_repo nearest
tag v1.2.0-rc1
commit_touching src/main.rs "A fix found by the test pass"
tag v1.2.0-rc2
commit_touching docs/release-1.2.0.md "Release notes for 1.2.0"
tag v1.2.0
expect 0 "an older candidate refused, a newer one qualifying: allowed" v1.2.0 "release candidate v1.2.0-rc2"

new_repo none
tag v1.2.0
expect 1 "no candidate is refused" v1.2.0 "has no release candidate"

new_repo other-version
tag v1.1.0-rc1
tag v1.20.0-rc1
tag v1.2.0-test
tag v1.2.0
expect 1 "a candidate of another version only is refused" v1.2.0 "has no release candidate"

new_repo not-ancestor
commit_touching src/main.rs "On main"
git -C "$repo" switch -q -c side HEAD~1
commit_touching src/main.rs "On a side branch"
tag v1.2.0-rc1
git -C "$repo" switch -q main
tag v1.2.0
expect 1 "a candidate on a commit that is not an ancestor is refused" v1.2.0 "is not an ancestor"

new_repo descendant
tag v1.2.0
commit_touching docs/notes.md "Later"
tag v1.2.0-rc1
expect 1 "a candidate cut after the final tag is refused" v1.2.0 "is not an ancestor"

new_repo override
tag v1.2.0
export RELEASE_WITHOUT_CANDIDATE="publish v1.2.0 without a candidate"
expect 0 "the exact sentence lets a version through without a candidate" v1.2.0 "without one"
export RELEASE_WITHOUT_CANDIDATE="without a candidate"
expect 1 "a sentence that is not the exact one is refused" v1.2.0 "is not that sentence"
export RELEASE_WITHOUT_CANDIDATE="publish v1.1.0 without a candidate"
expect 1 "the sentence for another version is refused" v1.2.0 "is not that sentence"
unset RELEASE_WITHOUT_CANDIDATE

new_repo outputs
tag v1.2.0
cases=$((cases + 1))
out_file="$work/github-output"
: > "$out_file"
(cd "$repo" && GITHUB_ACTIONS=true GITHUB_OUTPUT="$out_file" \
  RELEASE_WITHOUT_CANDIDATE="publish v1.2.0 without a candidate" bash "$gate" v1.2.0 > /dev/null)
if grep -qx "without_candidate=true" "$out_file" && grep -qx "candidate=" "$out_file"; then
  echo "ok   the workflow's outputs record a release without a candidate"
else
  echo "FAIL the workflow's outputs: $(tr '\n' ' ' < "$out_file")"
  failures=$((failures + 1))
fi

new_repo validation
tag v1.2.0-rc1
expect 2 "a hyphenated tag is not gated here" v1.2.0-rc1 "not a final version tag"
expect 2 "a missing tag cannot be judged" v9.9.9 "no tag v9.9.9"

echo
if [ "$failures" -gt 0 ]; then
  echo "$failures of $cases cases failed."
  exit 1
fi
echo "All $cases cases passed."
