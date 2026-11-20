# 2026-11-21 · A sample first: a few typical pages, another model on them, then the rest

## Context

The guide's best advice to a newcomer is to read three or four typical pages first, see how
well the model reads them and what it costs, compare two models there, and only then run the
rest ([#215](https://github.com/fmadore/Kalamu/issues/215)). New job could already do each
step, but in three gestures a beginner does not find: *Compare with another model*, choosing
it, then, far below under *Results*, *Try a sample first* with its first N pages. Nothing on
the sample's page then led to the rest, and the first pages of a document are often the ones
least like it: a cover, a title page, a table of contents.

What was there to build on: the result cache is keyed by what changes the answer (task, the
source's content, the piece, provider, model, prompt, options, languages and the fixes asked
for), never by the job, the set-up's id or label, nor the sample
(`CacheKey::compute`); a job runs one or more set-ups since the first version of `core` and
`store`; and `JobSettings::sample_pages` already cut each document to its first pages. None of
the reference projects tries a sample, so there was nothing to port.

## Decisions

- **Typical pages, chosen simply and said plainly.** A new setting,
  `JobSettings::sample_spread`, asks that a sample's pages be spread through each document
  rather than be its first ones. `core::PageSample` (`First(n)`, `Spread(n)`, `All`) is the one
  place that says which pages a job reads: spread pages leave the first page out (most often a
  cover or a title page) and take the middle page of each of `n` equal stretches of the rest,
  so three pages of a 40-page document are pages 8, 21 and 34. A document no longer than the
  sample is read whole. The same document always gives the same pages, so a sample resumed
  reads what it planned. `media`'s `pieces`, `try_pieces` and `text_pieces` take a
  `PageSample` instead of a count; a spread page that is a split spread is read as its two
  halves, as a first page is. Provenance names the sample (`sample_spread_3_pages`) as it named
  `sample_first_3_pages`; a pipeline's record says it in words. The field is left out of the
  stored settings when false, so jobs and pipelines saved before it read as they did.
- **One plain suggestion in New job, for extraction.** When a text extraction is given a
  document of more than 15 pages (a PDF, or images read as one document), New job shows *Try on
  a few pages first* under the files: *Try on 3 pages*, and *Try on 3 pages with two models*
  when another model can be used (`JobChoices::secondDraft`: the first model with a key, or on
  this computer, that the job does not use yet, asked what the first is asked, as *Compare with
  another model* chooses). One click sets the sample (three pages, spread) and, for two models,
  adds the second set-up; once chosen, the suggestion says what will be read and offers *Read
  every page*, which takes the sample back and the model it added. The results step's checkbox
  shows the same sample, *3 pages spread through each document*; ticked there by hand, a sample
  is of the first pages as before. Nothing changes for a short document, for other tasks, or
  for a researcher who passes the suggestion by: the form is otherwise as it was.
- **Other tasks keep the first pages.** A table of contents is on a document's first pages;
  records, corrections and translations keep the checkbox under *Results*, whose meaning is
  unchanged. A correction or translation given a spread sample (from a pipeline) reads the same
  spread pages of the text layer or the same passages; a translation of a transcript's timed
  lines is tried on its first passages.
- **The rest, from the sample's page, never paying twice.** Once a sample has read pages, its
  page says what it read and offers, for each of its set-ups, *Run the rest with [model]*,
  priced: *37 pages left, about $0.04. The 3 pages already read cost nothing.*
  `Engine::rest_of_sample` makes a job of every page of the same documents with that set-up
  **copied as it was frozen** (a new id, the same provider, model, prompt and options), the
  sample's other settings, outputs and destination, and no sample. Because the key leaves out
  the job, the set-up's id and the sample, the sample's pages are taken from the cache when it
  runs (`PieceStatus::Cached`, nothing sent, nothing spent), and a prompt edited since the
  sample changes nothing. `Engine::rest_estimate` prices it with `Engine::estimate`, leaving out
  the pages the sample read whose answers the cache still holds (a table of contents still
  combines every page). The new job joins every project the sample's files are documents of.
- **Another model on the same pages, as an ordinary comparison.** *Compare with another model*
  on the sample's page opens a small dialog: the model offered first is chosen as New job
  chooses a second one, the cost of the new model alone, and *Start*.
  `Engine::compare_sample` makes a job of the same sources and settings (the sample kept) with
  the sample's set-ups copied as frozen and the new model asked the sample's prompt and options
  (the reasoning level left to the model), checked as New job would check it. It is a job of
  several set-ups like any other: the sample's set-ups come from the cache, only the new model
  is paid, the review compares them side by side, and its own page offers *Run the rest* with
  either.
- **Everything through the engine.** Both jobs are made by the engine and started as any job
  is; the commands `rest_estimate`, `rest_of_sample` and `compare_sample` refuse a provider the
  researcher has not agreed to send to, and the interface asks first, as New job does.

## Proved by

- `crates/engine/src/tests/rest.rs`: a 40-page PDF sampled on pages 8, 21 and 34 makes three
  calls; the estimate of the rest prices 37 pages, 37/40 of the whole; after the preset is
  edited, the job of every page makes 37 more calls, three pages come from the cache and it
  spends 37 calls' price. A comparison on the sample makes only the new model's three calls,
  and the rest with that model 37 more.
- `crates/core/src/page_sample.rs` and `crates/media/src/tests.rs`: which pages are read.
- `e2e/sample-first.spec.ts`: the suggestion, two models in one click, *Read every page*, no
  suggestion for a short document; *Run the rest* and *Compare with another model* from a
  sample's page.

## Rejected

- **Pages drawn at random.** They would differ between a sample and its resume unless seeded,
  and a researcher cannot be told which pages to expect; evenly spread pages are as typical and
  can be said in one sentence.
- **A job of only the pages left.** Its files would hold 37 pages out of 40 and the sample's
  three would sit in another job; the job of every page, three of them from the cache, writes
  the whole document at the price of the rest.
- **The rest made from New job's form again.** A prompt edited since, or a setting read
  differently by a newer version, would change the cache keys and pay the sample's pages
  again; copying the frozen set-up cannot.
- **A comparison opened in New job.** It would re-freeze the sample's set-up from its preset,
  with the same risk, and lose the sample's settings to a form that may hold another job's.
- **A sample across a folder of separate images.** Each image is its own document of one page,
  so "three pages of each document" reads them all; a sample of three of the documents would
  need the job to remember the others for the rest. Not offered for now: images read as one
  document are, and the decision stays open.

## Consequences

- The privacy policy is unchanged: nothing new is sent, kept or sent elsewhere. A sample sends
  fewer pages; its rest and its comparison send what New job would have sent for them.
- Left to try by hand: the suggestion with a real long PDF, the rest's estimate against a real
  bill, a sample in economy mode (the rest's pages from the cache go into no batch), and all of
  it on a Mac.
