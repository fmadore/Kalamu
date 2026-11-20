# Output formats

Every result is written from the job database, with the user's corrections applied, into the
formats chosen for the job. Each source also gets a `provenance.json`. In comparison mode, file
names carry the set-up's label: `letter.gemini-3.8-flash.txt`. Two sources that would share a
name in one folder (`scan.pdf` and `scan.tif`, or two `scan.pdf` sent to one folder) keep apart:
the later ones in the job are numbered, `scan (2).txt`, regardless of case.

Images read as the pages of one document ([#52](https://github.com/fmadore/Kalamu/issues/52))
give one result, as a PDF does: one `.txt`, `.md`, `.docx`, `.json`, `.xlsx` and `.csv`, its pages
numbered in the document's order (`--- Page 3 ---`), named after the document and written in the
folder of its first page (`Box 3.txt`, `Box 3.provenance.json`). The images themselves are never
copied, moved or changed.

A page or segment that could not be read after its retries does not keep the rest from disk: it
stands as `[Not read: reason]` in the text formats and the CSV, with a `failed` reason in JSON,
and `provenance.json` lists it under `failed`. Retrying it writes the files again.

In the CSV files a cell that begins with `=`, `+`, `-`, `@`, a tab or a carriage return, and is
not simply a number, is written with an apostrophe before it (`'- Oui`), so a spreadsheet shows
it as text instead of computing it as a formula. The JSON beside it keeps the text exactly. A
table written through an export profile is the exception: its cells are written as they are,
since another tool's import would take the apostrophe as part of the value
([decision](decisions/2026-11-10-fourth-round.md)).

Every table written as CSV is also offered as an Excel workbook (`.xlsx`) with the same
headings, rows and cells, for a spreadsheet: it opens by double-click whatever language the
spreadsheet is set to, where a CSV's commas may all land in one column. It has one sheet named
`Table`, the heading row in bold and frozen, columns as wide as their longest line (8 to 60
characters) and long text wrapped. Every cell is text exactly as written, so nothing is guarded
with an apostrophe and nothing is read as a number, a date or a formula; Excel is told not to
flag numbers held as text. A cell longer than the 32,767 characters a workbook's cell holds is
cut and ends with a note saying so; the CSV and JSON keep the whole text. The workbook's dates
are fixed, so the same table gives the same file
([decision](decisions/2026-11-20-xlsx-tables-corrected-per-row.md)). The workbooks written beside
a JSON export (descriptions, export profiles, the review manifest, the dictionary, a table of
contents' items) are named as their CSV, with `.xlsx`.

## Text, Markdown and Word

- **`.txt`**: the text, with `--- Page N ---` before each page when there is more than one (as
  in zmo-ai-pipelines). Transcriptions are written as `[hh:mm:ss] Speaker: text` turns when the
  model gave timestamps and speakers.
- **`.md`**: a heading with the file name and the model, then `## Page N` per page.
- **`.docx`**: the same structure as Markdown; line breaks inside paragraphs are kept, since
  diplomatic transcriptions rely on them.

## Searchable PDF (`.searchable.pdf`)

The original PDF, unchanged, with an invisible text layer so it can be searched and its text
copied. Only models that report where text sits on the page can feed it: Tesseract, which
places every line and word, and Mistral OCR and LightOnOCR-3, whose block boxes are kept with
each page.
Text is set invisibly (render mode 3) in a glyph-less Unicode font, so any script stays
searchable: each of Tesseract's lines over its own box, and each of Mistral's or LightOnOCR's blocks spread
over the lines that fit its box; every line is stretched to its box's width. A corrected page
keeps its words' places where the engine placed each word (see ALTO below); otherwise it is set
as one block over the page. Only PDF sources get one, not images, a document of images included; it is written as `<name>.searchable.pdf`,
never replacing the source. New job says before a job whether it will have one: the format is
offered, and the task's diagram draws ALTO XML and the searchable PDF as a second result, only when a
chosen model reports where text sits; under any other model, the diagram notes that it gives text only.

## ALTO XML (`.alto.xml`)

[ALTO](https://www.loc.gov/standards/alto/) version 4, as digital libraries, eScriptorium and
Transkribus read it: one file per document, a `Page` per page, then `TextBlock`, `TextLine` and
`String` for each word, with `SP` between words. Positions are in pixels (`MeasurementUnit`
`pixel`): those of the image for an image source, those of the page as it was drawn for a PDF
(as Tesseract reads it: at 300 dpi, or less for a page whose longest side would pass 7,000
pixels, the resolution its result records; a result that records none, read by 1.7.0 or
earlier, is measured at 300), the page's own `WIDTH` and `HEIGHT` given; for a document of images,
each page measured in its own image, the file named after the document. A word carries the engine's
confidence (`WC`, 0 to 1) where it gave one. `Description` names the source file, the date,
Kalamu's version and, in `processingStepSettings`, the provider and model, the resolution PDF
pages are measured at (naming the pages drawn at less than 300 dpi), and says when positions
are estimated. A page split into halves is put back together on the whole page.

Offered only from models that report where text sits. Tesseract places every word. Mistral
OCR and LightOnOCR-3 place blocks only: a paragraph the model wrote on one line is wrapped into
as many lines as its block holds on the page (taking characters about half as wide as they are
tall, and lines 1.2 times their height apart), a line the model broke itself is kept, the lines
share the block's height and their words each line's width by their length, as the review draws
them; the file says these positions are estimated. A corrected page keeps its
words' places, as iwac-ai-pipelines' ALTO correction does: the researcher's text is matched to
the words the engine read, a word kept or changed staying in its box, words added sharing the
box before them, words removed leaving theirs; the engine's confidence is kept only for words
left as it read them.

## PAGE XML (`.pNNNN.page.xml`)

[PAGE](https://github.com/PRImA-Research-Lab/PAGE-XML) content, the 2019-07-15 schema, as
Transkribus, eScriptorium and Aletheia read it: one file per page, `<name>.p0001.page.xml`,
`<name>.p0002.page.xml` and on (four digits at least), beside the ALTO file and from the same
places. Offered when ALTO is, and the same pages are written: every page of a PDF, one read
without text and one that failed included (an empty `Page`, the failure in `Comments`), and
each page of images that has regions.

`Page` gives `imageWidth` and `imageHeight` in the pixels ALTO measures in and, for a PDF
page, the resolution it was drawn at (`imageXResolution`, `imageYResolution`, in PPI).
`imageFilename` is the image file's name; a PDF page has no image file, so it names
`<file>.pdf#page=N`. `ReadingOrder` lists the regions in the engine's order. Each block is a
`TextRegion`, its lines `TextLine`, and its words `Word` only where the engine placed each word
(Tesseract); `Coords` are rectangles as four points, in whole pixels kept within the page.
`TextEquiv/Unicode` holds the corrected text at every level, a corrected page keeping its words'
places as in ALTO; a word carries the engine's confidence (`conf`, 0 to 1) where it gave one.
No baselines and no region types are written, the engines giving none.

A block Mistral OCR or LightOnOCR-3 placed whole gets the lines ALTO estimates, each saying its
position is estimated, and no words: a word's place would be a guess within a guess. ALTO
writes those estimated words; this is the one difference between the two.

`Metadata` names Kalamu's version, the provider and model, and in `Comments` the resolution,
whether lines are estimated and whether the text was corrected. `Created` is when the page was
read and `LastChange` when it was last corrected, never when the file was written, so writing
the files again gives the same bytes.

### The researcher's regions in the searchable PDF, ALTO and PAGE XML

These three write the corrected view of where the text sits: the model's regions with the
boxes the researcher gave them in the review, and the regions the researcher drew that have
text. A file with none of either is byte for byte what it was.

- **Whose box.** A box the researcher gave a block, line or word of this set-up's reading
  takes the model's place. A box given to a reading no set-up shows any more is written as a
  region of the researcher's own; one given to another set-up's reading is left out, as in
  `provenance.json`, which lists the same regions.
- **Widened, never scaled.** A block moved while its lines were not keeps the lines where the
  model measured them and is widened just enough to hold them; a line or word moved outside
  what holds it widens that line, then that block. Each says which it is: the researcher's
  box, theirs widened, or the model's widened.
- **A corrected page.** The boxes are laid first, then the corrected words on them: a word
  given a box passes it to the word or words that replace it. Where the corrected text cannot
  be laid back on the model's words, the page is one block as before, and the researcher's
  boxes are written as regions of their own, without text.
- **A region the researcher drew** follows the model's on its page, in the order drawn. With
  text it is a block whose lines are estimated within its box and say so; a split spread's
  half is put back on the whole page. Without text, PAGE XML writes a `TextRegion` with its
  `Coords` alone, and ALTO and the searchable PDF leave it out.
- **ALTO** adds a second `Processing` step, `processing_2` (`contentModification`, dated by
  the last box), and every `TextBlock`, `TextLine` and `String` whose box is or involves the
  researcher's refers to it with `PROCESSINGREFS`. The researcher's own blocks have the id
  `researcher_<id>`.
- **PAGE XML** says it in each element's `comments`, where an estimated position is already
  said, naming the region's id as `provenance.json` lists it; in a
  `MetadataItem type="processingStep"`; and in `LastChange`, which counts the last box. The
  researcher's own regions have the id `researcher_<id>` and come last in `ReadingOrder`.
  `custom` is not used for this, since Transkribus reads tags there.
- **The searchable PDF** places its text by the corrected boxes and sets a drawn region's
  text invisibly over its box, so a marginal note is found by a search. A PDF does not say
  whose a box is.

Both XML formats were checked by hand against the official schemas (ALTO 4.4, PAGE
2019-07-15); no test does it. Not yet in the files: the reading's own identifier, which
`provenance.json` gives for each region.

## Analyses

Written as `<name>.analysis.txt` (and `.md`, `.docx`, `.json`, `.xlsx`, `.csv`) beside the
source, so an analysis of `interview.txt` never replaces it. The researcher's corrected answers
replace the model's where there are any.

- **Text formats** set out each field under its name: prose fields as a paragraph after their
  label, the others on one line (`Places: Bamako; Ségou`), and a field's reason on the next line
  (`Reason: …`).
- **JSON** adds to the part a `fields` object with each answer under its field's key (text for
  prose and single choices, an array for lists and several choices) and, when any field asked
  for one, a `reasons` object with the reasons by key. `provenance.json` records the fields
  asked for (`prompt.schema`).
- **CSV** has one row per document: `file`, then one column per field named as the researcher
  named it, a `<field> (reason)` column after each field that asks for a reason, then
  `corrected` and `reviewed`. List items are joined with semicolons.

### An analysis that quotes its text

An analysis whose fields include entries that quote the text (the default *Interview: themes
and quotations*) keeps, for each quotation, where Kalamu found it in the text it read. Only
quotations found word for word are written; the researcher's removals are applied.

- **Markdown** is a memo: each field under a `##` heading; an entry with several fields has
  its first as a `###` heading and the others below, then its quotations as block quotes that
  end with `> — Speaker, [hh:mm:ss]` (the turn's speaker and the time the turn starts, when the
  text has them), or `> — page 3` for a passage of a text read from pages. A field from which quotations were left out ends with a line in italics
  saying how many. Text and Word write each quotation on a line, `"…" (Speaker, hh:mm:ss)`.
- **JSON** gives such a field as `{ "entries": [{ "answers": [{ "key", "value" }], "evidence":
  [{ "anchor": { "part", "start", "end" }, "quote", "turn", "speaker", "start_ms", "end_ms",
  "page" }] }], "left_out", "dropped" }`, `page` (the piece, as `{ "kind": "page", "number" }`)
  only for a passage of a text read from pages. `start` and `end` count characters (Unicode scalar values, not
  bytes or UTF-16 units) in the text read, whose identifier and SHA-256 are in a top-level
  `"snapshot": { "id", "sha256" }`, written only for these analyses; `part` is the piece of
  that text, from 0; `turn` counts from 1; `quote` is the text's own characters, never the
  model's copy. `left_out` counts the quotations a model gave that were found nowhere,
  `dropped` the entries left with none.
- **CSV**: the analysis's own file keeps one row per document, the field's column holding its
  entries' names joined with semicolons. Beside it, `<name>.analysis.quotations.csv` has one
  row per quotation: `file`, `field`, `entry` (the entry's first field), `speaker`, `start`,
  `end` (`hh:mm:ss`, or empty), `quotation`, `turn`, `page` (empty but for a text read from
  pages). An entry that is quotations alone (a screened document's passages) is named in the
  analysis's own file by its quotations, each between quotation marks. Cells that a spreadsheet would read as a
  formula are guarded, as in every CSV Kalamu writes. The workbook does the same, as
  `<name>.analysis.xlsx` with `<name>.analysis.quotations.xlsx` beside it.
- **`provenance.json`** adds `cited`: for each such field, the quotations checked, those left
  out and the entries left out; `source.derived_from` names the text read.
- **A long text read in parts** writes the answer that brought the parts together, and only
  it; a part that could not be read stands after it as `[Not read: part 3 (turns 48–71,
  01:02:10 to 01:20:00): reason]` in text, Markdown and Word, and in JSON as a part whose piece
  is `{ "kind": "turns", "number", "first", "last" }` with its `failed` reason.
  `provenance.json` then has `merge`, the step that brought the parts together, whose `request`
  lists `turns_read` and, when a part could not be read, `turns_not_read` (`"1-24, 23-49"`,
  turns counted from 1); `pieces` counts the parts and that step. A text read whole has no
  `merge`.

### Records

An analysis whose fields are one repeating group (*One row per entry*) is read page by page,
and its files hold one row per entry, from the corrected rows.

- **CSV**: `file`, `page`, `position` (the entry's place on its page, from 1), then one column
  per column of the group, named as the researcher named it, with a `<column> (reason)` after
  each that asks for a reason, then `corrected` (the row's own: true when the researcher
  typed into the row or added it), `reviewed` (the row's page's) and `not_read`. A page that
  failed or was cut off is one row with no position and `[Not read: reason]` under `not_read`,
  so its entries are never taken for none. Cells that begin with `=`, `+`, `-` or `@` are
  written as in the other tables.
- **JSON**: the usual envelope, its parts without `fields`, and a top-level `rows` list: for
  each entry `file`, `page`, `position`, a `fields` object with each cell under its column's
  key (text, or an array for a list and several choices), `corrected`, `reviewed` (as in the
  CSV) and, where any column gave one, `reasons`. A page not read has no rows; its part says
  `failed`.
- **Which rows are corrected.** A corrected page's rows are lined up with the model's rows by
  their cells, in order (the longest run of rows the same in both); a row outside that run was
  typed into or added, and is `corrected`. Two rows are the same when every column holds the
  same value, a missing cell counting as empty and reasons not counting. So a row removed above
  marks none below it, a cell typed back as the model wrote it leaves its row the model's, and
  a removed row leaves no row to mark. The workbook has the same columns as the CSV.
- **Text formats** (`.txt`, `.md`, `.docx`) are not yet made for rows: each page shows only
  its rows' first column, joined with semicolons. Use the CSV or the JSON.

## Tables of contents

Written as `<name>.contents.md` (and `.docx`, `.xlsx`, `.csv`, `.txt`, `.json`) beside the
source. Only the combined entries are written, with the researcher's corrections; what the first
step read on each page stays in the job, for the review.

- **Markdown and Word**: a heading per entry, then its pages and fields as a list
  (`- **Pages:** 1, 3-4`).
- **Text**: the entries numbered, each followed by its pages and fields, indented.
- **CSV**: one row per entry: `file`, `Title`, `Pages`, one column per field named as the
  preset names it, then `corrected` and `reviewed`.
- **JSON**: the part has an `entries` array, each entry an object of its fields' values by key.
- **`provenance.json`** describes the first step as for any job and the second under `merge`:
  its `provider`, `model`, `model_versions`, `request` and `prompt` (with its fields), and its
  share of the `usage` and `cost_usd` of both steps.

### Items

The items made from a table of contents (#50) are exported, when the researcher selects some
and chooses *Export*, into a folder they choose, never replacing a file already there (a new
one is numbered, `(2)`):

- **One PDF per item**: its pages cut from the source in the item's order, named after the
  source, the item's place and its title (`an-nasr-12 03 Fatwa.pdf`).
- **`<source> units.csv`**: one row per item: `id`, `number`, `title`, `pages` (as typed,
  `1, 3-5, 8 (left)`), `file`, one column per entry field named as the preset names it, then
  `original` and `original_sha256`, the file the pages were cut from; and the same table as
  **`<source> units.xlsx`**.
- **`<source> units.json`**: `{"units": [...]}`, each with its `id` (stable across exports),
  `number`, `title`, `pages` (each `{"kind": "page", "number": 3}`, with `"half"` for half of a
  spread, numbered as the file's pages, not as printed), `pages_text`, `file`, `fields` by key,
  the `entry` it was made from (numbered from 1, or `null`), its `revision` and the `original`'s
  `file_name` and `sha256`.

A job that read an item's pages writes its results beside the source under the item's name,
and its `provenance.json` names the item under `source.derived_from.unit`.

## Corrected text

Written as `<name>.corrected.txt` (and `.md`, `.docx`, `.json`, `.xlsx`, `.csv`), so a text file is
never replaced by its correction. Pages keep their `--- Page N ---` markers; passages of a text
without pages carry none, as the cut means nothing to a reader, and are numbered `§1`, `§2` in
the CSV.

## Translated text

Written as `<name>.translated.<code>.txt` (and `.md`, `.docx`, `.json`, `.xlsx`, `.csv`),
`<code>` being the language translated into (`gallica.translated.en.txt`), so a text is never
replaced by its translation and translations into two languages do not replace each other.
Pages and passages are kept as for corrected text, and the CSV has the same columns.

A transcript translated with its times (frozen from a transcript whose every part had them) is
also written as `.srt` and `.vtt`, cut and timed exactly as a transcript's, each cue holding the
translation of the line it times, under that line's speaker. Each part of its JSON result has
`segments`: the transcript's lines with their times and speakers, the text translated. Its
`provenance.json` may carry `lines_changed`: the passages whose correction no longer has one
paragraph per line, whose times were therefore lost; no subtitles are written while it lists
any. A translation of any other text has no subtitles.

## Subtitles and spreadsheets

- **`.srt` / `.vtt`**: cues of at most 84 characters and 6 seconds, from timed segments or from
  `[hh:mm:ss]` markers in the text. WebVTT uses voice tags (`<v Speaker 1>`) for speakers.
- **`.csv`**: one row per page (`page, text, corrected, reviewed`) or per segment
  (`start, end, speaker, text`), UTF-8 with a byte-order mark so Excel reads accents correctly.
- **`.xlsx`**: the same rows as the CSV, in an Excel workbook (see above).

## JSON (`kalamu-result`, version 1)

```json
{
  "schema": "kalamu-result",
  "schema_version": 1,
  "task": "text_extraction",
  "source": { "file_name": "lettre.pdf", "sha256": "…", "size_bytes": 1024 },
  "setup": { "label": "gemini-3.8-flash", "provider": "gemini", "model": "gemini-3.8-flash", "prompt": "Printed or typed document" },
  "parts": [
    {
      "piece": { "kind": "page", "number": 1 },
      "label": "1",
      "text": "Bamako, le 3 mai 1957 …",
      "segments": [],
      "corrected": false,
      "reviewed": true
    }
  ],
  "provenance": { "…": "as below" }
}
```

- `piece` is `{ "kind": "page", "number": n }`, with `"half": "left" | "right"` when a double
  page was split, or `{ "kind": "segment", "index": i, "start_ms": …, "end_ms": … }`; a long
  text's part not read is `{ "kind": "turns", "number": n, "first": f, "last": l }`.
- `segments` are `{ "start_ms", "end_ms", "speaker", "text" }`, timed from the start of the
  recording.
- `text` is the corrected text when `corrected` is true; the model's own output stays in the
  job database.

## `provenance.json` (schema version 1)

Modelled on zmo-ai-pipelines' provenance record:

| Field | Meaning |
| --- | --- |
| `generator` | `{ "name": "Kalamu", "version": "…" }` |
| `created_at` | Milliseconds since 1970, UTC |
| `task`, `engine` | `text_extraction`, `audio_transcription`, `analysis`, `contents`, `text_correction` or `translation`; `cloud` or `offline` |
| `source` | File name, SHA-256 and size of the original; for a YouTube video, its title with the id (`<title> [<id>]`, or `youtube-<id>` when the title could not be read), the SHA-256 of its id, its `url`, `title` and `read_at`, when the provider first read it; for text frozen from an earlier result (*Use this text in a new job*, or a workflow step reading an earlier one), the frozen text's name, SHA-256 and size, and `derived_from`: that result's `job_id`, `setup_id`, `source_id` and `task`, the `original_path`, when it was frozen (`frozen_at`), how many pieces it holds and how many were `reviewed` and `corrected`, and the pieces `missing` from it; for an item's pages cut from a table of contents' source, the same with `task` `contents`, its pages as the pieces (`reviewed` counting those marked in the table of contents), and `unit`: the item's `id`, `title`, `revision` and `pages` as the original numbers them; for a document of images, the document's name, the SHA-256 of its pages' hashes in their order and their total size, and `pages`: each file in the order read, with `page` (the document's page it begins), its `id` (the same wherever the page is moved), `file_name`, `path`, `sha256`, `size_bytes`, `modified_ms`, `frames` (pages the file holds, more than one for a multi-page TIFF) and the researcher's `label`, if any |
| `documents` | The project documents the source is, in each project whose document it is: `project_id`, `project_name`, `document_id`, `title`, and `description`, each part of the source's description as it stood when the file was written (`repository`, `fonds`, `series`, `file`, `shelfmark`, `creator`, `date`, `url`, `citation`, `rights`) as `{ "value", "from" }`, `from` being `document` or `project`; parts with no value are left out. Supplied by the researcher, never by a model. Absent for a job tied to no project |
| `provider`, `model` | As requested |
| `model_versions` | The versions the provider reported, when they differ from `model` (including a fallback model) |
| `request` | What the requests carried besides the prompt, each as `{ "name", "value" }` named by its path in the request (`output_config.effort`, `generationConfig.maxOutputTokens`, `beam_size`): reasoning effort, output limit, media resolution; for Whisper, `vad` (whether silences were skipped with the voice-activity model, used when it is downloaded) and `vad_model`; for Tesseract, `engine` (`Tesseract 5.5.2`), `language_data` (`tessdata_best@e12c65a`), `ocr_engine_mode` (`lstm`), `page_segmentation` (`auto`) and, for a PDF, `page_resolution` (`300 dpi`: what the page was drawn at). For pages read several at a time, `pages` names the pages that went in one call (`5-8`). Recorded with each piece's result when it was sent, so a later version of Kalamu never rewrites it; a name listed twice means pieces were read with different values. Absent for results stored by 0.4.0 and earlier |
| `prompt` | Preset id and name, the full instructions and per-page request sent, and their SHA-256 |
| `merge` | For a table of contents, the model and prompt that combined the pages: `provider`, `model`, `model_versions`, `request`, `prompt`, and that step's `usage` and `cost_usd`, its share of the totals below |
| `languages`, `detected_languages` | What the user chose, and what the model reported |
| `options` | Diarization, timestamps, speaker count, and the reasoning level chosen or suggested for the set-up (`reasoning`, absent for a model that does not reason) |
| `pieces`, `corrected`, `reviewed` | Number of pages or segments, and which were corrected or reviewed |
| `failed` | Pages or segments that could not be read after their retries, marked in the files. Absent when none was |
| `truncated` | Pages or segments whose answer stopped before its end, at the model's output limit or for another reason the provider gave, so their end may be missing; for a table of contents, the pages read and the combined entries alike. Absent when none was |
| `cut_by_filter` | Those of `truncated` that a provider's filter (safety, a blocked term) stopped, rather than the output limit. Absent when none was |
| `preprocessing` | Fixes applied to working copies (`split_spreads`, `even_out_volume`, `sample_first_3_pages`): only those made, so `split_spreads` only where a spread was found, and a sample only of a document longer than it |
| `renamed_speakers` | The model's speaker labels the researcher gave names, which replace them in every file. The names are in the files, not here, so the record can be shared without them. Absent when none was |
| `usage`, `cost_usd` | Tokens, audio seconds or pages billed, the input read from or written to the provider's prompt cache (`cached_input_tokens`, `cache_write_tokens`, parts of `input_tokens`), whether the calls went through a batch service (`batch`), and the cost from catalogue prices with those discounts, for the results kept |
| `refused` | Answers the provider billed but nothing was kept from: analysis answers that broke the fields and were asked for again, or several pages read together whose page marks could not be used (each page then read again alone). `answers`, their `usage` and `cost_usd`, not included above. Absent when none was |
| `workflow` | For a result read by a workflow step: the run (`run_id`), the workflow (`workflow_id`, `name`) and its `version` when the run started, this `step` of `steps` and its `step_name`, the step it `reads` (numbered from 1; absent when it read the files, and `source.derived_from` names the snapshot), and the researcher's `approval` of that step (`at`, and `partial` when they went on although some pieces were not read). Absent for a job run on its own |

Added on 18 November 2026 without a new version, both absent when empty: `listed_by_hand`,
the items of an analysis's list answers that the researcher wrote (added, or renamed from
the model's form) and the model's own answer did not hold, each with its `part`, `field` and
`items`; and `passages`, the passages a model corrected at the researcher's request from the
review and that they accepted, each with its piece, provider, model, prompt, request
settings, character counts, whether the proposal was edited, its cost and when it was asked
and accepted. Their cost is not in `usage` or `cost_usd`, which count the job's own pieces.

Added later the same day, also absent when empty: `regions`, the regions the researcher drew
over the scan or whose box they adjusted, each with its `id`, `page` (and `half` of a split
spread), its box in fractions of the page or half (`x`, `y`, `width`, `height`), its `origin`
(`manual`), what it `adjusts` (the stored result's cache key and the region's id there, such
as `r1_l2`), whether it `has_text` (the text itself is not recorded), and when it was created
and updated. A set-up's file lists those drawn on the source, those adjusting its own
readings and those adjusting a reading no set-up shows any more. ALTO, PAGE XML and the
searchable PDF write the same regions (above). A model's region is named there as PAGE XML
names it (`r1`, `r1_l2`, `r1_l2_w3`), except on a split spread, whose halves the review
numbers each on its own and PAGE XML across the whole page.

And `region_readings`, the regions a model read again at the researcher's request and whose
reading they used, each with its `page` (and `half`), the `region`'s id and box, the
provider, model, prompt and request settings, where it went (`went_to`: `region`, the text of
a region the researcher drew, or `text`, in place of the region's words in the page's text),
whether it was `edited` first, its cost and when it was asked and accepted; never the text.
A reading that went to a drawn region is listed in every set-up's file, one that went into
the text in that set-up's alone. Their cost is not in `usage` or `cost_usd`.

## The search index (`search.sqlite`)

Not an output: a file in the app's data folder that project search reads, made only from the
job database ([decision](decisions/2026-11-03-project-search.md)). Each passage is one piece of
one result: the text as the review shows it, or an analysis's answers and reasons as separate
parts, kept as JSON beside the folded text in an FTS5 table (trigram tokenizer, contentless).
A stamp records what the passage was made from, and `PRAGMA user_version` the layout; a file of
another version, or one that cannot be read, is emptied and made again. Nothing outside Kalamu
should rely on its layout.

## The search by meaning's numbers (`vectors.sqlite`)

Not an output either: a file in the app's data folder, made only from the results
([decision](decisions/2026-11-18-search-by-meaning-map-passage-correction-added-items.md)).
One row for each stretch of text: the model (provider, id and length), the piece it is of,
its number in that piece, the piece's stamp and a hash of its text, where the stretch stands
in the text, and its numbers as little-endian 32-bit floats of length 1. Losing the file
loses only what preparing again costs. The job database keeps, for each project, the model
chosen and the consent given (`meaning_spaces`) and each preparation with its cost
(`meaning_runs`). Nothing outside Kalamu should rely on its layout.

## The review manifest (`kalamu-review-manifest`, version 1)

Written by **Export the list** in a project's *What to check*, where the researcher chooses:
a JSON file with an Excel workbook and a CSV beside it, every item of the list in the scope
shown (the project, a collection or a selection), whatever the filters show, the pages drawn at
random included. The workbook has the same columns as the CSV.

The JSON holds `format`, `version`, `project`, `project_name`, `scope` (`project`,
`collection:<id>` or `documents:<digest of the selection>`), `exported_at`, the scope's
`counts`, the `sample` (its seed, size, when it was drawn, how many pages it was drawn from,
how many were drawn and how many of them are reviewed; `null` without one), `set_aside` and
`items`. `set_aside` lists the results the researcher set aside from the list in that scope,
which the items and the sample leave out: each with its `job_id`, `job_name`, `task`,
`setup_id`, `setup_label`, `model` and `documents`; an empty list when there are none. It was
added on 18 November 2026 without a new version, a reader of version 1 ignoring it.

The CSV has one row per item, in these columns: `document_id`, `document`, `source_id`,
`job_id`, `job_name`, `task`, `setup_id`, `setup_label`, `model`, `preferred`, `piece`,
`place`, `position`, `revision`, `answered_at`, `reasons`, `reason_details`, `flagged`,
`processing`, `sample`, `sample_seed`, `reviewed`, `corrected`, `decided_at`, `open`,
`kept_aside`, `kept_note`, `kept_at`, `listed_for`. `piece` is the piece as Kalamu names it
(`result` for a whole result), `place` as exports label it, `position` counted from 1, and
`revision` the answer the row is about. `reasons` joins the kinds with `; ` and
`reason_details` what each found. Yes-or-no columns read `yes` or `no`, `kept_aside` also
`for an earlier answer`; `listed_for` reads `processing`, `flagged`, `not_reviewed` or
`sample`. Times are in UTC.

A page is drawn by ranking every page nothing flags by the SHA-256 of `<seed>:<identity>`, the
identity being `<document id>/<task>/<piece>`, and taking the lowest: the manifest's seed and
identities are enough to check a draw.
