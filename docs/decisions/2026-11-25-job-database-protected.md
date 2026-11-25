# 2026-11-25 · The job database protected: copies, a check, a way back (#275)

## Context

Every correction, review mark, project, dictionary decision and paid answer lives in one file,
`jobs.sqlite`. The review of 25 November ([safety nets](2026-11-25-review-safety-nets.md))
found nothing between that file and one bad migration or one damaged page: migrations ran on
the live file with no copy taken; `quick_check` ran on project files only; a database that
could not be opened ended the launch with "may be damaged"; no test migrated a database an
earlier release had written; one job record this version could not decode failed the whole
job list; and a frozen text cut short by a power cut was refused as changed for ever, since
replaced files were renamed without being flushed and a frozen file was never written again.

## Decision

**Where the copies live.** In a folder of their own, `backups`, in the app's data folder beside
the database (on Windows `%LOCALAPPDATA%\Kalamu Data\backups`, on macOS
`~/Library/Application Support/Kalamu/backups`, inside the package or container for the store
builds). Never beside the user's documents, never sent anywhere. Each file's name says what it
is and when it was made, in UTC: `jobs-daily-2026-11-25-143210.sqlite`,
`jobs-before-update-…`, `jobs-set-aside-…`. The names, not a separate record, are what the
app reads back, so a copy moved in or out by hand is understood.

**What is copied, and how many are kept.** The database can be large (paid answers, every
page's text), so few copies, each made with `VACUUM INTO` (a consistent, compacted copy that
holds what the write-ahead log holds), flushed to the disk, and given its name only once whole:

- *Before an update*: when `user_version` is behind this version's migrations, a copy is
  taken before the first migration runs. It is kept until **three daily copies are newer**
  (three launches, on three days, went well with the updated database), and at most the
  **two** newest are kept.
- *Daily*: once a day the app is opened, after the check below has passed, so every daily copy
  is of a database that read whole. The **three** newest are kept.
- *Set aside*: a database that could not be read, or that a copy replaced, is **moved** into
  the folder with its log, never deleted, and never removed by the app; Settings says the
  researcher may delete it once it is no longer needed. It can be opened again like a copy,
  which is how opening a copy is undone.

So the folder holds at most five copies besides what was set aside: for a 2 GB database, up to
10 GB. That is the cost of not losing paid work; *Clear unused results* shrinks every later
copy.

**Never filling the disk silently.** Before each copy the free space of the folder's disk is
asked (`fs4`, already in the tree); a copy is made only if, once made, at least **1 GB** stays
free beyond it. A copy skipped is logged and said in Settings (how much was free). The update
still runs when its copy was skipped: refusing to start for want of space would cost the
researcher more than the copy protects, and each migration is still a transaction.

**The check.** At launch, before anything changes the file, the database is probed cheaply
(its header and table list read on a connection of its own). A file SQLite calls damaged or not
a database leads to the question below. SQLite's `quick_check`, which reads every page and
takes seconds on a large file, runs **after launch, off the window's thread**, on a connection
of its own. When it finds damage the interface offers the newest copy at once (and Settings
says so until it is dealt with); otherwise the day's copy is made and old copies pruned.
`integrity_check` (indexes matched against tables, many times slower) is not run.

**A damaged file at launch.** The interface does not exist yet, so the shell asks in a native
message, in English or French as the start-up messages are, naming the folder the file will be
kept in and the logs folder: *Open the copy* (the newest daily or before-update copy, dated) or,
when there is none, *Start with an empty list*; or *Quit*, which changes nothing. Either way
forward the file is set aside first. Errors that are not the file's (a newer version's
database, no permission, a disk gone) are reported as before and nothing is moved.

**Opening a copy later.** Settings › General › *Safety copies* lists the copies and opens one
after asking. The database is in use, so the choice is written into the folder
(`open-at-next-launch.txt`) and the app restarts; the next launch sets the database aside and
puts the copy in its place before opening it. Refused while a job runs. Setting aside is tried
again for a few seconds while the ending process still holds the file. The request is
forgotten whatever happens, so a copy that cannot be opened is not tried at every launch.

**Fixtures.** `crates/store/fixtures` holds a database written by 1.0.0 and by each minor
release since: every one changed the schema (1.1, 1.2, 1.3, 1.6, 1.7, 1.8, 1.9) or a record the
store keeps as JSON (1.4, 1.5, 1.10); the patch releases changed neither. Each was written by
that release's own `core` and `store` crates, taken out of git into a scratch workspace by
`fixtures/make.sh`, with `fixtures/generator.rs` appended to that release's store tests so its
own helpers make the records: one job with two answers and a correction, and from 1.1.0 a
project with a collection, a document, a link and a chosen result. All eleven tags built. The
store's tests migrate each and read everything back; a list in the test must match the folder.
A release that changes the schema or a stored record runs `make.sh <tag>` and adds the version.

**One bad row.** `list_jobs` leaves out a job whose record or status this version cannot read,
and `unreadable_jobs` names it (id, name, why); the interface names it once at start-up and the
log records it. The record is kept unchanged, for the newer version that may read it.

**Files replaced.** `write_atomic` flushes the temporary file (`sync_all`) before the rename.
A frozen text whose file is missing or no longer matches its fingerprint is written again from
the store when the result it was frozen from, as corrected now, still gives exactly the same
text: when the same text is frozen again, and when a job that reads it finds it changed.
Otherwise it stays refused, as before.

## Consequences

- The privacy policy names the `backups` folder (effective 25 November 2026).
- Each launch reads the whole database once in the background; on a slow disk with a large
  database this is the price of finding damage before it spreads.
- `sync_all` costs some milliseconds per result file written; a job writes few files per
  source, so it is not felt beside the calls.
- `JobStore` has a new method with a default (`unreadable_jobs`), so stores in tests need no
  change; `AppInfo` carries the unreadable jobs.
- Not tried on a real Mac, nor in the packaged Windows app: the restart that opens a copy
  (`request_restart` with the single-instance plugin), and moving a file the ending process
  still holds, are covered by the retry and by tests below the shell only.
