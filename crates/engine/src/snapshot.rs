//! Snapshots of corrected text, read by later work instead of a file (#47's first increment).
//!
//! The corrected view of one source under one set-up (the text the researcher would export,
//! corrections and speaker names applied) is frozen as a text file named by its hash in the
//! app's data folder, never changed again, and recorded with where it came from. A later
//! analysis reads it as an ordinary text source carrying `derived_from`: its cache keys hold
//! the snapshot's hash, so a corrected text is analysed again and an unchanged one is not, and
//! a correction made afterwards never changes what an earlier analysis read; it only marks it
//! as out of date.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use kalamu_core::snapshot::{PartText, build};
use kalamu_core::{
    DerivedFrom, InputKind, Job, JobId, PieceStatus, SetupId, Snapshot, SourceFile, SourceId, Task,
    Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// Whether the result a source was frozen from has changed since.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Freshness {
    /// The corrected text is still the one frozen.
    Current,
    /// Corrected or read further since: this job read an earlier version.
    Changed,
    /// The job it came from was deleted; the frozen copy still stands.
    Gone,
}

/// A source of a job that read a snapshot, and how its snapshot stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DerivedState {
    pub source_id: SourceId,
    pub from: DerivedFrom,
    pub freshness: Freshness,
}

impl Engine {
    /// The corrected view of a source under a set-up as a snapshot's text, its parts, and the
    /// pieces not read (numbered from 1), failed or still to do.
    fn frozen_text(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<(String, Vec<kalamu_core::SnapshotPart>, Vec<u32>, Task), EngineError> {
        if !job.task.yields_text() {
            return Err(EngineError::NotText);
        }
        let doc = self.export_document(job, setup_id, source_id)?;
        let texts: Vec<PartText<'_>> = doc
            .parts
            .iter()
            .map(|p| PartText {
                piece: &p.piece,
                text: &p.text,
                reviewed: p.reviewed,
                corrected: p.corrected,
                failed: p.failed.is_some(),
            })
            .collect();
        let (text, parts) = build(&texts);
        let missing = self
            .store
            .source_pieces(job.id, setup_id, source_id)?
            .iter()
            .filter(|p| !p.status.is_finished() || p.status == PieceStatus::Failed)
            .map(|p| p.index + 1)
            .collect();
        Ok((text, parts, missing, job.task))
    }

    /// Freezes the corrected text of every source of a job under one set-up, for a later job
    /// to read, and returns them as sources. Sources with nothing read yet are left out.
    pub fn snapshot(
        &self,
        job_id: JobId,
        setup_id: SetupId,
    ) -> Result<Vec<SourceFile>, EngineError> {
        self.snapshot_sources(job_id, setup_id, None)
    }

    /// Freezes the corrected text of one source of a job, or of all of them with `None`: a
    /// project's document sends only its own text on.
    pub fn snapshot_sources(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        only: Option<SourceId>,
    ) -> Result<Vec<SourceFile>, EngineError> {
        let job = self.store.job(job_id)?;
        if !job.setups.iter().any(|s| s.id == setup_id)
            || only.is_some_and(|id| !job.sources.iter().any(|s| s.id == id))
        {
            return Err(EngineError::Store(kalamu_core::StoreError::NotFound));
        }
        let mut out = Vec::new();
        for source in job
            .sources
            .iter()
            .filter(|s| only.is_none_or(|id| s.id == id))
        {
            let (text, parts, missing, task) = self.frozen_text(&job, setup_id, source.id)?;
            if parts.is_empty() || text.trim().is_empty() {
                continue;
            }
            let sha256 = kalamu_core::fingerprint::sha256_hex(text.as_bytes());
            std::fs::create_dir_all(&self.config.snapshots)
                .map_err(|e| EngineError::Output(e.to_string()))?;
            let path = self.config.snapshots.join(format!("{sha256}.txt"));
            // Named by its content: the same text frozen twice is one file, never rewritten.
            if !path.is_file() {
                crate::output::write_atomic(&path, text.as_bytes())?;
            }
            let snapshot = Snapshot {
                id: Snapshot::new_id(),
                job_id,
                setup_id,
                source_id: source.id,
                task,
                original_path: source.path.clone(),
                sha256: sha256.clone(),
                parts,
                missing,
                created_at: Timestamp::now(),
            };
            self.store.put_snapshot(&snapshot)?;
            let stem = crate::source_stem(&source.file_name, &source.file_name);
            out.push(SourceFile {
                id: SourceId::new(),
                path: path.display().to_string(),
                file_name: format!("{stem}.txt"),
                sha256,
                size_bytes: text.len() as u64,
                modified_ms: None,
                kind: InputKind::Text,
                page_count: None,
                duration_ms: None,
                split_spreads: false,
                even_out_volume: false,
                title: None,
                derived_from: Some(DerivedFrom {
                    snapshot_id: snapshot.id,
                    job_id,
                    setup_id,
                    source_id: source.id,
                    original_path: source.path.clone(),
                    unit: None,
                }),
                pages: Vec::new(),
            });
        }
        if out.is_empty() {
            return Err(EngineError::NothingRead);
        }
        Ok(out)
    }

    /// Texts Kalamu wrote itself (a dictionary's rows, for a model to compare), kept by their
    /// content beside the snapshots, as sources a job reads. `texts` are file names with their
    /// text; the same text twice is one file, never rewritten.
    pub fn text_sources(&self, texts: &[(String, String)]) -> Result<Vec<SourceFile>, EngineError> {
        std::fs::create_dir_all(&self.config.snapshots)
            .map_err(|e| EngineError::Output(e.to_string()))?;
        let mut out = Vec::new();
        for (file_name, text) in texts {
            let sha256 = kalamu_core::fingerprint::sha256_hex(text.as_bytes());
            let path = self.config.snapshots.join(format!("{sha256}.txt"));
            if !path.is_file() {
                crate::output::write_atomic(&path, text.as_bytes())?;
            }
            out.push(SourceFile {
                id: SourceId::new(),
                path: path.display().to_string(),
                file_name: file_name.clone(),
                sha256,
                size_bytes: text.len() as u64,
                modified_ms: None,
                kind: InputKind::Text,
                page_count: None,
                duration_ms: None,
                split_spreads: false,
                even_out_volume: false,
                title: None,
                derived_from: None,
                pages: Vec::new(),
            });
        }
        Ok(out)
    }

    /// For each source of a job read from a snapshot, whether the text it was frozen from has
    /// changed since.
    pub fn derived_states(&self, job_id: JobId) -> Result<Vec<DerivedState>, EngineError> {
        let job = self.store.job(job_id)?;
        let mut out = Vec::new();
        // The jobs read from, each read once however many of this job's sources it gave.
        let mut upstream: HashMap<JobId, Option<Job>> = HashMap::new();
        for source in &job.sources {
            let Some(from) = &source.derived_from else {
                continue;
            };
            // A unit's pages are out of date when the unit now has other pages; its title only
            // names the files.
            if let Some(frozen) = &from.unit {
                let freshness = match self.store.unit(frozen.id) {
                    Ok(unit) if unit.pages == frozen.pages => Freshness::Current,
                    Ok(_) => Freshness::Changed,
                    Err(kalamu_core::StoreError::NotFound) => Freshness::Gone,
                    Err(e) => return Err(e.into()),
                };
                out.push(DerivedState {
                    source_id: source.id,
                    from: from.clone(),
                    freshness,
                });
                continue;
            }
            let read_from = match upstream.entry(from.job_id) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(match self.store.job(from.job_id) {
                    Ok(job) => Some(job),
                    Err(kalamu_core::StoreError::NotFound) => None,
                    Err(e) => return Err(e.into()),
                }),
            };
            let frozen = match read_from {
                Some(read_from) => self.frozen_text(read_from, from.setup_id, from.source_id),
                None => Err(EngineError::Store(kalamu_core::StoreError::NotFound)),
            };
            let freshness = match frozen {
                Ok((text, ..)) => {
                    if kalamu_core::fingerprint::sha256_hex(text.as_bytes()) == source.sha256 {
                        Freshness::Current
                    } else {
                        Freshness::Changed
                    }
                }
                Err(EngineError::Store(kalamu_core::StoreError::NotFound)) => Freshness::Gone,
                Err(e) => return Err(e),
            };
            out.push(DerivedState {
                source_id: source.id,
                from: from.clone(),
                freshness,
            });
        }
        Ok(out)
    }

    /// The snapshot a source was read from, for provenance.
    pub(crate) fn snapshot_of(&self, source: &SourceFile) -> Option<Snapshot> {
        let from = source.derived_from.as_ref()?;
        self.store.snapshot(&from.snapshot_id).ok()
    }
}
