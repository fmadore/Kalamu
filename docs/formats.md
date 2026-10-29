# Output formats

Every result is written from the job database, with the user's corrections applied, into the
formats chosen for the job. Each source also gets a `provenance.json`. In comparison mode, file
names carry the set-up's label: `letter.gemini-3.8-flash.txt`. Two sources that would share a
name in one folder (`scan.pdf` and `scan.tif`, or two `scan.pdf` sent to one folder) keep apart:
the later ones in the job are numbered, `scan (2).txt`, regardless of case.

A page or segment that could not be read after its retries does not keep the rest from disk: it
stands as `[Not read: reason]` in the text formats and the CSV, with a `failed` reason in JSON,
and `provenance.json` lists it under `failed`. Retrying it writes the files again.

## Text, Markdown and Word

- **`.txt`**: the text, with `--- Page N ---` before each page when there is more than one (as
  in zmo-ai-pipelines). Transcriptions are written as `[hh:mm:ss] Speaker: text` turns when the
  model gave timestamps and speakers.
- **`.md`**: a heading with the file name and the model, then `## Page N` per page.
- **`.docx`**: the same structure as Markdown; line breaks inside paragraphs are kept, since
  diplomatic transcriptions rely on them.

## Searchable PDF (`.searchable.pdf`)

The original PDF, unchanged, with an invisible text layer so it can be searched and its text
copied. Only models that report where text sits on the page can feed it; today that is Mistral
OCR, whose paragraph boxes are kept with each page. Each paragraph's words are set in invisible
text (render mode 3) over its box, spread over the lines that fit and stretched to its width,
in a glyph-less Unicode font, so any script stays searchable. A corrected page is set as one
block over the page. Only PDF sources get one; it is written as `<name>.searchable.pdf`, never
replacing the source.

## Analyses

Written as `<name>.analysis.txt` (and `.md`, `.docx`, `.json`, `.csv`) beside the source, so an
analysis of `interview.txt` never replaces it. The researcher's corrected answers replace the
model's where there are any.

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

## Tables of contents

Written as `<name>.contents.md` (and `.docx`, `.csv`, `.txt`, `.json`) beside the source. Only
the combined entries are written, with the researcher's corrections; what the first step read
on each page stays in the job, for the review.

- **Markdown and Word**: a heading per entry, then its pages and fields as a list
  (`- **Pages:** 1, 3-4`).
- **Text**: the entries numbered, each followed by its pages and fields, indented.
- **CSV**: one row per entry: `file`, `Title`, `Pages`, one column per field named as the
  preset names it, then `corrected` and `reviewed`.
- **JSON**: the part has an `entries` array, each entry an object of its fields' values by key.
- **`provenance.json`** describes the first step as for any job and the second under `merge`:
  its `provider`, `model`, `model_versions`, `request` and `prompt` (with its fields).

## Corrected text

Written as `<name>.corrected.txt` (and `.md`, `.docx`, `.json`, `.csv`), so a text file is
never replaced by its correction. Pages keep their `--- Page N ---` markers; passages of a text
without pages carry none, as the cut means nothing to a reader, and are numbered `§1`, `§2` in
the CSV.

## Subtitles and spreadsheets

- **`.srt` / `.vtt`**: cues of at most 84 characters and 6 seconds, from timed segments or from
  `[hh:mm:ss]` markers in the text. WebVTT uses voice tags (`<v Speaker 1>`) for speakers.
- **`.csv`**: one row per page (`page, text, corrected, reviewed`) or per segment
  (`start, end, speaker, text`), UTF-8 with a byte-order mark so Excel reads accents correctly.

## JSON (`kalamu-result`, version 1)

```json
{
  "schema": "kalamu-result",
  "schema_version": 1,
  "task": "text_extraction",
  "source": { "file_name": "lettre.pdf", "sha256": "…", "size_bytes": 1024 },
  "setup": { "label": "gemini-3.8-flash", "provider": "gemini", "model": "gemini-3.8-flash", "prompt": "Printed document, diplomatic" },
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
  page was split, or `{ "kind": "segment", "index": i, "start_ms": …, "end_ms": … }`.
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
| `task`, `engine` | `text_extraction`, `audio_transcription`, `analysis`, `contents` or `text_correction`; `cloud` or `offline` |
| `source` | File name, SHA-256 and size of the original; for a YouTube video, its title with the id (`<title> [<id>]`, or `youtube-<id>` when the title could not be read), the SHA-256 of its id, its `url`, `title` and `read_at`, when the provider first read it |
| `provider`, `model` | As requested |
| `model_versions` | The versions the provider reported, when they differ from `model` (including a fallback model) |
| `request` | What the requests carried besides the prompt, each as `{ "name", "value" }` named by its path in the request (`output_config.effort`, `generationConfig.maxOutputTokens`, `beam_size`): reasoning effort, output limit, media resolution. Recorded with each piece's result when it was sent, so a later version of Kalamu never rewrites it; a name listed twice means pieces were read with different values. Absent for results stored by 0.4.0 and earlier |
| `prompt` | Preset id and name, the full instructions and per-page request sent, and their SHA-256 |
| `merge` | For a table of contents, the model and prompt that combined the pages: `provider`, `model`, `model_versions`, `request`, `prompt` |
| `languages`, `detected_languages` | What the user chose, and what the model reported |
| `options` | Diarization, timestamps, speaker count |
| `pieces`, `corrected`, `reviewed` | Number of pages or segments, and which were corrected or reviewed |
| `failed` | Pages or segments that could not be read after their retries, marked in the files. Absent when none was |
| `truncated` | Pages or segments whose answer stopped at the model's output limit, so their end may be missing; for a table of contents, the pages read and the combined entries alike. Absent when none was |
| `preprocessing` | Fixes applied to working copies (`split_spreads`, `even_out_volume`, `sample_first_3_pages`) |
| `usage`, `cost_usd` | Tokens, audio seconds or pages billed, the input read from or written to the provider's prompt cache (`cached_input_tokens`, `cache_write_tokens`, parts of `input_tokens`), whether the calls went through a batch service (`batch`), and the cost from catalogue prices with those discounts |
