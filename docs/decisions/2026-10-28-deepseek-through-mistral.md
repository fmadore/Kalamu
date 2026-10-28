# 2026-10-28 · DeepSeek V4.1 Flash through Mistral

## Context

On 28 October Mistral started serving DeepSeek V4.1 Flash (`deepseek-v4p1-flash-1017`) on its
own servers, behind the same key and Chat Completions endpoint as GLM (decision of 27 October),
in public preview with limited capacity. The same announcement made GLM 5.3 Flash generally
available and gave its cached-input price. Issue #18 also weighed Mistral's new Responses API.

Mistral's documentation and API could not be reached from the session that made this change,
so the entry draws on Mistral's announcement and DeepSeek's model card on Hugging Face
(`deepseek-ai/DeepSeek-V4.1-Flash-1017`). Unlike GLM, no live call has been made yet.

## Decisions

- **DeepSeek V4.1 Flash for analyses**, as a catalogue entry under Mistral: preview, $0.30 input,
  $0.007 cached input and $1.20 output per million tokens.
- **Text only for now.** DeepSeek's card says the model reads images, but Mistral's deployment
  may not enable it (GLM 5.3 reads images too, yet Mistral refuses them), and whether a PDF goes
  through Document QnA is unknown. A scan is sent through text extraction first, as for GLM 5.3.
  Widening `inputs` is a catalogue change once a live call confirms it.
- **`reasoning_effort` is `low`**, as for GLM. DeepSeek V4.1's own settings are `low`, `high`
  and `max`, and Mistral rejected `none` for the other third-party models it hosts.
- **No economy mode yet.** The announcement does not say whether the batch service takes the
  model, and preview capacity is limited, so the entry has no `batch_discount`.
- **GLM 5.3 Flash** gains its cached-input price, $0.03.
- **Responses API: not adopted.** Kalamu sends each piece once with a strict schema, which Chat
  Completions already does, batches included. The Responses API is in preview and its batch
  support is unconfirmed. To revisit when it is generally available and batched; the change
  would stay inside `mistral/chat.rs` and `mistral/batch.rs`.

## Consequences

- One Mistral key now covers six chat models, three of them open-weight.
- The request is tested against a mock server, together with GLM. A live call should confirm
  `reasoning_effort: low`, strict `json_schema`, image and PDF input, and batch support; each
  is a one-line change to the catalogue or `lowest_effort` if it differs.
