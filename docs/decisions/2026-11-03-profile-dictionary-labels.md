# 2026-11-03 · Answers under the dictionary's labels in export profiles

## Context

A profile's answer column writes an analysis's list as the model gave it
([decision](2026-11-03-profile-result-columns.md)): `Moussa Traore` in one document and
`Traoré Moussa` in another, though the researcher gathered both in the project dictionary under
`Moussa Traoré` ([#48](https://github.com/fmadore/Kalamu/issues/48)). A dataset made for an
Omeka S import or for coding by hand needs the researcher's label, one per person, place or
term; [#55](https://github.com/fmadore/Kalamu/issues/55) left this for after the dictionary.

iwac-ai-pipelines matches folded names against existing authority items before linking them,
and writes the authority's title; it decides the match itself. Kalamu writes only what the
researcher decided: an entry's forms, as written.

## Decisions

- **A value of its own.** `ProfileValue::Labelled { key }` beside `Answer { key }`, so a
  profile can hold both the model's forms and the labels, side by side, and an existing
  profile's columns do not change meaning.
- **Only the researcher's entries.** Each form of the list is written under the label of the
  entry of that field (its namespace) that holds it; a form no entry holds is written as given
  (its spaces tidied, as the dictionary shows it); a label reached by two forms is written once,
  where it first appears. Look-alikes the dictionary only points out are not merged here, and
  another field's entry never applies. A text answer is written as it is.
- **Lookup in `core`, read by the app.** `kalamu_core::Labels` maps `(namespace, form)` to a
  label; `Dictionary::labels` builds it from the store, and the app passes it to the profiled
  export, reading the dictionary only when a column needs it. `projects` takes it as a value and
  gains no dependency on the dictionary's store or crate.

## Consequences

- `core` gains `Labels` and the value; `dictionary` `labels`; `projects`' profiled exports take
  the labels; the export dialog offers "an answer, under the dictionary's labels".
- No new data is kept and nothing leaves the computer: the privacy policy is unchanged.
- Left for later: the entry's id in a column of its own, for an import that links items rather
  than names; normalised and inferred values (with #54).
