# 2026-11-23 · Keep and Set aside read in every language

## Context

The [screening of 20 November](2026-11-20-research-question-screening.md) keeps the
researcher's decision as one of two tags, *Kept* or *Set aside*, named in the language the
interface had when they were given. It read them back in the current language only
([#240](https://github.com/fmadore/Kalamu/issues/240)). Once the language changed, or a project
file passed between a colleague working in French and one working in English, a document kept
in English showed no decision in French; setting it aside there added *Écarté* beside *Kept*,
so English showed it kept, French set aside, and the table's `tags` column exported both.
German will add a third pair.

## Decision

Still two tags, no table: the 20 November reasons hold, and a decision stored apart from tags
(or one decision per question) stays rejected.

- **Read in every language shipped.** `decisionOf` (`src/lib/screening.ts`) looks for the two
  tags of every catalogue in `LOCALES` (`everyCatalog()` in `i18n/index.ts`), so adding German's
  catalogue is all a third language needs. Tags are compared as the projects crate compares
  them: capitals and runs of spaces set aside (`same_tag`, after `tag_name`).
- **Written in the current language, one left.** The tag given is the interface's; taken off
  are every other decision tag the documents carry, the opposite in any language and the same
  given in another (*Kept* replaced by *Gardé*, never both). The row's toggle takes off the
  decision in whatever language it was given.
- **Both carried: a conflict, the same in every language.** `decisionOf` answers `both`; the
  row shows neither button pressed and *Kept and set aside: choose one* (*Gardé et écarté : à
  trancher*); Keep or Set aside, from the row or the selection bar, settles it and leaves one
  decision tag.

## Consequences

- No Rust change: the crates do not read these tags; the table's `tags` column still writes
  what the documents carry, which after a decision is one decision tag.
- The tag filter stays a filter on one tag's spelling: filtering on *Kept* does not show
  documents tagged *Gardé*. Keep and Set aside are read across languages; tags as tags are not.
- A word a researcher typed that happens to be another language's decision tag (*Gardé* as
  their own label) is read as a decision. Accepted: the words are the app's own and few.
