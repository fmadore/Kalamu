//! The tested default prompts, ported verbatim from zmo-ai-pipelines (`printed-*`) and
//! iwac-ai-pipelines (the transcription and HTR ones). The per-page requests come from the same
//! scripts. The analysis prompts are new, in English: the summaries keep the principles of
//! iwac-ai-pipelines' bilingual summary prompt (specific, faithful, searchable), the entities
//! its NER rules and the tone its sentiment scales (labels, never numbers, with a reason),
//! without their collection-specific parts.

use scriptorium_core::{AnalysisSchema, Field, FieldKind, Task};

pub(crate) struct Default {
    pub id: &'static str,
    pub task: Task,
    pub name: &'static str,
    pub description: &'static str,
    pub instructions: &'static str,
    pub user_prompt: &'static str,
    /// The fields an analysis preset asks for.
    pub schema: Option<fn() -> AnalysisSchema>,
}

/// zmo-ai-pipelines' `USER_PROMPT`.
const PAGE_REQUEST: &str = "Transcribe this page completely, following the instructions given.";

/// iwac-ai-pipelines' HTR `build_user_prompt`, without the language, which the instructions
/// already state.
const HANDWRITING_REQUEST: &str = "This is a legitimate handwritten text transcription (HTR) \
request for academic research and archival preservation. Transcribe ALL handwritten text with \
exact wording, spacing rules, accents, and WITHOUT summarizing or omitting any zones.";

const RECORDING_REQUEST: &str = "Transcribe this recording, following the instructions given.";

const ANALYSIS_REQUEST: &str =
    "Read the document above and fill in each field, following the instructions given.";

fn field(label: &str, instructions: &str, kind: FieldKind, reason: bool) -> Field {
    Field {
        key: String::new(),
        label: label.to_string(),
        instructions: instructions.to_string(),
        kind,
        reason,
    }
}

fn list(max: u32) -> FieldKind {
    FieldKind::List { max: Some(max) }
}

fn choice(values: &[&str]) -> FieldKind {
    FieldKind::Choice {
        values: values.iter().map(|v| (*v).to_string()).collect(),
    }
}

const KEYWORDS: &str = "Keywords and short key phrases in {languages} that someone would search \
for: people, organisations, places, events and specific themes. Prefer the specific (\"Hajj \
1957\") to the general (\"religion\"), and keep proper names as in the source.";

fn summary_research() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Summary",
                "What this document in particular says, following the instructions above.",
                FieldKind::Text,
                false,
            ),
            field("Keywords", KEYWORDS, list(12), false),
        ],
    }
}

fn summary_brief() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Summary",
                "Two or three dense sentences, following the instructions above.",
                FieldKind::Text,
                false,
            ),
            field("Keywords", KEYWORDS, list(8), false),
        ],
    }
}

fn entities() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field("People", "People named in the document.", list(30), false),
            field(
                "Organisations",
                "Associations, parties, administrations, companies, religious bodies.",
                list(20),
                false,
            ),
            field("Places", "Places named in the document.", list(20), false),
            field(
                "Subjects",
                "The document's specific subjects.",
                list(6),
                false,
            ),
        ],
    }
}

fn tone() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Main subject",
                "What the document is mainly about, in a few words, in {languages}.",
                FieldKind::Text,
                false,
            ),
            field(
                "Tone",
                "How the document presents its main subject.",
                choice(&["Positive", "Neutral", "Negative", "Mixed"]),
                true,
            ),
            field(
                "Register",
                "How much the document reports facts and how much it gives opinions.",
                choice(&[
                    "Factual",
                    "Mostly factual",
                    "Mixed",
                    "Mostly opinion",
                    "Opinion",
                ]),
                true,
            ),
        ],
    }
}

pub(crate) const DEFAULTS: &[Default] = &[
    Default {
        id: "text_extraction.printed_diplomatic",
        task: Task::TextExtraction,
        name: "Printed document, diplomatic",
        description: "Keeps spelling, abbreviations, deletions and insertions exactly as on the page.",
        instructions: include_str!("../defaults/printed-diplomatic.md"),
        user_prompt: PAGE_REQUEST,
        schema: None,
    },
    Default {
        id: "text_extraction.printed_normalized",
        task: Task::TextExtraction,
        name: "Printed document, normalized",
        description: "Readable text: wording and spelling kept, spacing tidied.",
        instructions: include_str!("../defaults/printed-normalized.md"),
        user_prompt: PAGE_REQUEST,
        schema: None,
    },
    Default {
        id: "text_extraction.newspaper",
        task: Task::TextExtraction,
        name: "Newspaper or magazine (columns)",
        description: "Follows the reading order of articles across columns.",
        instructions: include_str!("../defaults/newspaper.md"),
        user_prompt: "Please perform complete OCR transcription of this single page. Extract all \
visible text maintaining original formatting and structure.",
        schema: None,
    },
    Default {
        id: "text_extraction.handwriting_french",
        task: Task::TextExtraction,
        name: "Handwritten letter or document (French)",
        description: "French handwriting, keeping historical spelling and French typography.",
        instructions: include_str!("../defaults/handwriting-french.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
    },
    Default {
        id: "text_extraction.manuscript_arabic_script",
        task: Task::TextExtraction,
        name: "Arabic or Ajami manuscript",
        description: "Arabic-script handwriting, read right to left, without adding diacritics.",
        instructions: include_str!("../defaults/manuscript-arabic-script.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
    },
    Default {
        id: "text_extraction.handwriting_multilingual",
        task: Task::TextExtraction,
        name: "Handwriting, several languages or scripts",
        description: "Detects each language and script and keeps each in its reading direction.",
        instructions: include_str!("../defaults/handwriting-multilingual.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
    },
    Default {
        id: "audio_transcription.interview",
        task: Task::AudioTranscription,
        name: "Interview, verbatim with speakers",
        description: "Speaker labels and a timestamp at each turn; code-switching kept as spoken.",
        instructions: include_str!("../defaults/interview-transcription.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
    },
    Default {
        id: "audio_transcription.translate_english",
        task: Task::AudioTranscription,
        name: "Interview, translated into English",
        description: "An English translation of the recording, with speakers and timestamps.",
        instructions: include_str!("../defaults/interview-translation-english.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
    },
    Default {
        id: "audio_transcription.translate_french",
        task: Task::AudioTranscription,
        name: "Interview, translated into French",
        description: "A French translation of the recording, with speakers and timestamps.",
        instructions: include_str!("../defaults/interview-translation-french.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
    },
    Default {
        id: "analysis.summary_research",
        task: Task::Analysis,
        name: "Summary and keywords for research",
        description: "What this document in particular says, with names, figures and dates, and searchable keywords.",
        instructions: include_str!("../defaults/summary-research.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(summary_research),
    },
    Default {
        id: "analysis.summary_brief",
        task: Task::Analysis,
        name: "Short summary for a catalogue",
        description: "Two or three dense sentences that stand out in a list of search results.",
        instructions: include_str!("../defaults/summary-brief.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(summary_brief),
    },
    Default {
        id: "analysis.entities",
        task: Task::Analysis,
        name: "People, places and organisations",
        description: "The people, organisations, places and subjects each document names, spelled as in the text.",
        instructions: include_str!("../defaults/analysis-entities.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(entities),
    },
    Default {
        id: "analysis.tone",
        task: Task::Analysis,
        name: "Tone of the document",
        description: "Whether each document presents its subject positively or negatively, and how much it gives opinions, with a reason for each.",
        instructions: include_str!("../defaults/analysis-tone.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(tone),
    },
];
