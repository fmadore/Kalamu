//! Turning source files into pieces a provider accepts, and checking inputs before a job.
//!
//! PDFs are cut into single-page PDFs, never rendered, as in zmo- and iwac-ai-pipelines: every
//! first-stage provider reads PDF pages directly, at full quality. Images are passed through
//! when the provider takes their format, and converted otherwise. Recordings and videos are
//! split at pauses and converted by the bundled FFmpeg. A YouTube video is passed on as its
//! link, for the provider to fetch. Originals are only read, never modified.

mod audio;
mod audio_format;
mod checks;
mod ffmpeg;
mod image_source;
mod pdf;
mod text_source;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use kalamu_core::{
    InputKind, InputWarning, PieceRef, PreparedInput, SourceFile, SourceId, YOUTUBE_LINK,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub use audio::{AudioSource, DEFAULT_PIECE_MS, audio_mime, plan, plays_as_is};
pub use ffmpeg::{AudioFormat, Ffmpeg, Probe};
pub use image_source::ImageSource;
pub use pdf::PdfSource;
#[cfg(any(test, feature = "test-support"))]
pub use pdf::sample_pdf;
pub use text_source::TextSource;

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
    #[error("FFmpeg failed: {0}")]
    Tool(String),
}

/// Hashes a file and works out what it is, without loading it all into memory twice. FFmpeg,
/// when there is one, gives the length of recordings and videos.
pub fn inspect(path: &Path, ffmpeg: Option<&Ffmpeg>) -> Result<SourceFile, MediaError> {
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
    let duration_ms = match kind {
        InputKind::Audio | InputKind::Video => audio::probe(path, ffmpeg).map(|p| p.duration_ms),
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
        duration_ms,
        split_spreads: false,
        even_out_volume: false,
        title: None,
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
    /// A recording, or the sound track of a video.
    Audio(AudioSource),
    /// Plain text, Markdown, CSV or Word, for summaries.
    Text(TextSource),
    /// A YouTube video's address; the provider reads the video itself.
    YouTube(String),
}

impl OpenSource {
    pub fn open(source: &SourceFile, ffmpeg: Option<&Ffmpeg>) -> Result<Self, MediaError> {
        let path = Path::new(&source.path);
        match source.kind {
            InputKind::Pdf => Ok(Self::Pdf(Box::new(PdfSource::open(path)?))),
            InputKind::Image => Ok(Self::Image(ImageSource::open(path)?)),
            InputKind::Audio | InputKind::Video => {
                Ok(Self::Audio(AudioSource::open(source, ffmpeg)?))
            }
            InputKind::Text => Ok(Self::Text(TextSource::open(path)?)),
            InputKind::YouTube => Ok(Self::YouTube(source.path.clone())),
        }
    }

    /// The pieces to process, in reading order: every page, each spread as two pages when the
    /// user asked for it, and only the first `sample` pages when trying a sample; recordings
    /// in pieces of at most about `piece_ms`, cut at pauses (which decodes the whole file).
    pub fn pieces(&self, split_spreads: bool, sample: Option<u32>, piece_ms: u64) -> Vec<PieceRef> {
        let mut pieces = Vec::new();
        let pages = match self {
            Self::Pdf(pdf) => pdf.page_count(),
            Self::Image(img) => img.page_count(),
            Self::Audio(audio) => return audio.pieces(piece_ms),
            // A YouTube link cannot be clipped reliably: Gemini still hears the whole sound
            // track (see the decision of 27 October 2026), so a video is one piece.
            Self::Text(_) | Self::YouTube(_) => return vec![PieceRef::Whole],
        };
        let last = sample.map_or(pages, |n| n.min(pages));
        for number in 1..=last {
            let spread = split_spreads
                && match self {
                    Self::Pdf(pdf) => pdf.is_spread(number),
                    Self::Image(img) => img.is_spread(number),
                    Self::Audio(_) | Self::Text(_) | Self::YouTube(_) => false,
                };
            if spread {
                for half in [kalamu_core::PageHalf::Left, kalamu_core::PageHalf::Right] {
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

    /// One piece of a recording converted to listen to; see [`AudioSource::listening_copy`].
    pub fn listening_copy(&self, piece: &PieceRef) -> Result<PreparedInput, MediaError> {
        match self {
            Self::Audio(audio) => audio.listening_copy(piece),
            _ => Err(MediaError::UnsupportedKind),
        }
    }

    /// One piece in the first of `accepted` MIME types this crate can produce.
    pub fn prepare(
        &self,
        piece: &PieceRef,
        accepted: &[&str],
        file_stem: &str,
    ) -> Result<PreparedInput, MediaError> {
        if let Self::Audio(audio) = self {
            return audio.prepare(piece, accepted, file_stem);
        }
        if let Self::YouTube(url) = self {
            return youtube_link(url, piece, accepted, file_stem);
        }
        if *piece == PieceRef::Whole {
            return self.prepare_whole(accepted, file_stem);
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
            Self::Audio(_) | Self::Text(_) | Self::YouTube(_) => unreachable_audio()?,
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
            // Recordings are checked with FFmpeg by `recording_warnings`; a YouTube video is not
            // seen before the provider reads it.
            Self::Audio(_) | Self::Text(_) | Self::YouTube(_) => Vec::new(),
        };
        kinds
            .into_iter()
            .map(|kind| InputWarning { source_id, kind })
            .collect()
    }
}

impl OpenSource {
    /// The whole document, for an analysis: a PDF as it is, a photographed page as an image,
    /// other documents as text.
    fn prepare_whole(
        &self,
        accepted: &[&str],
        file_stem: &str,
    ) -> Result<PreparedInput, MediaError> {
        let (mime, bytes) = match self {
            Self::Pdf(pdf) if accepted.contains(&"application/pdf") => {
                ("application/pdf".to_string(), pdf.whole()?)
            }
            Self::Text(text) if accepted.contains(&"text/plain") => {
                ("text/plain".to_string(), text.text()?.into_bytes())
            }
            Self::Pdf(_) | Self::Image(_) if accepted == ["text/plain"] => {
                return Err(MediaError::NeedsConversion(
                    "This model reads text only. Extract the text first, then analyse the \
                     result, or choose a model that reads PDFs and images."
                        .into(),
                ));
            }
            // One image per call: the pages of a multi-page TIFF would be read apart.
            Self::Image(img) if img.page_count() > 1 => {
                return Err(MediaError::NeedsConversion(format!(
                    "This image holds {} pages, and an analysis reads a document as one. Save \
                     the pages as a single PDF to analyse them together.",
                    img.page_count()
                )));
            }
            Self::Image(img) => img.prepare(1, None, accepted)?,
            _ => {
                return Err(MediaError::NeedsConversion(
                    "This model cannot read this kind of document.".into(),
                ));
            }
        };
        let ext = mime
            .rsplit('/')
            .next()
            .unwrap_or("bin")
            .replace("jpeg", "jpg")
            .replace("plain", "txt");
        Ok(PreparedInput {
            piece: PieceRef::Whole,
            file_name: format!("{file_stem}.{ext}"),
            mime,
            bytes,
        })
    }
}

/// Checks on a recording or video before a job: FFmpeg missing, no sound track, quiet, long.
pub fn recording_warnings(source: &SourceFile, ffmpeg: Option<&Ffmpeg>) -> Vec<InputWarning> {
    let path = Path::new(&source.path);
    let probe = audio::probe(path, ffmpeg);
    audio::warnings(path, source.kind, probe.as_ref(), ffmpeg)
        .into_iter()
        .map(|kind| InputWarning {
            source_id: source.id,
            kind,
        })
        .collect()
}

/// A YouTube video as its link, for a model that reads YouTube links.
fn youtube_link(
    url: &str,
    piece: &PieceRef,
    accepted: &[&str],
    file_stem: &str,
) -> Result<PreparedInput, MediaError> {
    if !accepted.contains(&YOUTUBE_LINK) {
        return Err(MediaError::NeedsConversion(
            "This model cannot read YouTube videos. Choose a Gemini model that reads them.".into(),
        ));
    }
    Ok(PreparedInput {
        piece: piece.clone(),
        mime: YOUTUBE_LINK.to_string(),
        bytes: url.as_bytes().to_vec(),
        file_name: file_stem.to_string(),
    })
}

/// Audio returns earlier in `prepare`; this keeps the match exhaustive without a panic.
fn unreachable_audio() -> Result<(String, Vec<u8>), MediaError> {
    Err(MediaError::UnsupportedKind)
}

#[cfg(test)]
mod tests;
