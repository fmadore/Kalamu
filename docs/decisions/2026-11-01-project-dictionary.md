# 2026-11-01 · Project dictionary, first increment

## Context

[#48](https://github.com/fmadore/Kalamu/issues/48) asks for one dictionary per project of the
names and analytical labels its analyses found: the researcher reviews the inventory, merges
spellings of one person or place into an entry with aliases, keeps apart what only looks alike,
and later asks a model to propose merges. It builds on the project workspaces of #57.

iwac-ai-pipelines matches names extracted by a model against Omeka authority records, whose
title is the preferred label and whose alternative titles are the aliases. Its matching keys
(`common/reconciliation.py`, `AI_authority_curation/curation_core.py`) fold capitals, accents
and punctuation and sort words; fuzzy matches are only ever suggestions, and merge plans wait
for a curator's approval. It keeps no list of pairs judged different. zmo-ai-pipelines and Live
Translation reconcile nothing. AI-NER-Validator keeps a status per entity, without merges.

A new crate is an architecture checkpoint (AGENTS.md). Its findings are at the end.

## Decisions

- **A crate of its own, `dictionary`.** Reconciling names is neither organising documents
  (`projects`) nor running jobs (`engine`). It depends on `core` only, reads documents and links
  through `ProjectStore`, results through `JobStore`, and keeps its own rows through a new
  `DictionaryStore` trait, which the app wires to the same database.
- **Gathered, not stored.** The inventory is read from the results each time, through the
  corrected view of each answer, as the project views are; only the researcher's decisions are
  stored (`dictionary_entries`, `dictionary_apart`). A correction or a deleted job changes the
  inventory at once, and an entry left without evidence is marked rather than removed.
- **List fields only, one namespace per field key.** Lists are where analyses put names and
  terms. Choice fields draw on the researcher's own fixed values, so there is nothing to
  reconcile; free text holds no names to count. Fields with one key in different analyses share
  a namespace; different keys never mix, so a town and a person spelled alike stay apart.
- **Forms are kept as written.** A form is the model's string with its spaces tidied; capitals
  and accents are kept, since they may tell two names apart. An entry maps forms to a label,
  which need not be one of them. A form is in one entry of its namespace at most, so the
  mapping from original to canonical value is a function, as an export needs.
- **Every entry is a decision.** There are no automatic entries: a form no entry holds is shown
  as its own row, not reviewed. Merging marks the result reviewed; the oldest entry keeps its
  id and what it was kept apart from. "Oldest" is the store's order of making, since two
  entries made in one millisecond cannot be ordered by time. Taking every form out of an entry
  removes it, which undoes a merge.
- **Look-alikes are pointed out, never merged.** Forms with one folded key (capitals, accents
  and punctuation set aside, as iwac-ai-pipelines folds names) or one word-order key are offered
  as possibly the same. Unlike iwac-ai-pipelines, a decision that two are different is kept
  (`KeptApart`), refuses a merge until withdrawn, and silences the suggestion.
- **Documents and agreement, not mentions.** A list answer says that a document names someone,
  not where or how often. The dictionary counts documents, and over them the results that
  listed a form out of those that read the field, so two models listing a name reads as
  agreement, not as two mentions. Exact spans and mention counts wait for anchors (#47).
- **Export** writes the entries as JSON (`kalamu-dictionary`, version 1: ids, labels, forms,
  review status, pairs kept apart) and every form each result listed as CSV, with its entry's
  id and label, so raw and canonical values travel together.

## Left for the next increments

- Merges proposed by a model, through the engine as any job, so caching, resume, cost and
  provenance are those of any call: the inventory sent in bounded groups with overlap and
  explicit coverage, answers checked to refer to existing rows, an estimate before sending,
  nothing applied without the researcher, pairs kept apart never offered.
- Import of a dictionary, with conflicts and id collisions previewed.
- An accepted dictionary versioned and given to later analyses as a fingerprinted input, with
  workflow steps that read it (#49).
- Two people sharing a spelling in different documents: the mapping is per form, so they need
  mentions anchored in the text to be told apart.

## Checkpoint

- **Dependencies point inwards.** `dictionary` → `core`; `store` implements `DictionaryStore`;
  `app` wires it. The engine is unchanged and cannot reach the dictionary.
- **One job each.** `projects` organises documents, `dictionary` reconciles names, `engine`
  runs work, `store` persists all three behind their traits.
- **Resuming, caching, progress, provenance** are untouched: nothing in this increment calls a
  provider. Model proposals will go through the engine.
- **Size.** Every new file is under 500 lines. `projects/src/tests.rs` is at 501 lines, its
  newer tests already in a submodule.
- **AGENTS.md** now lists `projects` and `dictionary` in the crate table and `DictionaryStore`
  among the traits; the specification has a section on the dictionary; the privacy policy lists
  what it keeps.

## Consequences

- The project sidebar gains Dictionary, between the collections and Activity.
- The app sends nothing new. The privacy policy lists the dictionary among what a project
  keeps.
