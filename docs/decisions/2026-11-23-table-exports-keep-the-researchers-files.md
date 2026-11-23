# 2026-11-23 · Table exports never replace a file of the researcher's

## Context

Four exports write a table beside the file the save dialog asked about (#237): *Export
descriptions*, the dictionary's export and *Export the list* of the review ask for `name.json`
and also write `name.xlsx` and `name.csv`; *Export the whole table* of a screening or analysis
asks for `name.xlsx` and also writes `name.csv`. `write_table` wrote the companions with
`std::fs::write`, unchecked, so a workbook the researcher kept beside an earlier export, with
columns of their own, was replaced without a question. A pipeline's or workflow's record already
avoided this for its Markdown (`kalamu_prompts::markdown_beside`), recognising its own earlier
Markdown by its first lines.

## Decision

- **One rule, in `write_table`, for the four exports**: before anything is written, the files
  beside the chosen one are named with `kalamu_prompts::beside`, generalised from
  `markdown_beside` (which now calls it): `name.xlsx` and `name.csv` when each is free or
  Kalamu's own, else both under the first `name (2)`, `name (3)`… where they are. The workbook
  and the CSV keep one name between them. The command returns the files written under another
  name, and the interface says so after its usual message (`common.tableBeside`).
- **A free name, not a question.** Asking once by name would add a second dialog to every export
  and a new command and step in four components, for files the researcher did not choose; the
  record's Markdown already works the free-name way, so both kinds of export now behave alike.
- **Kalamu's own files are recognised by what it remembers, not by what they hold.** A CSV has no
  place for a mark, and a workbook opened and saved in Excel keeps any mark it had while holding
  the researcher's columns. So the app keeps `exports.json` in its data folder
  (`src-tauri/src/exports.rs`): each file a table export wrote, by path and SHA-256 of its bytes,
  the last 500. A file is Kalamu's only while its bytes are those written; once changed, it is
  the researcher's. The rule of where to write stays in `prompts`, which only receives a
  closure saying what is Kalamu's: the Markdown's recogniser reads its first lines, the tables'
  asks the list.
- **Each file is written whole** through a temporary file in the same folder (`tempfile`),
  persisted over the old one at the end (`commands::write_whole`); the chosen JSON or workbook as
  well as the companions.

## Consequences

- Exporting twice to the same name replaces Kalamu's own files, numbered or not, and never piles
  up copies; a test per export checks both this and that a researcher's file is kept byte for
  byte.
- Files exported before this version are not in the list: the first export again to the same
  name writes `name (2).xlsx` and `name (2).csv` once, and replaces those afterwards. Losing
  `exports.json` costs the same and nothing more.
- A file moved or renamed by the researcher is not recognised at its new place, which can only
  lead to a numbered name, never to a file replaced.
- The privacy policy names `exports.json`. The Mac App Store build already asked for the folder
  before these exports (`askForFolders`); the temporary file needs the same folder.
