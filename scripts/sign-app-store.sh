#!/usr/bin/env bash
# Signs the Mac App Store build again from the inside out, after `tauri build`
# (docs/decisions/2026-11-04-mac-app-store-feasibility.md, point 3).
#
# Tauri's bundler signs FFmpeg, ffprobe and llama.cpp's program with the app's own entitlements.
# In the App Sandbox an embedded tool needs exactly `app-sandbox` and `inherit`
# (src-tauri/appstore/Helper.plist) to run inside the app's sandbox, so each is signed again with
# those, under identifiers of their own; the app is then signed over them, since changing nested
# code breaks its seal.
#
#   scripts/sign-app-store.sh <Kalamu.app> <identity> <app entitlements>
#
# `identity` is the Apple Distribution certificate's name for the store, or `-` (ad hoc) for
# CI's sandbox check, with src-tauri/appstore/Sandbox.plist as the app's entitlements.
set -euo pipefail

app=$1
identity=$2
entitlements=$3
helper="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/appstore/Helper.plist"
bundle_id=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app/Contents/Info.plist")

for tool in ffmpeg ffprobe llama-mtmd-cli; do
  codesign --force --sign "$identity" --options runtime --timestamp=none \
    --identifier "$bundle_id.$tool" --entitlements "$helper" "$app/Contents/MacOS/$tool"
done
codesign --force --sign "$identity" --options runtime --timestamp=none \
  --entitlements "$entitlements" "$app"

codesign --verify --deep --strict --verbose=2 "$app"
for exe in "$app" "$app/Contents/MacOS/ffmpeg" "$app/Contents/MacOS/ffprobe" "$app/Contents/MacOS/llama-mtmd-cli"; do
  echo "== $exe"
  codesign -d --entitlements - --xml "$exe" 2>/dev/null | plutil -p - || true
done
