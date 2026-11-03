//! Export profiles (#55): the columns of a project's descriptions export chosen, ordered and
//! renamed by the researcher, with how lists, missing values and line breaks are written, saved
//! to be used again on any project. A profile holds no project's values, only where each column
//! takes its value from, and texts the researcher fixed for every row.
//!
//! A column can also take a value from the document's result for a task: the one the
//! researcher preferred, or else the only one there is. Read from what is stored, corrections
//! applied: another profile never calls a model.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{DescriptionField, Task, Timestamp};

/// Where a column takes its value from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum ProfileValue {
    DocumentId,
    Title,
    FileName,
    Path,
    /// The project's name.
    Project,
    /// The names of the collections the document is in: a list.
    Collections,
    /// The document's tags: a list.
    Tags,
    /// A part of the description as it stands, the document's own or the project's.
    Part {
        field: DescriptionField,
    },
    /// Where that part comes from: `document` or `project`.
    From {
        field: DescriptionField,
    },
    /// The same text in every row (an item set, a resource template, a licence).
    Fixed {
        text: String,
    },
    /// How far the researcher reviewed the document's result for a task: `reviewed`,
    /// `partly_reviewed` or `not_reviewed`.
    Status {
        task: Task,
    },
    /// The model that read that result.
    Model {
        task: Task,
    },
    /// That result's text as corrected, its pages or segments apart by a blank line; only for a
    /// task that yields text.
    Text {
        task: Task,
    },
    /// An answer of the document's analysis, by its field's key, as corrected; a list stays a
    /// list.
    Answer {
        key: String,
    },
    /// The same answer with each listed form written under the label of the project
    /// dictionary's entry that holds it; a form no entry holds is kept as the model wrote it,
    /// and a label reached by two forms is written once.
    Labelled {
        key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProfileColumn {
    pub heading: String,
    pub value: ProfileValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportProfile {
    /// Empty for a new profile; given when it is first saved.
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub columns: Vec<ProfileColumn>,
    /// Between the items of a list in a CSV cell; JSON keeps lists as arrays.
    #[serde(default = "default_separator")]
    pub list_separator: String,
    /// Written in a CSV cell that has no value; JSON writes `null`.
    #[serde(default)]
    pub missing: String,
    /// Line breaks inside a value replaced by a space, for tools that read one line per row.
    #[serde(default)]
    pub join_lines: bool,
    /// When it was last saved; the interface sends 0 and saving sets it.
    pub updated_at: Timestamp,
}

fn default_separator() -> String {
    "; ".into()
}

/// What a new profile starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProfileStarter {
    /// Every column of the default export, under the same headings.
    Every,
    /// The parts that have a Dublin Core term, under `dcterms:` headings, to adjust.
    DublinCore,
}

/// What keeps a profile from being saved, as the interface shows it beside the column at fault.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum ProfileError {
    #[error("An export profile needs a name")]
    NoName,
    #[error("An export profile needs at least one column")]
    NoColumns,
    /// Columns are numbered from 1, as the interface shows them.
    #[error("Column {column} has no heading")]
    Unnamed { column: u32 },
    #[error("The heading of column {column} holds a line break or a control character")]
    BadHeading { column: u32 },
    #[error("Column {column} has the same heading as column {first}")]
    Duplicate { column: u32, first: u32 },
    #[error("The list separator is empty or holds a line break")]
    BadSeparator,
    /// A result's text for a task that yields none, or an answer with no field's key.
    #[error("Column {column} takes a value Kalamu cannot give")]
    BadValue { column: u32 },
}

impl ProfileError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoName => "profile.no_name",
            Self::NoColumns => "profile.no_columns",
            Self::Unnamed { .. } => "profile.unnamed_column",
            Self::BadHeading { .. } => "profile.bad_heading",
            Self::Duplicate { .. } => "profile.duplicate_heading",
            Self::BadSeparator => "profile.bad_separator",
            Self::BadValue { .. } => "profile.bad_value",
        }
    }

    /// The column at fault, numbered from 1.
    pub fn column(&self) -> Option<u32> {
        match self {
            Self::Unnamed { column }
            | Self::BadHeading { column }
            | Self::Duplicate { column, .. }
            | Self::BadValue { column } => Some(*column),
            _ => None,
        }
    }
}

impl ProfileValue {
    /// Whether the value comes from the document's result for a task rather than the document.
    pub fn reads_result(&self) -> bool {
        matches!(
            self,
            Self::Status { .. }
                | Self::Model { .. }
                | Self::Text { .. }
                | Self::Answer { .. }
                | Self::Labelled { .. }
        )
    }

    /// Whether the value is written through the project dictionary's labels.
    pub fn reads_labels(&self) -> bool {
        matches!(self, Self::Labelled { .. })
    }
}

impl ExportProfile {
    /// The profile with its name, headings and answers' keys trimmed.
    pub fn tidied(mut self) -> Self {
        self.name = self.name.trim().to_string();
        for column in &mut self.columns {
            column.heading = column.heading.trim().to_string();
            if let ProfileValue::Answer { key } | ProfileValue::Labelled { key } = &mut column.value
            {
                *key = key.trim().to_string();
            }
        }
        self
    }

    /// Checks a tidied profile: a name, columns, a heading for each that no other column has
    /// (whatever its capitals, which spreadsheets and importers ignore) and that fits on one
    /// line, a value Kalamu can give (a text for a task that yields one, an answer's key), and a
    /// separator that fits on one line. A value may be taken by two columns.
    pub fn check(&self) -> Result<(), ProfileError> {
        if self.name.is_empty() {
            return Err(ProfileError::NoName);
        }
        if self.columns.is_empty() {
            return Err(ProfileError::NoColumns);
        }
        let mut seen: Vec<String> = Vec::new();
        for (i, column) in self.columns.iter().enumerate() {
            let number = i as u32 + 1;
            let given = match &column.value {
                ProfileValue::Text { task } => task.yields_text(),
                ProfileValue::Answer { key } | ProfileValue::Labelled { key } => {
                    !key.is_empty() && !key.chars().any(char::is_control)
                }
                _ => true,
            };
            if !given {
                return Err(ProfileError::BadValue { column: number });
            }
            if column.heading.is_empty() {
                return Err(ProfileError::Unnamed { column: number });
            }
            if column.heading.chars().any(char::is_control) {
                return Err(ProfileError::BadHeading { column: number });
            }
            let folded = column.heading.to_lowercase();
            if let Some(first) = seen.iter().position(|h| *h == folded) {
                return Err(ProfileError::Duplicate {
                    column: number,
                    first: first as u32 + 1,
                });
            }
            seen.push(folded);
        }
        if self.list_separator.is_empty() || self.list_separator.contains(['\n', '\r']) {
            return Err(ProfileError::BadSeparator);
        }
        Ok(())
    }

    /// The same profile with every fixed text emptied: what a template shared without one
    /// project's values keeps.
    pub fn without_fixed_texts(mut self) -> Self {
        for column in &mut self.columns {
            if let ProfileValue::Fixed { text } = &mut column.value {
                text.clear();
            }
        }
        self
    }
}

impl ProfileStarter {
    pub fn columns(self) -> Vec<ProfileColumn> {
        let column = |heading: &str, value| ProfileColumn {
            heading: heading.into(),
            value,
        };
        match self {
            Self::Every => {
                let mut out = vec![
                    column("document_id", ProfileValue::DocumentId),
                    column("title", ProfileValue::Title),
                    column("file_name", ProfileValue::FileName),
                    column("path", ProfileValue::Path),
                    column("collections", ProfileValue::Collections),
                    column("tags", ProfileValue::Tags),
                ];
                for field in DescriptionField::ALL {
                    out.push(column(field.as_str(), ProfileValue::Part { field }));
                    out.push(column(
                        &format!("{}_from", field.as_str()),
                        ProfileValue::From { field },
                    ));
                }
                out
            }
            Self::DublinCore => {
                let part = |field| ProfileValue::Part { field };
                vec![
                    column("dcterms:identifier", part(DescriptionField::Shelfmark)),
                    column("dcterms:title", ProfileValue::Title),
                    column("dcterms:creator", part(DescriptionField::Creator)),
                    column("dcterms:date", part(DescriptionField::Date)),
                    column("dcterms:isPartOf", part(DescriptionField::Fonds)),
                    column("dcterms:source", part(DescriptionField::Url)),
                    column(
                        "dcterms:bibliographicCitation",
                        part(DescriptionField::Citation),
                    ),
                    column("dcterms:rights", part(DescriptionField::Rights)),
                    column("dcterms:subject", ProfileValue::Tags),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(headings: &[&str]) -> ExportProfile {
        ExportProfile {
            id: String::new(),
            name: "Omeka".into(),
            columns: headings
                .iter()
                .map(|h| ProfileColumn {
                    heading: (*h).into(),
                    value: ProfileValue::Title,
                })
                .collect(),
            list_separator: default_separator(),
            missing: String::new(),
            join_lines: false,
            updated_at: Timestamp(0),
        }
    }

    #[test]
    fn a_profile_needs_a_name_columns_and_one_line_headings_of_its_own() {
        assert_eq!(profile(&["dcterms:title", "Titre"]).check(), Ok(()));
        let mut unnamed = profile(&["a"]);
        unnamed.name.clear();
        assert_eq!(unnamed.check(), Err(ProfileError::NoName));
        assert_eq!(profile(&[]).check(), Err(ProfileError::NoColumns));
        assert_eq!(
            profile(&["a", " "]).tidied().check(),
            Err(ProfileError::Unnamed { column: 2 })
        );
        assert_eq!(
            profile(&["a\nb"]).check(),
            Err(ProfileError::BadHeading { column: 1 })
        );
        assert_eq!(
            profile(&["Cote", "titre", "Côte", "COTE"]).check(),
            Err(ProfileError::Duplicate {
                column: 4,
                first: 1
            }),
            "capitals do not tell two headings apart, accents do"
        );
        let mut separator = profile(&["a"]);
        separator.list_separator = "\n".into();
        assert_eq!(separator.check(), Err(ProfileError::BadSeparator));
    }

    #[test]
    fn a_result_column_takes_a_value_kalamu_can_give() {
        let mut p = profile(&["texte", "personnes", "statut"]);
        p.columns[0].value = ProfileValue::Text {
            task: Task::TextExtraction,
        };
        p.columns[1].value = ProfileValue::Answer {
            key: " people ".into(),
        };
        p.columns[2].value = ProfileValue::Status {
            task: Task::Analysis,
        };
        let p = p.tidied();
        assert_eq!(p.check(), Ok(()));
        assert_eq!(
            p.columns[1].value,
            ProfileValue::Answer {
                key: "people".into()
            }
        );

        let mut analysis = p.clone();
        analysis.columns[0].value = ProfileValue::Text {
            task: Task::Analysis,
        };
        assert_eq!(analysis.check(), Err(ProfileError::BadValue { column: 1 }));
        let mut keyless = p.clone();
        keyless.columns[1].value = ProfileValue::Answer { key: "  ".into() };
        assert_eq!(
            keyless.tidied().check(),
            Err(ProfileError::BadValue { column: 2 })
        );
        let mut labelled = p;
        labelled.columns[1].value = ProfileValue::Labelled { key: "".into() };
        assert_eq!(
            labelled.clone().tidied().check(),
            Err(ProfileError::BadValue { column: 2 })
        );
        labelled.columns[1].value = ProfileValue::Labelled {
            key: " people ".into(),
        };
        let labelled = labelled.tidied();
        assert_eq!(labelled.check(), Ok(()));
        assert!(labelled.columns[1].value.reads_labels());
    }

    #[test]
    fn the_starters_are_valid_profiles() {
        for starter in [ProfileStarter::Every, ProfileStarter::DublinCore] {
            let mut p = profile(&[]);
            p.columns = starter.columns();
            assert_eq!(p.check(), Ok(()), "{starter:?}");
        }
        assert_eq!(ProfileStarter::Every.columns().len(), 26);
    }

    #[test]
    fn a_template_keeps_its_columns_but_not_its_fixed_texts() {
        let mut p = profile(&["a"]);
        p.columns.push(ProfileColumn {
            heading: "o:item_set".into(),
            value: ProfileValue::Fixed {
                text: "Fonds Mamadou".into(),
            },
        });
        let shared = p.clone().without_fixed_texts();
        assert_eq!(shared.columns.len(), 2);
        assert_eq!(
            shared.columns[1].value,
            ProfileValue::Fixed {
                text: String::new()
            }
        );
        assert_eq!(shared.columns[0], p.columns[0]);
    }
}
