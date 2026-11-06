#!/usr/bin/env bash
# Takes Mac App Store screenshots of the sandboxed app on a GitHub macOS runner, at 1440 × 900,
# one of the sizes App Store Connect accepts. Run by the Mac App Store workflow when its
# `screenshots` input is ticked; the images are kept as the run's artifact.
#
#   scripts/store-screenshots.sh <Kalamu.app> <output folder>
#
# The runners are Anka virtual machines: Anka's `change_res` sets the screen size, `bash` may
# record the screen and `osascript` may click (the image's TCC settings), so nothing asks.
# Buttons are found by their accessible name in the window, never by position.
set -uo pipefail

app="$1"
out="$2"
mkdir -p "$out"

log() { echo "== $*"; }

log "Screen before"
system_profiler SPDisplaysDataType | grep -iE "resolution|retina|display type" || true

change_res="/Library/Application Support/Veertu/Anka/addons/change_res"
if [ -x "$change_res" ]; then
	"$change_res" 1440x900x32@30 || echo "::warning::change_res failed."
	sleep 3
else
	echo "::warning::Anka's change_res is not on this runner: the screen keeps its size."
fi
log "Screen after"
system_profiler SPDisplaysDataType | grep -iE "resolution|retina|display type" || true

# Light mode, the Dock hidden, no desktop icons in the way.
osascript -e 'tell application "System Events" to tell appearance preferences to set dark mode to false' || true
defaults write com.apple.dock autohide -bool true
killall Dock 2>/dev/null || true

open "$app"
for _ in $(seq 1 60); do
	osascript -e 'tell application "System Events" to exists window 1 of process "Kalamu"' 2>/dev/null \
		| grep -q true && break
	sleep 1
done
sleep 5

# The window fills the screen below the menu bar, keeping its title bar and traffic lights.
osascript <<'EOF' || echo "::warning::Could not place the window."
tell application "System Events" to tell process "Kalamu"
	set frontmost to true
	set position of window 1 to {0, 0}
	set size of window 1 to {1440, 900}
end tell
EOF
sleep 2

# Clicks the first button, tab or link whose accessible name is the given text.
click() {
	osascript - "$1" <<'EOF'
on run argv
	set target to item 1 of argv
	tell application "System Events" to tell process "Kalamu"
		set frontmost to true
		repeat with e in (entire contents of window 1)
			try
				set r to role of e
				if r is "AXButton" or r is "AXRadioButton" or r is "AXLink" or r is "AXTab" then
					set n to ""
					try
						set n to name of e
					end try
					set d to ""
					try
						set d to description of e
					end try
					if n is target or d is target then
						try
							perform action "AXPress" of e
						on error
							click e
						end try
						return "clicked " & r & " " & target
					end if
				end if
			end try
		end repeat
	end tell
	error "No control named " & target
end run
EOF
}

# The window's controls, for writing the steps of new scenes.
osascript > "$out/controls.txt" 2>&1 <<'EOF' || true
set lines to {}
tell application "System Events" to tell process "Kalamu"
	repeat with e in (entire contents of window 1)
		try
			set r to role of e
			if r is "AXButton" or r is "AXRadioButton" or r is "AXLink" or r is "AXTab" or r is "AXTextField" then
				set n to ""
				try
					set n to name of e
				end try
				set d to ""
				try
					set d to description of e
				end try
				set end of lines to r & " | " & n & " | " & d
			end if
		end try
	end repeat
end tell
set AppleScript's text item delimiters to linefeed
return lines as text
EOF

shot() {
	screencapture -x "$out/$1.png"
	sips -g pixelWidth -g pixelHeight -g hasAlpha "$out/$1.png" | tail -n 3
}

log "Start"
shot 00-start

log "Settings › Offline"
click "Settings" && sleep 2 && click "Offline" && sleep 2
shot 10-offline

osascript -e 'quit app "Kalamu"' || pkill -x Kalamu || true
ls -la "$out"
