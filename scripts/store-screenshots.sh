#!/usr/bin/env bash
# Takes Mac App Store screenshots of the sandboxed app on a GitHub macOS runner, at 1440 × 900,
# one of the sizes App Store Connect accepts. Run by the Mac App Store workflow when its
# `screenshots` input is ticked; the images are kept as the run's artifact.
#
#   scripts/store-screenshots.sh <Kalamu.app> <output folder>
#
# The runners are Anka virtual machines: `bash` may record the screen and `osascript` may click
# (the image's TCC settings), so nothing asks. Controls are found by their accessible name in
# the window where the web view exposes them, else by their place in the window.
set -uo pipefail

app="$1"
out="$2"
mkdir -p "$out"

log() { echo "== $*"; }
screen() { system_profiler SPDisplaysDataType | grep -iE "resolution|retina|display type" || true; }

log "Screen before"
screen

# The runner's screen offers no size App Store Connect takes (its modes run from 640 × 480 to
# 1920 × 1080, none 16:10, and Anka's change_res is refused), so the screen is made larger than
# 1440 × 900 and the window alone is captured at exactly that size.
W=1440
H=900
brew install -q jakehilborn/jakehilborn/displayplacer >/dev/null 2>&1 || true
if command -v displayplacer >/dev/null; then
	displayplacer list > "$out/display-modes.txt" 2>&1
	id=$(awk '/^Persistent screen id:/ {print $4; exit}' "$out/display-modes.txt")
	[ -n "$id" ] && displayplacer "id:$id res:1600x1200 scaling:off origin:(0,0) degree:0" || true
	sleep 3
else
	echo "::warning::displayplacer could not be installed: the screen keeps its size."
fi
log "Screen after"
screen

# Light mode, the Dock hidden, a plain grey desktop behind the window's rounded corners.
osascript -e 'tell application "System Events" to tell appearance preferences to set dark mode to false' || true
defaults write com.apple.dock autohide -bool true
killall Dock 2>/dev/null || true
for grey in "/System/Library/Desktop Pictures/Solid Colors/Silver.png" \
	"/System/Library/Desktop Pictures/Solid Colors/Stone.png"; do
	if [ -f "$grey" ]; then
		osascript -e "tell application \"System Events\" to tell every desktop to set picture to \"$grey\"" || true
		break
	fi
done

open "$app"
for _ in $(seq 1 60); do
	osascript -e 'tell application "System Events" to exists window 1 of process "Kalamu"' 2>/dev/null \
		| grep -q true && break
	sleep 1
done
sleep 5

# The window, title bar and traffic lights included, is exactly W × H, just below the menu bar.
osascript -e "tell application \"System Events\" to tell process \"Kalamu\"
	set frontmost to true
	set position of window 1 to {40, 60}
	set size of window 1 to {$W, $H}
end tell" || echo "::warning::Could not place the window."
sleep 2

# The accessibility walker: with no argument it lists the window's controls (role | title |
# description); with a name it presses the first control whose title or description is that name.
ax_script="$out/.ax.applescript"
cat > "$ax_script" <<'APPLESCRIPT'
property found : missing value
property rows : {}

on label(e, key)
	tell application "System Events"
		try
			set v to value of attribute key of e
			if v is missing value then return ""
			return v as text
		on error
			return ""
		end try
	end tell
end label

on walk(e, depth, target)
	if found is not missing value then return
	tell application "System Events"
		try
			set kids to UI elements of e
		on error
			set kids to {}
		end try
	end tell
	repeat with k in kids
		set k to contents of k
		set r to my label(k, "AXRole")
		set t to my label(k, "AXTitle")
		set d to my label(k, "AXDescription")
		set end of rows to (depth as text) & " " & r & " | " & t & " | " & d
		if target is not "" and r is not "AXStaticText" and (t is target or d is target) then
			set found to k
			return
		end if
		if depth < 40 then my walk(k, depth + 1, target)
		if found is not missing value then return
	end repeat
end walk

on run argv
	set target to ""
	if (count of argv) > 0 then set target to item 1 of argv
	tell application "System Events" to tell process "Kalamu" to set frontmost to true
	tell application "System Events" to set w to window 1 of process "Kalamu"
	my walk(w, 0, target)
	if target is "" then
		set AppleScript's text item delimiters to linefeed
		return rows as text
	end if
	if found is missing value then error "No control named " & target
	tell application "System Events"
		try
			perform action "AXPress" of found
		on error
			click found
		end try
	end tell
	return "clicked " & target
end run
APPLESCRIPT

ax() { osascript "$ax_script" "$@"; }

# Clicks a place in the window: x from its left, y from its foot (the sidebar's foot keeps its
# place whatever the window's height).
click_at_bottom() {
	osascript -e "tell application \"System Events\" to tell process \"Kalamu\"
		set frontmost to true
		set {wx, wy} to position of window 1
		set {ww, wh} to size of window 1
		click at {wx + $1, wy + wh - $2}
	end tell" && echo "clicked at $1, foot - $2"
}

# Captures the window's rectangle (read back from macOS, so it is the window's real place), and
# keeps a JPEG copy, which has no alpha channel.
shot() {
	local rect
	rect=$(osascript -e 'tell application "System Events" to tell process "Kalamu"
		set {x, y} to position of window 1
		set {w, h} to size of window 1
	end tell
	return (x as text) & "," & (y as text) & "," & (w as text) & "," & (h as text)')
	screencapture -x -R"$rect" "$out/$1.png"
	echo "$1: window at $rect"
	sips -s format jpeg -s formatOptions best "$out/$1.png" --out "$out/$1.jpg" >/dev/null
	echo "$1: $(sips -g pixelWidth -g pixelHeight -g hasAlpha "$out/$1.jpg" | tail -n 3 | tr -s ' \n' ' ')"
}

osascript -e 'tell application "System Events" to tell process "Kalamu" to get {position, size} of window 1' || true
ax > "$out/controls-start.txt" 2>&1 || true
log "Elements in the window's tree: $(wc -l < "$out/controls-start.txt")"
head -n 30 "$out/controls-start.txt"

log "Start"
shot 00-start

log "Settings"
ax "Settings" || click_at_bottom 77 68
sleep 2
shot 09-settings
ax > "$out/controls-settings.txt" 2>&1 || true

log "Settings › Offline models"
ax "Offline models" || echo "::warning::No Offline models tab found by name."
sleep 2
shot 10-offline

osascript -e 'quit app "Kalamu"' || pkill -x Kalamu || true
rm -f "$ax_script"
ls -la "$out"
