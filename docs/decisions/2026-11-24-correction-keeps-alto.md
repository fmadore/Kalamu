# 2026-11-24 · A model's correction of an extraction keeps its words' places

Proposed, not built: tracked in [#262](https://github.com/fmadore/Kalamu/issues/262).

## Context

Researchers who publish scans online need ALTO XML: each word with its text and its place on
the page, for a viewer's search and highlighting. Kalamu writes ALTO only for a text
extraction, and only from models that say where text sits: Tesseract word by word, and
Mistral OCR and LightOnOCR-3 by block, whose lines and words ALTO then estimates. Tesseract
places words well but often misreads them. A language model corrects them well but does not
place them.

Two steps already run one after the other: a workflow can extract text, then correct it
(*Correct existing text*). The result has clean text and no places. A correction is exported
"as text extraction, without the searchable PDF or ALTO XML" (specification, *Export
formats*). Four things are missing:

- **The correction never sees the places.** It reads the extraction's text frozen through
  `ProvenanceDerived` (job, set-up, source). The frozen copy holds the text, not the regions.
- **The places of a corrected page already have code that keeps them.** `engine/src/realign.rs`
  lays a corrected page back onto the boxes of the words read. A word kept keeps its box, a
  word changed takes the box of the word it replaces, and a word added shares the box before
  it. A page the correction rewrites beyond what the boxes can hold (`MOST_PER_BOX`) is placed
  whole and marked as estimated. Today that code serves only the researcher's own corrections
  in the review.
- **The correction model reads text alone.** It never sees the page, so it guesses where the
  OCR is unreadable. The reference script this follows, iwac-ai-pipelines'
  `02_correct_alto_xml.py`, also sends text only, line by line, with the tokens aligned.
- **The default correction preset reorders the text.** *Printed document or newspaper* puts
  columns back in reading order. That is right for reading, and it breaks the match between
  words and boxes.

## Decision

1. **A correction of a placed extraction keeps that extraction's places.** When the text a
   correction reads was frozen from an extraction that placed text, the frozen copy also keeps
   that extraction's regions, page by page. It keeps them as the researcher left them, their
   own boxes included, so a later change to the extraction's boxes does not change the
   correction's export, just as the text does not change. At export, each page's corrected
   text (the model's answer, with the researcher's corrections on top) is laid onto those
   regions by `realign`. A correction with places therefore offers ALTO XML, PAGE XML and the
   searchable PDF, as an extraction does. A page `realign` refuses is placed whole and marked
   as estimated, as a researcher's own rewritten page is now. Provenance names the extraction
   whose places were used. The frozen regions travel in a project file with the frozen text.
2. **The correction may see the page.** Its new option, *with the page* (label to be written
   in `en.ts` and `fr.ts`), sends each page's image beside its text. It is offered only when
   the text comes from a scan or an image and the model reads pictures. The image is sent at
   the size the extraction read it. The estimate counts the image's tokens. Before Start, the
   screen says that the page is sent as well as its text. The model uses the image only to
   decide what the OCR misread, never to read the page again.
3. **A preset made for placed text: line for line.** A new correction preset (*OCR correction
   keeping the layout*, name to be settled) fixes misread characters and words only. It keeps
   every line in its place and order, does not join or split lines, moves no column, adds no
   heading or note, keeps end-of-line hyphens as they are on the page, and leaves an
   unreadable word as read. The text goes out as numbered lines. An answer is taken only when
   every line comes back once, the check a translated transcript's lines already pass, and is
   asked for again otherwise. `realign` then matches line by line before word by word, as the
   reference script does, so a wrong line cannot shift the words of the next one.
4. **A ready-made workflow.** Workflows' examples gain *Read and correct, keeping positions*.
   Its first step is Tesseract, free and on this computer. Its second is a correction with the
   page, on a cloud model that reads pictures (Mistral, Gemini or Claude, as the researcher
   chooses). The New job default stays as it is: the workflow is reached in Workflows and in
   *Then*.
5. **The tip says when not to use it.** The workflow's description and the guide say plainly
   that the correction can only fix an OCR that is already readable. On a page where Tesseract
   reads little (degraded type, columns, tables, handwriting), the model reads the page rather
   than correcting it, the words no longer fit Tesseract's boxes, and the page falls back to
   estimated places. There, extract with LightOnOCR-3 on this computer or Mistral OCR in the
   cloud, whose blocks are placed but whose lines and words ALTO estimates. If Tesseract's word
   confidences are kept, the step can name the pages read too poorly before anything is sent.

### Local models

- **Tesseract**: words placed, measured. The best source of ALTO.
- **LightOnOCR-3**, asked for its `grounding` mode: blocks placed. ALTO is written with
  estimated lines and words, and its correction keeps those blocks.
- **GLM-OCR**: text only. No ALTO, before or after a correction.

## Not chosen

- **One task that reads and corrects in one call.** Keeping two steps lets the extraction run
  free and offline and the correction run in the cloud. Each step keeps its own provenance and
  cost, and the review shows each change the model made against the OCR, with the original one
  click away.
- **Laying a vision model's text onto Tesseract's boxes** (reading with a model, placing with
  Tesseract). It uses the same `realign`, but two independent readings of a page diverge far
  more than a correction does, so most pages would fall back to estimated places. It can be
  tried once this ships.
- **Importing ALTO from elsewhere** (a digital library's) stays under *Later* in the roadmap.
  This work makes it simpler: such a file would be one more source of placed text for a
  correction.

## Consequences

- When built: the specification's export table gives *Correction* the extraction's formats
  when its text was placed. [formats.md](../formats.md), the guide in English and French
  (`export.md`, `review.md`), and [privacy.md](../privacy.md) change, the last because a
  correction can now send page images, so it needs a new effective date.
- The frozen snapshot gains regions, so the project file's format and its checks must carry
  them.
- A cloud correction with the page costs more than one without it, about one page of an
  extraction more. The estimate says so before Start.
