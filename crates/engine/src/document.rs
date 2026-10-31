//! The document exporters read: a source's stored results under one set-up, in reading order,
//! with the researcher's corrections and speaker names applied, and its `provenance.json`.

use std::collections::BTreeSet;

use kalamu_core::provenance::{GeneratorInfo, ProvenanceMerge, ProvenancePrompt, ProvenanceSource};
use kalamu_core::speakers::{rename_segments, rename_text};
use kalamu_core::{
    CacheKey, ExportDocument, ExportPart, InputKind, Job, PieceRef, PieceStatus, Provenance,
    RequestSetting, Segment, SetupId, SourceId, Task, Timestamp, Usage,
};

use crate::output::preprocessing;
use crate::{Engine, EngineError};

impl Engine {
    /// The document exporters read: stored results in reading order, corrections applied.
    pub fn export_document(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<ExportDocument, EngineError> {
        let setup = job
            .setups
            .iter()
            .find(|s| s.id == setup_id)
            .cloned()
            .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
        let source = job
            .sources
            .iter()
            .find(|s| s.id == source_id)
            .cloned()
            .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
        let corrections = self.store.corrections(job.id)?;
        // The researcher's names for this recording's speakers, applied to every part.
        let speaker_names = self
            .store
            .speaker_names(job.id)?
            .into_iter()
            .find(|n| n.setup_id == setup_id && n.source_id == source_id)
            .map(|n| n.names)
            .unwrap_or_default();
        let mut states: Vec<_> = self
            .store
            .pieces(job.id)?
            .into_iter()
            .filter(|p| p.setup_id == setup_id && p.source_id == source_id)
            .collect();
        states.sort_by_key(|p| p.index);

        let mut parts = Vec::new();
        let mut usage = Usage::default();
        let mut cost: Option<f64> = None;
        let mut versions = BTreeSet::new();
        let mut merge_versions = BTreeSet::new();
        let (mut request, mut merge_request) = (Vec::new(), Vec::new());
        let mut detected = BTreeSet::new();
        let mut first_read: Option<Timestamp> = None;
        let (mut corrected, mut reviewed, mut truncated) = (Vec::new(), Vec::new(), Vec::new());
        let mut failed = Vec::new();
        for state in &states {
            // A piece that failed after its retries is marked in its place, so the rest of the
            // document still reaches the researcher (a table of contents has only its entries).
            if state.status == PieceStatus::Failed && job.task != Task::Contents {
                failed.push(state.index + 1);
                parts.push(ExportPart {
                    piece: state.piece.clone(),
                    text: String::new(),
                    segments: Vec::new(),
                    corrected: false,
                    reviewed: false,
                    fields: Vec::new(),
                    regions: Vec::new(),
                    failed: Some(state.error.clone().unwrap_or_default()),
                });
                continue;
            }
            let Some(key) = &state.cache_key else {
                continue;
            };
            if !state.status.is_finished() {
                continue;
            }
            let Some(result) = self.store.cached_result(&CacheKey(key.clone()))? else {
                continue;
            };
            usage.add(&result.output.usage);
            first_read = Some(first_read.map_or(result.created_at, |t| t.min(result.created_at)));
            if let Some(c) = result.cost_usd {
                cost = Some(cost.unwrap_or(0.0) + c);
            }
            // A table of contents exports its combined entries; its pages count in the usage,
            // the cost and the model versions, as the reading they are.
            let combined = job.task == Task::Contents && state.piece == PieceRef::Whole;
            if result.output.truncated {
                truncated.push(state.index + 1);
            }
            gather(
                if combined {
                    &mut merge_request
                } else {
                    &mut request
                },
                &result.request,
            );
            if let Some(v) = &result.output.model_version {
                if combined {
                    if setup.merge.as_ref().is_none_or(|m| &m.model != v) {
                        merge_versions.insert(v.clone());
                    }
                } else if v != &setup.model {
                    versions.insert(v.clone());
                }
            }
            if job.task == Task::Contents && !combined {
                continue;
            }
            if let Some(lang) = &result.output.detected_language {
                detected.insert(lang.clone());
            }
            let correction = corrections.iter().find(|c| {
                c.setup_id == setup_id && c.source_id == source_id && c.index == state.index
            });
            let corrected_text = correction.and_then(|c| c.text.clone());
            // An analysis's corrected answers replace the model's, and its text is set out from
            // them again.
            let corrected_fields = correction.and_then(|c| c.fields.clone());
            let fields = corrected_fields
                .clone()
                .unwrap_or_else(|| result.output.fields.clone());
            let corrected_text = match (&corrected_fields, &setup.prompt.schema) {
                (Some(answers), Some(schema)) => Some(schema.render(answers)),
                _ => corrected_text,
            };
            let is_reviewed = correction.is_some_and(|c| c.reviewed);
            let number = state.index + 1;
            if corrected_text.is_some() {
                corrected.push(number);
            }
            if is_reviewed {
                reviewed.push(number);
            }
            // A correction replaces the text; the model's timings no longer match it.
            let segments = if corrected_text.is_some() {
                Vec::new()
            } else {
                let previous_end = parts.last().and_then(|p: &ExportPart| match p.piece {
                    PieceRef::Segment { end_ms, .. } => Some(end_ms),
                    PieceRef::Page { .. } | PieceRef::Whole | PieceRef::Passage { .. } => None,
                });
                let renamed = rename_segments(&result.output.segments, &speaker_names);
                absolute(&state.piece, &renamed, previous_end)
            };
            // A corrected page keeps its text searchable, as one block over the page.
            let regions = match (&corrected_text, result.output.regions.is_empty()) {
                (Some(text), false) => vec![kalamu_core::TextRegion {
                    x: 0.05,
                    y: 0.05,
                    width: 0.9,
                    height: 0.9,
                    text: text.clone(),
                }],
                _ => result.output.regions.clone(),
            };
            parts.push(ExportPart {
                piece: state.piece.clone(),
                text: rename_text(
                    corrected_text.as_deref().unwrap_or(&result.output.text),
                    &speaker_names,
                ),
                segments,
                corrected: corrected_text.is_some(),
                reviewed: is_reviewed,
                fields,
                regions,
                failed: None,
            });
        }

        // Offline engines are the providers that need no key.
        let engine = match self.provider(&setup.provider) {
            Ok(provider) if !provider.needs_key() => "offline",
            _ => "cloud",
        };
        let online = source.kind == InputKind::YouTube;
        let mut preprocessing = preprocessing(&source);
        if let Some(n) = job.settings.sample_pages {
            preprocessing.push(format!("sample_first_{n}_pages"));
        }
        let provenance = Provenance {
            schema_version: 1,
            generator: GeneratorInfo {
                name: "Kalamu".into(),
                version: self.config.app_version.clone(),
            },
            created_at: Timestamp::now(),
            task: job.task,
            source: ProvenanceSource {
                file_name: source.file_name.clone(),
                sha256: source.sha256.clone(),
                size_bytes: source.size_bytes,
                url: online.then(|| source.path.clone()),
                title: source.title.clone(),
                read_at: first_read.filter(|_| online),
            },
            engine: engine.into(),
            provider: setup.provider.clone(),
            model: setup.model.clone(),
            model_versions: versions.into_iter().collect(),
            request,
            prompt: ProvenancePrompt::from_snapshot(&setup.prompt),
            merge: setup.merge.as_ref().map(|m| ProvenanceMerge {
                provider: m.provider.clone(),
                model: m.model.clone(),
                model_versions: merge_versions.into_iter().collect(),
                request: merge_request,
                prompt: ProvenancePrompt::from_snapshot(&m.prompt),
            }),
            languages: job.settings.languages.clone(),
            detected_languages: detected.into_iter().collect(),
            options: setup.options.clone(),
            pieces: states.len() as u32,
            corrected,
            reviewed,
            truncated,
            failed,
            preprocessing,
            usage,
            cost_usd: cost,
        };
        Ok(ExportDocument {
            task: job.task,
            source,
            setup,
            parts,
            provenance,
            options: job.output.options.clone(),
        })
    }
}

/// Adds a piece's request settings to those gathered, each name and value once, in the order
/// first seen.
fn gather(into: &mut Vec<RequestSetting>, settings: &[RequestSetting]) {
    for setting in settings {
        if !into.contains(setting) {
            into.push(setting.clone());
        }
    }
}

/// Segment times from a provider are relative to the piece; exports want the recording's clock.
/// Where a piece starts before the previous one ended (a cut with no pause, repeated so the
/// word on it is heard whole), segments that end inside the repeat were already transcribed.
fn absolute(piece: &PieceRef, segments: &[Segment], previous_end: Option<u64>) -> Vec<Segment> {
    let offset = match piece {
        PieceRef::Segment { start_ms, .. } => *start_ms,
        PieceRef::Page { .. } | PieceRef::Whole | PieceRef::Passage { .. } => 0,
    };
    let repeated_until = previous_end.filter(|end| *end > offset);
    segments
        .iter()
        .map(|s| Segment {
            start_ms: s.start_ms + offset,
            end_ms: s.end_ms + offset,
            ..s.clone()
        })
        .filter(|s| repeated_until.is_none_or(|end| s.end_ms > end))
        .collect()
}
