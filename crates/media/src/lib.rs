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
mod page_set;
mod pdf;
pub mod program;
mod render;
mod text_source;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::UNIX_EPOCH;

use kalamu_core::{
    InputKind, InputWarning, PieceRef, PreparedInput, SourceFile, SourceId, StopSignal,
    YOUTUBE_LINK,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub use audio::{AudioSource, DEFAULT_PIECE_MS, Playback, audio_mime, plan};
pub use ffmpeg::{AudioFormat, Ffmpeg, Probe};
pub use image_source::ImageSource;
pub use page_set::PageSet;
pub use pdf::PdfSource;
#[cfg(any(test, feature = "test-support"))]
pub use pdf::sample_pdf;
pub use render::{THUMBNAIL_MAX_WIDTH, thumbnail};
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
    /// The job was paused or cancelled while FFmpeg worked; it was ended, and nothing failed.
    #[error("Stopped before the file was converted")]
    Stopped,
    #[error(
        "This file has changed since it was added to the job. Start a new job with it to read it as it is now"
    )]
    Changed,
    /// One file of a document of pages holds something else now than when it was added.
    #[error(
        "The page's file {0} has changed since it was added to the job. Start a new job with it to read it as it is now"
    )]
    PageChanged(String),
}

/// Hashes a file and works out what it is, without loading it all into memory twice. FFmpeg,
/// when there is one, gives the length of recordings and videos.
pub fn inspect(path: &Path, ffmpeg: Option<&Ffmpeg>) -> Result<SourceFile, MediaError> {
    let read_err = |e: std::io::Error| MediaError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    };
    let kind = InputKind::from_path(path).ok_or(MediaError::UnsupportedKind)?;
    let metadata = std::fs::metadata(path).map_err(read_err)?;
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
        size_bytes: metadata.len(),
        modified_ms: modified_ms(&metadata),
        kind,
        page_count,
        duration_ms,
        split_spreads: false,
        even_out_volume: false,
        title: None,
        derived_from: None,
        pages: Vec::new(),
    })
}

/// When a file was last modified, in milliseconds since 1970, where the system says.
pub(crate) fn modified_ms(metadata: &std::fs::Metadata) -> Option<i64> {
    let since = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(since.as_millis()).ok()
}

/// Checks that a file still holds what was hashed when it was added. Its size and modification
/// time settle it when they are known and unchanged; otherwise it is hashed again, so a file
/// touched but not changed is still read. A changed file is refused: what it holds now would be
/// cached under the fingerprint of what it held then.
pub fn check_unchanged(source: &SourceFile) -> Result<(), MediaError> {
    // A document of pages is checked page by page as each is read, so one file moved or
    // replaced fails its page alone (`PageSet`).
    if source.kind == InputKind::YouTube || source.is_page_set() {
        return Ok(());
    }
    let path = Path::new(&source.path);
    let metadata = std::fs::metadata(path).map_err(|e| MediaError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    if metadata.len() != source.size_bytes {
        return Err(MediaError::Changed);
    }
    if source.modified_ms.is_some() && modified_ms(&metadata) == source.modified_ms {
        return Ok(());
    }
    if hash_file(path)? == source.sha256 {
        Ok(())
    } else {
        Err(MediaError::Changed)
    }
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
    /// A document of image files, read as its pages in its order.
    Pages(PageSet),
    /// A recording, or the sound track of a video.
    Audio(AudioSource),
    /// Plain text, Markdown, CSV or Word, for summaries.
    Text(TextSource),
    /// A YouTube video's address; the provider reads the video itself.
    YouTube(String),
}

impl OpenSource {
    pub fn open(source: &SourceFile, ffmpeg: Option<&Ffmpeg>) -> Result<Self, MediaError> {
        Self::open_with_stop(source, ffmpeg, &StopSignal::default())
    }

    /// [`Self::open`] for a job's run: FFmpeg's work on a recording (the pass that finds its
    /// pauses, each piece cut) ends as soon as `stop` is raised, with [`MediaError::Stopped`].
    pub fn open_with_stop(
        source: &SourceFile,
        ffmpeg: Option<&Ffmpeg>,
        stop: &StopSignal,
    ) -> Result<Self, MediaError> {
        if source.is_page_set() {
            return Ok(Self::Pages(PageSet::new(source.pages.clone())));
        }
        let path = Path::new(&source.path);
        match source.kind {
            InputKind::Pdf => Ok(Self::Pdf(Box::new(PdfSource::open(path)?))),
            InputKind::Image => Ok(Self::Image(ImageSource::open(path)?)),
            InputKind::Audio | InputKind::Video => Ok(Self::Audio(AudioSource::open_with_stop(
                source, ffmpeg, stop,
            )?)),
            InputKind::Text => Ok(Self::Text(TextSource::open(path)?)),
            InputKind::YouTube => Ok(Self::YouTube(source.path.clone())),
        }
    }

    /// The pieces to process, in reading order: every page, each spread as two pages when the
    /// user asked for it, and only the first `sample` pages when trying a sample; recordings
    /// in pieces of at most about `piece_ms`, cut at pauses (which decodes the whole file).
    /// Empty for a recording whose pass was stopped; see [`Self::try_pieces`].
    pub fn pieces(&self, split_spreads: bool, sample: Option<u32>, piece_ms: u64) -> Vec<PieceRef> {
        self.try_pieces(split_spreads, sample, piece_ms)
            .unwrap_or_default()
    }

    /// [`Self::pieces`], or [`MediaError::Stopped`] when the source was opened with a stop
    /// signal that was raised while a recording's pauses were being found. That is the only
    /// error: a pass that fails otherwise cuts at fixed marks.
    pub fn try_pieces(
        &self,
        split_spreads: bool,
        sample: Option<u32>,
        piece_ms: u64,
    ) -> Result<Vec<PieceRef>, MediaError> {
        let mut pieces = Vec::new();
        let pages = match self {
            Self::Pdf(pdf) => pdf.page_count(),
            Self::Image(img) => img.page_count(),
            Self::Pages(set) => set.page_count(),
            Self::Audio(audio) => return audio.try_pieces(piece_ms),
            // A YouTube link cannot be clipped reliably: Gemini still hears the whole sound
            // track (see the decision of 27 October 2026), so a video is one piece.
            Self::Text(_) | Self::YouTube(_) => return Ok(vec![PieceRef::Whole]),
        };
        let last = sample.map_or(pages, |n| n.min(pages));
        for number in 1..=last {
            let spread = split_spreads
                && match self {
                    Self::Pdf(pdf) => pdf.is_spread(number),
                    Self::Image(img) => img.is_spread(number),
                    Self::Pages(set) => set.is_spread(number),
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
        Ok(pieces)
    }

    /// The pieces a correction sends: every page of a PDF (its text layer, so spreads are not
    /// split), the pages or passages of a text file; the first `sample` only when trying a
    /// sample. Images and recordings hold no text to correct.
    pub fn text_pieces(&self, sample: Option<u32>) -> Result<Vec<PieceRef>, MediaError> {
        let mut pieces: Vec<PieceRef> = match self {
            Self::Pdf(pdf) => (1..=pdf.page_count()).map(PieceRef::page).collect(),
            Self::Text(text) => text.sections()?.into_iter().map(|(p, _)| p).collect(),
            Self::Image(_) | Self::Pages(_) | Self::Audio(_) | Self::YouTube(_) => {
                return Err(MediaError::NeedsConversion(
                    "This file holds no text to correct. Extract its text first, then correct \
                     the result."
                        .into(),
                ));
            }
        };
        if let Some(n) = sample {
            pieces.truncate(n as usize);
        }
        Ok(pieces)
    }

    /// The text a page or passage holds before any model reads it, as a correction sends it.
    pub fn text_of(&self, piece: &PieceRef) -> Result<String, MediaError> {
        self.piece_text(piece)
    }

    /// The text of a page or passage, for a model that reads text only: a PDF page's text
    /// layer, or a text file's page or passage.
    fn piece_text(&self, piece: &PieceRef) -> Result<String, MediaError> {
        let text = match (self, piece) {
            (Self::Pdf(pdf), PieceRef::Page { number, .. }) => pdf.page_text(*number)?,
            (Self::Text(text), PieceRef::Page { .. } | PieceRef::Passage { .. }) => {
                text.section(piece)?
            }
            _ => {
                return Err(MediaError::NeedsConversion(
                    "This model reads text only, and this file has none. Choose a model that \
                     reads PDFs and images."
                        .into(),
                ));
            }
        };
        if !text.chars().any(char::is_alphanumeric) {
            return Err(MediaError::NeedsConversion(format!(
                "Page {} has no text to read: it is a scan without a text layer. Extract its \
                 text first, or choose a model that reads page images.",
                piece.label()
            )));
        }
        Ok(text)
    }

    /// One piece of a recording converted to listen to; see [`AudioSource::listening_copy`].
    pub fn listening_copy(
        &self,
        piece: &PieceRef,
        playback: &Playback,
    ) -> Result<PreparedInput, MediaError> {
        match self {
            Self::Audio(audio) => audio.listening_copy(piece, playback),
            _ => Err(MediaError::UnsupportedKind),
        }
    }

    /// Whole pages `first..=last` of a PDF as one input, for a model that reads several pages
    /// in one call. Only a PDF read as pages (not split into halves) has them.
    pub fn prepare_pages(
        &self,
        first: u32,
        last: u32,
        accepted: &[&str],
        file_stem: &str,
    ) -> Result<PreparedInput, MediaError> {
        let Self::Pdf(pdf) = self else {
            return Err(MediaError::UnsupportedKind);
        };
        if !accepted.contains(&"application/pdf") {
            return Err(MediaError::NeedsConversion(
                "This model does not read PDFs of several pages.".into(),
            ));
        }
        Ok(PreparedInput {
            piece: PieceRef::page(first),
            file_name: format!("{file_stem}-p{first}-{last}.pdf"),
            mime: "application/pdf".into(),
            bytes: pdf.pages_pdf(first, last)?,
        })
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
        // A model that reads text only (a correction, a text model reading a page) gets the
        // piece's text.
        let text_only = accepted == ["text/plain"];
        if text_only || matches!(piece, PieceRef::Passage { .. }) || matches!(self, Self::Text(_)) {
            if !accepted.contains(&"text/plain") {
                return Err(MediaError::NeedsConversion(
                    "This model cannot read text files.".into(),
                ));
            }
            let text = self.piece_text(piece)?;
            let label = piece.descriptor().replace(':', "-");
            return Ok(PreparedInput {
                piece: piece.clone(),
                file_name: format!("{file_stem}-{label}.txt"),
                mime: "text/plain".into(),
                bytes: text.into_bytes(),
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
                let page = pdf.page_pdf(*number, *half)?;
                if accepted.contains(&"application/pdf") {
                    ("application/pdf".to_string(), page)
                } else if accepted.contains(&"image/png") {
                    // An engine that reads images only (Tesseract): the same page, drawn.
                    ("image/png".to_string(), render::first_page_png(&page)?)
                } else {
                    return Err(MediaError::NeedsConversion(
                        "This model reads neither PDF pages nor page images.".into(),
                    ));
                }
            }
            Self::Image(img) => img.prepare(*number, *half, accepted)?,
            Self::Pages(set) => set.prepare(*number, *half, accepted)?,
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
            Self::Pages(set) => set.warnings(),
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
            // Several files are not one request; the engine refuses an analysis of them before
            // this is reached.
            Self::Pages(_) => {
                return Err(MediaError::NeedsConversion(
                    "An analysis reads a document as one file. Extract the text of these pages                      first, then analyse the result."
                        .into(),
                ));
            }
            Self::Pdf(_) | Self::Image(_) if accepted == ["text/plain"] => {
                return Err(MediaError::NeedsConversion(
                    "This model reads text only. Extract the text first, then analyse the \
                     result, or choose a model that reads PDFs and images."
                        .into(),
                ));
            }
            // A model that reads texts and single page images, not whole PDFs (the offline
            // language models).
            Self::Pdf(_) => {
                return Err(MediaError::NeedsConversion(
                    "This model does not read PDFs whole. Extract the text first, then analyse \
                     the result, or choose a model that reads PDFs."
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
