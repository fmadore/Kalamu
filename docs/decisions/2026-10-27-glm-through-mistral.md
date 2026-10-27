# 2026-10-27 · Z.ai GLM through Mistral, and inputs filtered per model

## Context

Mistral hosts Z.ai's GLM models on its own servers, behind the same key and Chat Completions
endpoint as its own models, so a researcher with a Mistral key can use them without another
account (issue #6). Mistral serves the weights unmodified, under the same regional controls as
its own models; the data does not go to Z.ai. This entry draws on Mistral's documentation as of
27 October (the `mistralai/platform-docs-public` repository: model cards, reasoning page and
changelog) and on a few one-sentence live calls made that day.

Until now the engine sent a model whatever its provider accepted for the task
(`Provider::accepted_inputs`); the `inputs` listed for each model in the catalogue limited
nothing. A text-only model behind Mistral would have been sent PDFs and images.

## Decisions

- **The engine sends a model only what both its provider and its catalogue entry accept**, in
  the provider's order of preference. A model missing from the catalogue keeps the provider's
  list. A text-only model given a PDF or picture fails that document with a message saying to
  extract the text first, without calling the provider. Gemini's entries now list HEIC, HEIF,
  AAC and WebM, which its provider already accepted, so nothing it read before is refused.
- **Two models, for analyses only**, as Mistral's own chat models (text extraction stays with
  Mistral OCR, decision of 26 October):
  - **Z.ai GLM 5.3** (`zai-glm-5-3`, generally available since 28 September, $1.40 / $4.40 per
    million tokens). Its card lists text input only, and a live call with an image was refused
    ("Image input is not enabled for this model"). A PDF sent as `document_url` was answered
    live, but the card does not list Document QnA, so the catalogue gives it text only.
  - **Z.ai GLM 5.3 Flash** (`zai-glm-5-3-flash`, available since 22 October, $0.15 / $0.50).
    Its card lists text and images, and Document QnA; both an image and a PDF were read live.
  - GLM 5.2 is left out: deprecated on 29 September, it retires on 31 October.
- **Strict `json_schema` works** with both models: their card lists structured outputs, and the
  live calls returned valid JSON matching the schema.
- **`reasoning_effort` is `low` for GLM.** Both models reject `none` (Mistral's own lowest
  setting) with a 400 error and accept `low`, `high` and `max`. The answer may then come as
  thinking and text chunks; only the text chunks are read, as already for Mistral's models.
  The choice is made from the model id's `zai-glm` prefix in `mistral/chat.rs`.

## Consequences

- One Mistral key now covers OCR, transcription and analyses with five chat models, two of them
  open-weight.
- A text-only model can analyse plain-text sources, and OCR output or transcripts passed on in a
  pipeline; a scan has to go through text extraction first.
- The filter applies to every provider, so a future text-only or image-only model needs only
  its catalogue entry.
- The GLM request and chunked answer are tested against a mock server, and the per-model filter
  by an engine test. Neither model has yet run through a whole job against the live API.
