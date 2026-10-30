use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ExportFormat, InputKind};

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
    /// Every task, in the order New job offers them.
    pub const ALL: [Task; 5] = [
        Self::TextExtraction,
        Self::TextCorrection,
        Self::Analysis,
        Self::Contents,
        Self::AudioTranscription,
    ];

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

    /// The kinds of source the task reads: pages for an extraction or a table of contents, text
    /// there is for a correction (a PDF's text layer or a text file), any document for an
    /// analysis, and recordings, videos and YouTube links for a transcription.
    pub fn source_kinds(self) -> &'static [InputKind] {
        match self {
            Self::TextExtraction | Self::Contents => &[InputKind::Pdf, InputKind::Image],
            Self::TextCorrection => &[InputKind::Text, InputKind::Pdf],
            Self::Analysis => &[InputKind::Text, InputKind::Pdf, InputKind::Image],
            Self::AudioTranscription => &[InputKind::Audio, InputKind::Video, InputKind::YouTube],
        }
    }

    /// Whether the task reads a source of this kind.
    pub fn reads_source(self, kind: InputKind) -> bool {
        self.source_kinds().contains(&kind)
    }

    /// The formats the task's results are written in, in the order New job offers them. A
    /// searchable PDF needs a model that reports where text sits on the page.
    pub fn formats(self) -> &'static [ExportFormat] {
        use ExportFormat::*;
        match self {
            Self::TextExtraction => &[Txt, Markdown, Docx, Json, Csv, SearchablePdf],
            Self::AudioTranscription => &[Txt, Markdown, Docx, Srt, Vtt, Json, Csv],
            Self::Analysis | Self::TextCorrection => &[Txt, Markdown, Docx, Json, Csv],
            Self::Contents => &[Markdown, Docx, Csv, Txt, Json],
        }
    }

    /// Whether the job's language is the one the model answers in, one only, rather than the
    /// source's: an analysis's fields and a table of contents' entries are written by the
    /// model; an extraction, a transcription or a correction keep the source's words.
    pub fn answers_in_language(self) -> bool {
        matches!(self, Self::Analysis | Self::Contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_task_reads_some_sources_and_writes_some_formats() {
        for task in Task::ALL {
            assert!(!task.source_kinds().is_empty(), "{}", task.as_str());
            assert!(task.formats().contains(&ExportFormat::Txt) || task == Task::Contents);
            assert!(task.formats().contains(&ExportFormat::Json));
        }
        assert!(Task::AudioTranscription.reads_source(InputKind::YouTube));
        assert!(!Task::TextExtraction.reads_source(InputKind::Text));
        assert!(!Task::TextCorrection.reads_source(InputKind::Image));
        assert!(
            Task::TextExtraction
                .formats()
                .contains(&ExportFormat::SearchablePdf)
        );
        assert!(!Task::Analysis.formats().contains(&ExportFormat::Srt));
    }
}
