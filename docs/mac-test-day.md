# A day on a rented Mac

A checklist for one day on a cloud Mac. The first such day, before 1.6.0, was Kalamu's first
run on real Apple hardware, with the Mac App Store build and the App Store screenshots
([decision](decisions/2026-11-02-macos.md)). This is the second: what that day did not write
down as tried, and everything written since, which CI builds and tests on macOS but no one has
opened on a Mac. The Windows share is in [windows-test-pass.md](windows-test-pass.md).

A virtual Mac on a Windows PC is ruled out by Apple's licence (macOS runs only on Apple
hardware) and would not run the Apple Silicon build anyway; a rented Mac mini is real hardware,
controlled from Windows over VNC. Apple's licence makes 24 hours the shortest rental, so plan
the day to use it.

Each step has a letter and a number, which the results name. Tick each box, and write what went
wrong in [Results](#results) as you go: the screen, what was done, what was expected, what
happened. A screenshot of anything odd costs nothing and saves guessing later.

## Before booking

Everything here costs nothing and makes the rented hours count.

- [ ] **A validation build of `main`**: a hyphenated tag (`v<version>-rc1`, the version the
  files of `main` give) pushed, and the release workflow finished
  ([development.md](development.md#releases)). Its draft holds `Kalamu_<version>_aarch64.dmg`,
  signed and notarised, and `kalamu-samples.zip`. A draft's files need a GitHub login, which
  stays off the rented Mac: download both on the PC, to copy across at the start of the day.
- [ ] **The same build in TestFlight**, for Part E: the *Mac App Store* workflow run on that
  tag with *upload* ticked, and the build shown as ready under TestFlight in App Store Connect
  (processing takes up to an hour; an email says if it was refused). If App Store Connect
  refuses a build while 1.6.0 is in review, leave Part E out and say so in the results.
- [ ] **A tester**: under TestFlight → Internal Testing, a group with the Apple ID you will sign
  in with on the rented Mac. A separate Apple ID is fine, and simpler to remove afterwards.
- [ ] **Test keys** for Gemini and Mistral, each with a low spending cap. They are revoked at
  the end.
- [ ] **A VNC viewer on Windows**: [TigerVNC](https://tigervnc.org/) or RealVNC Viewer. Windows'
  own OpenSSH (`ssh`, `scp`) copies files to the Mac and back.
- [ ] **This checklist open on Windows**, not on the Mac.

## If Claude runs the day, on the Mac itself

The Claude desktop app runs on the rented Mac, takes control of it and works through the whole
list there: commands in its own terminal, and the screens with computer use on the Mac's own
display, at full resolution.
That is faster and surer than driving the Mac from the PC through a VNC window. It starts as a
fresh session: the checklist and the brief below are all it needs. What it does not do: book or
pay, type a password, sign in to an account, paste an API key into a field, or change a system
setting.

**You do, at the start (30 minutes):**

- [ ] Book the Mac and connect over VNC as in the next section, typing the VNC password.
- [ ] Copy the disk image and the samples across, from Windows:

  ```bash
  scp Kalamu_<version>_aarch64.dmg kalamu-samples.zip "user@address:~/kalamu-test-day/"
  ```

- [ ] **Install the Claude desktop app** on the Mac from
  [claude.ai/download](https://claude.ai/download) and sign in with your account.
- [ ] Open the Code tab with `~/kalamu-test-day` as its working folder, and paste the brief
  below.
- [ ] **When computer use first asks**, allow it under System Settings → Privacy & Security:
  *Accessibility* and *Screen Recording* for Claude. macOS asks for the administrator password:
  type it yourself.

**You do, when Claude asks during the day:**

- [ ] Paste the test keys into Kalamu's Settings → API keys. The consent dialog at the first
  send to each provider is agreed to in the brief, for the sample files.
- [ ] System Settings → Keyboard → *Keyboard navigation*, off and then on, for Part C; and
  VoiceOver on and off (⌘F5).
- [ ] Sign in to TestFlight (and the App Store) with the tester's Apple ID, for Part E.
- [ ] Any password macOS asks for.

**You do, at the end** (before giving the Mac back, below): copy `~/kalamu-test-day` to the PC,
then sign out of Claude (Settings → Log out) as well as the Apple ID, revoke the keys and delete
the server. Keep GitHub off the rented Mac: the repository is public, so nothing there needs a
login, and the results come back to the PC to be turned into issues.

**Claude does:** opening the disk image, launching the apps, the steps on screen (jobs,
reviews, dialogs, folder pickers, closing and quitting), every check that reads files (the data
folder, the container, `grants.json`, the logs, results beside the sources), the programs
running (`pgrep`), the test disk image with `hdiutil`, and the results file. With VoiceOver on,
its caption panel shows on screen what it says, so Claude can read it. Safari is read-only to
computer use, so downloads go through the terminal; anything it cannot check (the player's
sound) is written down as such.

**The brief to paste:**

```text
Today is a test day for Kalamu, a desktop app for researchers (https://github.com/fmadore/Kalamu), on this rented Mac with Apple Silicon. The disk image and kalamu-samples.zip are already in this folder.

1. Download the two checklists into this folder (the repository is public; do not log in to GitHub on this Mac):
   https://raw.githubusercontent.com/fmadore/Kalamu/main/docs/mac-test-day.md
   https://raw.githubusercontent.com/fmadore/Kalamu/main/docs/windows-test-pass.md (Part B of the Mac's list sends you to its Parts B and C)
2. Work through mac-test-day.md, Parts A to E, in order. Skip "Before booking", "If Claude runs the day" and "Booking and first connection" except its display and quiet-desktop steps. Part F only if I ask.
3. A file copied with scp is not quarantined, so before opening the disk image mark it as a browser would, to test Gatekeeper's first-open question:
   xattr -w com.apple.quarantine "0081;$(printf %x $(date +%s));Safari;" Kalamu_*_aarch64.dmg
4. Write results.md in this folder after each step, not at the end: the step's letter and number, then passed, failed or not tried. For a failure: what you did, what you expected, what happened, and a screenshot in problems/. For "not tried": why.
5. I agree now to: the consent dialog at the first send to Gemini and Mistral, for the sample files only; and any job the checklist asks for whose estimate, which you read before starting, is under $0.25, up to $3 in all. Ask me for anything beyond that.
6. Stop and ask me whenever a step needs a password, an Apple ID sign-in, a key in Kalamu's Settings, or a system setting (keyboard navigation, VoiceOver): I do those, you never do.
7. Do not change Kalamu's files, settings outside the app, or anything outside this folder and the app's own data, except what a step asks for (the test folder with accents, the test disk image, ending Kalamu in Part D).
At the end, summarise what failed, what you could not try (for instance the player's sound), and what passed, in that order.
```

## Booking and first connection (30 minutes)

- [ ] Book a Mac mini, M4 if offered (Scaleway's *Apple silicon* in Paris is the cheapest found,
  about €0.22 an hour; AWS EC2 Mac and MacinCloud also work). Note the address, the VNC password
  and the SSH command the console gives.
- [ ] Connect with the VNC viewer. If the screen is black, wait a minute: the Mac may still be
  starting.
- [ ] **Display**: System Settings → Displays, **1440 × 900** or larger.
- [ ] **A quiet desktop**: Appearance *Light*; a plain wallpaper; System Settings → Desktop &
  Dock → hide recent apps. Leave notifications on: A9 looks at one.
- [ ] **Language**: System Settings → General → Language & Region → English.
- [ ] Unzip the samples into `~/kalamu-test-day/samples`.

## Part A: the disk image (2 hours)

The notarised app, as a researcher installs it. The first day tried the Mac App Store build;
most of this was never written down for the disk image.

- [ ] **A1. Gatekeeper**: open the disk image (marked as quarantined, as the brief says) and
  drag Kalamu to Applications. Opening Kalamu, macOS asks once whether to open an app
  downloaded from the internet, naming the developer; nothing more (no "damaged" or
  "unidentified developer").
- [ ] **A2. The window** opens, now that Kalamu's setup makes it rather than Tauri. The
  sidebar, New job and Settings show without error; Settings → About gives the validation
  build's version; shortcuts read ⌘, not Ctrl, in the glossary's help marks, the review's keys
  and the shortcut overview.
- [ ] **A3. The Keychain**: once the keys are pasted, no password prompt; quit and reopen: the
  keys are still there. If macOS asks for the login password to use a key, note the dialog's
  words and choose *Always Allow*.
- [ ] **A4. Whisper with Metal**: Settings → Offline, download Whisper Base and the
  voice-detection model. Transcribe `interview.wav` with *Whisper (on this computer)*. It
  finishes; the review shows the transcript beside the player; `provenance.json` says
  `"device": "metal"` and `"vad": true`. Pause `interview-two-speakers.wav` a moment after it
  starts: it stops within seconds, and resumes.
- [ ] **A5. The player**: play, click a line, the position moves to it, the passage heard is
  marked and kept in view. (VNC usually carries no sound: check that it plays, not what it
  sounds like.)
- [ ] **A6. Tesseract**: download French, read `bulletin-page.jpg`, with ALTO XML, PAGE XML
  and a searchable PDF. Open the PDF in Preview: a word selected highlights over the right
  place.
- [ ] **A7. The OCR vision models with Metal**: download LightOnOCR-3 and GLM-OCR, read
  `bulletin-page.jpg` with each. Both finish; LightOnOCR-3's blocks show under *Show text
  regions*; Settings shows the seconds a page took; `provenance.json` names the device.
- [ ] **A8. An offline analysis**: download Gemma 4 E2B and run *Summary and keywords* on
  `bulletin.txt` with it. It finishes with every field answered.
- [ ] **A9. The window closed for a whole job**: start a Tesseract job on
  `daily-nebraskan-1911-issue.pdf`, close the window at once. Kalamu stays in the Dock, the
  menu bar shows its icon with progress, a notification says the job finished, and a click on
  either opens the window with the job done.
- [ ] **A10. ⌘Q while a job runs**: Kalamu asks first, saying how many pieces are in flight;
  *Quit now* quits. At the next launch the sidebar's foot names the interrupted job with
  **Resume**; resumed, it finishes without reading again the pages already done.
- [ ] **A11. A cloud job and its consent**: an extraction of `dodington-1755.jpg` with Gemini.
  The dialog names the provider, says what is sent and gives the privacy policy's address;
  a second job to the same provider asks nothing.
- [ ] **A12. YouTube**: New job → Transcribe YouTube with a short public video, with Gemini;
  the review plays it in YouTube's player inside the window, and the links to YouTube open
  Safari.
- [ ] **A13. A folder with accents**: copy `interview.wav` and `bulletin-page.jpg` into
  `~/Documents/Données Frédérick`; transcribe and read them there. Results are written beside
  them, and *Show in Finder* selects one.
- [ ] **A14. WebKit at real sizes**: the window at its narrowest, at 640 × 560, and in full
  screen, on New job, a review, a project and Settings; nothing scrolls sideways or overlaps.
  In a text review, resize slowly through the point where the scan's scrollbar comes and goes
  (#192): the scan settles.
- [ ] **A15. French**: Settings → General → French. Every screen is in French; the macOS
  application menu stays in English (known). Back to English afterwards.

## Part B: what was written since 1.6.0, in WebKit (5 hours)

- [ ] **B1. The Windows list's Part B**, steps B1 to B15 of
  [windows-test-pass.md](windows-test-pass.md), here: with ⌘ for Ctrl, the trackpad's pinch
  if VNC carries it (say if it does not), and Finder for File Explorer.
- [ ] **B2. The Windows list's Part C**, steps C1 to C14, with the Gemini and Mistral keys.
  For C9, the interview's recording as an M4A if you have one.
- [ ] **B3. Text regions**, first tried on Windows on 17 November: on A6's and A7's reviews
  and on a Mistral OCR page, blocks, lines and words at each zoom, chosen with the pointer and
  with the keyboard.
- [ ] **B4. The model menu** follows its button when New job scrolls, and shows a model's
  whole description under the pointer.
- [ ] **B5. Files dropped**: a folder dragged from Finder onto a project and onto New job; the
  drop target shows; an `.xlsx` is refused at once, with a reason.
- [ ] **B6. A region drawn with the trackpad**, never tried in WKWebView: drag a box, move and
  resize it, pinch to zoom while one is chosen; the box stays under the pointer and on the
  scan. On a page of thousands of words, moving a region stays smooth.
- [ ] **B7. The passage menu**: on a selection in a text review, Ctrl+click and a two-finger
  click each open Kalamu's menu with *Correct with a model…*, not the system's own menu.
- [ ] **B8. A quotation played**: in an interview's review, **Listen** opens the transcript at
  the quotation's turn and plays it there, in each format the Mac's player is given (the
  recording as it was added, and one cut into pieces: a recording of more than half an hour).
  This has run in Chromium only.
- [ ] **B9. EmbeddingGemma 2 with Metal**: Windows' B10 here. Note the first five passages for
  each question and compare them with the Windows pass's for the same project: the model's card
  warns of half precision on a graphics processor. Note how long preparing took.

## Part C: keyboard and VoiceOver (1 hour)

WebKit skips buttons with Tab unless the system's *Keyboard navigation* is on, so each step is
done twice: with it off, then on (yours to switch).

- [ ] **C1. Tab through New job, a review and Settings**: with keyboard navigation on, every
  control is reached, in an order that follows the screen. With it off, note what can no longer
  be reached and whether the shortcuts still do the work.
- [ ] **C2. The menus**: a job's menu and a name's menu opened from the keyboard, moved with
  the arrows, Home and End, closed with Escape, the focus back on the button.
- [ ] **C3. The two language pickers**: typed into, chosen with the arrows and Return; Escape
  closes the list, keeps the text, and the list stays closed.
- [ ] **C4. The readings** of a comparison reached and taken from the keyboard; an analysis's
  names renamed and removed; a text region chosen with the arrows.
- [ ] **C4b. The newest screens**, D7 of the Windows list: a region drawn, moved and removed;
  the passage menu on a selection; an interview's cards, *Listen* and a quotation's *Remove*
  (with keyboard navigation off, Option+Tab reaches it); *Words* and *Meaning* in Search.
- [ ] **C5. What to check and the sidebar's Resume** from the keyboard, as D4 and D5 of the
  Windows list.
- [ ] **C6. With VoiceOver on**: C2 to C5 again. Each control is named and its state read out
  (the caption panel shows it); a failure that stays is read once; a region is read with its
  text; a proposal (a passage's, a region's) is announced when it arrives; an interview's card
  is read with its quotations, their speaker and time.

## Part D: after a crash or a Force Quit (30 minutes)

- [ ] **D1. A Force Quit**: start a LightOnOCR-3 job on `daily-nebraskan-1911-issue.pdf`, and
  while a page is being read, end Kalamu with Force Quit (⌥⌘Esc). In the terminal,
  `pgrep -fl llama` and `pgrep -fl ffmpeg` name nothing a few seconds later; if a program is
  left, note its name and whether it ends by itself. Open Kalamu: the job is interrupted, and
  resumes.
- [ ] **D2. A crash**: the same, ended with `kill -9` on Kalamu's process.
- [ ] **D3. Whisper**: the same once more during a Whisper transcription.
- [ ] **D4. The logs**: find the logs folder (`find ~/Library -name "kalamu.log*"`). It holds
  the day's log, and D1 to D3 left their last lines in it. If it holds a `panic.log`, note
  whether its lines name functions or only addresses, and give the size of
  `/Applications/Kalamu.app` (`du -sh`): the build is stripped until that size is known
  ([decision](decisions/2026-11-10-shell.md)).

## Part E: the Mac App Store build, through TestFlight (2 hours)

After the other parts, on the same Mac: the store app then finds the disk image's data, as a
researcher switching over would.

- [ ] **E1.** Quit Kalamu. Install TestFlight from the App Store and sign in with the tester's
  Apple ID (yours); accept the invitation and install Kalamu.
- [ ] **E2. Data carried over**: the TestFlight build shows the day's jobs and settings, in the
  sandbox's container
  (`~/Library/Containers/io.github.fmadore.kalamu/Data/Library/Application Support/Kalamu`),
  and the keys. If macOS asks to let the new build use a Keychain item, choose *Always Allow*
  and note it.
- [ ] **E3. The folders out of reach, shown at launch**: the store app was never given the
  folders the disk image's jobs read. At launch it says which folders are out of reach, and
  giving one from there makes its jobs work again (export one again).
- [ ] **E4. Results beside the files**: an extraction with a file from a folder not yet given,
  results beside the sources. Kalamu explains it needs the folder and opens the picker on it;
  chosen, the job runs. Choosing another folder instead: Kalamu says which was needed and
  starts nothing.
- [ ] **E5. Resume from a bookmark**: a long job in a folder already given, ⌘Q, *Quit now*,
  reopen: it resumes from the sidebar and writes its results, with no question.
- [ ] **E6. The programs in the sandbox**: an M4A made with
  `afconvert -f m4af -d aac interview.wav interview.m4a`, transcribed with Whisper (FFmpeg
  converts it); a LightOnOCR-3 page and a Gemma analysis (llama.cpp's programs start).
- [ ] **E7. A disk out of reach**: Disk Utility → File → New Image → Blank Image (100 MB),
  mount it, copy `interview.wav` onto it, and transcribe it there (give the volume when asked).
  Quit, eject the image, open Kalamu, quit again; mount the image, open Kalamu: transcribing
  from it again needs no new question.
- [ ] **E8. Exports that ask**: the review manifest of *What to check*, a project's
  descriptions export and the dictionary's export, each into a new folder: Kalamu asks for the
  folder for the second file, then writes both. PAGE XML's files, one per page, are all
  written beside the source.
- [ ] **E9. No support link**: neither the sidebar's foot nor Settings → About shows *Buy me a
  coffee*. *Report a problem* still opens GitHub in Safari.

## Part F: screenshots, only if the listing's scenes changed

The ten scenes, captions and the files each one uses are in
[store-listing.md](store-listing.md#screenshots), with "your Mac" for "your PC" in the tenth.
They were taken on the first day. Scenes 5 and 7 changed in 1.10.0: take search by meaning
(`05-search.png`) and screening (`07-screening.png`), and remove `05-dictionary.png` and
`07-analysis.png` from `store/screenshots/macos/`. Take another again only if its screen has
changed since, from the
**TestFlight build**, in light mode, with Do Not Disturb on, at the size of the others.
Use only the listing's material; nothing else goes in a shot, since everything legible in it
counts as the listing's text.

- [ ] Name jobs as a researcher would; no `/Users/<name>` path, key or personal data visible.
- [ ] **Read every shot word by word** at full size before keeping it. Retake it on another
  page or article if anything on race, religion, violence or politics could be taken out of
  context.
- [ ] **Capture the screen** with the window maximised, which gives the display's exact size,
  and check it: 1440 × 900 (or 2880 × 1800) and `hasAlpha: no`.

  ```bash
  screencapture -x ~/kalamu-test-day/screenshots/01-comparison.png
  ```

  ```bash
  sips -g pixelWidth -g pixelHeight -g hasAlpha ~/kalamu-test-day/screenshots/01-comparison.png
  ```

  If one has alpha, make a JPEG copy, which has none:

  ```bash
  sips -s format jpeg ~/kalamu-test-day/screenshots/01-comparison.png --out ~/kalamu-test-day/screenshots/01-comparison.jpg
  ```

## Before giving the Mac back (30 minutes)

- [ ] **Copy everything off**, from Windows (Git Bash or PowerShell), with the SSH address the
  console gave (type the password yourself): the results and the problems' screenshots.

  ```bash
  scp -r "user@address:~/kalamu-test-day" .
  ```

- [ ] **Sign out**: Claude (Settings → Log out), TestFlight and the App Store (Apple ID),
  Safari's saved logins.
- [ ] **Keys**: Settings → API keys → remove the test keys in Kalamu, then **revoke them** at
  each provider's console.
- [ ] **Delete the server** in the provider's console: billing runs until it is deleted, not
  until you disconnect.

## Results

One line per failure, and an issue for each real problem.

| Part | Step | What happened | Issue |
| --- | --- | --- | --- |
| | | | |

Afterwards, in the repository:

- the roadmap's *Next*: the "On a Mac" lines and item 3 (a first run on a real Mac, the rest)
  struck for what was tried, with what was found;
  [#148](https://github.com/fmadore/Kalamu/issues/148) and
  [#144](https://github.com/fmadore/Kalamu/issues/144) closed if their Mac lines passed;
- `docs/decisions/2026-11-02-macos.md` and the release notes' *Known limitations*: what has now
  been tried on a real Mac;
- the stripping of the Mac build ([shell](decisions/2026-11-10-shell.md)), decided from D4's
  size.
