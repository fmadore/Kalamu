//! The tested default prompts, ported verbatim from zmo-ai-pipelines (`printed-*`) and
//! iwac-ai-pipelines (the others). The per-page requests come from the same scripts.

use scriptorium_core::Task;

pub(crate) struct Default {
    pub id: &'static str,
    pub task: Task,
    pub name: &'static str,
    pub description: &'static str,
    pub instructions: &'static str,
    pub user_prompt: &'static str,
}

/// zmo-ai-pipelines' `USER_PROMPT`.
const PAGE_REQUEST: &str = "Transcribe this page completely, following the instructions given.";

/// iwac-ai-pipelines' HTR `build_user_prompt`, without the language, which the instructions
/// already state.
const HANDWRITING_REQUEST: &str = "This is a legitimate handwritten text transcription (HTR) \
request for academic research and archival preservation. Transcribe ALL handwritten text with \
exact wording, spacing rules, accents, and WITHOUT summarizing or omitting any zones.";

const RECORDING_REQUEST: &str = "Transcribe this recording, following the instructions given.";

pub(crate) const DEFAULTS: &[Default] = &[
    Default {
        id: "text_extraction.printed_diplomatic",
        task: Task::TextExtraction,
        name: "Printed document, diplomatic",
        description: "Keeps spelling, line breaks and end-of-line hyphens exactly as on the page.",
        instructions: include_str!("../defaults/printed-diplomatic.md"),
        user_prompt: PAGE_REQUEST,
    },
    Default {
        id: "text_extraction.printed_normalized",
        task: Task::TextExtraction,
        name: "Printed document, normalized",
        description: "Readable text: wrapped lines joined and line-break hyphens removed.",
        instructions: include_str!("../defaults/printed-normalized.md"),
        user_prompt: PAGE_REQUEST,
    },
    Default {
        id: "text_extraction.newspaper",
        task: Task::TextExtraction,
        name: "Newspaper or magazine (columns)",
        description: "Joins column lines into paragraphs and follows the reading order of articles.",
        instructions: include_str!("../defaults/newspaper.md"),
        user_prompt: "Please perform complete OCR transcription of this single page. Extract all \
visible text maintaining original formatting and structure.",
    },
    Default {
        id: "text_extraction.handwriting_french",
        task: Task::TextExtraction,
        name: "Handwritten letter or document (French)",
        description: "French handwriting, keeping historical spelling and French typography.",
        instructions: include_str!("../defaults/handwriting-french.md"),
        user_prompt: HANDWRITING_REQUEST,
    },
    Default {
        id: "text_extraction.manuscript_arabic_script",
        task: Task::TextExtraction,
        name: "Arabic or Ajami manuscript",
        description: "Arabic-script handwriting, read right to left, without adding diacritics.",
        instructions: include_str!("../defaults/manuscript-arabic-script.md"),
        user_prompt: HANDWRITING_REQUEST,
    },
    Default {
        id: "text_extraction.handwriting_multilingual",
        task: Task::TextExtraction,
        name: "Handwriting, several languages or scripts",
        description: "Detects each language and script and keeps each in its reading direction.",
        instructions: include_str!("../defaults/handwriting-multilingual.md"),
        user_prompt: HANDWRITING_REQUEST,
    },
    Default {
        id: "audio_transcription.interview",
        task: Task::AudioTranscription,
        name: "Interview, verbatim with speakers",
        description: "Speaker labels and a timestamp at each turn; code-switching kept as spoken.",
        instructions: include_str!("../defaults/interview-transcription.md"),
        user_prompt: RECORDING_REQUEST,
    },
    Default {
        id: "audio_transcription.translate_english",
        task: Task::AudioTranscription,
        name: "Interview, translated into English",
        description: "An English translation of the recording, with speakers and timestamps.",
        instructions: include_str!("../defaults/interview-translation-english.md"),
        user_prompt: RECORDING_REQUEST,
    },
    Default {
        id: "audio_transcription.translate_french",
        task: Task::AudioTranscription,
        name: "Interview, translated into French",
        description: "A French translation of the recording, with speakers and timestamps.",
        instructions: include_str!("../defaults/interview-translation-french.md"),
        user_prompt: RECORDING_REQUEST,
    },
];
