use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SourceId;

/// The broad kind of a source file, from its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InputKind {
    Pdf,
    Image,
    Audio,
    Video,
    Text,
}

impl InputKind {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        Some(match ext.as_str() {
            "pdf" => Self::Pdf,
            "png" | "jpg" | "jpeg" | "tif" | "tiff" | "webp" | "bmp" | "heic" | "heif" => {
                Self::Image
            }
            "mp3" | "wav" | "m4a" | "aac" | "flac" | "ogg" | "opus" | "wma" | "aiff" | "aif" => {
                Self::Audio
            }
            "mp4" | "mov" | "mkv" | "avi" | "webm" | "wmv" | "m4v" => Self::Video,
            "txt" | "md" | "docx" | "xlsx" | "csv" => Self::Text,
            _ => return None,
        })
    }
}

/// A file added to a job, identified by the hash of its content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceFile {
    pub id: SourceId,
    /// Absolute path on this PC.
    pub path: String,
    pub file_name: String,
    /// SHA-256 of the file's bytes, lowercase hex. Part of every cache key.
    pub sha256: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    pub kind: InputKind,
    /// Pages in a PDF or multi-page TIFF, once known.
    pub page_count: Option<u32>,
    /// Length of an audio or video file, once known.
    #[ts(type = "number | null")]
    pub duration_ms: Option<u64>,
}
