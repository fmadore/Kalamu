# 2026-11-02 · Gemini 3.5 Transcribe as Gemini's model for recordings

## Context

Google's dedicated speech-to-text model, `gemini-3.5-transcribe`, is listed as stable on the
Gemini API's model page. Unlike the Gemini chat models, which Kalamu asks for speakers and
`[hh:mm:ss]` timings in its prompt, it takes no prompt: a recording and
`generationConfig.audioTranscriptionConfig` (`diarization`, `wordTimestamp`, `languageCodes`,
`customVocabulary`, `mode`), through the same `generateContent` endpoint. Speakers and word
timings come back on the answer's parts (`audioTranscription.speakerLabel` and `.words`, with
offsets such as `"1.250s"`). Its page documents 83 language codes, 77 languages once regional
variants are folded, and a limit of 30 minutes of audio a request when speakers or timings are
asked for. Everything below follows Google's pages of 2 November 2026; no live call has been
made yet.

## Decisions

- **Gemini's default for audio transcription**, in place of Gemini 3.8 Flash, which stays
  offered. YouTube links still go to the chat models, which watch the video from its link.
- **A request of its own** (`providers/src/gemini/transcribe.rs`): no system instruction,
  prompt, thinking level, media resolution or safety settings. Speakers when the set-up asks
  for them; word timings whenever timestamps or speakers are asked for, since Kalamu places the
  lines of a transcript by them. Each part with words becomes a segment, a speaker's clauses
  joined into turns as for OpenAI and Voxtral, and `spk_1` reads as "Speaker 1". The provider
  tells the model apart by its name, as OpenAI's tells its diarizing model.
- **Languages**: the catalogue lists the model's 77 languages by Kalamu's codes (Mandarin as
  `zh`), so New job offers only those, and shows the list under the model, as it does for
  Whisper. A chosen language is hinted with the model's own code (`fr` as `fr-FR`); with none,
  or one it does not list, the model detects the language.
- **Price** at Google's per-minute estimate, $0.005 ($0.003 read, $0.002 written), and pieces of
  at most 30 minutes and 18 MB, sent inline as other Gemini requests are. Custom vocabulary and
  the smart mode are not used: Google documents that neither combines with speakers or timings.
- **Voxtral's languages**: Mistral states that its two transcription models read 50 languages
  but does not publish which; the descriptions say so, and the models keep "any language".

## Consequences

- To run live before it is relied on: the shape of a real answer, whether parts carry `text`
  beside their words, and what a minute costs against the estimate.
