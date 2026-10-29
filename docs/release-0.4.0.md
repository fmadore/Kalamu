Two new tasks for researchers who work with long documents and with text that libraries have
already machine-read. **This is still an early pre-release**: expect rough edges, and check
results against your sources.

## What's new

- **Detailed table of contents** of a long document scanned as one file: a newspaper or
  magazine issue, an archival file, a book or report. One model reads each page for the
  articles or items on it; another, which can be a cheaper model from any provider (Gemini Pro
  reading, Mistral combining, say), puts the pages together into one entry per article or item,
  with every page it is on and the fields you choose. Ready-made kinds of document:
  - a newspaper or magazine issue: section, title, authors, pages and a summary of each article;
  - an archival file, item by item: kind of document, date, sender, addressee, summary and
    names, as an archivist's inventory gives them;
  - a book, report or thesis: parts, chapters and sections with a summary of each.

  The review shows the scan beside the entries: click an entry to see its first page, and
  correct, add or remove entries. Results come as Markdown, Word, a spreadsheet with one row per
  entry, text and JSON.
- **Correct existing text**: the noisy text layer of a PDF, or a text or Word file downloaded
  from a digital library (Gallica, Europeana, Internet Archive…), corrected page by page with
  its spelling of the time kept. The review highlights every change against the original, and
  the original reading can be taken back in one click. A newspaper preset follows the
  OCR-correction prompt of iwac-ai-pipelines.
- **How it works**: under the tasks in New job, a short animated diagram shows how the chosen
  task splits and joins the work, drawn with your own files once they are added.
- **Jobs**: open the files a job wrote from its page, keep that list after a restart, and
  rename jobs.
- **DeepSeek V4.1 Flash**, hosted by Mistral, for analyses, in economy mode too.
- **Recordings play at once in the review**, straight from the file, instead of being converted
  first.

## Download

- `Kalamu_0.4.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_0.4.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).

Both bundle FFmpeg and offline Whisper; nothing else needs to be installed. Installing over
0.3.0 keeps your jobs, settings, presets and keys.

## Known limitations

- **Unsigned installers**: Windows SmartScreen will warn. Choose *More info*, then *Run anyway*.
- Tables of contents and corrections have not yet run against the live services. Try the first
  pages as a sample before a whole issue or file.
- A table of contents reads PDFs and multi-page images: photographs of one archival file should
  first be put together into a single PDF.
- Gemini, offline Whisper and economy mode have not yet run against the live services; OpenAI,
  Claude and Mistral have. Try a small job first.
- English interface only; French and German will follow.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). Offline
transcription uses whisper.cpp (MIT). The fonts are under the SIL Open Font License.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
