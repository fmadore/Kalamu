# 2026-11-02 · Fewer default prompts, one per kind of material

## Context

Kalamu shipped nineteen default prompts, six of them for text extraction. Several repeated
another with one difference:

- *Printed document, diplomatic* and *normalized*, ported from zmo-ai-pipelines, where they are
  two modes of the same reading. They differ in their lines (kept as on the page, or joined),
  which New job already asks for every extraction and adds to whichever prompt is used.
- *Handwritten letter or document (French)*, *Arabic or Ajami manuscript* and *Handwriting,
  several languages or scripts*, ported from iwac-ai-pipelines' three HTR prompts. They share
  their rules almost word for word; what differs is a language's conventions. The language is
  also chosen in New job.
- *Interview, translated into English* and *into French*: the same prompt in two target
  languages.
- *Summary and keywords for research* and *Short summary for a catalogue*: the same principles
  and fields at two lengths.
- *Printed document* and *Newspaper (columns, headlines)* for correction: the first was derived
  from the second for any printed text.

A researcher choosing a prompt saw near-duplicates, and the language-specific ones suggested
that a language without its own preset was not supported.

## Decisions

- **One default per kind of material, never per language, script or small variant.** Thirteen
  remain: three for text extraction (*printed or typed document*, *newspaper or magazine*,
  *handwritten document or manuscript*), two for recordings (*transcribed*, *translated into
  English*), four analyses, three tables of contents, one correction.
- **Each extraction prompt covers any language and either reading direction.** The handwriting
  prompt keeps what the French, Arabic and multilingual HTR prompts knew: right-to-left text
  written in reading order and never transliterated, Arabic vowel marks only where written, the
  extra letters of Ajami kept, French spacing before high punctuation, verse kept as verse. It
  drops the multilingual prompt's `[LANGUAGE DETECTED: …]` header and `[LANGUAGE SWITCH: …]`
  tags, which put text into the transcription that is not on the page and counted against its
  error rates after review.
- **Lines are left to New job.** Every extraction prompt writes running text by default and
  says that a choice of lines given at the end replaces that section, so *As on the page* gives
  what the diplomatic prompt gave.
- **One set of marks** across the text prompts: `[illegible]`, an uncertain reading as
  `[Kpémé?]` (as the interview prompts already wrote it), `[handwritten: …]`,
  `[struck through: …]`, `[margin: …]`, `[stamp: …]`; `[BLANK PAGE]` unchanged, since the
  engine reads it. Marks added by the holding library or archive (call numbers, ownership
  stamps, barcodes, colour charts) are left out; stamps of the document's own time are kept.
- **One translation prompt**, its target language named in one place (*Target language:
  English*) so a copy for French or Swahili changes one word. Speaker labels stay `Speaker 1`
  in any language, so the review finds and renames them.
- **The interview prompts state the number of speakers** through `{speakers}`, the variable
  presets already had: "an unknown number" while New job does not ask for it, the number
  itself once a set-up gives one.
- **One summary**, whose first sentence stands on its own for a catalogue or a list of search
  results and whose length otherwise follows the document.
- **One correction prompt** for any printed text, with a section on newspapers (headlines,
  bylines, figures, articles read as one).
- **The rewritten prompts are calm instructions**, without the capitals and repeated
  self-review sections of the ported ones, in the style of the analysis prompts.

## Compatibility

- A retired default's id resolves to its replacement (`defaults::RETIRED`): a saved pipeline
  or workflow shows and runs with the replacement, and a shared file that names a retired
  default brings its own text as a new preset of the researcher's, as any edited default does.
  A pipeline that used *translated into French* therefore shows *translated into English* in
  New job, where the researcher sees it before running.
- An edited copy of a retired default is kept as the researcher's own preset, under its old id
  and file, listed after the defaults.
- Jobs already run keep the prompt they froze, by name and text, in their provenance.

## Where Kalamu differs from the reference projects

zmo-ai-pipelines keeps separate diplomatic and normalized prompts, and iwac-ai-pipelines one
HTR prompt per language family. Kalamu keeps one of each because the line choice and the
language are settings of the job, added to any prompt, its own or the researcher's.

## Consequences

- The Prompts view and New job offer thirteen defaults instead of nineteen; *Duplicate* remains
  the way to a language-specific or collection-specific variant.
- The consolidated prompts have not yet been run live against every provider; the first runs
  on the sample files (the 1755 manuscript, the 1915 newspaper, the interview) should be
  compared with results from the retired prompts before the next release.
