use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An ISO 639 language code: two letters where one exists (`fr`), three otherwise (`mos`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(transparent)]
#[ts(export)]
pub struct LanguageCode(pub String);

impl LanguageCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into().to_ascii_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The languages a job expects in its sources.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(tag = "mode", content = "languages", rename_all = "snake_case")]
#[ts(export)]
pub enum LanguageChoice {
    /// Let the model detect the language.
    #[default]
    Auto,
    /// One or more languages chosen by the user, for mixed material.
    Explicit(Vec<LanguageCode>),
}

impl LanguageChoice {
    pub fn codes(&self) -> &[LanguageCode] {
        match self {
            Self::Auto => &[],
            Self::Explicit(codes) => codes,
        }
    }
}
