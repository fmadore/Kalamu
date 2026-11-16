# A test pass on Windows before 1.8.0

A checklist for trying the packaged app on a Windows PC before version 1.8.0 is tagged: what
the roadmap's *Next* gathers as "to try by hand", what was written after 1.7.0, and the live
runs that touch the researcher's own accounts. CI builds and tests everything here, but none of
it has been opened in an installed copy. The Mac's share waits for a Mac
([mac-test-day.md](mac-test-day.md)).

Tick each box, and write what went wrong in [Results](#results) as you go: the screen, what you
did, what you expected, what happened. A screenshot of anything odd costs nothing.

## Before starting

- [ ] **An installer of `main`**, built as [development.md](development.md#releases) says
  (FFmpeg and llama.cpp fetched first), with the configuration for this PC's architecture:
  `tauri.ffmpeg.windows-arm64.conf.json` on the Snapdragon, `tauri.ffmpeg.windows-x64.conf.json`
  on an Intel or AMD PC. Install it over 1.7.0; Settings → About gives the version of `main`.
- [ ] **The sample files** from `samples/`, and for the records part a few pages of a real
  directory and a real register (a scan of a few pages each is enough).
- [ ] **Keys** for OpenAI and Mistral (for the batch files) and Gemini (for Hydrogen), each with
  a low spending cap.
- [ ] **Narrator** (Ctrl + Windows + Enter) and a spreadsheet (Excel or LibreOffice Calc).

## Part A: launch and shell (30 minutes)

- [ ] **The window** opens at launch, now that Kalamu's setup makes it rather than Tauri. The
  sidebar, New job and Settings show without error.
- [ ] **A webview that cannot be made**: in a terminal, set
  `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` to a folder that does not exist and start Kalamu from
  that terminal. A native message says why it cannot start and names the logs folder; Kalamu
  ends. Clear the variable afterwards.
- [ ] **The window hidden for a whole job**: start a Tesseract job on
  `daily-nebraskan-1911-issue.pdf`, close the window at once. The job runs on from the
  notification area, the notification says it finished, and the window opens again from the
  icon with the job done.
- [ ] **Clear unused results** in Settings → General says what it will remove and removes it;
  the jobs listed still open their reviews.

## Part B: what 1.8.0 adds (1 hour)

- [ ] **Text regions, Tesseract**: open the Tesseract job's review, *Show text regions*. Blocks
  are drawn over the scan; zoom past 175% for lines and 350% for words, all solid. Choose a
  region with the pointer, then with the keyboard: its text shows and its place is marked in
  the page's text. With a touchpad, pinch to zoom and drag with the regions on.
- [ ] **Text regions, Mistral OCR**: the same on a Mistral OCR job's page. Lines and words
  within a block are dashed (estimated). A region whose words were corrected shows its own text
  and says it has no certain place.
- [ ] **Text regions, nothing placed**: a page read by a model that places nothing says so.
- [ ] **Text regions with Narrator**: a region chosen from the keyboard is read out with its
  text.
- [ ] **Records, a directory**: New job → Analyse texts on the directory's pages, with
  *Directory or list, one row per entry*. The review lists each page's rows beside the scan;
  correct a cell, add a row, remove one. Export `.csv` and `.json`: one row per entry with
  `file`, `page` and `position`, and `corrected` set on the changed rows.
- [ ] **Records, a register**, with *Register, one row per entry*, on the register's pages: the
  same, on handwriting. Note any entry left out or carried down from a heading.
- [ ] **Records, a page not read**: a page that fails stands in the `.csv` as one row saying so.
- [ ] **Gemini 4 Hydrogen** is the default in New job; the model menu follows its button when
  the page scrolls, and pointing at a model shows its whole description.

## Part C: the checkpoints' leftovers (45 minutes)

- [ ] **File types**: *Correct existing text* and *Translate a text* offer `.csv` in the file
  picker; an `.xlsx` dropped on New job is refused at once, with a reason.
- [ ] **A Tesseract job on a PDF with a large page** (an A3 or a newspaper page):
  `provenance.json` records `page_resolution`, and the ALTO file names any page drawn below
  300 dpi.
- [ ] **Language data deleted right after a job**: in Settings → Offline, delete the Tesseract
  language just used, as soon as the job ends. It is deleted at once, with no error.
- [ ] **A Whisper transcription from an accented data folder**: under a Windows account whose
  name has an accent (`é`), so that Kalamu's data folder has one, transcribe `interview.wav`
  with Whisper and voice detection on. It finishes and `provenance.json` says `"vad": true`.
  Skip it if no such account is at hand, and say so.
- [ ] **The offline engines on a graphics processor**, if a PC with one is at hand (the Intel
  Arc laptop, or one with an NVIDIA or AMD card): LightOnOCR-3 and Whisper on the same page and
  recording, with the seconds a page took shown in Settings.

## Part D: keyboard and Narrator (30 minutes)

- [ ] **The job's menu** (in the sidebar and on the Jobs page): opened from the keyboard, moved
  with the arrows, Home and End, closed with Escape, the focus back on its button.
- [ ] **A name's menu** in an analysis review: the same, Home and End included.
- [ ] **The two language pickers** (New job's languages, and Tesseract's language data in
  Settings → Offline): typed into,
  chosen with the arrows and Enter; Escape closes the list and it stays closed.
- [ ] **The readings** in a comparison's review: reached with Tab, one taken into the text; an
  empty reading taken leaves one space, not two.
- [ ] **All of the above with Narrator on**: each control is named and its state read out.

## Part E: exports in a spreadsheet (10 minutes)

- [ ] **The CSV apostrophe**: open an analysis's `.csv` holding a telephone number or a list
  item beginning with `-`, `+`, `=` or `@` in the spreadsheet. The cell shows the text, not a
  formula or an error; numbers are still numbers.

## Part F: live runs with real keys (1 to 2 hours, mostly waiting)

- [ ] **A batch's files deleted, OpenAI**: an economy-mode extraction of two or three pages
  with an OpenAI model. Before it is collected, the provider's Files page lists its upload;
  once its answers are stored, the upload, the answers and the errors file are gone.
- [ ] **A batch's files deleted, Mistral**: the same with a Mistral model. Note whether the file
  of its inline requests (about 1 MB for two pages) is among those deleted: if not, Kalamu
  cannot find it without listing the storage, which it will not do, and the roadmap's item 4
  says so.
- [ ] **A batch's files after a deleted job**: start a third economy job, delete the job while
  its batch waits, and check at the next launch that its files are gone too.
- [ ] **A records job in economy mode**, which has no test of its own: the directory's pages
  with a Gemini or Mistral model, collected after a restart.
- [ ] **Hydrogen through a whole job**: an extraction and an analysis of the sample files with
  Gemini 4 Hydrogen, the default; the cost after the run near the estimate before it.
- [ ] **An answer stopped by a filter**, if one occurs: the review and the error say it was the
  provider's filter, not the output limit. Do not try to provoke one.

## Results

One line per failure, and an issue for each real problem.

| Part | Step | What happened | Issue |
| --- | --- | --- | --- |
| | | | |

Afterwards, in the repository:

- the roadmap's *Next*: the lines tried struck from the "to try by hand" lists, with what was
  found; [#148](https://github.com/fmadore/Kalamu/issues/148) and
  [#144](https://github.com/fmadore/Kalamu/issues/144) closed once only their Mac lines are
  left, which move to item 3;
- item 4 (economy mode) with the batch files' live run, and
  [live runs](decisions/2026-11-03-live-runs.md) with its costs;
- then the 1.8.0 release notes, its *What's new* in [store-listing.md](store-listing.md), and
  the release steps in [development.md](development.md#releases).
