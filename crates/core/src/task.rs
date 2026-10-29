use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a job does with its sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Task {
    /// OCR and HTR: printed and handwritten text from PDFs and images.
    TextExtraction,
    /// Speech to text from audio or video.
    AudioTranscription,
    /// The researcher's own questions to each text, Word file or PDF: a summary, the people
    /// and places it names, its tone. Stored jobs from before the rename read as this.
    #[serde(alias = "summary")]
    Analysis,
    /// A detailed table of contents of a long document (a newspaper or magazine issue, an
    /// archival file scanned in one piece, a report): each page read for the articles or items
    /// on it, then the pages' readings combined into one entry per article or item, with its
    /// pages and the researcher's fields.
    Contents,
    /// Text that already exists (a PDF's text layer, a text file from a digital library)
    /// corrected page by page, its spelling and wording kept.
    TextCorrection,
}

impl Task {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextExtraction => "text_extraction",
            Self::AudioTranscription => "audio_transcription",
            Self::Analysis => "analysis",
            Self::Contents => "contents",
            Self::TextCorrection => "text_correction",
        }
    }

    /// The kind of call a provider makes for this task. Providers know three: reading a page,
    /// transcribing, and answering in fields. A table of contents and a correction are made of
    /// answers in fields (the entries on a page, the corrected text), so they reach providers
    /// as analyses and need nothing new from them.
    pub fn request_task(self) -> Task {
        match self {
            Self::Contents | Self::TextCorrection => Self::Analysis,
            other => other,
        }
    }

    /// Whether a model that does `self.request_task()` and reads `inputs` can do this task: a
    /// table of contents reads pages (PDF or images); a correction reads text.
    pub fn reads(self, inputs: &[String]) -> bool {
        let has = |mime: &str| inputs.iter().any(|i| i == mime);
        match self {
            Self::Contents => {
                has("application/pdf") || inputs.iter().any(|i| i.starts_with("image/"))
            }
            Self::TextCorrection => has("text/plain"),
            _ => true,
        }
    }

    /// Whether a table of contents' second step, combining the pages, can run on a model that
    /// reads `inputs`: it reads the pages' readings as text.
    pub fn merges(inputs: &[String]) -> bool {
        inputs.iter().any(|i| i == "text/plain")
    }
}
