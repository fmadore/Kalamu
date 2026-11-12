# 2026-11-12 · Gemini 4 Hydrogen, and what one Gemini request may write

## Context

Google released Gemini 4 Hydrogen (`gemini-4-hydrogen`) on 12 November 2026, stable, "for teams
migrating from Gemini 3.8 Flash", which its model list now calls "our previous-generation Flash
model". Google deprecates nothing, and names no successor for 3.8 Flash. Hydrogen costs $1.50 /
$7.50 per million tokens ($0.075 cached, half in batch mode), flat across its 1M-token context:
twice 3.8 Flash's introductory price until 31 December 2026, the same from 1 January 2027. It
reads text, images, PDFs, audio and video; its model card reports more unjustified refusals than
3.8 Flash, "especially for multimodal prompts", which matters for historical documents.

Its thinking levels are `low`, `medium`, `high` (the default) and a new `xhigh`; `minimal` and a
thinking budget are refused with a 400. Temperature, top-p and top-k are fixed.

Google advertises up to 1M output tokens, through *long decodes*: one request writes at most a
**hop**, 64K tokens for Hydrogen, and stops with `finishReason: CONTINUATION` and a
`continuationToken` the caller sends back for the next hop; only Google's SDK loops by itself.
Without streaming, as Kalamu calls `generateContent`, the hop of **Gemini 4 Carbon and Argon is
32K**, and a request asking more than 32,768 tokens of them without a continuation token is
refused with a 400. The catalogue asked 65,536 of both.

## Decision

- **Hydrogen is a catalogue entry beside 3.8 Flash**, not its successor: Google has not named it
  the replacement, and until January it costs twice as much. 3.8 Flash stays the recommended
  model and Gemini's default for extraction and analysis. The question comes back on 1 January,
  when the prices meet and the catalogue's prices are updated anyway: by then Hydrogen should be
  tried on real pages, handwriting included, against 3.8 Flash, for reading and for refusals.
- **Hydrogen asks for 65,536 tokens**, its whole hop, never more: Kalamu sends one piece per
  request, and a page or an analysis is far below 64K. Long decodes are not used.
- **Carbon and Argon ask for 32,768**, the most one non-streamed request may write.
- **`CONTINUATION` counts as cut off**, like `MAX_TOKENS`: a page keeps what was read, marked
  cut off (as any other early stop already was), and a YouTube transcript is refused as too long.
  The catalogue never asks beyond a hop, so this guards an entry that one day would.
- **Levels as Google lists them**; Kalamu suggests `low` for reading and analysing, and a
  `minimal` chosen elsewhere becomes `low`, the nearest level Hydrogen has.

## Consequences

- No code knows Hydrogen's name: it is one entry, one line in the list of shipped ids.
- A long analysis with Carbon or Argon may now stop at 32K tokens, thinking included, where it
  was refused before; it is marked cut off.
