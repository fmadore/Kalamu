# 2026-11-20 · The default prompts' fields in the interface language

## Context

The default prompts are written in English, which the models follow best, and only their names
and descriptions were translated (`localisePreset`). Their fields were not: in the French
interface an analysis's review read *Summary*, *Themes*, *Why it matters*, *Research question*,
a register's columns *Number · Date · Kind of entry*, a publication's methods *Archival
research*, and a project's dictionary grouped names under *People*, *Places*, *Organisations*. A
French-speaking researcher with little English read half the review in another language (#210).

A field has a key (`people`), a label, help for the model (`instructions`) and, for a choice, the
values the model must answer with. The key names the answer in results, corrections, exports,
the dictionary's namespaces and cache keys; the label and help are part of the prompt sent; the
model's answer to a choice is one of the values, stored as it is. A job freezes the prompt it ran
with (`PromptSnapshot`): its `preset_id` and its fields, with no flag saying whether the
researcher had edited the default.

## Decision

- **Display only.** One helper beside `localisePreset`, `localiseFields(presetId, schema)` in
  `src/lib/i18n/index.ts`, gives a copy of a schema for the screen with labels and help in the
  interface language and each choice's label beside its stored value (`choiceLabel`). Keys,
  kinds and choice values are never changed, nor is anything sent back to the app: results,
  corrections, cache keys, the prompt sent, provenance and the dictionary's keys stay
  byte-identical. A choice corrected through its menu stores the English value; the menu shows
  its French label.
- **Word for word, field by field.** A label, help or choice is shown translated only when it is
  exactly the default's shipped English for that field of that preset. This is how an
  unmodified default is recognised on a job's frozen schema without a new stored flag: the
  `preset_id` names the default, and its shipped fields are compared text by text with the
  frozen ones. Whatever the researcher wrote (a label renamed, a choice added, a prompt of their
  own) stays as written, and in an edited default only the words they did not change are shown
  translated. A job made when a default's English read differently shows that text as stored.
- **The shipped fields come from Rust.** A test in the prompts crate writes them to
  `src/lib/i18n/defaultFields.json`, as ts-rs writes the bindings, and CI checks the file is
  committed. The French texts live in `fr.ts` under the catalogue's `presets`, by preset id then
  field key, nested fields included; English needs none. A unit test fails when a shipped
  field, help or choice has no French text, or when a French text is left for one no longer
  shipped.
- **Where only a name is known.** A project's dictionary, the list of what to check, the
  agreement table, a project's search and the export dialog's field choice receive a field's
  label (and key) from views the app builds across jobs, without a preset id. `fieldLabel`
  shows the French label when that label (and key) is exactly a default field's; the unit test
  checks each English label has one French label.
- **Where it applies.** The three reviews (analysis, table of contents, records), their answers'
  choice controls and the Add menu, New job's list of what a prompt finds, the agreement table,
  a project's dictionary, a project's search results, the list of what to check and the export
  dialog's choice of field. **Prompts shows the fields as stored**: what is edited there is
  saved and sent to the model, and showing French there would save French labels, help and
  choices on the next edit.
- **Exports keep the stored English labels**, unchanged in this increment: a table sent to a
  colleague or read by a script does not change language with the interface, and no exporter
  changed. Headings in the interface language, named in `provenance.json`, are a possible later
  option.

## Consequences

- A change to a default's field (its label, help or a choice) changes `defaultFields.json` and
  fails the unit test until `fr.ts` has the French; German adds `de.ts`'s texts the same way.
- The help texts are translated though only Prompts shows help today, and Prompts shows it as
  stored: they are ready for a place that shows a field's help.
- The dictionary's prompt (`dictionary.merges`) is left out: its answer is read by the
  dictionary, never shown as fields.
- A researcher's own prompt whose field is labelled exactly as a default's (`People`, key
  `people`) is shown translated where only the name is known (the dictionary, the list of what
  to check): the words are the default's own.
