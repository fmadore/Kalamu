//! The tested default prompts, ported verbatim from zmo-ai-pipelines (`printed-*`) and
//! iwac-ai-pipelines (the transcription and HTR ones). The per-page requests come from the same
//! scripts. The analysis prompts are new, in English: the summaries keep the principles of
//! iwac-ai-pipelines' bilingual summary prompt (specific, faithful, searchable), the entities
//! its NER rules and the tone its sentiment scales (labels, never numbers, with a reason),
//! without their collection-specific parts. The scholarly publication summary has no ancestor
//! there: its fields follow what a social-science literature review compares.
//!
//! The tables of contents port iwac-ai-pipelines' `AI_summary_issue` prompts (page extraction
//! and consolidation), in English and without their Islamic-magazine specifics; the archival
//! file and the book are new, on the same two steps. The newspaper correction is
//! `AI_ocr_correction`'s prompt verbatim; the general one keeps its rules for any printed text.

use kalamu_core::{AnalysisSchema, Field, FieldKind, Task};

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
}

/// A table of contents' first step, sent with each page.
const CONTENTS_REQUEST: &str = "List the articles or items on this page and fill in their fields, following the \
     instructions given.";

const CORRECTION_REQUEST: &str =
    "Correct the machine-read text above, following the instructions given.";

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

/// The parts of a social-science publication a literature review compares: question, argument,
/// theory, method, case and findings, kept apart so each becomes a CSV column.
fn summary_publication() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Reference",
                "Authors, year, title, and the journal, book or institution it appeared in, as \
                 the publication gives them. Leave out what it does not state.",
                FieldKind::Text,
                false,
            ),
            field(
                "Type",
                "What kind of publication this is.",
                choice(&[
                    "Journal article",
                    "Book chapter",
                    "Book",
                    "Thesis",
                    "Working paper or report",
                    "Review",
                    "Other",
                ]),
                false,
            ),
            field(
                "Abstract",
                "An abstract of 150 to 250 words in your own words: the question, the case, the \
                 approach, the argument and the main findings.",
                FieldKind::Text,
                false,
            ),
            field(
                "Research question",
                "The question or puzzle the publication sets out to answer, in one or two \
                 sentences.",
                FieldKind::Text,
                false,
            ),
            field(
                "Argument",
                "The central argument or thesis, and the debate or received view it speaks to or \
                 argues against.",
                FieldKind::Text,
                false,
            ),
            field(
                "Theoretical framework",
                "The theories and concepts the analysis rests on, and the scholars it explicitly \
                 builds on or discusses.",
                FieldKind::Text,
                false,
            ),
            field(
                "Methods",
                "The methods the publication itself uses, not those it only discusses.",
                FieldKind::Choices {
                    values: [
                        "Ethnography and participant observation",
                        "Interviews",
                        "Oral history",
                        "Archival research",
                        "Press or media analysis",
                        "Discourse or content analysis",
                        "Survey",
                        "Statistical analysis",
                        "Comparative case study",
                        "Literature review or theoretical essay",
                    ]
                    .map(String::from)
                    .to_vec(),
                },
                false,
            ),
            field(
                "Data and sources",
                "What the study rests on: fieldwork (where, when, how long), interviews (how many, \
                 with whom), archives, press, surveys or corpora, as the publication describes \
                 them.",
                FieldKind::Text,
                false,
            ),
            field(
                "Places studied",
                "Countries, regions, towns or sites the study is about, not every place mentioned.",
                list(10),
                false,
            ),
            field(
                "Period studied",
                "The period the study covers, as precisely as the publication states it (\"1960 \
                 to 1990\", \"the 2010s\").",
                FieldKind::Text,
                false,
            ),
            field(
                "Main findings",
                "The main findings or claims, each in one sentence, the most important first, as \
                 plain prose.",
                FieldKind::Text,
                false,
            ),
            field(
                "Contribution and limits",
                "What the authors say the publication adds to the field, and the limits they \
                 acknowledge themselves.",
                FieldKind::Text,
                false,
            ),
            field(
                "Key concepts",
                "The analytical concepts the publication uses or proposes, in its own terms.",
                list(8),
                false,
            ),
            field(
                "Keywords",
                "Keywords in {languages} that someone would search for: the topic, groups, \
                 institutions, places and themes. Prefer the specific (\"Islamic reformism in \
                 Niger\") to the general (\"religion\").",
                list(10),
                false,
            ),
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

/// iwac-ai-pipelines' issue index: authors and a summary per article.
fn contents_periodical() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Authors",
                "The authors as the article names them (\"By …\", a signature at the end), \
                 including collective ones (\"The Editors\"); empty when none is named.",
                FieldKind::List { max: None },
                false,
            ),
            field(
                "Summary",
                "What the article says, with its names, figures and dates: two or three \
                 sentences for one page, four to six for the whole article.",
                FieldKind::Text,
                false,
            ),
        ],
    }
}

/// An item-level description of an archival file, as an archivist's inventory gives it.
fn contents_archival() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Kind of document",
                "What the item is, in a few words: letter, report, minutes, telegram, circular, \
                 petition, form, newspaper clipping, photograph, map, envelope…",
                FieldKind::Text,
                false,
            ),
            field(
                "Date",
                "The item's date as written, then in the form YYYY-MM-DD in parentheses when it \
                 is complete; empty when the item has none.",
                FieldKind::Text,
                false,
            ),
            field(
                "From",
                "Who wrote, signed or issued it, with their title or office, as written.",
                FieldKind::List { max: Some(5) },
                false,
            ),
            field(
                "To",
                "Who it is addressed or copied to, with their title or office, as written.",
                FieldKind::List { max: Some(8) },
                false,
            ),
            field(
                "Summary",
                "What the item says or records, in two to four sentences, with its names, places, \
                 figures and dates, and the annotations or stamps on it.",
                FieldKind::Text,
                false,
            ),
            field(
                "Names",
                "People, organisations and places the item names, as written.",
                list(20),
                false,
            ),
        ],
    }
}

fn contents_book() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Level",
                "Whether the entry is a part, a chapter (or a contribution to an edited volume), \
                 or a section within a chapter.",
                choice(&["Part", "Chapter", "Section", "Front or back matter"]),
                false,
            ),
            field(
                "Authors",
                "Its authors, when the entry names its own (in an edited volume); empty otherwise.",
                FieldKind::List { max: None },
                false,
            ),
            field(
                "Summary",
                "What it argues or describes, with its names, cases, figures and dates: one or \
                 two sentences for one page, three to five for a whole chapter or section.",
                FieldKind::Text,
                false,
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
        merge: None,
    },
    Default {
        id: "text_extraction.printed_normalized",
        task: Task::TextExtraction,
        name: "Printed document, normalized",
        description: "Readable text: wording and spelling kept, spacing tidied.",
        instructions: include_str!("../defaults/printed-normalized.md"),
        user_prompt: PAGE_REQUEST,
        schema: None,
        merge: None,
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
        merge: None,
    },
    Default {
        id: "text_extraction.handwriting_french",
        task: Task::TextExtraction,
        name: "Handwritten letter or document (French)",
        description: "French handwriting, keeping historical spelling and French typography.",
        instructions: include_str!("../defaults/handwriting-french.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "text_extraction.manuscript_arabic_script",
        task: Task::TextExtraction,
        name: "Arabic or Ajami manuscript",
        description: "Arabic-script handwriting, read right to left, without adding diacritics.",
        instructions: include_str!("../defaults/manuscript-arabic-script.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "text_extraction.handwriting_multilingual",
        task: Task::TextExtraction,
        name: "Handwriting, several languages or scripts",
        description: "Detects each language and script and keeps each in its reading direction.",
        instructions: include_str!("../defaults/handwriting-multilingual.md"),
        user_prompt: HANDWRITING_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "audio_transcription.interview",
        task: Task::AudioTranscription,
        name: "Interview, verbatim with speakers",
        description: "Speaker labels and a timestamp at each turn; code-switching kept as spoken.",
        instructions: include_str!("../defaults/interview-transcription.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "audio_transcription.translate_english",
        task: Task::AudioTranscription,
        name: "Interview, translated into English",
        description: "An English translation of the recording, with speakers and timestamps.",
        instructions: include_str!("../defaults/interview-translation-english.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "audio_transcription.translate_french",
        task: Task::AudioTranscription,
        name: "Interview, translated into French",
        description: "A French translation of the recording, with speakers and timestamps.",
        instructions: include_str!("../defaults/interview-translation-french.md"),
        user_prompt: RECORDING_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "analysis.summary_research",
        task: Task::Analysis,
        name: "Summary and keywords for research",
        description: "What this document in particular says, with names, figures and dates, and searchable keywords.",
        instructions: include_str!("../defaults/summary-research.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(summary_research),
        merge: None,
    },
    Default {
        id: "analysis.summary_brief",
        task: Task::Analysis,
        name: "Short summary for a catalogue",
        description: "Two or three dense sentences that stand out in a list of search results.",
        instructions: include_str!("../defaults/summary-brief.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(summary_brief),
        merge: None,
    },
    Default {
        id: "analysis.summary_publication",
        task: Task::Analysis,
        name: "Scholarly publication, social sciences",
        description: "Question, argument, theory, methods, data, case and findings of an article, chapter or book, field by field for a literature review.",
        instructions: include_str!("../defaults/summary-publication.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(summary_publication),
        merge: None,
    },
    Default {
        id: "analysis.entities",
        task: Task::Analysis,
        name: "People, places and organisations",
        description: "The people, organisations, places and subjects each document names, spelled as in the text.",
        instructions: include_str!("../defaults/analysis-entities.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(entities),
        merge: None,
    },
    Default {
        id: "analysis.tone",
        task: Task::Analysis,
        name: "Tone of the document",
        description: "Whether each document presents its subject positively or negatively, and how much it gives opinions, with a reason for each.",
        instructions: include_str!("../defaults/analysis-tone.md"),
        user_prompt: ANALYSIS_REQUEST,
        schema: Some(tone),
        merge: None,
    },
    Default {
        id: "contents.periodical",
        task: Task::Contents,
        name: "Newspaper or magazine issue",
        description: "One entry per article, with its section, authors, pages and a summary.",
        instructions: include_str!("../defaults/contents-periodical-page.md"),
        user_prompt: CONTENTS_REQUEST,
        schema: Some(contents_periodical),
        merge: Some(include_str!("../defaults/contents-periodical-merge.md")),
    },
    Default {
        id: "contents.archival_file",
        task: Task::Contents,
        name: "Archival file, item by item",
        description: "An inventory of a file scanned in one piece: each letter, report or clipping with its date, sender, addressee and a summary.",
        instructions: include_str!("../defaults/contents-archival-page.md"),
        user_prompt: CONTENTS_REQUEST,
        schema: Some(contents_archival),
        merge: Some(include_str!("../defaults/contents-archival-merge.md")),
    },
    Default {
        id: "contents.book",
        task: Task::Contents,
        name: "Book, report or thesis",
        description: "Parts, chapters and sections with their pages and a summary of each.",
        instructions: include_str!("../defaults/contents-book-page.md"),
        user_prompt: CONTENTS_REQUEST,
        schema: Some(contents_book),
        merge: Some(include_str!("../defaults/contents-book-merge.md")),
    },
    Default {
        id: "text_correction.general",
        task: Task::TextCorrection,
        name: "Printed document",
        description: "Corrects machine-reading errors only; spelling, wording and style of the time kept.",
        instructions: include_str!("../defaults/correction-general.md"),
        user_prompt: CORRECTION_REQUEST,
        schema: None,
        merge: None,
    },
    Default {
        id: "text_correction.newspaper",
        task: Task::TextCorrection,
        name: "Newspaper (columns, headlines)",
        description: "Also restores the reading order of columns and flags unclear names, figures and quotes.",
        instructions: include_str!("../defaults/correction-newspaper.md"),
        user_prompt: CORRECTION_REQUEST,
        schema: None,
        merge: None,
    },
];
