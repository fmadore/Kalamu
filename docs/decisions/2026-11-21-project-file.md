# 2026-11-21 · A project saved as one file, and opened on another computer

## Context

Everything a researcher does in Kalamu lives in the system's data folder: the job database
(`jobs.sqlite`), the frozen texts beside it (`snapshots/`), the search's and the search by
meaning's indexes. The researchers Kalamu is for never see that folder, and the review of
20 November ([decision](2026-11-20-review-for-researchers-new-to-computing.md)) heard three
needs in their own words ([#213](https://github.com/fmadore/Kalamu/issues/213)):

- *I have a new computer*: the projects, the dictionary and hundreds of corrected pages must go
  along.
- *My colleague shares the review with me*: a professor and a doctoral student check one
  corpus. Prompts, pipelines and workflows could be passed on as files; a project with its
  results and corrections could not.
- *I am afraid of losing my work*: a broken laptop must not cost months of checking.

One file, saved from a project and opened on another computer, Windows or macOS in either
direction, must bring back the project with its documents' results and the researcher's layer
intact, and resume or export its jobs without a call to a provider.

## Decision

### What is in the file

The file holds one project and what was done with its documents:

- **The project**: its name, description, the parts of a source's description its documents
  share; its collections, documents (title, file, tags, description), memberships; the links
  from documents to the jobs that read them; the results preferred for each task.
- **The jobs that read its documents**, each kept to the sources tied to this project's
  documents: a job that read five files of this project and five of another goes with five.
  With them their pieces, the models' answers they name (immutable, carried as stored, keyed
  by their fingerprint), the researcher's corrections and review marks, speakers' names, the
  table of contents' items, the passages and regions read again by a model, the regions the
  researcher drew, and what each call cost (`charges`).
- **The frozen texts** (snapshots) the jobs read, both their records and their files from
  `snapshots/`, always, whatever the choice about originals: an analysis of a corrected text,
  a translation, an item's pages cannot be reviewed or exported without them, and they are
  small.
- **The project's review decisions**: samples drawn with their seeds, items kept aside with
  their notes, results set aside.
- **The dictionary**: the entries with their forms and review marks, the pairs kept apart, the
  proposal requests and the proposals turned down; and the jobs that asked a model for those
  proposals, whole (they read only rows Kalamu wrote, not a file of the researcher's).

Never in the file:

- **API keys and anything from the keychain**. The keys never reach the job database, so a file
  made from it cannot hold them; a test writes a file and looks through its bytes for a known
  secret placed everywhere the store could hold one outside the project.
- **Anything of another project**: its documents, its dictionary, a job's sources tied to it.
- **Settings, prompts and pipelines**: they are the researcher's, on this computer; a job keeps
  the prompts it ran with, which is what reproducing a result needs.
- **The derived indexes**: the word search (`search.sqlite`) and the vectors (`vectors.sqlite`)
  are made again from the results on the other computer, as they are when lost.
- **The search by meaning's model, consent and costs** (`meaning_spaces`, `meaning_runs`).
  Consent is the researcher's on their computer for their account, so a colleague who opens the
  file chooses a model and agrees for themselves. Without the vectors, the record of what
  preparing them cost describes numbers that are not there; it stays with the project it was
  paid for.
- **Batches still at a provider, and the files a job marked out of date.** A batch belongs to
  the account and the computer that sent it. A piece the file finds held in a batch, or
  running, goes as not yet read, and its job as interrupted: resuming it on the other computer
  sends it again, which the job's page says as for any interrupted job.
- **Workflow runs.** A run's frozen definition names every file it started with, of any project.
  Its step jobs go as ordinary jobs; continuing a run on the other computer is later work.

**Cost records go with their jobs.** A job's cost is part of its record (the job's page and
`provenance.json` give what producing its results cost), and the file says what was done, not
who paid: a colleague who opens it sees what the work cost when it was done.

### The original files

Scans and recordings can be gigabytes, and they may be confidential, so **by default the file
holds the results without the originals**. One named control, *Include the original files
(scans, recordings)*, adds them; before saving, the dialog says what each choice weighs (*About
12 MB* and *About 3.4 GB with the original files*), how many files there are, and how many
cannot be found on this computer (those are left out, said in the sentence after saving).

- **Opened with the originals**, they are written into Kalamu's data folder (`originals/`, one
  folder per project, a folder per folder the files shared, each under its own name) and every
  path that named them is pointed there. Each is checked against its fingerprint as it is
  written; the job reads them as the files it was started with, so nothing is read again.
- **Opened without them**, the paths are kept as they were written on the other computer, and
  the documents show as *File missing*: the text, the review, the dictionary, the search and
  the exports of tables work from what is stored. The sentence after opening says so, and says
  the pages will show once the project is saved again with its original files.
  Pointing a document at a file found on this computer (by name and fingerprint) is later work
  ([#213](https://github.com/fmadore/Kalamu/issues/213)).

**A path is text, never parsed as the other system's.** A Windows path opened on a Mac, or a
Mac path on Windows, is kept as written and simply does not exist there. Where Kalamu needs a
file's name or folder from a path written on another computer, it splits at both `\` and `/`
(`project_file::path_parts`), and the names it writes are kept to what both systems accept.

### The format

**One SQLite database with the extension `.kalamu`**: the job database's own schema, at the
version the saving Kalamu has (`PRAGMA user_version`), holding only this project's rows, plus
three tables of the file's own: a manifest (format `kalamu-project`, format version 1, the
schema version, the app version, when it was saved, the project's name and counts, whether the
originals are in), the files it carries (Kalamu's frozen texts, and originals when chosen) and
their bytes in chunks of 4 MiB. The layout is in [formats](../formats.md#the-project-file-kalamu).

- SQLite is what the workspace already uses (`rusqlite`, bundled), so **no crate is added**. It
  reads the same on Windows and macOS, a file of gigabytes is no harder than one of kilobytes,
  and the store's own migrations bring a file saved by an older Kalamu up to date (on a copy in
  the temporary folder: the researcher's file is never changed).
- **Checked when opened**, each failure said in a sentence: not a Kalamu project file (any other
  file, or a database without the manifest); damaged (SQLite's check, or a carried file whose
  bytes do not match their fingerprint); made by a newer Kalamu (a format version, or a schema,
  this one does not know), with the advice to update Kalamu.

### Opening

**Always a new project; never merged, never written over.** Every identifier the file holds
(project, collections, documents, jobs, snapshots, items, entries, requests, passages, regions)
is given a new one as it comes in, wherever it appears, so the same file opened twice, or a
colleague's copy of a project already on this computer, comes in beside it and cannot touch it.
The models' answers are keyed by their fingerprint and taken as they are: an answer already
here is the same answer, and the first stored is kept, as everywhere in the store.
The project keeps its name; if one of this computer's projects has it, the name takes the date
the file was saved, `Letters (2026-11-21)`, then a number. It is opened unarchived.

Bringing a colleague's corrections into one's own project (a merge with its conflicts shown)
and a backup of the whole app are later work.

### Where it lives

The file is the job database's tables, so the **store** writes and reads it: it is the only
crate that knows them. `core` defines the trait, **`ProjectArchive`** (plan, save, open), its
types and its errors, beside `JobStore`, `ProjectStore` and `DictionaryStore`; `store`
implements it in a module of its own (`project_file/`); the shell holds an
`Arc<dyn ProjectArchive>` wired from the store, gives it the data folder's places (`snapshots/`,
`originals/`) and exposes three commands. No crate gains a dependency on another: the
architecture test is unchanged. `projects` stays as it was: organising never writes a result,
and opening a file writes results.

### Interrupted saving and opening

- **Saving** writes `<name>.kalamu.part` beside the file chosen and renames it once complete,
  so an interrupted save never leaves a file that looks whole; a failure deletes the part.
  The project's rows are read under one lock, so the file is one consistent moment of the
  project; the originals are written after, without holding the store.
- **Opening** writes the carried files first, each to a part renamed once checked, then puts
  every row in one transaction. A failure anywhere rolls the transaction back and deletes the
  originals it wrote, so the store is as it was. Frozen texts are named by their content and
  never rewritten, so one left behind is a file Kalamu would have written the same way.

## Consequences

- *Save to a file…* in a project's header opens a dialog with one default path (results only),
  the choice about originals behind its one named control, the sizes shown before saving,
  progress while it writes, and a sentence when done saying where the file is, with a button
  to show it in File Explorer or the Finder. *Open a project file…* on the list of projects
  opens one, with progress, and opens the new project with a sentence saying what came in.
- The privacy policy says that the file holds the documents' text, the researcher's corrections
  and, if chosen, the originals, and leaves the computer only when the researcher moves it.
- A file saved by this version opens in later ones; a file from a later one is refused with a
  sentence until Kalamu is updated.

## Rejected

- **A zip of JSON documents, one per table.** Readable by other tools, but it needs a zip
  crate in the app, a serialisation of every table kept in step with the schema by hand, and
  its own migrations; the store's migrations already do this for SQLite.
- **The whole `jobs.sqlite` copied.** Simplest, but it carries every other project, every
  job, and whatever else the researcher did, which a file sent to a colleague must not.
- **Originals always in.** A project of recordings would make a file too large to send, and
  originals often cannot be shared when results can.
- **Opening into the same project when its identifiers are already here.** It would write a
  colleague's corrections over one's own without a word; until a merge shows what differs,
  a new project beside the old is the honest answer.
- **Asking where to put the originals when opening.** A second question on the default path;
  Kalamu's data folder needs no permission on either system, the Mac App Store's sandbox
  included.

## Later

- Pointing a project opened without its originals at the files on this computer, by name and
  fingerprint, in a folder the researcher chooses.
- Bringing a colleague's corrections back into one's own project: a merge with its conflicts
  shown.
- Workflow runs carried with their jobs, so a run continues on the other computer.
- Writing a job's files on the other computer to a folder chosen then: a job set to write beside
  its sources, or into a folder of the first computer, has nowhere to write until then. The
  project's own exports (descriptions, answers, the review manifest, the dictionary) go where
  the researcher chooses, and work at once.
- A backup of the whole app, every project at once.
- The file opened by double-click (a file association), and a `.part` left by a crash cleaned
  at the next launch.
