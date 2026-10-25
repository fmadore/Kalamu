//! Domain types and the traits that connect Scriptorium's crates.
//!
//! This crate performs no I/O: it knows nothing about Tauri, SQLite, HTTP or FFmpeg. Types that
//! cross into the front end derive [`ts_rs::TS`]; `cargo test` writes their TypeScript bindings.

pub mod event;
pub mod export;
pub mod fingerprint;
pub mod ids;
pub mod job;
pub mod language;
pub mod provenance;
pub mod provider;
pub mod result;
pub mod setup;
pub mod source;
pub mod store;
pub mod task;
pub mod warning;

pub use event::ProgressEvent;
pub use export::{ExportDocument, ExportError, ExportFormat, ExportOptions, ExportPart, Exporter};
pub use fingerprint::CacheKey;
pub use ids::{JobId, ProviderId, SetupId, SourceId, Timestamp};
pub use job::{
    Job, JobSettings, JobStatus, JobSummary, LineBreaks, OutputDestination, OutputSettings,
};
pub use language::{LanguageChoice, LanguageCode};
pub use provenance::Provenance;
pub use provider::{
    ApiKey, PCM_16K_MONO, PreparedInput, ProcessRequest, Provider, ProviderError, StopSignal,
};
pub use result::{
    PageHalf, PieceRef, PieceState, PieceStatus, ProcessOutput, Segment, StoredResult, TextRegion,
    Usage,
};
pub use setup::{PromptSnapshot, Setup, SetupOptions};
pub use source::{InputKind, SourceFile};
pub use store::{Correction, JobStore, StoreError};
pub use task::Task;
pub use warning::{InputWarning, WarningKind};

/// Bumped when a change to preparation or parsing makes cached results stale.
pub const PIPELINE_VERSION: u32 = 1;
