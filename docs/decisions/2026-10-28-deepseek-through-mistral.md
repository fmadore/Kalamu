# 2026-10-28 · DeepSeek V4.1 Flash through Mistral

## Context

On 28 October Mistral started serving DeepSeek V4.1 Flash (`deepseek-v4-1-flash-1017`) on its
own servers, behind the same key and Chat Completions endpoint as GLM (decision of 27 October),
in public preview with limited capacity. The same announcement made GLM 5.3 Flash generally
available and gave its cached-input price. Issue #18 also weighed Mistral's new Responses API.

The entry follows Mistral's model page for DeepSeek V4.1 Flash 1017 (its web address says
`deepseek-v4-flash-1017`, but the API name is `deepseek-v4-1-flash-1017`) and its page for
GLM 5.3 Flash. Unlike GLM, no live call has been made yet.

## Decisions

- **DeepSeek V4.1 Flash for analyses**, as a catalogue entry under Mistral: preview, $0.30 input,
  $0.007 cached input and $1.20 output per million tokens; 1M tokens of context, 128k of output.
- **Text only.** Mistral describes it as a text model and, unlike GLM 5.3 Flash, does not list
  Document QnA for it, so a scan is sent through text extraction first, as for GLM 5.3.
- **`reasoning_effort` is `low`**, as for GLM. Mistral's reasoning page documents the accepted
  values for GLM only (`low`, `high` and `max`, not `none`); DeepSeek V4.1's own lowest setting
  is `low` as well.
- **Economy mode at half price.** Mistral's page lists batching (`/v1/batch`) for the model, and
  Mistral's batch service takes 50% off, so the entry has `batch_discount: 0.5` like its other
  chat models. Preview capacity is limited, so a large batch may wait longer than usual.
- **GLM 5.3 Flash** gains its cached-input price, $0.03.
- **Responses API: not adopted.** Kalamu sends each piece once with a strict schema, which Chat
  Completions already does, batches included. The Responses API is in preview and its batch
  support is unconfirmed. To revisit when it is generally available and batched; the change
  would stay inside `mistral/chat.rs` and `mistral/batch.rs`.

## Consequences

- One Mistral key now covers six chat models, three of them open-weight.
- The request is tested against a mock server, together with GLM. A live call should confirm
  the model name, `reasoning_effort: low` and strict `json_schema`, and a small batch the economy
  mode; each is a one-line change to the catalogue or `lowest_effort` if it differs.
