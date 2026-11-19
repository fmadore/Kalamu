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
    /// Pages or segments whose answer stopped before its end, at the model's output limit or
    /// for another reason the provider gave, so their end may be missing; for a table of
    /// contents, the pages read and the combined entries alike.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub truncated: Vec<u32>,
    /// Those of `truncated` that a provider's filter stopped, rather than the output limit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cut_by_filter: Vec<u32>,
    /// Pages or segments that could not be read after their retries, marked in the files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<u32>,
    /// Passages of a translated transcript whose correction no longer has one paragraph per
    /// line of the transcript: their times could not be kept, so no subtitles are written.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines_changed: Vec<u32>,
    /// Fixes applied to working copies before processing (`split_spread`, `rotate`…).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocessing: Vec<String>,
    /// The model's speaker labels (`Speaker 2`) the researcher gave names, which replace them
    /// in every file. The names themselves are in the files, not here, so that this record can
    /// be shared as a methods note without the names of the people recorded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub renamed_speakers: Vec<String>,
    /// The items of an analysis's list answers that the researcher wrote by hand, added or
    /// renamed, which the model's own answer did not hold: part by part and field by field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<ProvenanceListed>>")]
    pub listed_by_hand: Vec<ProvenanceListed>,
    /// The quotations of each field of entries that cite the text, part by part: how many the
    /// model gave, each looked for in the text read (`source.derived_from` names the snapshot),
    /// and how many could not be found there and were left out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<ProvenanceCited>>")]
    pub cited: Vec<ProvenanceCited>,
    /// Passages the researcher asked a model to correct from the review and accepted: each
    /// written into their corrections, the model's output left as it was. Not in `usage` or
    /// `cost_usd`, which count the results of the job's own pieces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passages: Vec<ProvenancePassage>,
    /// Regions the researcher drew over the scan, or whose box they adjusted, kept beside the
    /// stored results, which they never change. Exports' positions remain the engine's own.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<ProvenanceRegion>>")]
    pub regions: Vec<ProvenanceRegion>,
    /// Regions of the page the researcher asked a model to read again and accepted: each
    /// written as the text of a region they drew, or into their corrections, the model's output
    /// for the page left as it was. Not in `usage` or `cost_usd`, which count the results of the
    /// job's own pieces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<ProvenanceRegionReading>>")]
    pub region_readings: Vec<ProvenanceRegionReading>,
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

/// Items one list answer holds that the researcher wrote, not the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceListed {
    /// The page or segment, numbered as in the export.
    pub part: u32,
    /// The field's key.
    pub field: String,
    pub items: Vec<String>,
}

/// The quotations of one cited field in one part, as the model's answer was checked against
/// the text: the researcher's removals are in the files, not here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceCited {
    /// The page or segment, numbered as in the export.
    pub part: u32,
    /// The field's key.
    pub field: String,
    /// The quotations the model gave, each looked for in the text.
    pub checked: u32,
    /// Those not found, left out.
    pub left_out: u32,
    /// The model's entries left out because none of their quotations was found.
    #[serde(default, skip_serializing_if = "is_zero")]
    #[ts(optional, as = "Option<u32>")]
    pub entries_left_out: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
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

/// A passage corrected by a model at the researcher's request, as provenance records it: who
/// proposed it, with what prompt, and that the researcher accepted it (edited or as it was).
/// The texts themselves are in the files, as corrections are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenancePassage {
    /// The page or segment, numbered as in the export.
    pub piece: u32,
    pub provider: ProviderId,
    pub model: String,
    pub prompt: ProvenancePrompt,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request: Vec<RequestSetting>,
    /// Characters of the passage, and of the context sent on each side of it.
    pub passage_chars: u32,
    pub context_chars: u32,
    /// The researcher changed the model's proposal before accepting it.
    pub edited: bool,
    pub cost_usd: Option<f64>,
    pub asked_at: Timestamp,
    pub accepted_at: Timestamp,
}

/// A region the researcher drew or adjusted, as provenance records it: where it is, who placed
/// it and what it adjusts. Whether it has text, not the text itself, as for passages: a methods
/// note can be shared without what the researcher wrote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceRegion {
    pub id: String,
    /// The page's number in the source, from 1.
    pub page: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub half: Option<crate::PageHalf>,
    /// Fractions of the page, or of its half, from its top-left corner.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub origin: crate::RegionOrigin,
    /// The engine's region whose box it replaces: the stored result's cache key and the
    /// region's id there. None for a region drawn on its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub adjusts: Option<crate::AdjustedRegion>,
    pub has_text: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl ProvenanceRegion {
    pub fn of(region: &crate::DrawnRegion) -> Self {
        Self {
            id: region.id.clone(),
            page: region.page,
            half: region.half,
            x: region.x,
            y: region.y,
            width: region.width,
            height: region.height,
            origin: crate::RegionOrigin::Manual,
            adjusts: region.adjusts.clone(),
            has_text: !region.text.trim().is_empty(),
            created_at: region.created_at,
            updated_at: region.updated_at,
        }
    }
}

/// Where an accepted reading of a region went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ReadingPlace {
    /// The text of a region the researcher drew.
    Region,
    /// The page's text, in place of the words the region holds there.
    Text,
}

/// A region read again by a model at the researcher's request and accepted, as provenance
/// records it: which region, who read it, with what prompt, where the reading went and whether
/// the researcher changed it. Not the text, which is in the files or in the region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenanceRegionReading {
    /// The page's number in the source, from 1.
    pub page: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub half: Option<crate::PageHalf>,
    /// The region read: a drawn region's id, or an engine's.
    pub region: String,
    /// The part of the page sent, in fractions of the page or of its half.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub provider: ProviderId,
    pub model: String,
    pub prompt: ProvenancePrompt,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request: Vec<RequestSetting>,
    pub went_to: ReadingPlace,
    /// The researcher changed the model's reading before accepting it.
    pub edited: bool,
    pub cost_usd: Option<f64>,
    pub asked_at: Timestamp,
    pub accepted_at: Timestamp,
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
    /// For a document of image files, its pages in the order read: each file where it lies,
    /// its hash and identity, and the number of the first page it holds. `sha256` above is the
    /// hash of these files' contents in this order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<ProvenancePage>>")]
    pub pages: Vec<ProvenancePage>,
}

/// One file of a document of image files, as `provenance.json` records it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProvenancePage {
    /// The document's page this file begins, numbered from 1 as in the files written.
    pub page: u32,
    #[serde(flatten)]
    pub file: crate::PageFile,
}

impl ProvenancePage {
    /// A document's pages, each with the number of its first page.
    pub fn of(pages: &[crate::PageFile]) -> Vec<Self> {
        let mut first = 1;
        pages
            .iter()
            .map(|file| {
                let page = Self {
                    page: first,
                    file: file.clone(),
                };
                first += file.frames.max(1);
                page
            })
            .collect()
    }
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

/// The request setting that records the resolution a PDF page was drawn at for an engine that
/// reads images only (Tesseract), as `300 dpi`: kept with each result, so ALTO measures the page
/// as it was read. Results stored before it was recorded have none, and were drawn at 300 dpi.
pub const PAGE_RESOLUTION: &str = "page_resolution";

/// The resolution a result's page was drawn at, from its request settings ([`PAGE_RESOLUTION`]).
pub fn page_resolution(request: &[RequestSetting]) -> Option<f32> {
    request
        .iter()
        .find(|setting| setting.name == PAGE_RESOLUTION)
        .and_then(|setting| setting.value.trim_end_matches(" dpi").parse().ok())
        .filter(|dpi: &f32| dpi.is_finite() && *dpi > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_resolution_a_page_was_drawn_at_is_read_back() {
        let drawn = |value: &str| vec![RequestSetting::new(PAGE_RESOLUTION, value)];
        assert_eq!(page_resolution(&drawn("300 dpi")), Some(300.0));
        assert_eq!(page_resolution(&drawn("241.32 dpi")), Some(241.32));
        assert_eq!(page_resolution(&drawn("none")), None);
        assert_eq!(page_resolution(&[RequestSetting::new("engine", "x")]), None);
    }
}
