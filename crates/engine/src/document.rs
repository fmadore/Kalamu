//! The document exporters read: a source's stored results under one set-up, in reading order,
//! with the researcher's corrections and speaker names applied, and its `provenance.json`.

use std::collections::BTreeSet;

use kalamu_core::provenance::{
    GeneratorInfo, ProvenanceDerived, ProvenanceMerge, ProvenancePrompt, ProvenanceRefused,
    ProvenanceSource,
};
use kalamu_core::speakers::{rename_segments, rename_text, renamed_labels};
use kalamu_core::{
    CacheKey, ExportDocument, ExportPart, InputKind, Job, PieceRef, PieceState, PieceStatus,
    Provenance, RequestSetting, Segment, SetupId, SourceFile, SourceId, Task, Timestamp, Usage,
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
        // A table of contents' second step, as a share of both.
        let mut merge_usage = Usage::default();
        let mut merge_cost: Option<f64> = None;
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
            // A table of contents exports its combined entries; its pages count in the usage,
            // the cost and the model versions, as the reading they are.
            let combined = job.task == Task::Contents && state.piece == PieceRef::Whole;
            usage.add(&result.output.usage);
            add_cost(&mut cost, result.cost_usd);
            if combined {
                merge_usage.add(&result.output.usage);
                add_cost(&mut merge_cost, result.cost_usd);
            }
            first_read = Some(first_read.map_or(result.created_at, |t| t.min(result.created_at)));
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
        let preprocessing = done_to(job, &source, &states);
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
                derived_from: self.snapshot_of(&source).map(|s| ProvenanceDerived {
                    pieces: s.parts.len() as u32,
                    reviewed: s.parts.iter().filter(|p| p.reviewed).count() as u32,
                    corrected: s.parts.iter().filter(|p| p.corrected).count() as u32,
                    snapshot_id: s.id,
                    job_id: s.job_id,
                    setup_id: s.setup_id,
                    source_id: s.source_id,
                    task: s.task,
                    original_path: s.original_path,
                    frozen_at: s.created_at,
                    missing: s.missing,
                }),
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
                usage: merge_usage,
                cost_usd: merge_cost,
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
            renamed_speakers: renamed_labels(&speaker_names),
            usage,
            cost_usd: cost,
            refused: self.refused(job, setup_id, source_id)?,
            workflow: self.workflow_provenance(job.id)?,
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

impl Engine {
    /// The answers billed for this document but refused for breaking its fields.
    fn refused(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Option<ProvenanceRefused>, EngineError> {
        let mut refused: Option<ProvenanceRefused> = None;
        for charge in self.store.charges(job.id)? {
            if charge.setup_id != setup_id || charge.source_id != source_id {
                continue;
            }
            let entry = refused.get_or_insert_with(|| ProvenanceRefused {
                answers: 0,
                usage: Usage::default(),
                cost_usd: None,
            });
            entry.answers += 1;
            entry.usage.add(&charge.usage);
            add_cost(&mut entry.cost_usd, charge.cost_usd);
        }
        Ok(refused)
    }
}

/// The fixes provenance lists: of those the source's settings ask for (which its cache key
/// carries all the same), only what was done to it. A spread is split only where one was
/// found, the volume evened out only in a recording, and a sample taken only of a document cut
/// into more pages than it keeps.
fn done_to(job: &Job, source: &SourceFile, states: &[PieceState]) -> Vec<String> {
    let mut done: Vec<String> = preprocessing(source)
        .into_iter()
        .filter(|step| match step.as_str() {
            "split_spreads" => states
                .iter()
                .any(|s| matches!(s.piece, PieceRef::Page { half: Some(_), .. })),
            "even_out_volume" => matches!(source.kind, InputKind::Audio | InputKind::Video),
            _ => true,
        })
        .collect();
    let cut = states
        .iter()
        .any(|s| matches!(s.piece, PieceRef::Page { .. } | PieceRef::Passage { .. }));
    if let Some(n) = job.settings.sample_pages
        && cut
        && source.page_count.is_none_or(|pages| pages > n)
    {
        done.push(format!("sample_first_{n}_pages"));
    }
    done
}

/// Adds a cost to a total that stays unknown until one is known.
fn add_cost(total: &mut Option<f64>, cost: Option<f64>) {
    if let Some(c) = cost {
        *total = Some(total.unwrap_or(0.0) + c);
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
