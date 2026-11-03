Kalamu 1.3.0 speaks French, searches the text of a whole project, lists what still needs
checking across it, and keeps each source's archival description, exported in your own columns
beside what was read from it. For what 1.2.0 changed, see the
[1.2.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.2.0).

## What's new since 1.2.0

- **A French interface**: choose *Same as the system*, *English* or *Français* in **Settings ›
  Appearance and language**; by default Kalamu takes the first of the system's languages it
  has. Dates, numbers and costs follow the language, and items' pages are typed as `8 (gauche)`.
  The default prompts' names are translated; the prompts themselves, and the files Kalamu
  writes, stay in English, and the answers' language is still chosen in New job.
- **Search a whole project**: every passage of its documents that holds some words, in the text
  as you corrected it, capitals, accents and Arabic vowel marks set aside. Look for the exact
  phrase, all the words in the same page, or words near each other; narrow by collection, tag,
  task or review; include analyses' answers and tables of contents if you wish. Each match opens
  its review at that place, a recording at its line. The search runs on the computer: nothing
  is sent, and its index is made again from your results when needed.
- **What to check**: one list, for a project, a collection or a selection, of the pages and
  answers that need you, each with its reason: failed or not read, cut off at the model's limit,
  read differently by two set-ups, read from a text corrected since, or not yet reviewed. Each
  opens its review, from which the previous and next items are a click away. Only your review
  resolves an item; models agreeing never counts as one.
- **Describe each source as an archive does**: repository, fonds, series, box or folder,
  shelfmark, creator, date as written, web address, citation and rights. Give the parts a
  project's documents share once, in the project; a document fills in its own. *Export
  descriptions* writes them as CSV and JSON, each value beside where it came from, and each
  result's `provenance.json` names the documents its source is, with their descriptions.
- **Export profiles**: the same export in your own columns, order and headings, for a
  spreadsheet or an Omeka S import: start from every column or from Dublin Core terms, add fixed
  texts, choose the list separator and what an empty cell holds. A column can also read each
  document's result: how far it was reviewed, the model, its text as corrected, or an analysis's
  answer, either as the model gave it or under your dictionary's labels. Profiles are kept for
  every project, shared as templates and imported.
- **Rename a tag** across a project from the table's tag filter; onto a tag in use, the two
  become one.
- **Languages in New job**: it now says when a transcription model takes only one of the
  languages chosen, or a language only without speakers and timestamps.
- **Report a problem** leads with bugs and ideas, and opens GitHub's choice of forms; offensive
  or harmful output can still be flagged there. A misreading is for the review to correct, not
  to report.

## Fixed

- Gemini 3.5 Transcribe names speakers Speaker 1, Speaker 2 (the live service labels them
  `spk:0`, `spk:1`), and its cost is counted from its tokens: it showed $0.
- OpenAI caches only the instructions: implicit caching wrote each whole page to the cache at
  1.25 times the input price.
- Descriptions, profiles and the dictionary's CSV files open in Excel with their accents.
- Tokens billed for an answer a provider's filter stopped count in the job's cost.
- Splitting and joining items is saved in one step, with clearer messages; cutting out or
  exporting items no longer freezes the window.
- Project search waits for a word of three letters before searching, and a database error is
  said as such rather than counted as text not read.
- After renaming a tag, the table stays filtered on it.
- A result's `provenance.json` is no longer overwritten with older details when a document is
  renamed while its job finishes.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.3.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.3.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.3.0_aarch64.dmg`: macOS 13 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.3.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg and offline Whisper; nothing else needs to be installed.
Installing 1.3.0 over an earlier version keeps your jobs, settings, presets and keys; project
search builds its index the first time Search is opened.

## Known limitations

- **Still in English**: the macOS application menu, model descriptions, the error a failed page
  keeps, and the files Kalamu writes. German will follow.
- **Search** finds words of three letters or more, and one word or another is not offered yet.
- **Items are whole pages of a PDF.** Two articles on one page each take the whole page; a
  multi-page image's items can be made and corrected, but not read or exported on their own.
- **Unsigned installers**: Windows SmartScreen will warn. Choose *More info*, then *Run anyway*,
  or install from the Store.
- **The Mac app is not signed with an Apple Developer ID**, so the first time macOS refuses to
  open it: open **System Settings → Privacy & Security**, choose **Open Anyway** beside Kalamu,
  and confirm. macOS may also ask once, and again after an update, to let Kalamu use the key
  saved in your Keychain.
- **The Mac version is new**: it is built and tested on Macs in GitHub Actions, but has not yet
  been used on a real Mac. Please report what does not work with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose). Intel Macs are not
  supported.
- Offline Whisper, tables of contents and corrections have had fewer runs against the live
  services; OpenAI's batches had not answered within 35 minutes in the last check. Try a small
  job first.
- A workflow run again starts from the models' text, not the corrections made in the last run.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). Offline
transcription uses whisper.cpp (MIT). The fonts are under the SIL Open Font License. Every other
component is listed with its licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
