# 2026-11-03 · Source descriptions, project defaults, and renaming a tag

## Context

[#57](https://github.com/fmadore/Kalamu/issues/57) left a document's descriptive metadata and
renaming a tag across a project; [#55](https://github.com/fmadore/Kalamu/issues/55) asks for
supplied archival metadata kept apart from what models read or infer, project defaults with
per-document overrides, and exports that say where each value came from, before reusable
export profiles.

iwac-ai-pipelines leaves descriptions to Omeka S, where they are an item's Dublin Core
properties: its scripts find items by `dcterms:identifier` and write model output (entities,
summaries, transcriptions) back into the same items. zmo-ai-pipelines keeps no description of a
source, only each file's run signature (name, size, hash). Neither has defaults shared by a set
of items. Kalamu keeps the researcher's description apart from every result, as it keeps
corrections apart from AI output.

## Decisions

- **A fixed set of parts, typed.** `DescriptionField`: repository, fonds or collection, series,
  box or folder, shelfmark, creator, date, web address, citation, rights: #55's list with the
  creator and the date as written, which every citation needs. A fixed set gives each part a
  translated label and a stable column, and leaves the export profiles to come a known
  vocabulary to map from. Free fields of the researcher's own are left for later.
- **Supplied only.** A description is typed by the researcher. No model reads or writes it and
  no result changes it; dates or places a model infers (#54) will sit beside it, never in it.
- **Defaults on the project, values on the document.** `Project.source_defaults` holds the parts
  its documents share; `Document.source_description` its own. A part the document leaves empty
  takes the project's: there is no way to say "none" against a default, which the first users
  are not expected to need. Values are trimmed, line breaks inside kept, empty parts dropped.
  Both are a JSON column (`{}` by default) on the existing tables, so adding a part later needs
  no migration.
- **Where each value came from is decided in Rust** (`kalamu_projects::described`) and shown on
  the document's page (the project's value as the field's hint, *From the project* under it),
  and written into every export.
- **Export: the documents shown, as JSON and CSV.** The project's, or a collection's when one is
  open. JSON (`kalamu-descriptions`, version 1) gives each part as `{value, from}`; CSV gives one
  row per document, each part in a column with a `<part>_from` column beside it, lists joined
  with `; `. The file name column is `file_name`, not `file`, which is a part of the description
  (the box or folder): a duplicate column the first test caught.
- **Renaming a tag** works from the table's tag filter, on every document of the project that
  carries it, whatever its capitals. A new name differing only in capitals respells it; one the
  project uses already, in any capitals, merges the two under the project's spelling.

## Consequences

- `Projects` gains `describe_document`, `set_source_defaults`, `rename_tag` and two exports; the
  app four commands; `DocumentDetail` its description as it stands.
- The job database keeps what the researcher types; the privacy policy says so, under the
  effective date of 3 November 2026, not yet released.
- Left for later in #55: saved export profiles, descriptions carried into results' exports and
  `provenance.json`, normalised and inferred values beside the literal ones.
