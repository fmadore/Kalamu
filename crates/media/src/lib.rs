//! Turning source files into pieces a provider accepts, and checking inputs before a job.
//!
//! PDFs are cut into single-page PDFs, never rendered, as in zmo- and iwac-ai-pipelines: every
//! first-stage provider reads PDF pages directly, at full quality. Images are passed through
//! when the provider takes their format, and converted otherwise. Originals are only read,
//! never modified.

mod checks;
mod image_source;
mod pdf;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use scriptorium_core::{InputKind, InputWarning, PieceRef, PreparedInput, SourceFile, SourceId};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub use image_source::ImageSource;
pub use pdf::PdfSource;
#[cfg(any(test, feature = "test-support"))]
pub use pdf::sample_pdf;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Could not read {path}: {message}")]
    Read { path: String, message: String },
    #[error("This file type is not supported yet")]
    UnsupportedKind,
    #[error("The PDF could not be opened: {0}")]
    Pdf(String),
    #[error("The PDF is password-protected")]
    Encrypted,
    #[error("The image could not be read: {0}")]
    Image(String),
    #[error("Page {0} does not exist in this file")]
    NoSuchPage(String),
    #[error("{0}")]
    NeedsConversion(String),
}

/// Hashes a file and works out what it is, without loading it all into memory twice.
pub fn inspect(path: &Path) -> Result<SourceFile, MediaError> {
    let read_err = |e: std::io::Error| MediaError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    };
    let kind = InputKind::from_path(path).ok_or(MediaError::UnsupportedKind)?;
    let size_bytes = std::fs::metadata(path).map_err(read_err)?.len();
    let sha256 = hash_file(path)?;
    let page_count = match kind {
        InputKind::Pdf => Some(PdfSource::open(path)?.page_count()),
        InputKind::Image => ImageSource::open(path).map(|i| i.page_count()).ok(),
        _ => None,
    };
    Ok(SourceFile {
        id: SourceId::new(),
        path: path.display().to_string(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        sha256,
        size_bytes,
        kind,
        page_count,
        duration_ms: None,
        split_spreads: false,
    })
}

pub fn hash_file(path: &Path) -> Result<String, MediaError> {
    let read_err = |e: std::io::Error| MediaError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    };
    let mut reader = BufReader::new(File::open(path).map_err(read_err)?);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader.read(&mut buf).map_err(read_err)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// A source opened for processing: parsed once, then asked for one piece at a time.
pub enum OpenSource {
    Pdf(Box<PdfSource>),
    Image(ImageSource),
    /// A recording sent whole, in its own format. Splitting long recordings and converting
    /// formats a provider does not take need the bundled FFmpeg, which comes later.
    Audio(AudioSource),
}

pub struct AudioSource {
    path: std::path::PathBuf,
    mime: &'static str,
    duration_ms: u64,
}

/// The MIME type providers expect for an audio file, from its extension.
pub fn audio_mime(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "ogg" | "opus" => "audio/ogg",
        "webm" => "audio/webm",
        _ => return None,
    })
}

impl OpenSource {
    pub fn open(source: &SourceFile) -> Result<Self, MediaError> {
        let path = Path::new(&source.path);
        match source.kind {
            InputKind::Pdf => Ok(Self::Pdf(Box::new(PdfSource::open(path)?))),
            InputKind::Image => Ok(Self::Image(ImageSource::open(path)?)),
            InputKind::Audio => Ok(Self::Audio(AudioSource {
                path: path.to_path_buf(),
                mime: audio_mime(path).ok_or(MediaError::UnsupportedKind)?,
                duration_ms: source.duration_ms.unwrap_or(0),
            })),
            InputKind::Video => Err(MediaError::NeedsConversion(
                "Video needs its audio extracted, which comes with the bundled FFmpeg. Export the                  sound as MP3 or M4A in the meantime."
                    .into(),
            )),
            InputKind::Text => Err(MediaError::UnsupportedKind),
        }
    }

    /// The pieces to process, in reading order: every page, each spread as two pages when the
    /// user asked for it, and only the first `sample` pages when trying a sample.
    pub fn pieces(&self, split_spreads: bool, sample: Option<u32>) -> Vec<PieceRef> {
        let mut pieces = Vec::new();
        let pages = match self {
            Self::Pdf(pdf) => pdf.page_count(),
            Self::Image(img) => img.page_count(),
            Self::Audio(audio) => {
                return vec![PieceRef::Segment {
                    index: 0,
                    start_ms: 0,
                    end_ms: audio.duration_ms,
                }];
            }
        };
        let last = sample.map_or(pages, |n| n.min(pages));
        for number in 1..=last {
            let spread = split_spreads
                && match self {
                    Self::Pdf(pdf) => pdf.is_spread(number),
                    Self::Image(img) => img.is_spread(number),
                    Self::Audio(_) => false,
                };
            if spread {
                for half in [
                    scriptorium_core::PageHalf::Left,
                    scriptorium_core::PageHalf::Right,
                ] {
                    pieces.push(PieceRef::Page {
                        number,
                        half: Some(half),
                    });
                }
            } else {
                pieces.push(PieceRef::page(number));
            }
        }
        pieces
    }

    /// One piece in the first of `accepted` MIME types this crate can produce.
    pub fn prepare(
        &self,
        piece: &PieceRef,
        accepted: &[&str],
        file_stem: &str,
    ) -> Result<PreparedInput, MediaError> {
        if let Self::Audio(audio) = self {
            if !accepted.contains(&audio.mime) {
                return Err(MediaError::NeedsConversion(format!(
                    "This model does not accept {} audio; converting it needs the bundled FFmpeg,                      which comes later. Choose another model, or convert the file to MP3.",
                    audio.mime.trim_start_matches("audio/")
                )));
            }
            let bytes = std::fs::read(&audio.path).map_err(|e| MediaError::Read {
                path: audio.path.display().to_string(),
                message: e.to_string(),
            })?;
            let ext = audio
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp3");
            return Ok(PreparedInput {
                piece: piece.clone(),
                file_name: format!("{file_stem}.{ext}"),
                mime: audio.mime.to_string(),
                bytes,
            });
        }
        let PieceRef::Page { number, half } = piece else {
            return Err(MediaError::NoSuchPage(piece.label()));
        };
        let suffix = match half {
            None => String::new(),
            Some(h) => format!("-{h:?}").to_lowercase(),
        };
        let (mime, bytes) = match self {
            Self::Pdf(pdf) => {
                if !accepted.contains(&"application/pdf") {
                    return Err(MediaError::NeedsConversion(
                        "This model needs page images; reading PDFs with it is not available yet. \
                         Choose a model that accepts PDF pages."
                            .into(),
                    ));
                }
                ("application/pdf".to_string(), pdf.page_pdf(*number, *half)?)
            }
            Self::Image(img) => img.prepare(*number, *half, accepted)?,
            Self::Audio(_) => unreachable_audio()?,
        };
        let ext = mime
            .rsplit('/')
            .next()
            .unwrap_or("bin")
            .replace("jpeg", "jpg");
        Ok(PreparedInput {
            piece: piece.clone(),
            file_name: format!("{file_stem}-p{number}{suffix}.{ext}"),
            mime,
            bytes,
        })
    }

    /// Problems likely to hurt the result, found from page sizes and metadata only.
    pub fn warnings(&self, source_id: SourceId) -> Vec<InputWarning> {
        let kinds = match self {
            Self::Pdf(pdf) => checks::pdf_warnings(pdf),
            Self::Image(img) => checks::image_warnings(img),
            Self::Audio(_) => Vec::new(),
        };
        kinds
            .into_iter()
            .map(|kind| InputWarning { source_id, kind })
            .collect()
    }
}

/// Audio returns earlier in `prepare`; this keeps the match exhaustive without a panic.
fn unreachable_audio() -> Result<(String, Vec<u8>), MediaError> {
    Err(MediaError::UnsupportedKind)
}

#[cfg(test)]
mod tests;
