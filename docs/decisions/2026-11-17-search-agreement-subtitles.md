# 2026-11-17 · Search alternatives and two letters, agreement between set-ups, subtitles of a translated transcript

## Context

After 1.8.1, three things the roadmap listed as open needed neither a live run nor a Mac, and
their files did not overlap: what was left of the project search, a job-level measure of how
far the set-ups of an analysis agree, and the subtitles of a translated transcript
([#167](https://github.com/fmadore/Kalamu/issues/167)). Each was written by an agent in a
worktree of its own, reviewed, and gathered on one branch where every check was run again.

## Decision

### Project search: one or another, and two letters

- **A word, not a symbol.** OR and OU in capitals are typed without a manual on either
  keyboard; `|` is AltGr and 6 on an AZERTY keyboard and unknown to most researchers. Both are
  read whatever the interface's language, so a query means the same in each. Capitals only, so
  *or* (gold, in French) and *ou* stay words. An operator at either end is dropped, so a search
  fired while typing does no harm. Among several words OR joins the words on either side into
  one group, as web search engines read it; the index is asked for `(a OR b) AND c`.
  `SearchIndex::find` takes groups of terms.
- **Two letters as a word on its own.** Found inside words, *de* or *ba* would be listed on
  nearly every page. A match is still numbered among the review's own matches, which count
  those inside longer words, so opening it lands on the right one. Scripts written without
  spaces have no word boundaries, so two of their characters stay too short.
- **The index.** It matches three characters at a time
  ([3 November](2026-11-03-search-words-and-preparation.md)). Rather than a second index or a
  scan, it now holds every character that is neither a letter, a digit nor a space between
  spaces, and a space at either end, so a two-letter word always stands as ` ab `. Queries are
  spaced alike, so longer terms are found as before. The passages' version rose to 3: each is
  made again from the job store at the next search, the researcher doing nothing. The index
  grew by 0.5% on the benchmark and 5% on heavily punctuated text.
- **Folding.** Each part and line is folded once for all the terms, and ASCII characters skip
  decomposition: a common word on all 4,000 pages of the benchmark went from about 1.1 s to
  about 0.23 s; `de` on its own takes 0.2 to 0.35 s.
- `searchRules.json` carries the operators, the scripts without spaces and each case's groups,
  so the interface reads a query as Rust does, and a test compares the two.

### Agreement between set-ups across a job

- **Where.** The statistics are in `core` (`agreement`), with no I/O; the engine feeds them
  one piece at a time (`job_agreement`), so a large job never holds all its answers at once,
  and the command runs off the window's thread: 192 ms for 5,000 documents by two set-ups.
- **What is measured.** A choice: exact agreement and Cohen's kappa, computed in whole
  numbers and left out when chance alone would agree every time. A list or several choices:
  the Jaccard overlap of the items, averaged over documents, two empty lists agreeing. Free
  text and repeating groups are not measured: two models rarely word a summary alike, and the
  review's side-by-side view says more.
- **Which answers.** The models' own, not the corrections, as the comparison's agreement does:
  a correction is the researcher's reading. Only the pieces both set-ups of a pair answered.
- **Folding shared.** Items are compared with the dictionary's folding, moved to
  `core::list_items::folded` and used by both; `dictionary` no longer depends on
  `unicode-normalization` itself.

### Subtitles of a translated transcript

- **What was there.** A translation read the frozen text. Most transcriptions keep their times
  in segments, which never reached the snapshot; markers in the text were relative to each
  piece of a cut recording. So a translated transcript had no usable times.
- **Lines frozen with the snapshot** (`Snapshot::lines`, `DerivedFrom::timed`), all or
  nothing: only when every part read has its times. The text and its hash are unchanged, so
  analyses and earlier translations keep their cache keys.
- **Numbered lines.** A translation of a timed source plans passages of whole lines (6,000
  characters or 120 lines), sends `[n] text` with the request to give each line back under its
  number, and answers in the entries structure records use, so no provider changed. Times and
  speakers are never sent and never read from the answer: they are copied by number.
- **A broken answer is refused, never shifted.** Every number must come back once, with text.
  Otherwise the answer is an `InvalidAnswer`, billed and asked again by the usual retries; the
  piece fails with the reason after them. Cache keys add `timed_lines:<hash>`.
- **Corrections.** The times hold while a corrected passage keeps one paragraph per line. Any
  other count loses them: provenance lists the passage (`lines_changed`) and SRT and WebVTT are
  refused with the reason, the other formats written as corrected.
- **Formats.** `Task::writes` keeps subtitles out of a translation with no timed source; New
  job says why they are not offered.

## Consequences

- Nothing new is sent to a provider: a translated transcript's words went there already, and
  now go without their times or speakers. The privacy policy does not change.
- **Economy mode sends timed passages at once**, since a batch's collection does not check an
  answer line by line: they cost the full price where the estimate counts half. To take up
  with the batch code.
- A transcript corrected in the review under a segment-based engine loses that part's times,
  as before, so its snapshot has no lines and its translation no subtitles.
- The workflow editor still offers SRT and WebVTT on a translation step whatever it reads; the
  engine drops them when the input has no times.
- The lint's allowance of files outside `tsconfig.json` rose from 16 to 24 with the new
  browser tests.

## Not tried

A real model answering numbered lines; the search's one-time preparation on a large real
project; the agreement table with three set-ups and more on screen; any of it in the packaged
app or on a Mac.
