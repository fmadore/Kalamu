//! Whether a model that takes these formats can be given a source of this kind at all, decided
//! from the kind alone. [`OpenSource::prepare`](crate::OpenSource::prepare) asks it before
//! preparing a piece, and the engine asks it before a job is made, so a job a model cannot read
//! is refused up front with the message each piece would otherwise fail with. What depends on
//! the file itself (a scan without a text layer, a TIFF of several pages) is still found piece
//! by piece.

use kalamu_core::{InputKind, YOUTUBE_LINK};

use crate::MediaError;

/// How a task reads its sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// The whole document at once: an analysis.
    Whole,
    /// Page by page, passage by passage or piece by piece: every other task. A correction or a
    /// translation is given text alone, so its formats are `text/plain` only.
    Pieces,
}

/// Refuses a source of this kind for a model taking `accepted` formats (the provider's for the
/// task, narrowed to the model's), with the message a researcher reads. Recordings are left to
/// FFmpeg, which converts them to what the model takes.
pub fn readable(kind: InputKind, reading: Reading, accepted: &[&str]) -> Result<(), MediaError> {
    let has = |mime: &str| accepted.contains(&mime);
    let images = accepted.iter().any(|m| m.starts_with("image/"));
    let text_only = accepted == ["text/plain"];
    let refuse = |message: &str| Err(MediaError::NeedsConversion(message.into()));
    match (kind, reading) {
        (InputKind::Audio | InputKind::Video, _) => Ok(()),
        (InputKind::YouTube, _) if has(YOUTUBE_LINK) => Ok(()),
        (InputKind::YouTube, _) => {
            refuse("This model cannot read YouTube videos. Choose a Gemini model that reads them.")
        }
        (InputKind::Text, _) if has("text/plain") => Ok(()),
        (InputKind::Text, _) => refuse("This model cannot read text files."),
        (InputKind::Pdf, Reading::Whole) if has("application/pdf") => Ok(()),
        (InputKind::Image, Reading::Whole) if images => Ok(()),
        (InputKind::Pdf | InputKind::Image, Reading::Whole) if text_only => refuse(
            "This model reads text only. Extract the text first, then analyse the result, or \
             choose a model that reads PDFs and images.",
        ),
        // A model that reads texts and single page images, not whole PDFs (the offline language
        // models).
        (InputKind::Pdf, Reading::Whole) => refuse(
            "This model does not read PDFs whole. Extract the text first, then analyse the \
             result, or choose a model that reads PDFs.",
        ),
        (InputKind::Image, Reading::Whole) => {
            refuse("This model cannot read this kind of document.")
        }
        // A model that reads text only gets a PDF page's text layer.
        (InputKind::Pdf, Reading::Pieces)
            if text_only || has("application/pdf") || has("image/png") =>
        {
            Ok(())
        }
        (InputKind::Pdf, Reading::Pieces) => {
            refuse("This model reads neither PDF pages nor page images.")
        }
        (InputKind::Image, Reading::Pieces) if text_only => refuse(
            "This model reads text only, and this file has none. Choose a model that reads PDFs \
             and images.",
        ),
        (InputKind::Image, Reading::Pieces) if images => Ok(()),
        (InputKind::Image, Reading::Pieces) => refuse("This model does not accept images."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &[&str] = &["text/plain"];
    const ALL: &[&str] = &["application/pdf", "image/png", "image/jpeg", "text/plain"];
    const IMAGES_AND_TEXT: &[&str] = &["image/png", "image/jpeg", "text/plain"];

    #[test]
    fn a_text_model_reads_text_and_a_pdfs_text_layer_but_no_image() {
        assert!(readable(InputKind::Text, Reading::Whole, TEXT).is_ok());
        assert!(readable(InputKind::Pdf, Reading::Pieces, TEXT).is_ok());
        assert!(readable(InputKind::Pdf, Reading::Whole, TEXT).is_err());
        assert!(readable(InputKind::Image, Reading::Whole, TEXT).is_err());
        assert!(readable(InputKind::Image, Reading::Pieces, TEXT).is_err());
    }

    #[test]
    fn a_model_reading_images_but_not_pdfs_reads_pdf_pages_not_whole_pdfs() {
        assert!(readable(InputKind::Image, Reading::Whole, IMAGES_AND_TEXT).is_ok());
        assert!(readable(InputKind::Pdf, Reading::Pieces, IMAGES_AND_TEXT).is_ok());
        assert!(readable(InputKind::Pdf, Reading::Whole, IMAGES_AND_TEXT).is_err());
    }

    #[test]
    fn a_model_reading_everything_reads_every_document() {
        for kind in [InputKind::Pdf, InputKind::Image, InputKind::Text] {
            for reading in [Reading::Whole, Reading::Pieces] {
                assert!(readable(kind, reading, ALL).is_ok());
            }
        }
        assert!(readable(InputKind::YouTube, Reading::Whole, ALL).is_err());
    }
}
