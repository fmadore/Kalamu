//! The default prompts: one per kind of material, never one per language or per small variant
//! (decision of 2 November 2026). The text extraction ones consolidate zmo-ai-pipelines'
//! diplomatic and normalized prompts (which differ only in their lines, a choice New job makes
//! for every preset) and iwac-ai-pipelines' OCR and its French, Arabic and multilingual HTR
//! prompts, keeping their rules for any language and either reading direction. The interview
//! prompts come from iwac-ai-pipelines' transcription prompts, one translation kept, its target
//! language named in one place to be changed. The analysis prompts are new, in English: the
//! summary keeps the principles of iwac-ai-pipelines' bilingual summary prompt (specific,
//! faithful, searchable), the entities its NER rules and the tone its sentiment scales (labels,
//! never numbers, with a reason), without their collection-specific parts. The scholarly
//! publication summary has no ancestor there: its fields follow what a social-science literature
//! review compares. The image description has none either: it keeps the summary's rules (only
//! what the source shows, no inference, the first sentence on its own) for what an image shows,
//! and lists only the names its own text gives (#168).
//!
//! The tables of contents port iwac-ai-pipelines' `AI_summary_issue` prompts (page extraction
//! and consolidation), in English and without their Islamic-magazine specifics; the archival
//! file and the book are new, on the same two steps. The correction keeps `AI_ocr_correction`'s
//! rules for any printed text, its newspaper ones included. The translation is new: it takes
//! the translated interview's rules (everything in order, names kept, the original term beside a
//! culturally specific one) to any text, its target named by `{languages}` as an analysis's
//! answer language is. The directory and the register are records analyses (#171), new: one row
//! per entry, read page by page, every cell as written ([`records`]).
//!
//! The list below keeps the order New job shows; the analyses' schemas live in [`analysis`]
//! and [`records`], the tables of contents' in [`contents`].

use kalamu_core::{AnalysisSchema, Field, FieldKind, InputKind, Task};

mod analysis;
mod contents;
mod records;

pub(crate) struct Default {
    pub id: &'static str,
    pub task: Task,
    pub name: &'static str,
    pub description: &'static str,
    pub instructions: &'static str,
    pub user_prompt: &'static str,
    /// The fields an analysis preset asks for, or a table of contents' entry fields.
    pub schema: Option<fn() -> AnalysisSchema>,
    /// A table of contents' second step.
    pub merge: Option<&'static str>,
    /// Pages full of text (a newspaper's), of which fewer go together in one call.
    pub dense: bool,
    /// The kind of source it is made for, which New job suggests it for: the image description's
    /// images. Most are made for whatever their task reads.
    pub made_for: Option<InputKind>,
}

const CORRECTION_REQUEST: &str =
    "Correct the machine-read text above, following the instructions given.";

const TRANSLATION_REQUEST: &str = "Translate the text above completely, without summarising or \
leaving out any part, following the instructions given.";

/// zmo-ai-pipelines' `USER_PROMPT`.
const PAGE_REQUEST: &str = "Transcribe this page completely, following the instructions given.";

/// After iwac-ai-pipelines' HTR `build_user_prompt`, which says why the page is read: some
/// models otherwise decline a manuscript.
const HANDWRITING_REQUEST: &str = "This is a legitimate request to transcribe a handwritten \
document for academic research and archival preservation. Transcribe all the writing on this \
page with its exact wording, spelling and diacritics, without summarising or leaving out any \
part, following the instructions given.";

const RECORDING_REQUEST: &str = "Transcribe this recording, following the instructions given.";

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

/// Defaults no longer shipped, each with the one that replaced it: a pipeline, a workflow or a
/// shared file that names one runs with its replacement.
pub(crate) const RETIRED: &[(&str, &str)] = &[
    (
        "text_extraction.printed_diplomatic",
        "text_extraction.printed",
    ),
    (
        "text_extraction.printed_normalized",
        "text_extraction.printed",
    ),
    (
        "text_extraction.handwriting_french",
        "text_extraction.handwritten",
    ),
    (
        "text_extraction.manuscript_arabic_script",
        "text_extraction.handwritten",
    ),
    (
        "text_extraction.handwriting_multilingual",
        "text_extraction.handwritten",
    ),
    (
        "audio_transcription.translate_french",
        "audio_transcription.translate_english",
    ),
    ("analysis.summary_brief", "analysis.summary_research"),
    ("text_correction.newspaper", "text_correction.general"),
];

/// The id a preset goes by now: a retired default's replacement, any other id as it is.
pub(crate) fn current(id: &str) -> &str {
    RETIRED
        .iter()
        .find(|(old, _)| *old == id)
        .map_or(id, |(_, new)| new)
}

pub(crate) const DEFAULTS: &[Default] = &[
    Default {
        id: "text_extraction.printed",
        task: Task::TextExtraction,
        name: "Printed or typed document",
        description: "Books, reports, typed letters and forms in any language, left to right or right to left; spelling kept as printed.",
        instructions: include_str!("../defaults/printed.md"),
        user_prompt: PAGE_REQUEST,
        schema: None,
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "text_extraction.newspaper",
        task: Task::TextExtraction,
        name: "Newspaper or magazine",
        description: "Article by article across the columns, with the words the columns split rejoined.",
        instructions: include_str!("../defaults/newspaper.md"),
        user_prompt: PAGE_REQUEST,
        schema: None,
        merge: None,
        dense: true,
        made_for: None,
    },
    Default {
        id: "text_extraction.handwritten",
        task: Task::TextExtraction,
        name: "Handwritten document or manuscript",
        description: "Letters, registers and manuscripts in any script, Latin, Arabic and Ajami included; spelling, diacritics and reading direction as written.",
        instructions: include_str!("../defaults/handwritten.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "audio_transcription.interview",
        task: Task::AudioTranscription,
        name: "Interview or recording, transcribed",
        description: "Word for word in the languages spoken, with speaker labels and a timestamp at each turn.",
        instructions: include_str!("../defaults/interview-transcription.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "audio_transcription.translate_english",
        task: Task::AudioTranscription,
        name: "Interview or recording, translated into English",
        description: "Everything said, in English, with speakers and timestamps. For another language, duplicate it and change the target language.",
        instructions: include_str!("../defaults/interview-translation-english.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.summary_research",
        task: Task::Analysis,
        name: "Summary and keywords",
        description: "What each document in particular says, with names, figures and dates, as long as the document needs, and keywords to search by.",
        instructions: include_str!("../defaults/summary-research.md"),
        user_prompt: analysis::ANALYSIS_REQUEST,
        schema: Some(analysis::summary_research),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.summary_publication",
        task: Task::Analysis,
        name: "Scholarly publication, social sciences",
        description: "Question, argument, theory, methods, data, case and findings of an article, chapter or book, field by field for a literature review.",
        instructions: include_str!("../defaults/summary-publication.md"),
        user_prompt: analysis::ANALYSIS_REQUEST,
        schema: Some(analysis::summary_publication),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.entities",
        task: Task::Analysis,
        name: "People, places and organisations",
        description: "The people, organisations, places and subjects each document names, spelled as in the text.",
        instructions: include_str!("../defaults/analysis-entities.md"),
        user_prompt: analysis::ANALYSIS_REQUEST,
        schema: Some(analysis::entities),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.tone",
        task: Task::Analysis,
        name: "Tone of the document",
        description: "Whether each document presents its subject positively or negatively, and how much it gives opinions, with a reason for each.",
        instructions: include_str!("../defaults/analysis-tone.md"),
        user_prompt: analysis::ANALYSIS_REQUEST,
        schema: Some(analysis::tone),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.image_description",
        task: Task::Analysis,
        name: "Image description and keywords",
        description: "What each photograph, poster, map or other image shows, as a catalogue would describe it, with keywords, the text written on it and the names that text gives.",
        instructions: include_str!("../defaults/analysis-image-description.md"),
        user_prompt: analysis::IMAGE_REQUEST,
        schema: Some(analysis::image_description),
        merge: None,
        dense: false,
        made_for: Some(InputKind::Image),
    },
    Default {
        id: "analysis.directory",
        task: Task::Analysis,
        name: "Directory or list, one row per entry",
        description: "Every entry on every page, in reading order: the name, occupation, address and remarks as printed, for a city directory, a membership list or a printed table.",
        instructions: include_str!("../defaults/records-directory.md"),
        user_prompt: records::RECORDS_REQUEST,
        schema: Some(records::directory),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.register",
        task: Task::Analysis,
        name: "Register, one row per entry",
        description: "Every entry on every page of a parish or civil register or a ledger: number, date, kind of entry, name, place, the people named and remarks, as written, repetition marks included.",
        instructions: include_str!("../defaults/records-register.md"),
        user_prompt: records::RECORDS_REQUEST,
        schema: Some(records::register),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "analysis.interview",
        task: Task::Analysis,
        name: "Interview: themes and quotations",
        description: "A transcript read for your research: a short summary, the themes, keywords and the passages worth quoting, each quotation found word for word in the transcript with its speaker and time.",
        instructions: include_str!("../defaults/analysis-interview.md"),
        user_prompt: analysis::INTERVIEW_REQUEST,
        schema: Some(analysis::interview),
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "contents.periodical",
        task: Task::Contents,
        name: "Newspaper or magazine issue",
        description: "One entry per article, with its section, authors, pages and a summary.",
        instructions: include_str!("../defaults/contents-periodical-page.md"),
        user_prompt: contents::REQUEST,
        schema: Some(contents::periodical),
        merge: Some(include_str!("../defaults/contents-periodical-merge.md")),
        dense: false,
        made_for: None,
    },
    Default {
        id: "contents.archival_file",
        task: Task::Contents,
        name: "Archival file, item by item",
        description: "An inventory of a file scanned in one piece: each letter, report or clipping with its date, sender, addressee and a summary.",
        instructions: include_str!("../defaults/contents-archival-page.md"),
        user_prompt: contents::REQUEST,
        schema: Some(contents::archival),
        merge: Some(include_str!("../defaults/contents-archival-merge.md")),
        dense: false,
        made_for: None,
    },
    Default {
        id: "contents.book",
        task: Task::Contents,
        name: "Book, report or thesis",
        description: "Parts, chapters and sections with their pages and a summary of each.",
        instructions: include_str!("../defaults/contents-book-page.md"),
        user_prompt: contents::REQUEST,
        schema: Some(contents::book),
        merge: Some(include_str!("../defaults/contents-book-merge.md")),
        dense: false,
        made_for: None,
    },
    Default {
        id: "text_correction.general",
        task: Task::TextCorrection,
        name: "Printed document or newspaper",
        description: "Corrects machine-reading errors only, columns put back in order; spelling, wording and style of the time kept.",
        instructions: include_str!("../defaults/correction-general.md"),
        user_prompt: CORRECTION_REQUEST,
        schema: None,
        merge: None,
        dense: false,
        made_for: None,
    },
    Default {
        id: "translation.research",
        task: Task::Translation,
        name: "Translation for research",
        description: "Everything translated in order, nothing summarised; names kept as written, the original beside culturally specific terms, the layout and a transcript's speakers and times kept.",
        instructions: include_str!("../defaults/translation-research.md"),
        user_prompt: TRANSLATION_REQUEST,
        schema: None,
        merge: None,
        dense: false,
        made_for: None,
    },
];
