#!/usr/bin/env bash
# The release gate (docs/decisions/2026-11-25-release-gate.md): a final version tag (vX.Y.Z) is
# published only when a release candidate of the same version (vX.Y.Z-rcN) was cut from the
# same commit, or from an ancestor of it with nothing but documentation changed since. The
# candidate is the build the test pass tried; a commit that changes anything else after it is a
# commit no one has tried.
#
#   scripts/release-gate.sh <tag>
#
# Run inside a clone that holds the full history and every tag (`fetch-depth: 0`). Exits 0 when
# the tag may be published, 1 when it may not, 2 when it cannot tell (not a final version tag,
# the tag is missing). A hyphenated tag is a validation build and is not gated: release.yml
# does not call this for one.
#
# RELEASE_WITHOUT_CANDIDATE, set from the workflow's manual input, lets a final version through
# without a candidate when it reads exactly "publish vX.Y.Z without a candidate" for this tag;
# the release notes then say so. A candidate that qualifies always wins over it.
#
# In GitHub Actions, writes `candidate=<tag or empty>` and `without_candidate=<true|false>` to
# $GITHUB_OUTPUT. Tested by scripts/release-gate.test.sh against throwaway repositories.
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: scripts/release-gate.sh <tag>" >&2
  exit 2
fi
tag=$1

error() {
  if [ "${GITHUB_ACTIONS:-}" = true ]; then
    # A workflow annotation holds one line; the details follow in the log.
    echo "::error::$1"
  else
    echo "Refused: $1" >&2
  fi
}

output() {
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "$1" >> "$GITHUB_OUTPUT"
  fi
}

if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "$tag is not a final version tag (vX.Y.Z); only those are gated." >&2
  exit 2
fi
if ! commit=$(git rev-parse --verify --quiet "refs/tags/$tag^{commit}"); then
  echo "There is no tag $tag in this clone; it needs every tag (fetch-depth: 0)." >&2
  exit 2
fi

# What may change between a candidate and the final tag without a new candidate: the
# documentation folder (release notes, test-pass results, the guide) and the Markdown files at
# the repository's root. Not Markdown elsewhere: the default prompts (crates/prompts/defaults)
# are Markdown compiled into the app.
documentation_only() {
  local path
  while IFS= read -r -d '' path; do
    case $path in
      docs/*) ;;
      */*) return 1 ;;
      *.md) ;;
      *) return 1 ;;
    esac
  done < <(git diff --no-renames --name-only -z "$1" "$2")
}

files_outside_documentation() {
  local path
  while IFS= read -r -d '' path; do
    case $path in
      docs/*) ;;
      */*) echo "    $path" ;;
      *.md) ;;
      *) echo "    $path" ;;
    esac
  done < <(git diff --no-renames --name-only -z "$1" "$2")
}

# Candidates are vX.Y.Z-rcN exactly: another hyphenated tag (v1.2.0-test) is a validation build
# but not a candidate.
mapfile -t candidates < <(git tag --list "$tag-rc*" | grep -E "^${tag//./\\.}-rc[0-9]+$" || true)

chosen=''
chosen_distance=''
report=()
for candidate in "${candidates[@]}"; do
  candidate_commit=$(git rev-parse "refs/tags/$candidate^{commit}")
  short=$(git rev-parse --short "$candidate_commit")
  if [ "$candidate_commit" = "$commit" ]; then
    chosen=$candidate
    chosen_distance=0
    break
  fi
  if ! git merge-base --is-ancestor "$candidate_commit" "$commit"; then
    report+=("  $candidate ($short) is not an ancestor of $tag: it was cut on another line of history.")
    continue
  fi
  distance=$(git rev-list --count "$candidate_commit..$commit")
  if documentation_only "$candidate_commit" "$commit"; then
    # The nearest qualifying candidate, when several do.
    if [ -z "$chosen" ] || [ "$distance" -lt "$chosen_distance" ]; then
      chosen=$candidate
      chosen_distance=$distance
    fi
  else
    report+=("  $candidate ($short) is $distance commit(s) behind $tag, and they change more than documentation:")
    while IFS= read -r line; do report+=("    $line"); done < <(git log --format='%h %s' "$candidate_commit..$commit")
    report+=("  Files changed outside docs/ and the root's Markdown files:")
    while IFS= read -r line; do report+=("$line"); done < <(files_outside_documentation "$candidate_commit" "$commit")
  fi
done

if [ -n "$chosen" ]; then
  if [ "$chosen_distance" = 0 ]; then
    echo "$tag is on the commit of release candidate $chosen."
  else
    echo "$tag is $chosen_distance commit(s) after release candidate $chosen, documentation only:"
    git log --format='  %h %s' "refs/tags/$chosen..$commit"
  fi
  output "candidate=$chosen"
  output "without_candidate=false"
  exit 0
fi

phrase="publish $tag without a candidate"
given=${RELEASE_WITHOUT_CANDIDATE:-}
if [ "$given" = "$phrase" ]; then
  echo "No release candidate qualifies for $tag; published without one, as the manual run asked. The release notes say so."
  for line in "${report[@]}"; do echo "$line"; done
  output "candidate="
  output "without_candidate=true"
  exit 0
fi

if [ ${#candidates[@]} -eq 0 ]; then
  error "$tag has no release candidate: no tag $tag-rcN exists."
else
  error "No release candidate of $tag was cut from its commit, or from an ancestor with only documentation changed since."
  for line in "${report[@]}"; do echo "$line" >&2; done
fi
{
  echo "Push $tag-rcN on $tag's commit ($(git rev-parse --short "$commit")), let its build finish and run the test pass on it, then run this again."
  echo "To publish without a candidate, run the Release workflow by hand from the tag and type, in its last box: $phrase"
} >&2
if [ -n "$given" ]; then
  echo "The manual run's box read \"$given\", which is not that sentence." >&2
fi
exit 1
