# 2026-10-26 · GPT-6.1 Terra and Mistral's chat models

## Context

OpenAI released GPT-6.1 Terra on 26 October 2026, and the researcher asked for it in the
catalogue, together with Mistral's general models: until now Mistral served only OCR and
Voxtral, so it could not run analyses (issue #4). This entry draws on Mistral's documentation
as of 26 October (the `mistralai/platform-docs-public` repository behind docs.mistral.ai: model
cards, the chat completions API, structured output, reasoning and Document QnA) and on OpenAI's
API specification and the Terra announcement.

## Decisions

- **GPT-6.1 Terra is a catalogue entry only** (`gpt-6.1-terra`, $1 / $5 per million tokens,
  the same inputs as GPT-6.1 Sol), going through the existing Responses API code. OpenAI
  presents it as a model for computer use. Sol stays OpenAI's default until Terra has been
  tried on historical print and handwriting.
- **Mistral's chat models run analyses, not text extraction.** Mistral's own documentation
  sends document parsing to its OCR, and `Provider::accepted_inputs` is set per provider and
  task, not per model: chat models taking text extraction would receive the PDF pages that OCR
  takes, and Mistral reads a PDF sent to a chat model through that same OCR first. Text
  extraction therefore stays with Mistral OCR 4.1.
- **Three models**, each a pinned identifier from its model card: Mistral Medium 3.5
  (`mistral-medium-3-5`, stable, Mistral's default for analyses, matching the mid-range
  defaults of OpenAI and Claude), Mistral Large 4 (`mistral-large-4-0`, public preview since
  6 October, a million-token context, priced below Medium) and Mistral Small 4
  (`mistral-small-2603`, the inexpensive one). Large 4 can become the default once it leaves
  preview.
- **The request** follows the other providers: the document first and the request last, the
  fields as a strict `json_schema` response format, and the lowest reasoning setting
  (`reasoning_effort: none`), which also returns the answer as one string. Text goes as a text
  chunk, PDFs as `document_url`, images as `image_url`, all inline as data URLs.
- **`mistral/chat.rs`** holds the chat request and response, so `mistral.rs` stays below the
  ~500-line mark.

## Consequences

- Mistral now covers all three tasks, so one key is enough for OCR followed by an analysis.
- The Mistral analysis is tested against a mock server, in the same test as the other three
  providers, and the live example runs it with Small 4. Neither new model has run against the
  live API yet.
- With a PDF, a Mistral chat model works from the text Mistral's OCR extracts. A text file or
  photographed page goes to the model as it is.
