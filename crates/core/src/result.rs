use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{FieldAnswer, JobId, LanguageCode, ProviderId, SetupId, SourceId, Timestamp};

/// One half of an open double-page spread, as seen on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PageHalf {
    Left,
    Right,
}

/// Which part of a source a piece covers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum PieceRef {
    /// One page of a PDF or image, numbered from 1 as in the original; `half` when a spread was
    /// split into two pages.
    Page {
        number: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        half: Option<PageHalf>,
    },
    /// One segment of a recording.
    Segment {
        index: u32,
        #[ts(type = "number")]
        start_ms: u64,
        #[ts(type = "number")]
        end_ms: u64,
    },
    /// A whole document, read at once: what an analysis covers.
    Whole,
}

impl PieceRef {
    pub fn page(number: u32) -> Self {
        Self::Page { number, half: None }
    }

    /// How the piece is labelled in exports: `3`, `3 (left)`, `00:20:00`.
    pub fn label(&self) -> String {
        match self {
            Self::Page { number, half: None } => number.to_string(),
            Self::Page {
                number,
                half: Some(PageHalf::Left),
            } => format!("{number} (left)"),
            Self::Page {
                number,
                half: Some(PageHalf::Right),
            } => format!("{number} (right)"),
            Self::Segment { start_ms, .. } => clock(*start_ms),
            Self::Whole => String::new(),
        }
    }

    /// A stable string for cache keys and database rows.
    pub fn descriptor(&self) -> String {
        match self {
            Self::Page { number, half: None } => format!("page:{number}"),
            Self::Page {
                number,
                half: Some(half),
            } => format!("page:{number}:{half:?}").to_lowercase(),
            Self::Segment {
                index,
                start_ms,
                end_ms,
            } => format!("segment:{index}:{start_ms}-{end_ms}"),
            Self::Whole => "whole".to_string(),
        }
    }
}

/// `hh:mm:ss` for a position in a recording.
pub fn clock(ms: u64) -> String {
    let s = ms / 1000;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// A timed stretch of a transcription.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Segment {
    #[ts(type = "number")]
    pub start_ms: u64,
    #[ts(type = "number")]
    pub end_ms: u64,
    pub speaker: Option<String>,
    pub text: String,
}

/// Where a block of text sits on its page, for a searchable PDF: fractions of the page's width
/// and height, measured from its top-left corner as displayed. Only models that report layout
/// (Mistral OCR) give them, at paragraph level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TextRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}

/// What a provider call consumed, for cost tracking.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Usage {
    #[ts(type = "number")]
    pub input_tokens: u64,
    #[ts(type = "number")]
    pub output_tokens: u64,
    /// Seconds of audio billed, for per-minute pricing.
    pub audio_seconds: f64,
    /// Pages billed, for per-page pricing.
    pub pages: u32,
}

impl Usage {
    pub fn add(&mut self, other: &Usage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.audio_seconds += other.audio_seconds;
        self.pages += other.pages;
    }
}

/// The result of processing one piece.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProcessOutput {
    pub text: String,
    /// Timed segments for transcriptions, with times relative to the start of the piece;
    /// empty for text extraction.
    #[serde(default)]
    pub segments: Vec<Segment>,
    pub detected_language: Option<LanguageCode>,
    #[serde(default)]
    pub usage: Usage,
    /// The exact model version the provider reports, which may differ from the name requested.
    pub model_version: Option<String>,
    /// An analysis's answers, one per field of its schema; `text` holds them as plain text.
    #[serde(default)]
    pub fields: Vec<FieldAnswer>,
    /// Text blocks and their place on the page, from models that report layout.
    #[serde(default)]
    pub regions: Vec<TextRegion>,
}

/// A cached result, keyed by [`crate::CacheKey`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoredResult {
    pub cache_key: String,
    pub provider: ProviderId,
    pub model: String,
    pub output: ProcessOutput,
    pub cost_usd: Option<f64>,
    #[ts(type = "number")]
    pub duration_ms: u64,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PieceStatus {
    Pending,
    Running,
    Done,
    /// Done earlier with the same source and settings; not sent again.
    Cached,
    Failed,
}

impl PieceStatus {
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Done | Self::Cached)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Cached => "cached",
            Self::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => Self::Pending,
            "running" => Self::Running,
            "done" => Self::Done,
            "cached" => Self::Cached,
            "failed" => Self::Failed,
            _ => return None,
        })
    }
}

/// Progress of one piece of one source under one set-up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PieceState {
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    /// Position within the source, from 0.
    pub index: u32,
    pub piece: PieceRef,
    pub status: PieceStatus,
    pub cache_key: Option<String>,
    /// A readable reason when the piece failed.
    pub error: Option<String>,
    pub attempts: u32,
    pub updated_at: Timestamp,
}
