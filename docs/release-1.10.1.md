Kalamu 1.10.1 repairs what a review of the whole repository found on 23 November 2026, the day
after 1.10.0: a project could be saved to a file without a text its jobs had read, an export
could replace a workbook kept beside it, two files could be given one name, and economy mode's
estimate promised a discount the run did not give. A project file can now be checked without
opening it, and the review of a text extraction gains menus on a right click. For what 1.10.0 brought, see the
[1.10.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.10.0).

## What is fixed since 1.10.0

- **A project file is never saved incomplete**
  ([#247](https://github.com/fmadore/Kalamu/pull/247),
  [#239](https://github.com/fmadore/Kalamu/issues/239),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-23-frozen-texts-always-carried.md)).
  When an analysis, a translation or a correction reads your corrected text, Kalamu keeps that
  text as it was then. 1.10.0 saved a project without such a text when it was missing from
  Kalamu's data folder or had changed, and said nothing. **Save to a file…** now refuses, names
  the documents and the jobs concerned, and leaves a file saved earlier under that name as it
  was. A file 1.10.0 saved that way still opens, and the summary shown after opening lists
  what is missing.
- **An export never replaces a table you kept**
  ([#248](https://github.com/fmadore/Kalamu/pull/248),
  [#237](https://github.com/fmadore/Kalamu/issues/237),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-23-table-exports-keep-the-researchers-files.md)).
  *Export descriptions*, the dictionary's export, the review's *Export the list* and *Export
  the whole table* write a workbook or a CSV beside the file you chose. Where a file of that
  name is already there and is not one Kalamu wrote, or is one you have edited since, the new
  ones are written as `name (2).xlsx` and `name (2).csv`, and the message says so. To tell its
  own files from yours, Kalamu keeps on your computer the place and fingerprint of the last
  500 tables it exported, never their contents: the
  [privacy policy](https://fmadore.github.io/Kalamu/privacy) says so.
- **Every file gets a name of its own**
  ([#254](https://github.com/fmadore/Kalamu/pull/254),
  [#238](https://github.com/fmadore/Kalamu/issues/238),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-23-file-names-reserved.md)).
  With `scan.pdf`, `scan.tif` and `scan (2).pdf` in one job, two sources wrote their results to
  the same `scan (2)` files; and two originals in a project file whose names differ only by a
  character a system refuses, or by capitals, were written as one file when the project was
  opened. A name once given is now kept, and the next one is numbered again.
- **Economy mode's estimate is what the run charges**
  ([#249](https://github.com/fmadore/Kalamu/pull/249),
  [#242](https://github.com/fmadore/Kalamu/issues/242),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-23-economy-estimate-matches-run.md)).
  A screening, an interview analysis and a transcript translated with its times are sent at
  once, at the usual price, because Kalamu checks each quotation and each line as it arrives.
  The estimate counted the discount for them all the same. It now counts it only for what is
  sent in a batch, **Speed and price** says in a sentence why a model gets none, and *Cheaper*
  is not offered when nothing in the job can be batched.
- **Keep and Set aside hold in every language**
  ([#252](https://github.com/fmadore/Kalamu/pull/252),
  [#240](https://github.com/fmadore/Kalamu/issues/240),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-23-screening-decision-any-language.md)).
  A document kept with the interface in English showed no decision in French, and setting it
  aside there left it both kept and set aside. A decision is now read whatever language it was
  given in, as it must be when a project file passes between colleagues, and a document that
  carries both says so until you choose.
- **A question asked under a changed prompt stands apart**
  ([#251](https://github.com/fmadore/Kalamu/pull/251),
  [#241](https://github.com/fmadore/Kalamu/issues/241),
  [#244](https://github.com/fmadore/Kalamu/issues/244),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-23-answers-grouped-by-frozen-prompt.md)).
  Answers given before and after a preset was edited were shown and exported as one group.
  They are now two, each with the date it was first asked, the exported table names the job
  and the prompt behind each row, and Kalamu says before sending anything when new answers
  will be kept apart from the earlier ones. A large project's table also opens faster.
- **An export fails rather than write an incomplete record**
  ([#253](https://github.com/fmadore/Kalamu/pull/253),
  [#243](https://github.com/fmadore/Kalamu/issues/243)). When the passages corrected by a model
  or the regions read again could not be read from the job database, `provenance.json` was
  written as if there were none. The export now stops and says what could not be read.

## What is new

- **Check a project file**
  ([#260](https://github.com/fmadore/Kalamu/pull/260),
  [#246](https://github.com/fmadore/Kalamu/issues/246),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-24-check-project-file.md)).
  Before you rely on a `.kalamu` file as your only copy, or move to a new computer, **Check a
  project file…** in the list of projects reads it without opening it: whether it is whole,
  whether every text and original file in it is intact, which documents will have to be found
  again on the other computer, and which files would take another name there. Nothing is
  sent, changed or added. **Save to a file…** now checks its own file the same way before the
  file takes its name, so a file that does not check never replaces one you saved earlier.
- **Menus on a right click in the review, and a card that moves**
  ([#233](https://github.com/fmadore/Kalamu/pull/233)). A right click on a region of the scan
  (or the menu key) offers what its card offers and **Copy its text**; a right click in the
  text compared offers **Copy**, and on a difference **Use this reading**. A region's card can
  be dragged out of the way by its grip, and the caret in its text box is now visible.

- **Gemini 4 Neon in place of Gemini 4 Argon**
  ([decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-24-gemini-4-argon-replaced-by-neon.md)).
  New job no longer offers Argon, whose price doubles on 1 December 2026 to more than Neon's.
  Google still serves it: a pipeline, a workflow or a comparison that names it keeps it,
  marked as replaced, with Neon a click away.

## Also

- `cookie` 0.18.3, for the advisory RUSTSEC-2026-0594
  ([#250](https://github.com/fmadore/Kalamu/pull/250)), and the dependency updates held back
  from 1.10.0 ([#235](https://github.com/fmadore/Kalamu/pull/235)): rusqlite 0.43.0, Tauri
  2.12.7, SvelteKit 3.4.0, Vite 8.3.8 among them.
- The screenshots of the guide and the listings, taken again on 1.10.0
  ([#234](https://github.com/fmadore/Kalamu/pull/234)), and a home page in English and French
  at [fmadore.github.io/Kalamu](https://fmadore.github.io/Kalamu/)
  ([#255](https://github.com/fmadore/Kalamu/pull/255)).

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.10.1_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.10.1_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.10.1_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.10.1.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.10.1 over an earlier version keeps your jobs, settings, presets and
keys. A job database opened by 1.10.0 is read by 1.10.1 and the other way round; a version
before 1.10.0 refuses it.

## Known limitations

Those of [1.10.0](https://github.com/fmadore/Kalamu/blob/main/docs/release-1.10.0.md#known-limitations)
still hold. What this version changes among them, and adds:

- **These fixes have run against recorded answers and in the tests only**, on Windows and in
  CI on a Mac: none has yet been tried by hand in the packaged app.
- **Tables exported before this version are not known to Kalamu as its own**: the first export
  again under the same name writes `name (2).xlsx` and `name (2).csv` once, and replaces those
  afterwards.
- **A job written by 1.10.0 with two sources under one name** keeps its old files until the
  sources are written again: the later source's results then appear under `name (2) (2)`.
- **A project file has still not been carried from Windows to a Mac**, nor checked there. The
  check does not find rows the job database would refuse: those still show only on opening.
- **The menus on a right click and the card that moves** have run in WebKit in the tests, not
  on a real Mac.

## Included software

Unchanged since 1.10.0: see
[its notes](https://github.com/fmadore/Kalamu/blob/main/docs/release-1.10.0.md#included-software),
and `licenses/THIRD-PARTY-NOTICES.txt` in every download for each component and its licence.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
