use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{PieceRef, Provenance, Segment, Setup, SourceFile, Task};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ExportFormat {
    Txt,
    Markdown,
    Docx,
    Json,
    Srt,
    Vtt,
    Csv,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Markdown => "md",
            Self::Docx => "docx",
            Self::Json => "json",
            Self::Srt => "srt",
            Self::Vtt => "vtt",
            Self::Csv => "csv",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportOptions {
    /// Mark where each page or segment starts.
    pub piece_markers: bool,
    pub speaker_labels: bool,
    pub timestamps: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            piece_markers: true,
            speaker_labels: true,
            timestamps: true,
        }
    }
}

/// One page or segment, with the user's correction applied when there is one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportPart {
    pub piece: PieceRef,
    pub text: String,
    /// Timed segments, with times relative to the start of the recording.
    pub segments: Vec<Segment>,
    pub corrected: bool,
    pub reviewed: bool,
    /// Keywords, for summaries.
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Everything an exporter needs for one source under one set-up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDocument {
    pub task: Task,
    pub source: SourceFile,
    pub setup: Setup,
    pub parts: Vec<ExportPart>,
    pub provenance: Provenance,
    pub options: ExportOptions,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("{0} export is not available for this task")]
    Unsupported(&'static str),
    #[error("Could not write the {format} file: {message}")]
    Write {
        format: &'static str,
        message: String,
    },
}

/// Turns stored results into one output format. Adding a format means adding an exporter and
/// nothing else.
pub trait Exporter: Send + Sync {
    fn format(&self) -> ExportFormat;

    fn supports(&self, task: Task) -> bool;

    fn export(&self, document: &ExportDocument) -> Result<Vec<u8>, ExportError>;
}
