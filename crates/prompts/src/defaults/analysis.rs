//! The analysis presets' requests and schemas: the summaries, the entities, the tone and the
//! image description.

use kalamu_core::{AnalysisSchema, FieldKind};

use super::{choice, field, list};

pub(super) const ANALYSIS_REQUEST: &str =
    "Read the document above and fill in each field, following the instructions given.";

pub(super) const IMAGE_REQUEST: &str =
    "Look at the image above and fill in each field, following the instructions given.";

const KEYWORDS: &str = "Keywords and short key phrases in {languages} that someone would search \
for: people, organisations, places, events and specific themes. Prefer the specific (\"Hajj \
1957\") to the general (\"religion\"), and keep proper names as in the source.";

pub(super) fn summary_research() -> AnalysisSchema {
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

/// The parts of a social-science publication a literature review compares: question, argument,
/// theory, method, case and findings, kept apart so each becomes a CSV column.
pub(super) fn summary_publication() -> AnalysisSchema {
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

pub(super) fn entities() -> AnalysisSchema {
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

pub(super) fn tone() -> AnalysisSchema {
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

/// A catalogue entry for an image: what it shows, what it is, keywords, its own text, and the
/// names that text gives. The lists carry the entities preset's labels, so the project
/// dictionary gathers their names with the documents'.
pub(super) fn image_description() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "Description",
                "What is visibly there, following the instructions above, in {languages}; the \
                 first sentence a summary on its own.",
                FieldKind::Text,
                false,
            ),
            field(
                "Kind of image",
                "What the image is as an object, not what it shows.",
                choice(&[
                    "Photograph",
                    "Poster or advertisement",
                    "Map or plan",
                    "Drawing or painting",
                    "Printed page",
                    "Manuscript page",
                    "Object",
                    "Other",
                ]),
                false,
            ),
            field(
                "Keywords",
                "Subjects visibly present, in {languages}, specific before general, with no two \
                 saying the same thing.",
                list(12),
                false,
            ),
            field(
                "Text in the image",
                "Captions, signs, inscriptions and stamps, as written, one per line; empty when \
                 there is none.",
                FieldKind::Text,
                false,
            ),
            field(
                "People",
                "People named in the image's own text.",
                list(20),
                false,
            ),
            field(
                "Places",
                "Places named in the image's own text.",
                list(20),
                false,
            ),
            field(
                "Organisations",
                "Organisations named in the image's own text.",
                list(20),
                false,
            ),
        ],
    }
}
