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
pub mod pages;
pub mod profile;
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
pub mod task;
pub mod tied;
pub mod units;
pub mod warning;
pub mod workflow;
pub mod youtube;

pub use analysis::{AnalysisSchema, Field, FieldAnswer, FieldKind, FieldValue, SchemaError};
pub use attention::{
    QueueCounts, QueueItem, ReasonCounts, ReviewQueue, ReviewScope, ReviewTarget, Signal,
};
pub use batch::{
    BATCH_WINDOW_MS, BatchContext, BatchItem, BatchLimits, BatchPhase, BatchProgress, BatchRecord,
};
pub use dictionary::{
    DictionaryError, DictionaryStore, Entry, KeptApart, Labels, ProposalGroup, ProposalRequest,
};
pub use event::ProgressEvent;
pub use export::{
    ExportDocument, ExportError, ExportFormat, ExportOptions, ExportPart, Exporter, Table,
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
pub use pages::PagesPerCall;
pub use profile::{ExportProfile, ProfileColumn, ProfileError, ProfileStarter, ProfileValue};
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
    TextLine, TextRegion, TextWord, Usage,
};
pub use search::{IndexSize, Passage, PassageKey, PassagePart, SearchIndex};
pub use setup::{MergeModel, MergeStep, PromptSnapshot, Setup, SetupOptions, SetupRequest, Step};
pub use snapshot::{Anchor, AnchorError, DerivedFrom, Snapshot, SnapshotPart};
pub use source::{InputKind, SourceFile};
pub use speakers::{SpeakerIndex, SpeakerNames};
pub use store::{Correction, JobStore, StoreError};
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
    /// source description in order, the tasks whose results are a text, and the queue's reasons.
    /// `cargo test` keeps `src/lib/coreLists.json` current, as it does `searchRules.json`; the
    /// interface imports it, so the two sides cannot drift.
    #[test]
    fn export_lists_for_the_interface() -> Result<(), Box<dyn std::error::Error>> {
        let lists = serde_json::json!({
            "description_fields": crate::DescriptionField::ALL,
            "text_tasks": crate::Task::ALL
                .iter()
                .filter(|t| t.yields_text())
                .collect::<Vec<_>>(),
            "queue_reasons": crate::Signal::KINDS,
        });
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/lib/coreLists.json");
        let json = serde_json::to_string_pretty(&lists)? + "\n";
        if std::fs::read_to_string(&path).ok().as_deref() != Some(json.as_str()) {
            std::fs::write(&path, json)?;
        }
        Ok(())
    }
}
