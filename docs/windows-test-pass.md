# A test pass on Windows

A checklist for trying the packaged app on a Windows PC before a version is tagged: what the
roadmap's *Next* gathers as "to try by hand", what was written since the last pass, and the live
runs that touch the researcher's own accounts. CI builds and tests everything here, but none of
it has been opened in an installed copy. The first pass, before 1.8.0, is in
[Results](#results); this is the second, for 1.9.0: what `main` holds after 1.8.0. The Mac's share is
in [mac-test-day.md](mac-test-day.md).

Each step has a letter and a number, which the results name. Tick each box, and write what went
wrong in [Results](#results) as you go: the screen, what was done, what was expected, what
happened. A screenshot of anything odd costs nothing.

## Before starting

- [ ] **A validation build of `main`**: a hyphenated tag (`v<version>-rc1`, the version the
  files of `main` give) pushed, and the release workflow finished, as
  [development.md](development.md#releases) says. Its draft holds the installer for this PC's
  architecture (`arm64` on the Snapdragon, `x64` on an Intel or AMD PC) and
  `kalamu-samples.zip`.
- [ ] **Installed from File Explorer**, over the copy already there, by you or by Claude with
  computer use: never from a terminal Claude started (see below). Settings → About gives the
  version.
- [ ] **A test folder**, `Downloads\kalamu-test-pass`, with the sample files unzipped into
  `samples`, and for the records part a few pages of a real directory and a real register.
  Everything the pass writes goes there.
- [ ] **An interview of yours**, an hour or so, with its recording if you have it, for C9 and
  F12: the sample `interview.wav` lasts 20 seconds, too short for themes. And **a long text**,
  over 12,000 words in one `.txt` file, for B15.
- [ ] **Room for the models on the computer**: EmbeddingGemma 2 (310 MB) for B10, and Gemma 4
  E2B (4.3 GB, about 5 GB of free memory while it runs) for B15 and F12.
- [ ] **Keys**, each with a low spending cap, pasted by you into Settings → API keys: Gemini
  and Mistral for Parts B to E, OpenAI and Claude as well for Part F.
- [ ] **A spreadsheet** (Excel or LibreOffice Calc), and for Part D **Narrator**
  (Ctrl + Windows + Enter).

## If Claude runs the pass

Claude takes control of this PC from the Claude desktop app and works through the whole list:
computer use on the screen (File Explorer, Kalamu's window, the spreadsheet), and its own
terminal for reading files. It starts as a fresh session: this checklist and the brief below
are all it needs. It downloads the validation build's installer and samples into
`Downloads\kalamu-test-pass` (`gh release download`), installs Kalamu, runs every step it can,
and writes the results.

**One trap, which the brief names:** the Claude desktop app is itself a Store package, so a
program started from its terminal sees a private copy of `AppData\Local`. An installer run
from the terminal installs into that copy and the real Kalamu stays as it was: Claude installs
by opening the installer in File Explorer, on screen. For the same reason it never reads
Kalamu's job database or install folder from its terminal (an old copy answers). It reads what
Kalamu writes beside the sources, in `Downloads\kalamu-test-pass`, and the data folder
`AppData\Local\Kalamu Data` (settings, logs, the models' speeds), which has no private copy.

**What stays yours:**

- **Keys and passwords.** You paste each key into Settings → API keys when Claude asks, and
  type any password Windows asks for. Claude never types either.
- **System settings**: networking off and on for F8, another Windows account for F9.
- **Narrator's voice**, which Claude cannot hear: D6. It does the keyboard half of Part D.
- **The providers' own pages** in Part F, unless you are signed in to them in Chrome and let
  Claude read them there with Claude in Chrome. Otherwise Claude reads what Kalamu's log says
  it deleted, and you look at the page.

**What you agree to once, in the brief:** the consent dialog at the first send to each
provider, sends of the sample files under the brief's limits, and *Clear unused results* (A4),
which deletes stored answers for good. Take a line out of the brief to keep that step yours.

**The brief to paste**, in the Code tab opened on `Downloads\kalamu-test-pass`:

```text
This is a test pass of Kalamu, a desktop app for researchers (https://github.com/fmadore/Kalamu), my own project, on this Windows PC. Take control of the computer with computer use and run the whole pass; I am nearby for the few things that are mine.

1. Read docs/windows-test-pass.md from the repository's main branch (https://raw.githubusercontent.com/fmadore/Kalamu/main/docs/windows-test-pass.md), and its section "If Claude runs the pass" first.
2. Download the validation build <tag> into this folder with `gh release download <tag> --repo fmadore/Kalamu` (the installer for this PC's architecture, SHA256SUMS and kalamu-samples.zip), check the installer against SHA256SUMS, and unzip the samples into samples\.
3. Install Kalamu by opening that installer in File Explorer with computer use, never from your terminal: your terminal sees a private copy of AppData, and an install from it would not reach the real Kalamu. Then check Settings → About gives the build's version.
4. Ask me to paste the keys into Settings → API keys (Gemini and Mistral first; OpenAI and Claude before Part F). You never type a key or a password.
5. Work through Parts A to F in order. Use only the files in samples\. Check results by reading the files Kalamu writes beside them and Kalamu's logs, never Kalamu's database.
6. Write results.md in this folder after each step, not at the end: the step's letter and number, then passed, failed or not tried. For a failure: what you did, what you expected, what happened, and a screenshot in problems\. For "not tried": why.
7. I agree now to: the consent dialog at the first send to Gemini, Mistral, OpenAI and Claude, for the sample files only; any job the checklist asks for whose estimate, which you read before starting, is under $0.25, up to $3 in all; and Clear unused results in step A4. Ask me for anything beyond that.
8. Ask me for what the checklist keeps mine: system settings (networking, another account), Narrator's voice, and the providers' web pages.
9. Change nothing outside this folder, Kalamu and its own data, and install nothing else.
At the end, summarise what failed, what you could not try, and what passed, in that order.
```

## Part A: launch and shell (30 minutes)

- [ ] **A1. The window** opens at launch. The sidebar, New job and Settings show without error,
  and Settings → About gives the validation build's version.
- [ ] **A2. The window hidden for a whole job**: start a Tesseract job on
  `daily-nebraskan-1911-issue.pdf`, close the window at once. The job runs on from the
  notification area, whose icon shows it; a notification says it finished, with Kalamu's icon;
  the window opens again from the icon with the job done.
- [ ] **A3. A job that waits, in the sidebar**: start the same job again under another name,
  pause it, and open Settings. The sidebar's foot, under *To resume*, names the job with how
  far it got and **Resume**; resume it from the keyboard; the row goes, a message says the job
  resumed, and the focus lands on the job in the list. With two jobs waiting, the one stopped
  last is shown and the other is under *1 more*. In a short window the sidebar scrolls by
  itself and the page does not. Then start it a third time and
  quit Kalamu while it runs (*Quit now*); at the next launch the interrupted job is there with
  **Resume**, and finishes without reading again the pages already done.
- [ ] **A4. Clear unused results** in Settings → General says what it will
  remove and removes it; the jobs listed still open their reviews.
- [ ] **A5. The review's scan outside full screen** (#192, fixed in 1.9.0): open a text review
  in a window small enough for the scan to need a scrollbar, and resize the window slowly
  through the point where the scrollbar comes and goes. The scan settles; it never redraws
  without end, and the processor's use falls back to nothing.
- [ ] **A6. A webview that cannot be made**, whose message now comes from a newer library
  (`rfd` 0.18.0): start Kalamu from a terminal opened from the Start menu, not Claude's, with
  `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` set to a folder that does not exist. A native message
  says why Kalamu cannot start and names the logs folder; Kalamu ends.

## Part B: written since 1.8.0, without a key (3 hours)

- [ ] **B1. PAGE XML**: on A2's Tesseract job, write *PAGE XML* among the formats. One
  `.pNNNN.page.xml` per page appears beside the ALTO file; each is well formed, names its page's
  size, and its `TextEquiv` holds the page's text. Correct a word in the review and write the
  files again: that page's file holds the correction, its words keep their places, and the
  other pages' files are the same bytes as before.
- [ ] **B2. Text regions with a touchpad**, not tried in the first pass: *Show text regions*
  on the same review, pinch to zoom and drag with two fingers; the regions follow the scan.
- [ ] **B3. Project search**: put the Tesseract jobs in a project and search it. `Lincoln OR
  Omaha` finds pages with either; a word of two letters standing alone (`of`) is found; each
  match is marked where the text shows it. The first search after updating says the index is
  being prepared, once.
- [ ] **B4. What to check, the sample**: in the project's *What to check*, draw 5 pages at
  random. They are listed apart, as drawn at random and never as a problem, with the seed
  shown. Close and reopen Kalamu: the same five. Type the seed by hand in a second project
  holding the same documents: the same five again.
- [ ] **B5. What to check, kept aside and set aside**: keep one item aside with a note, and set
  a result aside from the list. Both are still so after Kalamu is closed and reopened. Correct
  the page of the item kept aside: it is listed again, marked as kept aside with an earlier
  answer.
- [ ] **B6. What to check, the manifest**: *Export the list* into the test folder. The JSON
  and the CSV hold every item in scope, the five drawn among them with the seed, and the note
  of B5.
- [ ] **B7. Several images as one document**: New job → Extract text with
  `bulletin-page.jpg` and `dodington-1755.jpg` as the pages of one document, Tesseract. Move a
  page before starting; the results are one document, its pages in the order chosen.
- [ ] **B8. The newest models**: in New job, OpenAI's default is GPT-6.2 Sol, and GPT-6.2 Astra
  is offered; GPT-6.1 Sol and GPT-6 Astra are not, but a saved pipeline that names one still
  shows it, marked, with its replacement a click away. Gemini 4 Neon is listed between Hydrogen,
  still Gemini's default, and Carbon, and marked costly. Listing models by tier works across
  providers.
- [ ] **B9. The look of small things**: an error message beside a correction in blue and a
  highlighted match are told apart at a glance; an empty value in a table reads as empty, and
  the apparatus of a comparison keeps its bracket and colon.
- [ ] **B10. Search by meaning, on the computer**: in Settings → Offline, download
  EmbeddingGemma 2. In B3's project, Search → *Meaning*, choose it and *Prepare the text*: the
  estimate says it costs nothing, no consent is asked, and the progress shows. Pause it part
  way, close and reopen Kalamu, and go on: it goes on where it stopped. Ask in your own words
  for something a page speaks of without using its words, and once in French: passages are
  listed with their documents, and each opens where it stands. *Words* is still what Search
  opens on, and still works. Note how long preparing took.
- [ ] **B11. The map**: the project's *Map* shows one point per document, grouped by collection
  or by tag; choosing a point lists its nearest documents beside it, and the list is reached
  and read from the keyboard. Before the text is prepared, the screen says what to do first.
- [ ] **B12. A region of your own**: on a Tesseract review, nothing new shows until *Show
  regions*. Then *Draw a region*: drag a box on the scan, write what is there; move it, resize
  it, remove it and bring it back with Undo. A slip (a click with no drag) keeps nothing and
  shows no error; Escape leaves the drawing. Close and reopen the review: the region is there.
  The same in the review of an analysis and of a table of contents.
- [ ] **B13. A model's region moved**: move and resize one of Tesseract's blocks; *Put back the
  model's box* returns it to where Tesseract placed it.
- [ ] **B14. A region read again, and the exports**: on a region you drew, *Read this region
  again…* with Tesseract: the dialog shows the part to be read and what it costs (nothing, on
  the computer), and the answer is a proposal; use it, then undo it. Write ALTO, PAGE XML and
  the searchable PDF: your region and the block you moved are in them, where you put them, and
  the PDF's text is found there by searching it.
- [ ] **B15. A text too long for the model**: with Gemma 4 E2B downloaded, New job → Analyse
  texts with the long `.txt`, the prompt *Interview: themes and quotations* and Gemma 4 E2B.
  Start is refused with a sentence that says the text is too long for the model chosen, and no
  job is made; nothing ran. With a cloud model in its place the job starts (stop it at once,
  or leave this half for Part C).

## Part C: with a Gemini or Mistral key (3 hours)

Cents in all. Each first send to a provider shows the consent dialog, which the brief agrees
to for the sample files.

- [ ] **C1. Mistral OCR's estimated lines**, fixed after the first pass: an extraction of
  `daily-nebraskan-1911.pdf` with Mistral OCR, then *Show text regions*. A paragraph's
  estimated lines are several, of a line's height, dashed; PAGE XML written from it holds those
  lines and no words, ALTO holds both.
- [ ] **C2. A records analysis**: *Directory or list, one row per entry* on the directory's
  pages. New job's diagram shows the records steps and the table is ticked (`.csv` in 1.9.0,
  `.xlsx` after the plain-first round); the cost after the run
  is near the estimate before it. Then *Register, one row per entry* on the register's pages,
  on handwriting: note any entry left out or carried down from a heading.
- [ ] **C3. Agreement between set-ups**: an analysis of `daily-nebraskan-1911.pdf` with *Summary
  and keywords* and three set-ups (two providers, or three models). The job's page shows how
  far they agree field by field over the whole job, each set-up under its siglum.
- [ ] **C4. A name pointed at** (#194): in that analysis's review, point at a name in a list
  field. It is brought into view in the text beside it and marked.
- [ ] **C5. A translation**: *Translate a text* on `bulletin.txt` into English. The review
  shows each page of the source beside its translation; the files are `.translated.en`.
- [ ] **C6. A translated transcript's subtitles**: transcribe `interview.wav` (French) with a
  cloud model, *Use this text in a new job* → Translate into English, then write `.srt` and
  `.vtt`. Each cue keeps the time and speaker of the line it translates.
- [ ] **C7. An image described**: New job with `dodington-1755.jpg` and `bulletin-page.jpg`
  alone and *Analyse*: *Image description and keywords* is chosen by itself; text-only models
  are not offered.
- [ ] **C8. The dictionary's proposed merges**: in the project, run two analyses that list
  names (*People, places and organisations*) on the two newspaper files, open the dictionary
  and ask for proposed merges on a field. Proposals name rows to merge with their documents;
  accept one and refuse one; nothing changes until accepted. With a field of a few hundred rows
  (only with a larger corpus of yours), note whether names far apart in the alphabet are
  proposed together (a surname first and last, initials, two transliterations).
- [ ] **C9. Analyse this interview**: transcribe your interview with a cloud model that labels
  speakers, name the speakers, then *Use this text in a new job* → **Analyse this interview**.
  One box, the model and its cost in one line, one button; type a sentence about the research,
  tick *Keep this for my next interviews*, and Start. The review shows a summary, themes,
  keywords and passages worth quoting, each theme a card with its quotations, their speaker
  and time. **Listen** opens the transcript at that turn and plays it. Read five quotations
  against the transcript: each is there word for word, said by the speaker named. Note how many
  quotations the result says were left out. Edit an entry, remove a quotation and an entry,
  and undo each. Open the dialog on another transcription: the description kept is offered.
  Note the cost against the estimate, and whether the themes are of use to you. Then open the
  dialog once more and, under *More choices*, choose Gemma 4 E2B: when the transcript is over
  about 11,000 words, a sentence where the cost was says it is too long for that model, with
  how many words each holds, and Start is off; choosing a cloud model brings both back. The
  same in French.
- [ ] **C10. The interview's files**: the Markdown memo names each quotation's speaker and
  time, the `.json` gives each quotation's place, and `.analysis.quotations.csv` has one row
  per quotation. A quotation you removed is in none of them.
- [ ] **C11. A passage corrected by a model**: in a text review, select a sentence, right-click
  and *Correct with a model…*; the dialog names the model, what is sent and the cost. The
  answer is shown against the selection: accept one, edit one before accepting, discard one,
  and undo an accepted one. While a model answers, Escape stops the wait and leaves the dialog
  open, saying an answer on its way may still be charged; a second Escape closes it. The job's
  cost has risen by the passage's.
- [ ] **C12. An item added to a list**: in C8's analysis review, add a name the model missed to
  a list field. It is marked as yours, and it is in the `.csv`, the project's dictionary and
  the project's search.
- [ ] **C13. Search by meaning with a cloud model**: in a second project holding the same
  documents, Search → *Meaning* with Gemini Embedding 2 or Mistral Embed. Before anything is
  sent, the estimate shows and a dialog names the provider and what is sent, for this project
  and this model; refuse it once (nothing is sent), then agree. Ask B10's questions: note
  whether the first passages are the same as EmbeddingGemma 2's. The cost kept after preparing
  is near the estimate.
- [ ] **C14. A region read again by a cloud model**: on `dodington-1755.jpg`, draw a region
  round two lines of handwriting and *Read this region again…* with a Gemini or Mistral model:
  the dialog shows the part cut from the page and its cost; the answer reads those lines only.
  On a model's own region, the answer takes the place of that region's words in the page's
  text; undo it. The page's own reading is unchanged throughout.

## Part D: keyboard and Narrator (30 minutes)

Claude does each from the keyboard; the Narrator half (D6) is yours.

- [ ] **D1. The two language pickers** (New job's languages, and Tesseract's language data in
  Settings → Offline), fixed after the first pass: with text typed in the box, Escape closes
  the list, keeps the text, and the list stays closed.
- [ ] **D2. An empty reading** in a comparison's review reads "(no text)", set apart from the
  models' text; taken into the text it leaves one space, not two.
- [ ] **D3. Text regions from the keyboard**: a region chosen with the arrows shows its text.
- [ ] **D4. What to check from the keyboard**: draw a sample, type a seed, keep an item aside
  with a note, open an item's review and come back with Back, all without the pointer.
- [ ] **D5. The sidebar's Resume** from the keyboard (with A3).
- [ ] **D6. With Narrator on** (yours): D1 to D5 and D7, the job's menu and a name's menu; each
  control is named and its state read out; a region is read with its text; a proposal (a
  passage's, a region's) is announced when it arrives.
- [ ] **D7. The newest screens from the keyboard**: draw a region with *Draw a region* pressed
  twice, move and resize it with the arrows, write its text, remove it and undo; open the
  passage menu on a selection with the menu key or Shift+F10; in an interview's review, reach
  each card, *Listen* and a quotation's *Remove*, and undo from the message that follows; in
  Search, change between *Words* and *Meaning*. Nothing needs the pointer, and the focus is
  never lost.

## Part E: exports in a spreadsheet (15 minutes)

- [ ] **E1. The CSV apostrophe**: open an analysis's `.csv` holding a cell that begins with
  `-`, `+`, `=` or `@` in the spreadsheet. The cell shows the text, not a formula or an error;
  numbers are still numbers.
- [ ] **E2. French Excel** (from the plain-first round, after 1.9.0): the analysis's `.xlsx`
  opened by double-click in an Excel set to French. It opens in columns, the heading row bold
  and kept in view, long text wrapped, a telephone number, a date and a page number as written
  and with no green corner. The `.csv` beside it still opens in one column there: New job and
  the glossary say which to pick.
- [ ] **E3. A records table**: on a page where one row was corrected, one added and one
  removed, `corrected` is set on the changed and the added rows only, in the `.xlsx`, the
  `.csv` and the `.json`; `reviewed` on every row of a reviewed page.
- [ ] **E4. The quotations `.csv`** of C10 opened in the spreadsheet: one row per quotation,
  accents and quotation marks whole, the times readable.

## Part F: live runs with real keys (3 hours, mostly waiting)

A few dollars at most. The providers' own pages are yours to look at, or Claude's through
Chrome where you are signed in.

- [ ] **F1. A batch's files deleted, OpenAI**: an economy-mode extraction of two or three pages
  with GPT-6.2 Sol. Before it is collected, OpenAI's Files page lists its upload; once its
  answers are stored, the upload, the answers and the errors file are gone. Note how long the
  batch took: on 3 November one had answered nothing after 35 minutes.
- [ ] **F2. A batch's files deleted, Mistral**: the same with a Mistral model. Note whether the
  file of its inline requests (about 1 MB for two pages) is among those deleted.
- [ ] **F3. A Claude batch deleted**: the same with a Claude model. Once its answers are
  stored, Anthropic's console no longer lists the batch.
- [ ] **F4. A Gemini batch through a file**: an economy-mode extraction large enough to pass
  20 MB (a PDF of scans of 40 pages or more). It is sent as a file, collected, and its files
  are gone from Google's storage afterwards. A small economy job with Gemini still goes as
  before.
- [ ] **F5. A batch's files after a deleted job**: start one more economy job, delete the job
  while its batch waits, and check at the next launch that its files are gone too.
- [ ] **F6. A records job in economy mode**: the directory's pages with a Gemini or Mistral
  model, collected after Kalamu was closed and reopened.
- [ ] **F7. GPT-6.2 Sol and Astra on real pages**: an extraction of `dodington-1755.jpg`
  (handwriting) and `daily-nebraskan-1911.pdf` with each; the cost near the estimate; note how
  the handwriting reads against Gemini 4 Hydrogen's.
- [ ] **F8. Offline with networking off** (yours to switch): a Tesseract extraction, a Whisper
  transcription of `interview.wav` and a LightOnOCR-3 page, each to its end with the network
  off; `provenance.json` names the device each ran on.
- [ ] **F9. A Whisper transcription from an accented data folder**: under a Windows account
  whose name has an accent, so that Kalamu's data folder has one, with voice detection on. It
  finishes and `provenance.json` says `"vad": true`. Skip it if no such account is at hand.
- [ ] **F10. The offline engines on a graphics processor**, on a PC that has one (the Intel
  Arc laptop, or one with an NVIDIA or AMD card): LightOnOCR-3 and Whisper on the same page and
  recording, with the seconds a page took shown in Settings.
- [ ] **F11. An answer stopped by a filter**, if one occurs: the review and the error say it
  was the provider's filter, not the output limit. Do not try to provoke one.
- [ ] **F12. The interview analysis, model by model**: your interview analysed by each
  provider's default model (Gemini, OpenAI, Claude, Mistral) and by Gemma 4 E2B on the
  computer, as five jobs or one job of several set-ups. For each, note the quotations given,
  those left out as found nowhere, the cost against the estimate, and whether the themes are of
  use. An hour's interview is near what Gemma reads at once: note whether it is refused before
  Start, and how long it takes if not.
- [ ] **F13. Search by meaning, each cloud model**: prepare a small project with Gemini
  Embedding 2, with one of OpenAI's two and with Mistral Embed, each in a project of its own.
  For each, the cost kept against the estimate, and against the provider's own usage page;
  whether Gemini's cost shows at all (its answer may not say the tokens billed); and whether
  Mistral refuses a call as too large (its limit for one call is a guess).
- [ ] **F14. Gemini 4 Neon on real pages**: with F7's two files, the cost near the estimate,
  and the handwriting against Hydrogen's.
- [ ] **F15. EmbeddingGemma 2 on a graphics processor**, with F10's PC: B10's project prepared
  there; the same questions give the same first passages as on the processor.
- [ ] **F16. A long interview in parts**: a transcript of two hours analysed with a cloud
  model whose limit it passes, and with Gemma 4 E2B on the computer. Before Start the dialog
  says in how many parts it will be read; the cost against the estimate; the themes brought
  together name each quotation once; no quotation is left with words the transcript does not
  have. Then with your own themes under *More choices*: each theme says where it comes from,
  and one the interview never takes up is listed as not found.
- [ ] **F17. A quotation of your own**: in F16's review, a passage selected in the transcript,
  **Add as a quotation**, the theme chosen. It shows as added by you, goes into the memo and
  the quotations table (`added_by`), and Undo takes it back.
- [ ] **F18. Which documents speak of my question**: in a project of twenty documents already
  read, some of them on your question and some not, **Which of these speak of my question?**
  with a cloud model. Note the answers you agree with, the passages against the pages they
  name, and the cost against the estimate. Filter by answer, **Keep** and **Set aside**,
  **Export this table** and open the `.xlsx`.
- [ ] **F19. Photographs taken twice and blurred**: a real batch from a reading room, 100
  photographs or more, added to New job. Note how long the check takes, each pair it calls
  the same page and each photograph it calls blurred against what you see, and what it
  missed. Nothing is sent for this step.
- [ ] **F20. A sample first**: a PDF of 40 pages or more in New job, **Try on 3 pages**. Note
  which pages are read, then **Run the rest** from the job's page: the cost against the
  price shown, and on the provider's usage page that the three pages were not sent again.
  Then **Compare with another model** on the sample, and the rest with that one.
- [ ] **F21. A long document screened**: F18's project with a document of 200 pages or more
  added, **Read the new documents against this question**. Only the new document is paid;
  its answer against what you know of it; its passages against the pages they name.
- [ ] **F22. A project carried to another computer**: a project with corrections, tags and a
  dictionary saved with **Save to a file…**, without the originals and then with them. Open
  each file on the Mac (and one saved on the Mac here): every correction, review mark and
  tag is there, the pages show when the originals were included, the exports work, and the
  other projects are untouched. Open the same file twice. Open the `.kalamu` in a text
  editor and search it for your API key: it is not there.

## Results

One line per failure, and an issue for each real problem. The first pass's steps are named as
that pass numbered them, in order within each part; its checklist is in the file's history.

### 17 November 2026, 1.8.0-rc1 on the Snapdragon (Windows 11 ARM64)

The `v1.8.0-rc1` validation build's ARM64 installer, installed over an earlier GitHub copy,
driven by hand through the interface. About $0.09 of Gemini and Mistral calls in all.

**Passed**: A1, A2 (WebView2's own message first, then Kalamu's naming the logs folder, in the
system's language rather than the interface's), A3 (the window reopened with the job done), B1
with the mouse and the arrow keys, B3, B5 on the four-page *Daily Nebraskan* issue's Directory
of Advertisers (59 rows; a cell corrected, a row added and one removed; `.csv` and `.json` with
`file`, `page` and `position`), B8, C1, C2 (`page_resolution` in `provenance.json`; ALTO not
written by default, so not read), C3, D1, D2, D4, and E as far as the file goes (`'=`, `'+` and
`'@` written, `-2201` kept a number).

| Part | Step | What happened | Fix |
| --- | --- | --- | --- |
| B2 | Text regions, Mistral OCR | Mistral writes each paragraph on one line, so the estimated lines were one per paragraph, the block's full height, and the words tall slivers across it | Estimated lines wrap a paragraph into the lines its block's shape holds, in the review and in ALTO |
| B5 | Records, cost | Estimated $0.019, cost $0.038: the estimate priced a records analysis as one whole read | Priced page by page, as it runs |
| B5 | Records, New job | "How it works" said "Read whole, as one piece" and "One row per document"; `.csv` unticked | The records steps of their own; `.csv` ticked |
| B5 | Records, export | `corrected` is set on every row of a corrected page, as [formats.md](formats.md) says, where this checklist expected the changed rows alone | To decide |
| C1 | `.xlsx` refused | The refusal went on to say Kalamu reads Excel | Excel taken out of the sentence |
| C3 | Language data deleted | "In use by a job" stayed after the job ended, Remove disabled until another tab was opened | The list follows the end of a job |
| D3 | Language pickers | With text in the box, Escape emptied it and the whole list opened again (WebView2 clears a search box on Escape, which reopened the list) | Escape closes the list and keeps the text |
| D4 | Readings | An empty reading was labelled "om." | "(no text)", set apart from the models' text |
| E | CSV in Excel | French Excel opens a comma-separated file in one column, so the apostrophe could not be seen at work there | To decide |

**Not tried**: the notification and the notification-area icon in A3, and the touchpad in B1
(the external screen, in HDR, could not be seen from the session driving the test); A4 (it
deletes stored answers for good: left to the maintainer); B4 and D5 (Narrator); B6 and B7 (no
register scan, no page that fails); C4 and C5; all of Part F (OpenAI's first-send consent left
to the maintainer, and the providers' Files pages are on the maintainer's accounts).

Afterwards, in the repository:

- the roadmap's *Next*: the lines tried struck from the "to try by hand" lists, with what was
  found; [#148](https://github.com/fmadore/Kalamu/issues/148) and
  [#144](https://github.com/fmadore/Kalamu/issues/144) closed once only their Mac lines are
  left, which move to item 3;
- item 4 (economy mode) with the batch files' live run, and
  [live runs](decisions/2026-11-03-live-runs.md) with its costs;
- then the 1.8.0 release notes, its *What's new* in [store-listing.md](store-listing.md), and
  the release steps in [development.md](development.md#releases).
