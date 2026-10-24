# Output formats

Every result is written from the job database, with the user's corrections applied, into the
formats chosen for the job. Each source also gets a `provenance.json`. In comparison mode, file
names carry the set-up's label: `letter.gemini-3.8-flash.txt`.

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

## Summaries

Written as `<name>.summary.txt` (and `.md`, `.docx`, `.json`, `.csv`) beside the source, so a
summary of `interview.txt` never replaces it. The text formats end with a `Keywords:` line;
JSON adds a `keywords` array to the part; CSV has one row per file
(`file, summary, keywords, corrected, reviewed`).

## Subtitles and spreadsheets

- **`.srt` / `.vtt`**: cues of at most 84 characters and 6 seconds, from timed segments or from
  `[hh:mm:ss]` markers in the text. WebVTT uses voice tags (`<v Speaker 1>`) for speakers.
- **`.csv`**: one row per page (`page, text, corrected, reviewed`) or per segment
  (`start, end, speaker, text`), UTF-8 with a byte-order mark so Excel reads accents correctly.

## JSON (`scriptorium-result`, version 1)

```json
{
  "schema": "scriptorium-result",
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
| `generator` | `{ "name": "Scriptorium", "version": "…" }` |
| `created_at` | Milliseconds since 1970, UTC |
| `task`, `engine` | `text_extraction` or `audio_transcription`; `cloud` or `offline` |
| `source` | File name, SHA-256 and size of the original |
| `provider`, `model` | As requested |
| `model_versions` | The versions the provider reported, when they differ from `model` (including a fallback model) |
| `prompt` | Preset id and name, the full instructions and per-page request sent, and their SHA-256 |
| `languages`, `detected_languages` | What the user chose, and what the model reported |
| `options` | Diarization, timestamps, speaker count |
| `pieces`, `corrected`, `reviewed` | Number of pages or segments, and which were corrected or reviewed |
| `preprocessing` | Fixes applied to working copies (`split_spreads`, `even_out_volume`, `sample_first_3_pages`) |
| `usage`, `cost_usd` | Tokens, audio seconds or pages billed, and the cost from catalogue prices |
