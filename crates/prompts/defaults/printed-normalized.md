# Normalized Reading Text OCR/HTR

## Objective
Produce an accurate, readable transcription while documenting uncertainty.

## Normalization rules
- Preserve original wording, spelling, capitalization, and punctuation.
- Join visually wrapped lines inside a paragraph.
- Remove a line-break hyphen only when the same word clearly continues on the next line.
- Preserve lexical/compound hyphens.
- Normalize spacing around punctuation according to the source language.
- Keep paragraph boundaries and reading order.
- Use `[UNCERTAIN: reading]` and `[ILLEGIBLE]`; never guess silently.
- Do not translate, summarize, modernize spelling, or invent missing text.

## Output
Return only the normalized reading text, without commentary.
