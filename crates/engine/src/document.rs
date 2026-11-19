//! The document exporters read: a source's stored results under one set-up, in reading order,
//! with the researcher's corrections and speaker names applied, and its `provenance.json`.

use std::collections::BTreeSet;

use kalamu_core::provenance::{
    GeneratorInfo, ProvenanceDerived, ProvenanceDocument, ProvenanceMerge, ProvenancePage,
    ProvenancePrompt, ProvenanceRefused, ProvenanceSource,
};
use kalamu_core::speakers::{rename_segments, renamed_labels, text_as_left};
use kalamu_core::{
    CacheKey, CutReason, ExportDocument, ExportPart, InputKind, Job, PieceRef, PieceState,
    PieceStatus, Provenance, RequestSetting, Segment, SetupId, SourceFile, SourceId, Task,
    Timestamp, Usage,
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
        // Only this source's rows are read: a job of thousands of sources writes each one's
        // files without reading the others'.
        let corrections = self.store.source_corrections(job.id, setup_id, source_id)?;
        // The researcher's names for this recording's speakers, applied to every part.
        let speakers = kalamu_core::SpeakerIndex::new(
            self.store
                .source_speaker_names(job.id, setup_id, source_id)?,
        );
        let speaker_names = speakers.of(setup_id, source_id);
        let states = self.store.source_pieces(job.id, setup_id, source_id)?;
        // The researcher's regions this set-up's files carry, handed to each page's part.
        let mut drawn = self.export_regions(job, source_id, &states)?;

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
        let (mut failed, mut cut_by_filter) = (Vec::new(), Vec::new());
        let mut lines_changed = Vec::new();
        let mut listed_by_hand = Vec::new();
        for state in &states {
            // A piece that failed after its retries is marked in its place, so the rest of the
            // document still reaches the researcher (a table of contents has only its entries).
            if state.status == PieceStatus::Failed && job.task != Task::Contents {
                failed.push(state.index + 1);
                let (regions, researcher) = self.export_geometry(
                    &source,
                    &state.piece,
                    &[],
                    None,
                    None,
                    &drawn.on(&state.piece),
                );
                parts.push(ExportPart {
                    piece: state.piece.clone(),
                    text: String::new(),
                    segments: Vec::new(),
                    corrected: false,
                    reviewed: false,
                    fields: Vec::new(),
                    regions,
                    failed: Some(state.error.clone().unwrap_or_default()),
                    drawn_dpi: None,
                    read_at: None,
                    corrected_at: None,
                    researcher,
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
                if result.output.cut_reason == CutReason::Filter {
                    cut_by_filter.push(state.index + 1);
                }
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
            let correction = kalamu_core::Correction::of(&corrections, state);
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
            // The items of the list answers the researcher wrote, not the model.
            if let Some(answers) = &corrected_fields {
                listed_by_hand.extend(
                    kalamu_core::list_items::by_hand(&result.output.fields, answers)
                        .into_iter()
                        .map(|(field, items)| kalamu_core::provenance::ProvenanceListed {
                            part: number,
                            field,
                            items,
                        }),
                );
            }
            // A correction replaces the text; the model's timings no longer match it. A
            // translated transcript's lines keep theirs while the correction keeps one
            // paragraph per line; otherwise its times are lost, and said to be.
            let segments = if let Some(text) = &corrected_text {
                if job.task == Task::Translation && !result.output.segments.is_empty() {
                    let lines = rename_segments(&result.output.segments, speaker_names);
                    kalamu_core::aligned::as_corrected(&lines, text).unwrap_or_else(|_| {
                        lines_changed.push(number);
                        Vec::new()
                    })
                } else {
                    Vec::new()
                }
            } else {
                let previous_end = parts.last().and_then(|p: &ExportPart| match p.piece {
                    PieceRef::Segment { end_ms, .. } => Some(end_ms),
                    PieceRef::Page { .. } | PieceRef::Whole | PieceRef::Passage { .. } => None,
                });
                let renamed = rename_segments(&result.output.segments, speaker_names);
                absolute(&state.piece, &renamed, previous_end)
            };
            // A corrected page keeps its words' places where the engine placed each word;
            // otherwise its text stays searchable as one block over the page, also when the
            // engine read nothing there and the researcher typed the page. The researcher's
            // boxes are laid on the model's regions first, and their own regions beside them.
            let (regions, researcher) = self.export_geometry(
                &source,
                &state.piece,
                &result.output.regions,
                Some(key),
                corrected_text.as_deref(),
                &drawn.on(&state.piece),
            );
            parts.push(ExportPart {
                piece: state.piece.clone(),
                text: text_as_left(
                    corrected_text.as_deref(),
                    &result.output.text,
                    speaker_names,
                ),
                segments,
                corrected: corrected_text.is_some(),
                reviewed: is_reviewed,
                fields,
                regions,
                failed: None,
                drawn_dpi: kalamu_core::provenance::page_resolution(&result.request),
                read_at: Some(result.created_at),
                corrected_at: correction
                    .filter(|_| corrected_text.is_some())
                    .map(|c| c.updated_at),
                researcher,
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
                    unit: source.derived_from.as_ref().and_then(|d| d.unit.clone()),
                }),
                pages: ProvenancePage::of(&source.pages),
            },
            documents: self.source_documents(job, source_id),
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
            cut_by_filter,
            failed,
            lines_changed,
            preprocessing,
            renamed_speakers: renamed_labels(speaker_names),
            listed_by_hand,
            passages: self.passages_accepted(job, setup_id, source_id, &states)?,
            regions: self.regions_recorded(job, source_id, &states)?,
            region_readings: self.readings_accepted(job, setup_id, source_id, &states)?,
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
        for charge in self.store.source_charges(job.id, setup_id, source_id)? {
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

impl Engine {
    /// The project documents a source is, as they stand. A project that cannot be read leaves
    /// them out rather than the files unwritten.
    fn source_documents(&self, job: &Job, source_id: SourceId) -> Vec<ProvenanceDocument> {
        let Some(documents) = self.documents.get() else {
            return Vec::new();
        };
        documents.documents(job.id, source_id).unwrap_or_else(|e| {
            tracing::warn!(job = %job.id, error = %e, "documents left out of provenance");
            Vec::new()
        })
    }
}
