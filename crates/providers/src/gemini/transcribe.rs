//! Google's speech-to-text models (`gemini-3.5-transcribe`): the recording and
//! `audioTranscriptionConfig` only, no prompt, system instruction or thinking. Speakers and word
//! timings come back beside the text, one part per utterance, and become timed segments.

use kalamu_core::{
    LanguageChoice, ProcessOutput, ProcessRequest, ProviderError, RequestSetting, Segment,
    SetupOptions,
};
use serde_json::{Value, json};

use super::Gemini;
use crate::http::{base64, join_turns, speaker_label};

/// The language codes Gemini 3.5 Transcribe documents, the only ones its hints take. The
/// catalogue lists the same languages, by Kalamu's codes, for New job to offer.
const TRANSCRIBE_LANGUAGES: &[&str] = &[
    "af-ZA",
    "am-ET",
    "ar-EG",
    "as-IN",
    "az-AZ",
    "be-BY",
    "bg-BG",
    "bn-BD",
    "bn-IN",
    "bs-BA",
    "ca-ES",
    "ceb",
    "cmn-Hans-CN",
    "cs-CZ",
    "da-DK",
    "de-DE",
    "el-GR",
    "en-GB",
    "en-IN",
    "en-US",
    "es-419",
    "es-US",
    "et-EE",
    "fa-IR",
    "fi-FI",
    "fil-PH",
    "fr-FR",
    "gl-ES",
    "gu-IN",
    "ha-NG",
    "he-IL",
    "hi-IN",
    "hr-HR",
    "hu-HU",
    "hy-AM",
    "id-ID",
    "is-IS",
    "it-IT",
    "ja-JP",
    "jv-ID",
    "ka-GE",
    "kea-CV",
    "kk-KZ",
    "km-KH",
    "kn-IN",
    "ko-KR",
    "ky-KG",
    "ln-CD",
    "lt-LT",
    "lv-LV",
    "mk-MK",
    "ml-IN",
    "mn-MN",
    "mr-IN",
    "ms-MY",
    "mt-MT",
    "my-MM",
    "nb-NO",
    "ne-NP",
    "nl-NL",
    "or-IN",
    "pa-IN",
    "pa-Guru-IN",
    "pl-PL",
    "pt-BR",
    "pt-PT",
    "ro-RO",
    "ru-RU",
    "rup-BG",
    "sd-Arab-IN",
    "sk-SK",
    "sl-SI",
    "sr-RS",
    "sv-SE",
    "sw-KE",
    "te-IN",
    "tg-TJ",
    "th-TH",
    "tr-TR",
    "uk-UA",
    "uz-UZ",
    "vi-VN",
    "yue-Hant-HK",
];

/// A speech-to-text model's request: the recording and what to mark in its transcript.
pub(super) fn body(request: &ProcessRequest<'_>) -> Value {
    let input = request.input;
    let mut config = config(request.options);
    // The languages chosen, as the model's own codes; with none it detects the language.
    if let LanguageChoice::Explicit(codes) = request.languages {
        let hints: Vec<&str> = codes
            .iter()
            .filter_map(|c| transcribe_language(c.as_str()))
            .collect();
        if !hints.is_empty() {
            config["languageCodes"] = json!(hints);
        }
    }
    json!({
        "contents": [{
            "role": "user",
            "parts": [{
                "inlineData": {
                    "mimeType": input.mime,
                    "data": base64(&input.bytes),
                }
            }],
        }],
        "generationConfig": {
            "audioTranscriptionConfig": config,
        },
    })
}

/// A speech-to-text model's answer: its text, and a segment for each utterance timed by its
/// words, a speaker's clauses joined into one turn.
pub(super) fn parse(response: &Value) -> Result<ProcessOutput, ProviderError> {
    let mut output = Gemini::parse(response)?;
    let parts = response
        .pointer("/candidates/0/content/parts")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let segments: Vec<Segment> = parts
        .iter()
        .filter_map(|part| {
            let said = part.get("audioTranscription")?;
            let words = said.get("words").and_then(Value::as_array)?;
            let (first, last) = (words.first()?, words.last()?);
            // The part's own text keeps the spacing of languages written without spaces.
            let text = part
                .get("text")
                .and_then(Value::as_str)
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| {
                    words
                        .iter()
                        .filter_map(|w| w.get("word").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join(" ")
                });
            Some(Segment {
                start_ms: offset_ms(first.get("startOffset")),
                end_ms: offset_ms(last.get("endOffset")),
                speaker: said
                    .get("speakerLabel")
                    .and_then(Value::as_str)
                    .map(speaker_label),
                text,
            })
        })
        .collect();
    // Without word timings there is nothing to place the lines by.
    if segments.iter().any(|s| s.end_ms > 0) {
        output.segments = join_turns(segments);
    }
    let lines: Vec<&str> = parts
        .iter()
        .filter_map(|p| p.get("text").and_then(Value::as_str))
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    output.text = if lines.is_empty() {
        output
            .segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        lines.join("\n")
    };
    Ok(output)
}

/// One of Google's speech-to-text models, which take no prompt.
pub(super) fn transcriber(model: &str) -> bool {
    model.contains("transcribe")
}

/// Speakers when asked for; word timings whenever timestamps or speakers are, since the lines
/// of a transcript are placed by them.
fn config(options: &SetupOptions) -> Value {
    json!({
        "diarization": options.diarize,
        "wordTimestamp": options.timestamps || options.diarize,
    })
}

/// The settings a speech-to-text request carries, decided as `transcription_config` decides.
pub(super) fn settings(options: &SetupOptions) -> Vec<RequestSetting> {
    let config = config(options);
    ["diarization", "wordTimestamp"]
        .into_iter()
        .map(|name| {
            RequestSetting::new(
                &format!("generationConfig.audioTranscriptionConfig.{name}"),
                config[name].as_bool().unwrap_or(false),
            )
        })
        .collect()
}

/// Gemini 3.5 Transcribe's code for a language: the first it documents for it, Mandarin for
/// Chinese.
fn transcribe_language(code: &str) -> Option<&'static str> {
    let code = if code == "zh" { "cmn" } else { code };
    TRANSCRIBE_LANGUAGES
        .iter()
        .find(|c| c.split('-').next() == Some(code))
        .copied()
}

/// A duration as Google writes it (`"1.250s"`), in milliseconds.
fn offset_ms(value: Option<&Value>) -> u64 {
    let seconds = match value {
        Some(Value::String(s)) => s.trim_end_matches('s').parse::<f64>().ok(),
        Some(v) => v.as_f64(),
        None => None,
    };
    seconds.map_or(0, |s| (s * 1000.0).round().max(0.0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_is_hinted_by_the_code_the_model_documents() {
        assert_eq!(transcribe_language("fr"), Some("fr-FR"));
        assert_eq!(transcribe_language("zh"), Some("cmn-Hans-CN"));
        assert_eq!(transcribe_language("kea"), Some("kea-CV"));
        // A language the model does not list gets no hint: it detects the language instead.
        assert_eq!(transcribe_language("yo"), None);
    }

    #[test]
    fn offsets_are_read_in_seconds() {
        assert_eq!(offset_ms(Some(&json!("1.250s"))), 1_250);
        assert_eq!(offset_ms(Some(&json!("3s"))), 3_000);
        assert_eq!(offset_ms(Some(&json!(0.5))), 500);
        assert_eq!(offset_ms(None), 0);
    }
}
