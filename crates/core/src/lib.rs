//! Domain types and the traits that connect Kalamu's crates.
//!
//! This crate performs no I/O: it knows nothing about Tauri, SQLite, HTTP or FFmpeg. Types that
//! cross into the front end derive [`ts_rs::TS`]; `cargo test` writes their TypeScript bindings.

pub mod analysis;
pub mod attention;
pub mod batch;
pub mod contents;
pub mod correction;
pub mod dictionary;
pub mod event;
pub mod export;
pub mod fingerprint;
pub mod ids;
pub mod job;
pub mod language;
pub mod list_items;
pub mod ordered;
pub mod pages;
pub mod profile;
pub mod progress;
pub mod project;
pub mod provenance;
pub mod provider;
pub mod result;
pub mod search;
pub mod setup;
pub mod snapshot;
pub mod source;
pub mod speakers;
pub mod store;
pub mod structured;
pub mod task;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod tied;
pub mod timed;
pub mod units;
pub mod warning;
pub mod workflow;
pub mod youtube;

pub use analysis::{AnalysisSchema, Field, FieldAnswer, FieldKind, FieldValue, SchemaError};
pub use attention::{
    QueueCounts, QueueItem, ReasonCounts, ReviewQueue, ReviewScope, ReviewTarget, Signal,
};
pub use batch::{
    BATCH_WINDOW_MS, BatchCleanup, BatchContext, BatchItem, BatchLimits, BatchPhase, BatchProgress,
    BatchRecord,
};
pub use dictionary::{
    DictionaryError, DictionaryStore, Entry, KeptApart, Labels, ProposalGroup, ProposalRequest,
};
pub use event::ProgressEvent;
pub use export::{
    CsvCells, ExportDocument, ExportError, ExportFormat, ExportOptions, ExportPart, Exporter, Table,
};
pub use fingerprint::CacheKey;
pub use ids::{
    CollectionId, DocumentId, EntryId, JobId, ProjectId, ProviderId, RunId, SetupId, SourceId,
    Timestamp, UnitId,
};
pub use job::{
    Job, JobSettings, JobSources, JobStatus, JobSummary, LineBreaks, OutputDestination,
    OutputSettings, Pace,
};
pub use language::{LanguageChoice, LanguageCode};
pub use ordered::{PageFile, PageNote, natural_cmp, natural_sort, page_notes};
pub use pages::PagesPerCall;
pub use profile::{ExportProfile, ProfileColumn, ProfileError, ProfileStarter, ProfileValue};
pub use progress::{ERRORS_LISTED, FileProgress, JobProgress};
pub use project::{
    Collection, DescribedBy, DescriptionField, Document, DocumentLink, PreferredResult, Project,
    ProjectError, ProjectStore, SourceDescription, cleaned, same_tag, tag_name,
};
pub use provenance::{Provenance, SourceDocuments};
pub use provider::{
    ApiKey, PCM_16K_MONO, PreparedInput, ProcessRequest, Provider, ProviderError, RequestSetting,
    StopSignal, Tuning,
};
pub use result::{
    Charge, PageHalf, PieceRef, PieceState, PieceStatus, ProcessOutput, Segment, StoredResult,
    TextLine, TextRegion, TextWord, Usage, plain_words,
};
pub use search::{IndexSize, Passage, PassageKey, PassagePart, SearchIndex};
pub use setup::{MergeModel, MergeStep, PromptSnapshot, Setup, SetupOptions, SetupRequest, Step};
pub use snapshot::{Anchor, AnchorError, DerivedFrom, Snapshot, SnapshotPart};
pub use source::{InputKind, SourceFile};
pub use speakers::{SpeakerIndex, SpeakerNames};
pub use store::{Correction, JobStore, OutdatedFiles, PrunedResults, StoreError};
pub use task::Task;
pub use tied::{TiedResult, TiedResults};
pub use units::{DocumentUnit, FrozenUnit, PagesError, SeedProblem, SharedPage, UnitError};
pub use warning::{InputWarning, WarningKind};
pub use workflow::{
    Approval, RunStatus, RunStep, StepInput, StepProgress, StepState, WorkflowError, WorkflowRun,
};
pub use youtube::YOUTUBE_LINK;

/// Bumped when a change to preparation or parsing makes cached results stale.
pub const PIPELINE_VERSION: u32 = 1;

#[cfg(test)]
mod interface_lists {
    /// The lists the interface would otherwise copy from Rust's enums and rules: the parts of a
    /// source description in order, every task in New job's order and those whose results are
    /// a text, the queue's reasons, the extensions of each kind of file and the kinds each task
    /// reads (the file pickers), and the formats written only from a model that reports layout.
    /// `cargo test` keeps `src/lib/coreLists.json` current, as it does `searchRules.json`; the
    /// interface imports it, so the two sides cannot drift.
    #[test]
    fn export_lists_for_the_interface() -> Result<(), Box<dyn std::error::Error>> {
        use crate::{ExportFormat, InputKind, Task};
        let extensions: serde_json::Map<String, serde_json::Value> = InputKind::EXTENSIONS
            .iter()
            .map(|(kind, exts)| Ok((name(kind)?, serde_json::json!(exts))))
            .collect::<Result<_, serde_json::Error>>()?;
        let task_sources: serde_json::Map<String, serde_json::Value> = Task::ALL
            .iter()
            .map(|t| (t.as_str().to_string(), serde_json::json!(t.source_kinds())))
            .collect();
        let mut layout_formats: Vec<ExportFormat> = Vec::new();
        for format in Task::ALL.iter().flat_map(|t| t.formats()) {
            if format.needs_layout() && !layout_formats.contains(format) {
                layout_formats.push(*format);
            }
        }
        let lists = serde_json::json!({
            "description_fields": crate::DescriptionField::ALL,
            "tasks": Task::ALL,
            "text_tasks": Task::ALL
                .iter()
                .filter(|t| t.yields_text())
                .collect::<Vec<_>>(),
            "queue_reasons": crate::Signal::KINDS,
            "extensions": extensions,
            "task_sources": task_sources,
            "layout_formats": layout_formats,
        });
        crate::test_support::write_interface_json("src/lib/coreLists.json", &lists)
    }

    /// A value's name as it is sent to the interface (`"pdf"`, `"youtube"`).
    fn name(value: &impl serde::Serialize) -> Result<String, serde_json::Error> {
        match serde_json::to_value(value)? {
            serde_json::Value::String(s) => Ok(s),
            other => Ok(other.to_string()),
        }
    }
}
