use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SourceId;

/// Something about an input that is likely to hurt the result. Never blocks a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum WarningKind {
    /// Pages that look like an open double-page spread (wider than tall).
    DoublePageSpread { pages: Vec<u32> },
    /// Pages whose resolution is too low for small print or handwriting.
    LowResolution { pages: Vec<u32>, dpi: u32 },
    /// A PDF that already has a text layer.
    HasTextLayer { pages_with_text: u32 },
    /// An image file the app cannot decode or convert yet.
    UnsupportedImage { detail: String },
    /// A file larger than the provider accepts in one request, before splitting.
    LargeFile {
        #[ts(type = "number")]
        size_bytes: u64,
    },
    /// A video or an audio format that needs FFmpeg, which was not found.
    ToolMissing,
    /// A video or recording with no sound to transcribe.
    NoSoundTrack,
    /// A recording long enough to be split into several pieces.
    LongRecording { minutes: u32, pieces: u32 },
    /// Speech so quiet that words may be missed.
    QuietRecording { mean_db: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InputWarning {
    pub source_id: SourceId,
    #[serde(flatten)]
    pub kind: WarningKind,
}
