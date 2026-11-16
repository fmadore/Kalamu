//! The ready-made records analyses (#171): one repeating group each, read page by page, one row
//! per entry. Every column is read as written, repetition marks kept and nothing carried down
//! from a heading, so a row says what its entry says; whether a ditto mark or a street printed
//! above should fill the cells it stands for is left open.

use kalamu_core::{AnalysisSchema, Field, FieldKind};

use super::{choice, field};

/// Sent with each page of a records analysis.
pub(super) const RECORDS_REQUEST: &str = "List every entry on this page, in reading order, and \
fill in its fields, following the instructions given.";

fn rows(instructions: &str, columns: Vec<Field>) -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![field(
            "Entries",
            instructions,
            FieldKind::Entries { fields: columns },
            false,
        )],
    }
}

/// A city or trade directory, a membership list, a printed table of names.
pub(super) fn directory() -> AnalysisSchema {
    rows(
        "Every entry on this page, one row each, in reading order, column by column.",
        vec![
            field(
                "Name",
                "The name as printed, surname and forenames in the order given.",
                FieldKind::Text,
                false,
            ),
            field(
                "Occupation",
                "The occupation, trade or title as printed; empty when none.",
                FieldKind::Text,
                false,
            ),
            field(
                "Address",
                "The street and number, or the place, as printed; empty when none.",
                FieldKind::Text,
                false,
            ),
            field(
                "Remarks",
                "Anything else the entry gives (a telephone number, a date, an abbreviation), \
                 as printed; empty when nothing.",
                FieldKind::Text,
                false,
            ),
        ],
    )
}

/// A parish or civil register, a register of admissions or of letters, a ledger.
pub(super) fn register() -> AnalysisSchema {
    rows(
        "Every entry on this page, one row each, in the order written.",
        vec![
            field(
                "Number",
                "The entry's number as written; empty when none.",
                FieldKind::Text,
                false,
            ),
            field(
                "Date",
                "The entry's date as written.",
                FieldKind::Text,
                false,
            ),
            field(
                "Kind of entry",
                "What the entry records.",
                choice(&["Birth or baptism", "Marriage", "Death or burial", "Other"]),
                false,
            ),
            field(
                "Name",
                "The person the entry is about, as written.",
                FieldKind::Text,
                false,
            ),
            field(
                "Place",
                "The place the entry names for them, as written; empty when none.",
                FieldKind::Text,
                false,
            ),
            field(
                "People named",
                "The other people the entry names (parents, spouse, witnesses, the clerk), \
                 each as written.",
                FieldKind::List { max: None },
                false,
            ),
            field(
                "Remarks",
                "Marginal notes and anything else the entry records, as written; empty when \
                 nothing.",
                FieldKind::Text,
                false,
            ),
        ],
    )
}
