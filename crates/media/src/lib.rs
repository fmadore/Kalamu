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
}

impl OpenSource {
    pub fn open(source: &SourceFile) -> Result<Self, MediaError> {
        let path = Path::new(&source.path);
        match source.kind {
            InputKind::Pdf => Ok(Self::Pdf(Box::new(PdfSource::open(path)?))),
            InputKind::Image => Ok(Self::Image(ImageSource::open(path)?)),
            _ => Err(MediaError::UnsupportedKind),
        }
    }

    /// The pieces to process, in reading order: every page, each spread as two pages when the
    /// user asked for it, and only the first `sample` pages when trying a sample.
    pub fn pieces(&self, split_spreads: bool, sample: Option<u32>) -> Vec<PieceRef> {
        let mut pieces = Vec::new();
        let pages = match self {
            Self::Pdf(pdf) => pdf.page_count(),
            Self::Image(img) => img.page_count(),
        };
        let last = sample.map_or(pages, |n| n.min(pages));
        for number in 1..=last {
            let spread = split_spreads
                && match self {
                    Self::Pdf(pdf) => pdf.is_spread(number),
                    Self::Image(img) => img.is_spread(number),
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
        };
        kinds
            .into_iter()
            .map(|kind| InputWarning { source_id, kind })
            .collect()
    }
}

#[cfg(test)]
mod tests;
