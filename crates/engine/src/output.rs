//! Writing results to disk: every chosen format plus `provenance.json`, next to the source or
//! into the chosen folder, from stored results with corrections applied.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use scriptorium_core::provenance::{GeneratorInfo, ProvenancePrompt, ProvenanceSource};
use scriptorium_core::speakers::{rename_segments, rename_text};
use scriptorium_core::{
    CacheKey, ExportDocument, ExportPart, InputKind, Job, JobId, OutputDestination, PieceRef,
    PieceState, ProgressEvent, Provenance, Segment, Setup, SetupId, SourceFile, SourceId,
    Timestamp, Usage,
};

use crate::{Engine, EngineError};

impl Engine {
    /// Writes the outputs of each set-up whose pieces of a source are all done; true when one
    /// could not be written.
    pub(crate) fn export_finished(
        &self,
        job: &Job,
        source_id: SourceId,
        setups: &[&Setup],
    ) -> Result<bool, EngineError> {
        let states = self.store.pieces(job.id)?;
        let mut failed = false;
        for setup in setups {
            let mine: Vec<&PieceState> = states
                .iter()
                .filter(|p| p.setup_id == setup.id && p.source_id == source_id)
                .collect();
            if !mine.is_empty() && mine.iter().all(|p| p.status.is_finished()) {
                match self.write_outputs(job, setup.id, source_id) {
                    Ok(files) => self.emit(ProgressEvent::SourceExported {
                        job_id: job.id,
                        setup_id: setup.id,
                        source_id,
                        files,
                    }),
                    Err(e) => {
                        failed = true;
                        self.emit(ProgressEvent::SourceFailed {
                            job_id: job.id,
                            source_id,
                            error: e.to_string(),
                        });
                    }
                }
            }
        }
        Ok(failed)
    }

    pub(crate) fn write_outputs(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Vec<String>, EngineError> {
        let document = self.export_document(job, setup_id, source_id)?;
        let (dir, base) = output_base(job, &document.source, &document.setup.label);
        std::fs::create_dir_all(&dir).map_err(|e| EngineError::Output(e.to_string()))?;

        let mut written = Vec::new();
        for format in &job.output.formats {
            let Some(exporter) = scriptorium_export::exporter(*format) else {
                continue;
            };
            if !exporter.supports(job.task) {
                continue;
            }
            let bytes = match exporter.export(&document) {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::warn!(?format, error = %e, "export skipped");
                    continue;
                }
            };
            let mut path = dir.join(format!("{base}.{}", format.extension()));
            // Never overwrite the source itself (a .txt or .csv input exported to the same name).
            if path == Path::new(&document.source.path) {
                path = dir.join(format!("{base}.scriptorium.{}", format.extension()));
            }
            write_atomic(&path, &bytes)?;
            written.push(path.display().to_string());
        }
        let provenance = serde_json::to_vec_pretty(&document.provenance)
            .map_err(|e| EngineError::Output(e.to_string()))?;
        let path = dir.join(format!("{base}.provenance.json"));
        write_atomic(&path, &provenance)?;
        written.push(path.display().to_string());
        Ok(written)
    }

    /// The plain-text results a finished job wrote, for summarising them in a new job.
    pub fn text_outputs(&self, job_id: JobId) -> Result<Vec<String>, EngineError> {
        let job = self.store.job(job_id)?;
        let mut out = Vec::new();
        for source in &job.sources {
            for setup in &job.setups {
                let (dir, base) = output_base(&job, source, &setup.label);
                let path = [
                    dir.join(format!("{base}.txt")),
                    dir.join(format!("{base}.scriptorium.txt")),
                    dir.join(format!("{base}.md")),
                ]
                .into_iter()
                .find(|p| p.is_file() && p != Path::new(&source.path));
                if let Some(path) = path {
                    out.push(path.display().to_string());
                }
            }
        }
        Ok(out)
    }

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
            .ok_or(EngineError::Store(scriptorium_core::StoreError::NotFound))?;
        let source = job
            .sources
            .iter()
            .find(|s| s.id == source_id)
            .cloned()
            .ok_or(EngineError::Store(scriptorium_core::StoreError::NotFound))?;
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
        let mut detected = BTreeSet::new();
        let mut first_read: Option<Timestamp> = None;
        let (mut corrected, mut reviewed) = (Vec::new(), Vec::new());
        for state in &states {
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
            if let Some(v) = &result.output.model_version
                && v != &setup.model
            {
                versions.insert(v.clone());
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
                    PieceRef::Page { .. } | PieceRef::Whole => None,
                });
                let renamed = rename_segments(&result.output.segments, &speaker_names);
                absolute(&state.piece, &renamed, previous_end)
            };
            // A corrected page keeps its text searchable, as one block over the page.
            let regions = match (&corrected_text, result.output.regions.is_empty()) {
                (Some(text), false) => vec![scriptorium_core::TextRegion {
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
            });
        }

        let online = source.kind == InputKind::YouTube;
        let mut preprocessing = preprocessing(&source);
        if let Some(n) = job.settings.sample_pages {
            preprocessing.push(format!("sample_first_{n}_pages"));
        }
        let provenance = Provenance {
            schema_version: 1,
            generator: GeneratorInfo {
                name: "Scriptorium".into(),
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
            engine: "cloud".into(),
            provider: setup.provider.clone(),
            model: setup.model.clone(),
            model_versions: versions.into_iter().collect(),
            prompt: ProvenancePrompt::from_snapshot(&setup.prompt),
            languages: job.settings.languages.clone(),
            detected_languages: detected.into_iter().collect(),
            options: setup.options.clone(),
            pieces: parts.len() as u32,
            corrected,
            reviewed,
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

/// Where a source's results go and the name they share: the source's name, with the set-up's
/// label in comparison mode, and `.analysis` for analyses so they never replace the
/// transcription or extraction they summarise.
fn output_base(job: &Job, source: &SourceFile, setup_label: &str) -> (PathBuf, String) {
    let dir = match &job.output.destination {
        OutputDestination::BesideSources => Path::new(&source.path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default(),
        OutputDestination::Folder(folder) => PathBuf::from(folder),
    };
    // A video's name is its title, whose dots are not an extension.
    let stem = if source.kind == InputKind::YouTube {
        source.file_name.clone()
    } else {
        Path::new(&source.file_name)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "result".into())
    };
    let mut base = if job.is_comparison() {
        format!("{stem}.{setup_label}")
    } else {
        stem
    };
    if job.task == scriptorium_core::Task::Analysis {
        base.push_str(".analysis");
    }
    (dir, base)
}

/// Fixes applied to a source's working copies, for provenance and the cache key.
pub(crate) fn preprocessing(source: &SourceFile) -> Vec<String> {
    let mut out = Vec::new();
    if source.split_spreads {
        out.push("split_spreads".to_string());
    }
    if source.even_out_volume {
        out.push("even_out_volume".to_string());
    }
    out
}

/// Segment times from a provider are relative to the piece; exports want the recording's clock.
/// Where a piece starts before the previous one ended (a cut with no pause, repeated so the
/// word on it is heard whole), segments that end inside the repeat were already transcribed.
fn absolute(piece: &PieceRef, segments: &[Segment], previous_end: Option<u64>) -> Vec<Segment> {
    let offset = match piece {
        PieceRef::Segment { start_ms, .. } => *start_ms,
        PieceRef::Page { .. } | PieceRef::Whole => 0,
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

/// Write to a temporary file, then rename, so a crash never leaves half a file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    let tmp = path.with_extension(format!(
        "{}.pending",
        path.extension().and_then(|e| e.to_str()).unwrap_or("out")
    ));
    std::fs::write(&tmp, bytes)
        .map_err(|e| EngineError::Output(format!("{}: {e}", path.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| EngineError::Output(format!("{}: {e}", path.display())))
}
