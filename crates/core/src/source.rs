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
    /// A YouTube video added by its link: no file, read by the provider from its address.
    #[serde(rename = "youtube")]
    YouTube,
}

impl InputKind {
    /// The file extensions of each kind of file, lowercase: the one table [`Self::from_path`]
    /// reads, and the one the interface's file pickers are given (`coreLists.json`), so a file
    /// is offered exactly when it is read. A YouTube video has no file, so none.
    pub const EXTENSIONS: [(InputKind, &'static [&'static str]); 5] = [
        (Self::Pdf, &["pdf"]),
        (
            Self::Image,
            &[
                "png", "jpg", "jpeg", "tif", "tiff", "webp", "bmp", "heic", "heif",
            ],
        ),
        (
            Self::Audio,
            &[
                "mp3", "wav", "m4a", "aac", "flac", "ogg", "opus", "wma", "aiff", "aif",
            ],
        ),
        (
            Self::Video,
            &["mp4", "mov", "mkv", "avi", "webm", "wmv", "m4v"],
        ),
        // What `kalamu-media`'s text source opens: a spreadsheet (`.xlsx`) is not read.
        (Self::Text, &["txt", "md", "docx", "csv"]),
    ];

    /// The extensions of files of this kind.
    pub fn extensions(self) -> &'static [&'static str] {
        Self::EXTENSIONS
            .iter()
            .find(|(kind, _)| *kind == self)
            .map_or(&[], |(_, extensions)| *extensions)
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        Self::EXTENSIONS
            .iter()
            .find(|(_, extensions)| extensions.contains(&ext.as_str()))
            .map(|(kind, _)| *kind)
    }
}

/// A file added to a job, identified by the hash of its content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceFile {
    pub id: SourceId,
    /// Absolute path on this PC, or the address of a YouTube video.
    pub path: String,
    pub file_name: String,
    /// SHA-256 of the file's bytes, lowercase hex, or of a YouTube video's id. Part of every
    /// cache key.
    pub sha256: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// When the file was last modified, in milliseconds since 1970, as it was when hashed: a
    /// run reads the file again only if its size or this changed since.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub modified_ms: Option<i64>,
    pub kind: InputKind,
    /// Pages in a PDF or multi-page TIFF, once known.
    pub page_count: Option<u32>,
    /// Length of an audio or video file, once known.
    #[ts(type = "number | null")]
    pub duration_ms: Option<u64>,
    /// Split open double-page spreads into two pages before processing.
    #[serde(default)]
    pub split_spreads: bool,
    /// Even out a recording's volume before transcription (quiet or uneven speakers).
    #[serde(default)]
    pub even_out_volume: bool,
    /// A YouTube video's title, as YouTube gives it, when it could be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    /// A frozen copy of an earlier result's corrected text, read instead of a file of the
    /// researcher's: which result, and the original beside which results are written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub derived_from: Option<crate::DerivedFrom>,
    /// For a document of image files read as its pages, the files in the document's order
    /// (#52); empty for every other source. `path` then names the document in its first
    /// page's folder and `sha256` is [`crate::ordered::manifest_sha256`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<crate::PageFile>>")]
    pub pages: Vec<crate::PageFile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_extension_reads_as_its_kind_in_any_case() {
        for (kind, extensions) in InputKind::EXTENSIONS {
            assert!(!extensions.is_empty(), "{kind:?}");
            assert_eq!(kind.extensions(), extensions);
            for ext in extensions {
                assert_eq!(*ext, ext.to_ascii_lowercase(), "{ext}");
                for name in [
                    format!("a.{ext}"),
                    format!("A.{}", ext.to_ascii_uppercase()),
                ] {
                    assert_eq!(InputKind::from_path(Path::new(&name)), Some(kind), "{name}");
                }
            }
        }
    }

    #[test]
    fn no_extension_belongs_to_two_kinds() {
        let all: Vec<&str> = InputKind::EXTENSIONS
            .iter()
            .flat_map(|(_, e)| e.iter().copied())
            .collect();
        let unique: std::collections::BTreeSet<&str> = all.iter().copied().collect();
        assert_eq!(all.len(), unique.len());
    }

    #[test]
    fn files_kalamu_does_not_read_have_no_kind() {
        for name in [
            "table.xlsx",
            "notes.odt",
            "archive.zip",
            "README",
            "dossier.",
        ] {
            assert_eq!(InputKind::from_path(Path::new(name)), None, "{name}");
        }
        assert!(InputKind::YouTube.extensions().is_empty());
    }
}
