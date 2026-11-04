# A day on a rented Mac

A checklist for one day on a cloud Mac: the first run of Kalamu on real Apple hardware, the Mac
App Store build through TestFlight, and the App Store screenshots. A virtual Mac on a Windows PC
is ruled out by Apple's licence (macOS runs only on Apple hardware) and would not run the Apple
Silicon build anyway; a rented Mac mini is real hardware, controlled from Windows over VNC.
Apple's licence makes 24 hours the shortest rental, so plan the day to use it.

Tick each box, and write what went wrong in the notes column of [Results](#results) as you go:
the screen, what you did, what you expected, what happened. A screenshot of anything odd costs
nothing and saves guessing later.

## Before booking

Everything here costs nothing and makes the rented hours count. Steps 1 to 4 can take days at
Apple, so start them first.

- [ ] **Apple setup and secrets done** ([mac-app-store.md](mac-app-store.md)): App ID,
  certificates, profile, API key, the app in App Store Connect, the ten secrets and
  `APPLE_APP_ID`.
- [ ] **1.4.1 released**: tag pushed, the release workflow finished, `Kalamu_1.4.1_aarch64.dmg`
  and `kalamu-samples.zip` on the [releases page](https://github.com/fmadore/Kalamu/releases).
- [ ] **Store build in TestFlight**: *Mac App Store* workflow run on `v1.4.1` with *upload*
  ticked, and the build shown as ready under TestFlight in App Store Connect (processing takes
  up to an hour; an email says if it was refused).
- [ ] **A tester**: under TestFlight → Internal Testing, a group with the Apple ID you will sign
  in with on the rented Mac. A separate Apple ID is fine, and simpler to remove afterwards.
- [ ] **A test key** for one cloud provider, with a low spending cap (Gemini or Mistral), for the
  consent dialog and a cloud job. It is revoked at the end.
- [ ] **Material for the screenshots**, from
  [store-listing.md](store-listing.md#screenshots): six to ten issues of *The Monitor* from
  [Chronicling America](https://chroniclingamerica.loc.gov/), the sample files, and an MP4 or
  M4A recording you may use (for FFmpeg). Put them in one zip somewhere you can download from
  (a GitHub release draft, a cloud drive link).
- [ ] **A VNC viewer on Windows**: [TigerVNC](https://tigervnc.org/) or RealVNC Viewer. Windows'
  own OpenSSH (`ssh`, `scp`) copies files back at the end.
- [ ] **This checklist open on Windows**, not on the Mac.

## If Claude runs the day, on the Mac itself

The Claude desktop app runs on the rented Mac and works through the list there: commands in its
own terminal, and the screens with computer use on the Mac's own display, at full resolution.
That is faster and surer than driving the Mac from the PC through a VNC window. It starts as a
fresh session: the checklist and the brief below are all it needs. What it does not do: book or
pay, type a password, sign in to an account, or paste an API key into a field.

**You do, at the start (30 minutes):**

- [ ] Book the Mac and connect over VNC as in the next section, typing the VNC password.
- [ ] **Install the Claude desktop app** on the Mac from
  [claude.ai/download](https://claude.ai/download) and sign in with your account.
- [ ] Open the Code tab with a new folder, `~/kalamu-test-day`, as its working folder, and paste
  the brief below.
- [ ] **When computer use first asks**, allow it under System Settings → Privacy & Security:
  *Accessibility* and *Screen Recording* for Claude. macOS asks for the administrator password:
  type it yourself.

**You do, when Claude asks during the day:**

- [ ] Sign in to TestFlight (and the App Store) with the tester's Apple ID.
- [ ] Paste the test key into Kalamu's Settings → API keys.
- [ ] Any other password macOS asks for.

**You do, at the end** (before giving the Mac back, below): copy `~/kalamu-test-day` to the PC,
then sign out of Claude (Settings → Log out) as well as the Apple ID, revoke the key and delete
the server. Keep GitHub off the rented Mac: the repository is public, so nothing there needs a
login, and the results come back to the PC to be turned into issues.

**Claude does:** the downloads and installs, launching the apps, the steps on screen (jobs,
reviews, dialogs, folder pickers, the consent dialog, closing and quitting), every check that
reads files (the data folder, the container, `grants.json`, the logs, results beside the
sources), the test disk image with `hdiutil`, the screenshots with `screencapture` and their
checks with `sips`, and the results file. Safari is read-only to computer use, so downloads go
through the terminal; anything it cannot check (the player's sound) is written down as such.

**The brief to paste:**

```text
Today is a test day for Kalamu, a desktop app for researchers (https://github.com/fmadore/Kalamu), on this rented Mac with Apple Silicon. Nothing has run on a real Mac before; you are the first.

1. Download the checklist and the release notes into this folder (the repository is public; do not log in to GitHub on this Mac, and do not clone the repository unless git is already installed):
   https://raw.githubusercontent.com/fmadore/Kalamu/main/docs/mac-test-day.md
   https://raw.githubusercontent.com/fmadore/Kalamu/main/docs/release-1.4.1.md
   https://raw.githubusercontent.com/fmadore/Kalamu/main/docs/store-listing.md (the "Screenshots" table: ten scenes and captions)
2. Work through mac-test-day.md, Part A (the disk image), Part B (the Mac App Store build through TestFlight) and Part C (screenshots), in order. Skip "Before booking", "If Claude runs the day" and "Booking and first connection" except its display and quiet-desktop steps.
3. Download files with curl in the terminal (Safari is read-only to you). A file fetched with curl is not quarantined, so before opening the disk image mark it as a browser would, to test Gatekeeper's first-open question:
   xattr -w com.apple.quarantine "0081;$(printf %x $(date +%s));Safari;" Kalamu_1.4.1_aarch64.dmg
4. Write results.md in this folder as you go: for every step, passed, failed or not checked, and for each failure what you did, what you expected, what happened, with a screenshot of it in problems/. Write it after each step, not at the end.
   The sample files: https://github.com/fmadore/Kalamu/releases/download/v1.4.1/kalamu-samples.zip. Ask me for the link to the screenshot material (issues of The Monitor and a recording).
5. Save the App Store screenshots in screenshots/, named 01-comparison … 10-offline, checked with sips (1440x900 or 2880x1800, hasAlpha no).
6. Stop and ask me whenever a step needs a password, an Apple ID sign-in, or the test API key in Kalamu's Settings: I type those, you never do. Ask before anything that would send files to a cloud provider beyond the steps that use the test key, and keep to the estimate shown before each job.
7. Do not change Kalamu's files, settings outside the app, or anything outside this folder and the app's own data, except what a step asks for (the test folder with accents, the test disk image).
At the end, summarise what passed, what failed and what you could not check (for instance the player's sound), with the failures first.
```

## Booking and first connection (30 minutes)

- [ ] Book a Mac mini, M4 if offered (Scaleway's *Apple silicon* in Paris is the cheapest found,
  about €0.22 an hour; AWS EC2 Mac and MacinCloud also work). Note the address, the VNC password
  and the SSH command the console gives.
- [ ] Connect with the VNC viewer. If the screen is black, wait a minute: the Mac may still be
  starting.
- [ ] **Display**: System Settings → Displays. For screenshots choose **1440 × 900**, which App
  Store Connect accepts. For 2880 × 1800 a Retina mode is needed: VNC to a headless Mac mini
  usually offers none, but a virtual display made with BetterDisplay can.
- [ ] **A quiet desktop**: Appearance *Light*; a plain wallpaper; Focus → Do Not Disturb on (no
  notifications in the shots); System Settings → Desktop & Dock → hide recent apps; the clock
  and menu-bar extras as few as possible.
- [ ] **Language**: System Settings → General → Language & Region → English (for the English
  screenshots; French later for French ones).
- [ ] Download the material zip with Safari and unzip it into `~/Documents/Kalamu material`.

## Part A: the disk image (2 hours)

The notarised app from the releases page, as a researcher installs it.

- [ ] Download `Kalamu_1.4.1_aarch64.dmg` with Safari (or with `curl`, marked as quarantined as
  the brief says, when Claude runs it), open it and drag Kalamu to Applications.
- [ ] Open Kalamu from Applications. macOS asks once whether to open an app downloaded from the
  internet, naming the developer; nothing more (no "damaged" or "unidentified developer").
- [ ] The window opens; the sidebar, New job and Settings show without error. Settings → About
  gives version 1.4.1.
- [ ] **Shortcuts read ⌘**, not Ctrl: in the glossary's help marks and the review's keys.
- [ ] **Keychain**: Settings → API keys, paste the test key. No password prompt now; quit and
  reopen: the key is still there. If macOS asks for the login password to use the key, note it
  and choose *Always Allow*.
- [ ] **Whisper offline**: Settings → Offline, download Whisper Base. New job → Transcribe audio,
  `interview.wav`, *Whisper (on this computer)*, Whisper Base, Start. It finishes; the review
  shows the transcript beside the player.
- [ ] **The player**: play, click a line, the position moves to it. (VNC usually carries no
  sound: check that it plays, not what it sounds like.)
- [ ] **Search**: ⌘F, type `Mopti`, the match is marked.
- [ ] **A correction**: edit a word and save; it shows as corrected; the `.txt` beside the
  recording has it.
- [ ] **Tesseract offline**: Settings → Offline, download French. New job → Extract text,
  `bulletin-page.jpg`, Tesseract, French, Start. Formats include ALTO XML and a searchable PDF.
- [ ] Open the searchable PDF in Preview: select a word, it highlights over the right place.
- [ ] **A photo turned by EXIF**: take `bulletin-page.jpg`, rotate it 180° in Preview with
  *Tools → Rotate* and save (Preview may write an EXIF orientation rather than turn the pixels);
  read it with Tesseract: the text comes out as it should, not garbled.
- [ ] **Opening results**: on the job's page, open a result (TextEdit or Word opens it) and
  *Show in Finder* (Finder opens with the file selected).
- [ ] **The consent dialog**: New job → Extract text, `dodington-1755.jpg`, the cloud provider of
  the test key, Start. A dialog names the provider, says what is sent and gives the privacy
  policy's address, with *Send* and *Cancel*. Cancel: nothing starts. Start again, *Send*: the
  job runs. A second job to the same provider asks nothing.
- [ ] **YouTube** (if the test key is Gemini's): New job → Transcribe YouTube with a short public
  video; the review plays it in YouTube's player inside the window.
- [ ] **Closing while a job runs**: start a long job (several pages), close the window. The app
  stays in the Dock and the menu bar shows its icon with progress; clicking it opens the window
  again.
- [ ] **⌘Q while a job runs**: the app asks first, saying how many pieces are in flight; *Quit
  now* quits.
- [ ] **Resume**: open Kalamu again: the job is listed as interrupted; resume it; it finishes
  without reading the pages already done again.
- [ ] **French**: Settings → General → French. Every screen is in French; the macOS application
  menu stays in English (known).
- [ ] **A folder with accents**: copy `interview.wav` and `bulletin-page.jpg` into
  `~/Documents/Données Frédérick`; transcribe and read them there. Results are written beside
  them.
- [ ] **WebKit at real sizes**: make the window narrow, then full screen; nothing scrolls
  sideways, nothing overlaps.

## Part B: the Mac App Store build, through TestFlight (2 hours)

Do this after Part A, on the same Mac: the store app then finds the disk image's data, as a
researcher switching over would.

- [ ] Quit Kalamu. Install TestFlight from the App Store and sign in with the tester's Apple ID;
  accept the invitation (the email, or TestFlight's list) and install Kalamu.
- [ ] **Data carried over**: open the TestFlight build. Part A's jobs and settings are there, in
  the sandbox's container
  (`~/Library/Containers/io.github.fmadore.kalamu/Data/Library/Application Support/Kalamu`).
- [ ] **Keys**: the test key is still in Settings. If macOS asks to let the new build use the
  Keychain item, choose *Always Allow* and note it.
- [ ] **No support link**: neither the sidebar's foot nor Settings → About shows *Buy me a
  coffee*. *Report a problem* still opens GitHub in Safari.
- [ ] **An old job needs its folder**: reopen Part A's Tesseract job and export it again, or
  retry it. Expect a failure to reach the file (the store app was never given that folder).
  Then add that folder to a new job (it asks for the folder): the old job now works too.
- [ ] **Results beside the files**: New job → Extract text with a file from a folder not yet
  granted, results beside the sources. Before starting, Kalamu explains it needs the folder and
  opens the folder picker on it; choose it: the job runs and writes beside the file.
- [ ] **The wrong folder**: do the same, and choose another folder in the picker: Kalamu says
  which folder was needed and starts nothing.
- [ ] **Resume from a bookmark**: start a long job in a granted folder, ⌘Q, *Quit now*, reopen:
  the job resumes and writes its results, with no question.
- [ ] **FFmpeg in the sandbox**: transcribe the MP4 or M4A from a granted folder with Whisper;
  it converts and finishes. In the review the player plays it.
- [ ] **A disk out of reach**: Disk Utility → File → New Image → Blank Image (100 MB), mount it,
  copy `interview.wav` onto it, and transcribe it there (grant the volume when asked). Quit,
  eject the image, open Kalamu, quit again; mount the image, open Kalamu: transcribing from it
  again needs no new question (the grant was kept while the disk was away).
- [ ] **Exports that ask**: a project's descriptions export or the dictionary's export into a
  new folder: it asks for the folder for the second file, then writes both.
- [ ] **The consent dialog** for a provider not yet agreed to (if there is a second test key), or
  check that the first one is remembered.

## Part C: screenshots (3 to 4 hours)

The ten scenes and captions are in [store-listing.md](store-listing.md#screenshots), with "your
Mac" for "your PC" in the tenth. Take them from the **TestFlight build** (the store app), in
light mode, one size throughout.

- [ ] **The demo project**, as the listing describes: *The Monitor, Omaha 1915–1916*, its
  collections and tags, the workflow *Newspapers* run on the issues, so the dictionary has the
  same people under several spellings. The workflow uses the test key: check the estimate first.
- [ ] Name jobs as a researcher would; no `/Users/<name>` path, key or personal data visible.
- [ ] **Capture the window alone**: ⌘⇧4, then Space, then hold ⌥ and click the window, which
  leaves out the shadow and its transparency. From the terminal (as Claude does), maximise the
  window and capture the screen silently, which gives the display's exact size:

  ```bash
  screencapture -x ~/kalamu-test-day/screenshots/01-comparison.png
  ```
- [ ] **Check each image** in Terminal: size and no alpha channel.

  ```bash
  sips -g pixelWidth -g pixelHeight -g hasAlpha ~/Desktop/01-comparison.png
  ```

  Expect 1440 × 900 (or 2880 × 1800) and `hasAlpha: no`. A window capture is the window's size,
  not the screen's: for an exact size, maximise the window (double-click its title bar) and
  capture the whole screen with ⌘⇧3 instead, then check the menu bar shows nothing private.
  If one has alpha, make a JPEG copy, which has none:

  ```bash
  sips -s format jpeg ~/Desktop/01-comparison.png --out ~/Desktop/01-comparison.jpg
  ```

- [ ] Name them after the listing's rows: `01-comparison` … `10-offline`.
- [ ] French screenshots, if time is left: switch Kalamu to French and take the first five again.

## Before giving the Mac back (30 minutes)

- [ ] **Copy everything off**, from Windows (Git Bash or PowerShell), with the SSH address the
  console gave (type the password yourself): the results, the problems' screenshots and the App
  Store screenshots, all in one folder when Claude ran the day.

  ```bash
  scp -r "user@address:~/kalamu-test-day" .
  ```

  Done by hand, copy `~/Desktop/*.png` and your notes the same way.
- [ ] **Sign out**: Claude (Settings → Log out), TestFlight and the App Store (Apple ID),
  Safari's saved logins.
- [ ] **Keys**: Settings → API keys → remove the test key in Kalamu, then **revoke it** at the
  provider's console.
- [ ] **Delete the server** in the provider's console: billing runs until it is deleted, not
  until you disconnect.

## Results

Copy this table into an issue or a note, one line per failure, and open an issue for each real
problem.

| Part | Step | What happened | Issue |
| --- | --- | --- | --- |
| | | | |

Afterwards, in the repository:

- the roadmap's *Next*, item 2 (a first run on a real Mac): done, with what was found;
- `docs/decisions/2026-11-02-macos.md` and the release notes' *Known limitations*: what has now
  been tried on a real Mac;
- the screenshots uploaded to App Store Connect, and the submission
  ([mac-app-store.md](mac-app-store.md), *Each version*, steps 5 and 6).
