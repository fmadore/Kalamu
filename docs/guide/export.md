---
layout: guide
title: Exporting and citing
order: 8
lede: Take your checked results where you need them, with a record of how they were made that you can cite.
description: Export formats and provenance in Kalamu.
---

[Français](fr/export.md)

## Formats

In **New job**, under **Results**, you tick the formats that suit the kind of result. Kalamu
writes the files beside your originals, or in a folder you choose, and writes them again after
your corrections.

| Format | Good for |
| --- | --- |
| Plain text, Markdown | Reading, quoting, any other tool. |
| Word | Sharing with colleagues, editing further. |
| Spreadsheet for Excel (`.xlsx`) | Reading, sorting and filtering a table in Excel, LibreOffice or Numbers: it opens with a double click, whatever language your spreadsheet is set to. Analyses export one row per document, or one per entry for a register. |
| Table for other tools (`.csv`), JSON | The same table for R, Python, statistics software or a database import. |
| Subtitles | Transcripts with their timestamps, to lay over a video. |
| Searchable PDF | The original scan with the text behind it, so it can be searched and copied. |
| ALTO XML, PAGE XML | Each word with its place on the page, for digital libraries and editions. Your corrections are included. |

Exports always use the text **as you corrected it**. The [output formats](../formats.md) page
describes every file field by field.

**Which table to pick.** Choose the `.xlsx` to open a table yourself. A `.csv` opened with a
double click in Excel set to French or German can show the whole table in the first column,
because those versions of Excel expect semicolons; the `.xlsx` has no such problem. Every cell
of the `.xlsx` is kept as text exactly as it was read or corrected, so a telephone number, a
date written in the source's own way or a page number stays as it is. A table of contents and
a register or directory tick the `.xlsx` for you.

In a register or directory, the `corrected` column marks only the rows you changed or added;
`reviewed` marks every row of a page you reviewed.

## Provenance

Every export comes with a `provenance.json`. It records, for each result:

- the provider and the exact model;
- the full prompt and the settings;
- when it was run;
- whether the text was corrected and reviewed by a person;
- how the source is described in its project (repository, shelfmark, citation).

Keep it with the exported files. It is what lets you, or a reviewer, say exactly how a text was
produced, and it is what makes a result reproducible as far as these models allow.

## Citing your method

For a methods section, a **pipeline** can be exported with its model, settings and full
prompts. A sentence along these lines is then enough, with the files in an appendix or a
repository:

> The pages were transcribed with *model* (*provider*, *month and year*) using Kalamu
> *version*, with the prompt given in the appendix. *N* pages of *M* were checked against the
> scans and corrected by the author; the uncorrected and corrected texts are both preserved.

For a large collection reviewed by sample, export the project's review list as well: it
records which pages were drawn and how many were checked. See
[What to check](projects.md#what-to-check).

## Citing Kalamu

Kalamu is open source under the MIT licence. Cite the version you used and the repository:
[github.com/fmadore/Kalamu](https://github.com/fmadore/Kalamu).
