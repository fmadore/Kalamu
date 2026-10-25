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
}

impl Task {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextExtraction => "text_extraction",
            Self::AudioTranscription => "audio_transcription",
            Self::Analysis => "analysis",
        }
    }
}
