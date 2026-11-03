# 2026-11-03 · Project search, first delivery

## Context

[#86](https://github.com/fmadore/Kalamu/issues/86), the full-text search left from
[#57](https://github.com/fmadore/Kalamu/issues/57): a researcher with hundreds of newspaper
articles or interviews in a project wants every passage that names an organisation or a person,
without opening each document, and to go from a passage to its source. The review's search
([decision](2026-10-29-search-in-the-review.md)) already finds words in one document as the
researcher reads it; project search covered only titles, file names and tags.

None of the three reference projects searches across results: iwac-ai-pipelines and
zmo-ai-pipelines write files and leave searching to Omeka or the file system, and Live
Translation keeps no corpus. The design is Kalamu's own. iwac-ai-pipelines' name folding
(capitals, accents, punctuation) is already the dictionary's; this search folds as the review
does instead, since it must find a phrase where it stands.

A new crate and a new trait make this an architecture checkpoint (AGENTS.md). Its findings are
at the end.

## Decisions

- **A crate of its own, `search`.** Finding passages is neither organising documents
  (`projects`) nor reconciling names (`dictionary`) nor running work (`engine`). It depends on
  `core` only: documents, links and preferences through `ProjectStore`, pieces, answers and
  corrections through `JobStore`, and its index through a new `SearchIndex` trait. It never
  calls a model and never changes a result.
- **The text searched is the review's.** For an extraction, a transcription or a correction,
  each piece's correction in place of the model's text, the speakers' names in place of their
  labels; for an analysis, the corrected answers and the reasons, each field apart; for a table
  of contents, its combined entries, each field under the entry's title. A match never runs
  from one answer into the next.
- **Text read from a document and text a model wrote about it are kept apart.** Analyses and
  tables of contents are searched only when the researcher asks for them, and each of their
  matches names its field (and entry), and says when it was found in a reason.
- **Matching is the review's**: capitals, accents and Arabic vowel marks set aside, a run of
  spaces or line breaks matching one space, the query one literal phrase, found anywhere in the
  text, inside words too. `search/src/fold.rs` is `src/lib/search.ts` in Rust, counting
  characters as anchors do (#47), so a match maps back to the exact characters shown, and the
  review, opened on it, finds the same match with the same number. Three letters at least,
  where the review needs two: the index matches three characters at a time.
- **SQLite FTS5 with the trigram tokenizer**, from the SQLite already bundled (`rusqlite`'s
  `bundled` builds FTS5 in). Trigrams find a phrase inside a word, which a word index cannot:
  Arabic joins its article and some prepositions to the word (`بالمسجد`), French elides
  (`l'Union`). The text arrives folded, so the tokenizer folds nothing more and both sides fold
  alike. The index is contentless (`content=''`, `contentless_delete=1`); each passage's text as
  shown is kept beside it, so a search reads no stored answer. No vector database: lexical
  search needs none.
- **The index is derived, never authoritative.** It lives in a file of its own,
  `search.sqlite`, beside the job database, with its own connection, so indexing never holds
  the lock the review and the engine use. Each passage carries a stamp of what it was made from
  (the stored answer's key, when the piece was last corrected or reviewed, the speakers' names,
  a version of the folding). Before each search, the stamps of the results searched are read
  from the store without reading any answer; only passages whose stamp changed are made again,
  in batches of 64, and those of pieces gone, failed or no longer searched are removed. So a
  correction, a speaker named, a preference, a link removed or a deleted job shows at the next
  search, with no hook in the engine. A file that cannot be opened, or of another layout
  version, is emptied and made again from the store; *Prepare the text again* empties it on
  request. Deleting a job removes its passages, and the next start removes those of jobs
  deleted while the index could not be told.
- **A newer search stops an older one** still preparing text, between batches, keeping what it
  prepared (a generation counter in the app). The search runs off the window's thread.
- **The result policy is said, not guessed.** By default each document's preferred result of a
  task, where the researcher chose one, and every result where none is preferred, which the
  view counts (*several readings and none preferred: every reading was searched*). *Every
  reading* searches them all. Two readings of one document stay two results under one document,
  never two documents.
- **Coverage is shown with every search**: documents in scope, those with text searched, those
  with nothing read yet of the kinds searched, the pieces of the results searched still to read
  or failed, and, when nothing is found, that finding nothing is not absence.
- **Filters**: a collection, a tag, a task, reviewed or not yet reviewed pieces, the
  preferred or every reading, and the generated text.
- **Opening a match.** The review opens on the source with the words in its search and goes to
  the match: its piece, and the same match there (the n-th in that piece), playing a recording
  from its line. A match in another set-up's text opens its page with that text in the compare
  pane. Since every search brings the index up to date first, the match is in the text as it
  stood; if a correction has moved it since, the review says so rather than opening another
  place silently. An analysis or a table of contents opens with its own search filled in.
- **The last search of each project is kept in memory** for the session, so coming back from a
  review shows it again. Nothing about searches is written down.
- **Stale analyses are marked** by the app, from the engine's `derived_states`, as the job's
  page marks them.

## Measured

A synthetic project of 500 documents, 4,000 pages and 12 MB of text (an ignored test,
`cargo test -p kalamu-search --release -- --ignored --nocapture scale`), on the development
machine while other builds ran: the first search, which prepares every page, took 3.6 to 5 s;
a phrase then took 70 to 120 ms; a three-letter word on every page (43,586 matches, the first
1,000 listed) took 1.3 to 1.8 s, most of it folding each candidate page to place its matches.
The index took 38 MB, about three times the text: the trigrams, and the text as shown kept
beside them.

## Left for later

- Several words anywhere in a passage, alternatives and proximity, beside the literal phrase.
- Preparing the text in the background before the first search, with its progress shown.
- Opening a table of contents' match at its entry, and an analysis's at its field.

All but alternatives were done the same day
([decision](2026-11-03-search-words-and-preparation.md)).
- Searching two letters, and limiting the folding work for very common words.
- Semantic search, only with explicit local or cloud execution, consent to sending text,
  versions, costs and coverage (#86's later part); answers and chat stay out of scope.

## Checkpoint

- **Dependencies point inwards.** `search` → `core`; `store` implements `SearchIndex` in its
  own file and connection; `app` wires `Search` and marks stale analyses through the engine.
  The engine is unchanged and cannot reach the index.
- **One job each.** `projects` organises, `dictionary` reconciles names, `search` finds
  passages, `engine` runs work, `store` persists them all behind their traits.
- **Resuming, caching, progress, provenance** are untouched: nothing here calls a provider.
  The index is a cache of the store with its own recovery; losing it costs a slower first
  search.
- **Duplication.** The rule for a piece's text as shown now lives in three places: the
  engine's `export_document`, the review's `PieceView` and `search/src/passages.rs`, which is
  the shortest. Folding lives in `search.ts` and `fold.rs`, tested on the same cases. Moving
  "the corrected view of a piece" into `core` would let all three share it; noted for the next
  checkpoint rather than done here, as it touches the export path.
- **Size.** Every new file is under 500 lines; `search/src/query.rs` is the largest, at about
  400.
- **AGENTS.md** lists `search` and `SearchIndex`; the specification, roadmap, README, store
  listing and the privacy policy (with a new effective date, for `search.sqlite`) are updated.

## Consequences

- The project's sidebar gains Search, above the dictionary.
- The app sends nothing new. It keeps a new file, `search.sqlite`, with a copy of the text of
  the results searched, made again from the job database whenever needed.
