//! The tables of contents' first step and their entry fields: a periodical's issue, an
//! archival file and a book.

use kalamu_core::{AnalysisSchema, FieldKind};

use super::{choice, field, list};

/// A table of contents' first step, sent with each page.
pub(super) const REQUEST: &str = "List the articles or items on this page and fill in their fields, following the \
     instructions given.";

/// iwac-ai-pipelines' issue index: authors and a summary per article.
pub(super) fn periodical() -> AnalysisSchema {
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
pub(super) fn archival() -> AnalysisSchema {
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

pub(super) fn book() -> AnalysisSchema {
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
