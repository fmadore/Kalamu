use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    ExportFormat, ExportOptions, JobId, LanguageChoice, Setup, SourceFile, Task, Timestamp,
};

/// Settings shared by every set-up in a job.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobSettings {
    #[serde(default)]
    pub languages: LanguageChoice,
    /// Process only the first pages of each document, for trying a sample.
    pub sample_pages: Option<u32>,
    /// How a text extraction lays out lines; `None` leaves it to the prompt.
    #[serde(default)]
    pub line_breaks: Option<LineBreaks>,
    /// Whether cloud calls go one by one or as a cheaper batch.
    #[serde(default)]
    pub pace: Pace,
    /// How many pages of a PDF a text extraction sends in one call; one unless chosen.
    #[serde(default, skip_serializing_if = "is_one")]
    #[ts(optional)]
    pub pages_per_call: Option<crate::PagesPerCall>,
}

fn is_one(pages: &Option<crate::PagesPerCall>) -> bool {
    matches!(pages, None | Some(crate::PagesPerCall::One))
}

/// How a job's cloud calls are sent. Offline engines and models without a batch service run
/// at once whatever the choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Pace {
    /// One call per piece, results as they come.
    #[default]
    Fast,
    /// Every piece sent together to the provider's batch service, at about half the price;
    /// results can take up to a day.
    Economy,
}

/// How extracted text keeps the source's lines. The two serve different work: checking against
/// the scan, or reading and searching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LineBreaks {
    /// One line of text per line in the source, end-of-line hyphens kept.
    AsOnPage,
    /// Lines joined into paragraphs and words split across lines rejoined.
    RunningText,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "path", rename_all = "snake_case")]
#[ts(export)]
pub enum OutputDestination {
    /// Next to each source file.
    #[default]
    BesideSources,
    /// Into one chosen folder.
    Folder(String),
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OutputSettings {
    pub formats: Vec<ExportFormat>,
    #[serde(default)]
    pub destination: OutputDestination,
    #[serde(default)]
    pub options: ExportOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum JobStatus {
    /// Created, not started yet.
    Pending,
    Running,
    /// Stopped by the user; resumable.
    Paused,
    /// Stopped because the app closed or crashed; resumable.
    Interrupted,
    Completed,
    /// Finished, but some pieces failed and can be retried.
    CompletedWithErrors,
    Cancelled,
}

impl JobStatus {
    /// Jobs the app offers to resume on the next launch.
    pub fn is_unfinished(self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Running | Self::Paused | Self::Interrupted
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Interrupted => "interrupted",
            Self::Completed => "completed",
            Self::CompletedWithErrors => "completed_with_errors",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => Self::Pending,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "interrupted" => Self::Interrupted,
            "completed" => Self::Completed,
            "completed_with_errors" => Self::CompletedWithErrors,
            "cancelled" => Self::Cancelled,
            _ => return None,
        })
    }
}

/// One run of a task over a set of sources, with one or more set-ups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Job {
    pub id: JobId,
    pub name: String,
    pub task: Task,
    pub created_at: Timestamp,
    pub sources: Vec<SourceFile>,
    /// One set-up for a normal job, several in comparison mode.
    pub setups: Vec<Setup>,
    #[serde(default)]
    pub settings: JobSettings,
    #[serde(default)]
    pub output: OutputSettings,
    pub status: JobStatus,
}

impl Job {
    pub fn is_comparison(&self) -> bool {
        self.setups.len() > 1
    }
}

/// A job as listed in the queue and on the resume screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobSummary {
    pub id: JobId,
    pub name: String,
    pub task: Task,
    pub status: JobStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub source_count: u32,
    pub setup_count: u32,
    pub pieces_total: u32,
    pub pieces_finished: u32,
    pub pieces_failed: u32,
    /// Finished pieces whose answer stopped at the model's output limit.
    #[serde(default)]
    pub pieces_truncated: u32,
    /// What the job's own calls cost over all its runs, in US dollars; `None` before any
    /// priced answer.
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// The models its set-ups called, each once, in set-up order, a table of contents'
    /// combining model after the one that read the pages.
    #[serde(default)]
    pub models: Vec<String>,
    /// The workflow it is a step of, by the name the run was started under.
    #[serde(default)]
    pub workflow: Option<String>,
}

/// The names of a job's sources, for finding it on the Jobs page by a file it read: each
/// file's name, and a YouTube video's title when it was read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobSources {
    pub job_id: JobId,
    pub names: Vec<String>,
}

impl Job {
    /// The models its set-ups call, each once, in set-up order.
    pub fn models(&self) -> Vec<String> {
        let mut models: Vec<String> = Vec::new();
        let called = self
            .setups
            .iter()
            .flat_map(|s| std::iter::once(&s.model).chain(s.merge.as_ref().map(|m| &m.model)));
        for model in called {
            if !models.contains(model) {
                models.push(model.clone());
            }
        }
        models
    }
}
