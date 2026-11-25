#!/usr/bin/env bash
# Starts the test build of Kalamu on macOS (a bundle built with `--features test-overrides`),
# checks that it opens and checks its job database, quits it as the system asks an app to quit,
# and checks that it ended cleanly. WKWebView has no WebDriver, so this is the Mac's share of
# .github/workflows/app-driven.yml; Windows drives the flows themselves.
#
#   scripts/app-driven/launch-check.sh <path to Kalamu.app>
#
# The app runs in a data folder of its own, every cloud provider at the stand-in provider
# (started here, and never asked anything at launch), its keys in memory: the runner's keychain
# is never touched.

set -euo pipefail

app="${1:?the path to Kalamu.app}"
here="$(cd "$(dirname "$0")" && pwd)"
exe="$(find "$app/Contents/MacOS" -maxdepth 1 -type f -iname kalamu | head -n 1)"
[ -n "$exe" ] || { echo "::error::No Kalamu executable in $app/Contents/MacOS."; exit 1; }

root="$(mktemp -d)"
data="$root/data"
logs="$data/logs"
mkdir -p "$data"
printf '{"language":"en","refresh_catalog":false}\n' > "$data/settings.json"

node "$here/stand-in.mjs" --port 0 > "$root/stand-in.txt" &
stand_in=$!
for _ in $(seq 1 50); do
  grep -q 'stand-in provider at' "$root/stand-in.txt" 2>/dev/null && break
  sleep 0.2
done
url="$(sed -n 's/^stand-in provider at //p' "$root/stand-in.txt")"
[ -n "$url" ] || { echo "::error::The stand-in provider did not start."; exit 1; }

KALAMU_TEST_DATA="$data" KALAMU_TEST_PROVIDERS="$url" "$exe" > "$root/app.txt" 2>&1 &
pid=$!

finish() {
  kill "$stand_in" 2> /dev/null || true
  kill "$pid" 2> /dev/null || true
  echo "== app output"; cat "$root/app.txt" || true
  echo "== log"; cat "$logs"/kalamu.log* 2> /dev/null || true
}
trap finish EXIT

# Open: the database made, then checked whole and copied for the day once the window is up.
opened=""
for _ in $(seq 1 120); do
  if ! kill -0 "$pid" 2> /dev/null; then
    echo "::error::The app ended at launch."
    exit 1
  fi
  if [ -f "$data/jobs.sqlite" ] && grep -q "job database copied" "$logs"/kalamu.log* 2> /dev/null; then
    opened=yes
    break
  fi
  sleep 1
done
[ -n "$opened" ] || { echo "::error::The app did not open and check its job database within two minutes."; exit 1; }
if grep -q "the job database's check found damage" "$logs"/kalamu.log*; then
  echo "::error::The app found its new job database damaged."
  exit 1
fi

# Quit as the system asks an app to (the menu's Quit, logging out): Apple's quit event.
osascript -e 'tell application id "io.github.fmadore.kalamu" to quit' || true
status=""
for _ in $(seq 1 60); do
  if ! kill -0 "$pid" 2> /dev/null; then
    status=0
    wait "$pid" || status=$?
    break
  fi
  sleep 1
done
[ -n "$status" ] || { echo "::error::The app did not end within a minute of being asked to quit."; exit 1; }
[ "$status" = 0 ] || { echo "::error::The app ended with status $status."; exit 1; }
grep -q "the app ends" "$logs"/kalamu.log* \
  || { echo "::error::The app ended without passing through its exit (no \"the app ends\" in its log)."; exit 1; }
if [ -f "$logs/panic.log" ]; then
  echo "::error::The app panicked."; cat "$logs/panic.log"; exit 1
fi
echo "The app started, opened its database and ended cleanly: $app"
