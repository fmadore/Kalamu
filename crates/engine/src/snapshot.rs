//! Snapshots of corrected text, read by later work instead of a file (#47's first increment).
//!
//! The corrected view of one source under one set-up (the text the researcher would export,
//! corrections and speaker names applied) is frozen as a text file named by its hash in the
//! app's data folder, never changed again, and recorded with where it came from. A later
//! analysis reads it as an ordinary text source carrying `derived_from`: its cache keys hold
//! the snapshot's hash, so a corrected text is analysed again and an unchanged one is not, and
//! a correction made afterwards never changes what an earlier analysis read; it only marks it
//! as out of date.
//!
//! A transcript whose every part read has its times is frozen with its timed lines too
//! (`Snapshot::lines`, `DerivedFrom::timed`), for a translation that keeps them
//! ([`crate::aligned`]). The text itself, and so every cache key made from it, is the same as
//! without them.
//!
//! Every snapshot of a text keeps its turns too (`Snapshot::turns`): where each speaker's
//! words, or each paragraph, sit in the text, read from the export document's parts with the
//! text itself ([`kalamu_core::snapshot::build_with_turns`]), so the two cannot disagree.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use kalamu_core::snapshot::{Built, PartText, TimedPart, build_with_turns};
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

/// A result's corrected text as a snapshot keeps it.
struct Frozen {
    text: String,
    parts: Vec<kalamu_core::SnapshotPart>,
    missing: Vec<u32>,
    task: Task,
    /// A transcript's timed lines, for a translation that keeps them; empty for other text.
    lines: Vec<kalamu_core::Segment>,
    /// Where each turn sits in the text ([`kalamu_core::Turn`]).
    turns: Vec<kalamu_core::Turn>,
}

impl Engine {
    /// The corrected view of a source under a set-up as a snapshot's text, its parts, the
    /// pieces not read (numbered from 1), failed or still to do, and a transcript's timed lines
    /// when every part read has them.
    fn frozen_text(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Frozen, EngineError> {
        if !job.task.yields_text() {
            return Err(EngineError::NotText);
        }
        let doc = self.export_document(job, setup_id, source_id)?;
        // Each part with its segments, on the recording's clock as the export document gives
        // them, so the turns are read with the text and cannot disagree with it.
        let texts: Vec<TimedPart<'_>> = doc
            .parts
            .iter()
            .map(|p| TimedPart {
                text: PartText {
                    piece: &p.piece,
                    text: &p.text,
                    reviewed: p.reviewed,
                    corrected: p.corrected,
                    failed: p.failed.is_some(),
                },
                segments: &p.segments,
            })
            .collect();
        let Built { text, parts, turns } = build_with_turns(job.task, &texts);
        let missing = self
            .store
            .source_pieces(job.id, setup_id, source_id)?
            .iter()
            .filter(|p| !p.status.is_finished() || p.status == PieceStatus::Failed)
            .map(|p| p.index + 1)
            .collect();
        Ok(Frozen {
            text,
            parts,
            missing,
            task: job.task,
            lines: kalamu_core::aligned::lines_of(job.task, &doc.parts).unwrap_or_default(),
            turns,
        })
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
            let Frozen {
                text,
                parts,
                missing,
                task,
                lines,
                turns,
            } = self.frozen_text(&job, setup_id, source.id)?;
            let timed = !lines.is_empty();
            if parts.is_empty() || text.trim().is_empty() {
                continue;
            }
            let sha256 = kalamu_core::fingerprint::sha256_hex(text.as_bytes());
            std::fs::create_dir_all(&self.config.snapshots)
                .map_err(|e| EngineError::Output(e.to_string()))?;
            let path = self.config.snapshots.join(format!("{sha256}.txt"));
            // Named by its content: the same text frozen twice is one file, written again only
            // when the file there no longer holds it.
            write_frozen(&path, &text, &sha256)?;
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
                lines,
                turns,
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
                dense_pages: None,
                title: None,
                derived_from: Some(DerivedFrom {
                    snapshot_id: snapshot.id,
                    job_id,
                    setup_id,
                    source_id: source.id,
                    original_path: source.path.clone(),
                    unit: None,
                    timed,
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
            write_frozen(&path, text, &sha256)?;
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
                dense_pages: None,
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
                Ok(Frozen { text, .. }) => {
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

    /// Writes again the frozen text a source reads when its file in the snapshots folder is
    /// missing or no longer holds it (cut short by a power cut, say), from the store: only when
    /// the result it was frozen from, as corrected now, still gives the very same text, so
    /// nothing else is ever read under its fingerprint. True when it was written.
    pub(crate) fn rewrite_frozen(&self, source: &SourceFile) -> Result<bool, EngineError> {
        let Some(from) = &source.derived_from else {
            return Ok(false);
        };
        let path = std::path::Path::new(&source.path);
        // A unit's pages are a PDF cut from the original, not a text the store can give.
        if from.unit.is_some() || path.parent() != Some(self.config.snapshots.as_path()) {
            return Ok(false);
        }
        let whole = std::fs::read(path)
            .is_ok_and(|bytes| kalamu_core::fingerprint::sha256_hex(&bytes) == source.sha256);
        if whole {
            return Ok(false);
        }
        let job = match self.store.job(from.job_id) {
            Ok(job) => job,
            Err(kalamu_core::StoreError::NotFound) => return Ok(false),
            Err(e) => return Err(e.into()),
        };
        let text = match self.frozen_text(&job, from.setup_id, from.source_id) {
            Ok(Frozen { text, .. }) => text,
            Err(EngineError::Store(kalamu_core::StoreError::NotFound)) => return Ok(false),
            Err(e) => return Err(e),
        };
        if kalamu_core::fingerprint::sha256_hex(text.as_bytes()) != source.sha256 {
            return Ok(false);
        }
        crate::output::write_atomic(path, text.as_bytes())?;
        Ok(true)
    }
}

/// Keeps a frozen text under its fingerprint at `path`: written unless the file there already
/// holds exactly it, so one cut short or emptied by a power cut is made whole the next time the
/// same text is frozen.
fn write_frozen(path: &std::path::Path, text: &str, sha256: &str) -> Result<(), EngineError> {
    let whole = std::fs::read(path)
        .is_ok_and(|bytes| kalamu_core::fingerprint::sha256_hex(&bytes) == sha256);
    if !whole {
        crate::output::write_atomic(path, text.as_bytes())?;
    }
    Ok(())
}
