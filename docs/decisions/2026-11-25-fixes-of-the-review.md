# 2026-11-25 · The six bugs of the review of 25 November

## Context

The [review of 25 November](2026-11-25-review-safety-nets.md) found six bugs (#269 to #274)
and put them first. They were written the same day by agents in worktrees, one or two bugs
each, gathered on one branch and read again there. The crates were built and tested on
Linux; the shell does not build there, so CI on Windows and macOS is the first to compile
what changed in `src-tauri`.

## Decision

- **The limit on calls at the same time holds across a job's files (#269).** A job has one
  pool of calls. Sources are prepared one after another, as economy mode's gathering needs,
  and a new one is opened only when those open have nothing left to send, never more of them
  than the limit: a long PDF is still read alone, a folder of photographs as many at once as
  the limit. A source's files are written once none of its calls is running. An engine error
  while a source is prepared or finished now also lets the calls already started be stored
  first. The per-source steps moved to `engine/src/run/opened.rs`.
- **A 429 that states a wait is waited out, whatever it calls itself (#270).** The wait is
  read from `Retry-After`, from Gemini's `RetryInfo`, or from "retry in" or "try again in"
  anywhere in the provider's whole message, before it is cut for the researcher. One bound
  decides, 300 seconds. An exhausted account still stops the job: a 402, `insufficient_quota`,
  a low credit balance, a quota per day, a longer wait, or quota words with no wait.
- **An index file is replaced only when SQLite says it cannot be read (#273)**: not a
  database, or damaged. Any other failure is returned and the file left; the app then runs
  on an index in memory, as it already did when opening failed. A vector index that is
  replaced is set aside as `.unreadable`, since making it was paid for; the word index, made
  again for nothing, is deleted.
- **A project's copied-in originals go with it (#274).** Deleting a project removes
  `originals/<project>`, after its rows, and the question before deleting counts the files.
  Kept: a file there that a document of another project names, and everything outside that
  folder. The jobs made in the project stay, as they always did, and show the file as
  missing: keeping what a surviving job read would keep everything, which is the bug. A file
  that cannot be removed is logged and tried again at launch, when folders whose project no
  longer exists are swept.
- **An older project file is brought up to date in the data folder (#274)**, in
  `project-file-scratch`, given to the store as a value and emptied at launch, where it was
  copied whole into the system's temporary folder.
- **Text takes its direction from what it holds (#272).** One class, `.own-direction`
  (`unicode-bidi: plaintext`, `text-align: start`), on every block and text box that shows or
  edits a model's or the researcher's text, so each paragraph follows its first letter;
  `dir="auto"` only on one-line fields and single values, since on a block it reads the
  first letter of the whole. A transcript's speaker label is set apart (`speakerOf`), so an
  Arabic turn is not read as left to right for its Latin label. The interface itself stays
  left to right. Not covered: prompt editors, the names of jobs, projects and tags, the
  dictionary table's labels.
- **Closing the window or quitting asks before changes not saved are lost (#271).** The
  interface tells the shell whether a draft holds changes (`set_draft_open`); the shell knows
  nothing more. A close that would lose them is held and `close-requested` sent, now saying
  whether jobs run; the interface asks its existing question, then the one about the jobs.
  On macOS the red button only hides the window and loses nothing, so it asks about jobs
  alone; ⌘Q asks about both. The tray's Quit quits at once, as it must not depend on the
  interface. A correction is still not kept as it is typed: a crash loses it.

## Consequences

- **Identical files in one job may each be paid for.** With one pool, two files with the
  same content can be sent at the same time, where the second used to wait for the first and
  come from the cache. Identical pages within one file already behaved so. New job flags
  photographs taken twice before anything is sent (#216); a pool that waits for a call
  already in flight with the same fingerprint is left for #284.
- `SourceExported` events may arrive out of the sources' order; what the files hold does not
  change.
- #270's wordings are as documented or remembered, and marked so in the tests: the issue
  stays open until a captured answer from each provider replaces them (#278).
- The interaction tests' files outside `tsconfig.json` passed ESLint's limit of 32 with the
  two new ones; it is 48.
- **WebKit starts a line on the side of `direction` alone.** CI's WebKit lane showed that
  `unicode-bidi: plaintext` orders an Arabic line's words there but leaves the line starting
  on the left. It also read a transcript's speaker label, though set apart, into the
  direction it found. Each line of the text being checked therefore carries `dir`, worked
  out from its first letter (`directionOf`), in place of the class. A block of several paragraphs (a compared text, the
  correction editor) cannot carry one `dir`: in WebKit its Arabic paragraphs are ordered
  rightly and still start on the left, to see on a Mac.
- **On Windows the working copy of an older project file was never deleted**, in the
  temporary folder before as in the data folder now: the copy was removed while the
  connection reading it was still open, which Windows refuses. The new test showed it on
  CI's Windows lane; the connection is now closed first.
- To try by hand, none of it having run in the packaged app: an Arabic page and transcript in
  WebView2 and WebKit; closing with a correction open on Windows; ⌘Q and the red button on a
  Mac, with and without jobs; a project opened with its originals, then deleted.
- The privacy policy says where copied-in originals are kept and when they go, with a new
  effective date.
