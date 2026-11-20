---
layout: guide
title: Exporting and citing
order: 8
lede: Take your checked results where you need them, with a record of how they were made that you can cite.
description: Export formats and provenance in Kalamu.
---

[Français](fr/export.md)

## Formats

From a job's review, **Export…** offers the formats that suit its kind of result.

| Format | Good for |
| --- | --- |
| Plain text, Markdown | Reading, quoting, any other tool. |
| Word | Sharing with colleagues, editing further. |
| JSON, CSV | Analysis in a spreadsheet, R or Python; analyses export one row per document, or one per entry for a register. |
| Subtitles | Transcripts with their timestamps, to lay over a video. |
| Searchable PDF | The original scan with the text behind it, so it can be searched and copied. |
| ALTO XML, PAGE XML | Each word with its place on the page, for digital libraries and editions. Your corrections are included. |

Exports always use the text **as you corrected it**. The [output formats](../formats.md) page
describes every file field by field.

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
