use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    DescribedBy, DescriptionField, DocumentId, JobId, LanguageChoice, LanguageCode, ProjectId,
    ProviderId, RequestSetting, SetupOptions, SourceId, StoreError, Task, Timestamp, Usage,
};

/// Written as `<source>.provenance.json` next to every output: enough to tell how a result was
/// produced and to reproduce it.
///
/// Modelled on zmo-ai-pipelines' `provenance.json` (schema 2): source hash, full prompt text and
/// its hash, per-run usage. Kalamu starts its own numbering at 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Provenance {
    /// Bumped when fields change meaning; follows zmo-ai-pipelines' `schema_version`.
    pub schema_version: u32,
    pub generator: GeneratorInfo,
    pub created_at: Timestamp,
    pub task: Task,
    pub source: ProvenanceSource,
    /// The project documents the source is, with what the researcher says of each, as they
    /// stood when the files were written. Absent for a job tied to no project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub documents: Vec<ProvenanceDocument>,
    /// `cloud` or `offline`.
    pub engine: String,
    pub provider: ProviderId,
    pub model: String,
    /// The versions the provider reported, when they differ from `model`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_versions: Vec<String>,
    /// What the requests carried besides the prompt (reasoning effort, output limit, media
    /// resolution), as recorded with each piece's result. A name listed twice means pieces were
    /// read with different values; results stored before these were kept add nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request: Vec<RequestSetting>,
    pub prompt: ProvenancePrompt,
    /// For a table of contents, the model and prompt that combined the pages' readings; the
    /// fields above are those that read each page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge: Option<ProvenanceMerge>,
    pub languages: LanguageChoice,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detected_languages: Vec<LanguageCode>,
    pub options: SetupOptions,
    pub pieces: u32,
    /// Pages or segments the user corrected (numbered as in the export).
    #[serde(default)]
    pub corrected: Vec<u32>,
    /// Pages or segments the user marked as reviewed.
    #[serde(default)]
    pub reviewed: Vec<u32>,
    /// Pages or segments whose answer stopped at the model's output limit, so their end may be
    /// missing; for a table of contents, the pages read and the combined entries alike.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub truncated: Vec<u32>,
    /// Pages or segments that could not be read after their retries, marked in the files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<u32>,
    /// Fixes applied to working copies before processing (`split_spread`, `rotate`…).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocessing: Vec<String>,
    /// The model's speaker labels (`Speaker 2`) the researcher gave names, which replace them
    /// in every file. The names themselves are in the files, not here, so that this record can
    /// be shared as a methods note without the names of the people recorded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub renamed_speakers: Vec<String>,
    /// What the results kept cost; for a table of contents, both steps (`merge` gives the
    /// second's share).
    pub usage: Usage,
    pub cost_usd: Option<f64>,
    /// Answers the provider billed but that nothing was kept from: refused for breaking the
    /// fields and asked for again, or several pages read together whose page marks could not be
    /// used, each page then read again alone. Not in `usage` or `cost_usd`, which count the
    /// results kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub refused: Option<ProvenanceRefused>,
    /// The workflow step this result was read by, when a workflow ran it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub workflow: Option<ProvenanceWorkflow>,
}

/// A project document a source is, as provenance records it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceDocument {
    pub project_id: ProjectId,
    pub project_name: String,
    pub document_id: DocumentId,
    pub title: String,
    /// Each part of its source's description as it stands: the document's own, or else the
    /// project's. Supplied by the researcher, never by a model.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub description: BTreeMap<DescriptionField, ProvenanceDescribed>,
}

/// A part of a source's description, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceDescribed {
    pub value: String,
    pub from: DescribedBy,
}

/// The project documents a job's source is, for the files written from its results. Read only:
/// the engine records them and never organises a project.
pub trait SourceDocuments: Send + Sync {
    fn documents(
        &self,
        job: JobId,
        source: SourceId,
    ) -> Result<Vec<ProvenanceDocument>, StoreError>;
}

/// Where a result stands in the workflow run that produced it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceWorkflow {
    pub run_id: crate::RunId,
    pub workflow_id: String,
    pub name: String,
    /// The definition's version when the run started.
    pub version: u32,
    /// This step, and how many the run has, numbered from 1.
    pub step: u32,
    pub steps: u32,
    pub step_name: String,
    /// The step whose corrected text this one read (numbered from 1); none when it read the
    /// run's files. `source.derived_from` names the snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reads: Option<u32>,
    /// The researcher's go-ahead on the step read, when one was given: after a review, or to
    /// go on although some of its pieces were not read (`partial`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub approval: Option<crate::Approval>,
}

/// Answers billed but not kept, as provenance records them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceRefused {
    pub answers: u32,
    pub usage: Usage,
    pub cost_usd: Option<f64>,
}

/// A table of contents' second step, as provenance records it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceMerge {
    pub provider: ProviderId,
    pub model: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_versions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request: Vec<RequestSetting>,
    pub prompt: ProvenancePrompt,
    /// This step's share of the document's `usage` and `cost_usd`; the rest is the reading of
    /// the pages.
    #[serde(default)]
    pub usage: Usage,
    #[serde(default)]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeneratorInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceSource {
    pub file_name: String,
    pub sha256: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// A YouTube video's address: what was read, since there is no file to keep.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub url: Option<String>,
    /// The video's title, as YouTube gave it when the link was added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    /// When the provider first read the video at that address; it may change or be removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub read_at: Option<Timestamp>,
    /// For text frozen from an earlier result: which result, and how much of it the researcher
    /// had checked when it was frozen. `sha256` above is the frozen text's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub derived_from: Option<ProvenanceDerived>,
}

/// The earlier result a source's text was frozen from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceDerived {
    pub snapshot_id: String,
    pub job_id: crate::JobId,
    pub setup_id: crate::SetupId,
    pub source_id: crate::SourceId,
    pub task: Task,
    /// The original the earlier result was read from.
    pub original_path: String,
    pub frozen_at: Timestamp,
    /// Pieces in the frozen text, and how many the researcher had reviewed or corrected.
    pub pieces: u32,
    pub reviewed: u32,
    pub corrected: u32,
    /// Pieces left out, failed or not yet read (numbered from 1).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<u32>,
    /// For a document unit's pages: the unit as it was read, its pages numbered as in the
    /// original. `pieces` and `reviewed` then count those pages in the table of contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unit: Option<crate::FrozenUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenancePrompt {
    pub preset_id: Option<String>,
    pub preset_name: String,
    /// The full text sent to the model, so results stay reproducible after a preset changes.
    pub text: String,
    pub user_prompt: String,
    /// SHA-256 of `text` followed by `user_prompt`.
    pub sha256: String,
    /// For an analysis, the fields asked for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::AnalysisSchema>,
}

impl ProvenancePrompt {
    pub fn from_snapshot(prompt: &crate::PromptSnapshot) -> Self {
        Self {
            preset_id: prompt.preset_id.clone(),
            preset_name: prompt.preset_name.clone(),
            text: prompt.text.clone(),
            user_prompt: prompt.user_prompt.clone(),
            sha256: crate::fingerprint::sha256_hex(
                format!("{}\n{}", prompt.text, prompt.user_prompt).as_bytes(),
            ),
            schema: prompt.schema.clone(),
        }
    }
}
