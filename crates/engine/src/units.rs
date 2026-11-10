//! Document units (#50's first increment): a table of contents' articles and items as pages of
//! its source, which the researcher corrects and then reads, analyses or exports one by one.
//!
//! Units are made once from the corrected entries and then belong to the researcher; the
//! table of contents is never changed by them. A unit is read by freezing its pages, in its
//! order, as a PDF named by its hash in the app's data folder, as corrected text is frozen
//! (#47): the later job reads an ordinary PDF carrying `derived_from`, so caching, resuming,
//! the review and provenance need no case of their own, and a unit changed afterwards marks
//! that job out of date without changing what it read.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};

use kalamu_core::units::{FrozenUnit, file_stem, format_pages, parse_pages, seed, shared_pages};
use kalamu_core::{
    DerivedFrom, DocumentUnit, FieldAnswer, InputKind, JobId, PieceRef, SeedProblem, SetupId,
    SharedPage, Snapshot, SnapshotPart, SourceFile, SourceId, Task, Timestamp, UnitError, UnitId,
};
use kalamu_media::PdfSource;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// The sources of a batch of units, each checked unchanged and opened the first time a unit
/// needs it, so fifty items of one volume hash and parse it once.
type Opened = HashMap<(JobId, SourceId), (SourceFile, PdfSource)>;

/// A unit as the list shows it: its pages as typed, and the pages it shares with others.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnitView {
    pub unit: DocumentUnit,
    pub pages: String,
    pub shared: Vec<SharedPage>,
}

/// A table of contents' units for one source under one set-up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnitsView {
    pub units: Vec<UnitView>,
    /// The entries there are to make units from: none until the pages are combined.
    pub entries: u32,
    /// Whether the units can be read on their own: only a PDF's pages are cut out for now.
    pub readable: bool,
}

/// What making units from the entries gave.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SeedReport {
    pub made: u32,
    /// Entries whose pages could not all be found in the document.
    pub problems: Vec<SeedProblem>,
}

/// A unit made or changed by the researcher: `id` none for a new one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnitEdit {
    #[ts(optional)]
    pub id: Option<UnitId>,
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub title: String,
    /// The pages as typed: `1, 3-5, 8 (left)`.
    pub pages: String,
    /// The entry's other fields, when edited; left as they were when none.
    #[ts(optional)]
    pub fields: Option<Vec<FieldAnswer>>,
}

impl Engine {
    /// The source's pages under the set-up, as the job planned them, in order.
    fn unit_pages(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Vec<PieceRef>, EngineError> {
        Ok(self
            .store
            .source_pieces(job_id, setup_id, source_id)?
            .into_iter()
            .filter(|p| matches!(p.piece, PieceRef::Page { .. }))
            .map(|p| p.piece)
            .collect())
    }

    /// The combined entries as the researcher left them, or none before the pages are combined.
    fn corrected_entries(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Option<Vec<Vec<FieldAnswer>>>, EngineError> {
        let job = self.store.job(job_id)?;
        if job.task != Task::Contents {
            return Err(UnitError::NotContents.into());
        }
        let corrections = self.store.source_corrections(job_id, setup_id, source_id)?;
        let Some(merged) = self
            .store
            .source_pieces(job_id, setup_id, source_id)?
            .into_iter()
            .find(|p| p.piece == PieceRef::Whole)
        else {
            return Ok(None);
        };
        Ok(self
            .store
            .answers(&merged, &corrections)?
            .map(|answers| kalamu_core::contents::entries(&answers).to_vec()))
    }

    fn units_of(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Vec<DocumentUnit>, EngineError> {
        Ok(self
            .store
            .units(job_id)?
            .into_iter()
            .filter(|u| u.setup_id == setup_id && u.source_id == source_id)
            .collect())
    }

    pub fn units_view(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<UnitsView, EngineError> {
        let entries = self
            .corrected_entries(job_id, setup_id, source_id)?
            .map_or(0, |e| e.len() as u32);
        let job = self.store.job(job_id)?;
        let readable = job
            .sources
            .iter()
            .any(|s| s.id == source_id && s.kind == InputKind::Pdf);
        let units = self.units_of(job_id, setup_id, source_id)?;
        let shared = shared_pages(&units);
        Ok(UnitsView {
            units: units
                .into_iter()
                .zip(shared)
                .map(|(unit, shared)| UnitView {
                    pages: format_pages(&unit.pages),
                    unit,
                    shared,
                })
                .collect(),
            entries,
            readable,
        })
    }

    /// One unit per entry of the corrected table of contents, once: afterwards the units are
    /// the researcher's, and a later correction of the entries does not remake them.
    pub fn seed_units(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<SeedReport, EngineError> {
        let entries = self
            .corrected_entries(job_id, setup_id, source_id)?
            .filter(|e| !e.is_empty())
            .ok_or(EngineError::NothingRead)?;
        if !self.units_of(job_id, setup_id, source_id)?.is_empty() {
            return Err(UnitError::AlreadyMade.into());
        }
        let pages = self.unit_pages(job_id, setup_id, source_id)?;
        let (units, problems) = seed((job_id, setup_id, source_id), &entries, &pages, 0);
        self.store.replace_units(&units, &[])?;
        Ok(SeedReport {
            made: units.len() as u32,
            problems,
        })
    }

    /// Saves a unit's title, pages and fields, or makes a new one at the end of the list.
    pub fn save_unit(&self, edit: UnitEdit) -> Result<DocumentUnit, EngineError> {
        let pieces = self.unit_pages(edit.job_id, edit.setup_id, edit.source_id)?;
        let pages = parse_pages(&edit.pages, &pieces).map_err(UnitError::from)?;
        let mut unit = match edit.id {
            Some(id) => {
                let mut unit = self.store.unit(id)?;
                // The pages were checked against the result named, so it must be the unit's.
                if (unit.job_id, unit.setup_id, unit.source_id)
                    != (edit.job_id, edit.setup_id, edit.source_id)
                {
                    return Err(UnitError::OtherResult.into());
                }
                unit.revision += 1;
                unit.updated_at = Timestamp::now();
                unit
            }
            None => {
                self.corrected_entries(edit.job_id, edit.setup_id, edit.source_id)?;
                let units = self.units_of(edit.job_id, edit.setup_id, edit.source_id)?;
                let next = units.iter().map(|u| u.position + 1).max().unwrap_or(0);
                DocumentUnit::new(edit.job_id, edit.setup_id, edit.source_id, next)
            }
        };
        unit.title = edit.title.trim().to_string();
        unit.pages = pages;
        if let Some(fields) = edit.fields {
            unit.fields = fields;
        }
        self.store.put_unit(&unit)?;
        Ok(unit)
    }

    pub fn delete_unit(&self, id: UnitId) -> Result<(), EngineError> {
        Ok(self.store.delete_unit(id)?)
    }

    /// `second`'s pages added after `first`'s, `second` removed: an article and its
    /// continuation, which the table of contents gave as two entries.
    pub fn join_units(&self, first: UnitId, second: UnitId) -> Result<DocumentUnit, EngineError> {
        let mut kept = self.store.unit(first)?;
        let joined = self.store.unit(second)?;
        if first == second
            || (kept.job_id, kept.setup_id, kept.source_id)
                != (joined.job_id, joined.setup_id, joined.source_id)
        {
            return Err(UnitError::NotJoinable.into());
        }
        kept.pages = kalamu_core::units::join(&kept, &joined);
        kept.revision += 1;
        kept.updated_at = Timestamp::now();
        self.store
            .replace_units(std::slice::from_ref(&kept), &[second])?;
        Ok(kept)
    }

    /// Splits a unit before its page `at` (counted from 0): two items an entry took for one.
    /// The second, with the same title for the researcher to change, follows the first.
    pub fn split_unit(&self, id: UnitId, at: u32) -> Result<[DocumentUnit; 2], EngineError> {
        let mut first = self.store.unit(id)?;
        let (before, after) =
            kalamu_core::units::split(&first, at as usize).ok_or(UnitError::BadSplit)?;
        // The units after it move down one place to make room, in the same write.
        let mut changed: Vec<DocumentUnit> = self
            .units_of(first.job_id, first.setup_id, first.source_id)?
            .into_iter()
            .filter(|u| u.position > first.position)
            .map(|mut later| {
                later.position += 1;
                later
            })
            .collect();
        let mut second = DocumentUnit::new(
            first.job_id,
            first.setup_id,
            first.source_id,
            first.position + 1,
        );
        second.title = first.title.clone();
        second.fields = first.fields.clone();
        second.pages = after;
        first.pages = before;
        first.revision += 1;
        first.updated_at = Timestamp::now();
        changed.extend([first.clone(), second.clone()]);
        self.store.replace_units(&changed, &[])?;
        Ok([first, second])
    }

    /// A unit's pages cut out of its source, in its order, and the source they come from.
    fn unit_pdf(
        &self,
        unit: &DocumentUnit,
        opened: &mut Opened,
    ) -> Result<(Vec<u8>, SourceFile), EngineError> {
        if unit.pages.is_empty() {
            return Err(UnitError::NoPages(unit.title.clone()).into());
        }
        let (source, pdf) = match opened.entry((unit.job_id, unit.source_id)) {
            Entry::Occupied(known) => known.into_mut(),
            Entry::Vacant(slot) => {
                let job = self.store.job(unit.job_id)?;
                let source = job
                    .sources
                    .into_iter()
                    .find(|s| s.id == unit.source_id)
                    .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
                if source.kind != InputKind::Pdf {
                    return Err(UnitError::NotPdf(source.file_name.clone()).into());
                }
                kalamu_media::check_unchanged(&source)?;
                let pdf = PdfSource::open(Path::new(&source.path))?;
                slot.insert((source, pdf))
            }
        };
        Ok((pdf.chosen_pages_pdf(&unit.page_numbers())?, source.clone()))
    }

    /// Freezes each unit's pages as a PDF for a later job to read, and returns them as
    /// sources, in the units' order. The same pages frozen twice are one file.
    pub fn unit_sources(&self, ids: &[UnitId]) -> Result<Vec<SourceFile>, EngineError> {
        let mut units = ids
            .iter()
            .map(|id| self.store.unit(*id))
            .collect::<Result<Vec<_>, _>>()?;
        units.sort_by_key(|u| (u.job_id, u.position));
        let mut opened = Opened::new();
        let mut out = Vec::new();
        for unit in units {
            let (bytes, source) = self.unit_pdf(&unit, &mut opened)?;
            let sha256 = kalamu_core::fingerprint::sha256_hex(&bytes);
            std::fs::create_dir_all(&self.config.snapshots)
                .map_err(|e| EngineError::Output(e.to_string()))?;
            let path = self.config.snapshots.join(format!("{sha256}.pdf"));
            if !path.is_file() {
                crate::output::write_atomic(&path, &bytes)?;
            }
            let reviewed = self.reviewed_pages(&unit)?;
            let snapshot = Snapshot {
                id: Snapshot::new_id(),
                job_id: unit.job_id,
                setup_id: unit.setup_id,
                source_id: unit.source_id,
                task: Task::Contents,
                original_path: source.path.clone(),
                sha256: sha256.clone(),
                parts: unit
                    .pages
                    .iter()
                    .map(|page| SnapshotPart {
                        piece: page.clone(),
                        start: 0,
                        end: 0,
                        reviewed: reviewed.contains(page),
                        corrected: false,
                    })
                    .collect(),
                missing: Vec::new(),
                created_at: Timestamp::now(),
            };
            self.store.put_snapshot(&snapshot)?;
            out.push(SourceFile {
                id: SourceId::new(),
                path: path.display().to_string(),
                file_name: format!(
                    "{}.pdf",
                    file_stem(&stem(&source), unit.position, &unit.title)
                ),
                sha256,
                size_bytes: bytes.len() as u64,
                modified_ms: None,
                kind: InputKind::Pdf,
                page_count: Some(unit.pages.len() as u32),
                duration_ms: None,
                split_spreads: false,
                even_out_volume: false,
                title: None,
                derived_from: Some(DerivedFrom {
                    snapshot_id: snapshot.id,
                    job_id: unit.job_id,
                    setup_id: unit.setup_id,
                    source_id: unit.source_id,
                    original_path: source.path.clone(),
                    unit: Some(FrozenUnit::from(&unit)),
                }),
            });
        }
        if out.is_empty() {
            return Err(UnitError::NoneChosen.into());
        }
        Ok(out)
    }

    /// The unit's pages the researcher marked as reviewed in the table of contents.
    fn reviewed_pages(&self, unit: &DocumentUnit) -> Result<Vec<PieceRef>, EngineError> {
        let corrections =
            self.store
                .source_corrections(unit.job_id, unit.setup_id, unit.source_id)?;
        Ok(self
            .store
            .source_pieces(unit.job_id, unit.setup_id, unit.source_id)?
            .into_iter()
            .filter(|p| p.status.is_finished())
            .filter(|p| kalamu_core::Correction::of(&corrections, p).is_some_and(|c| c.reviewed))
            .map(|p| p.piece)
            .filter(|piece| unit.pages.contains(piece))
            .collect())
    }

    /// Writes each unit's pages as a PDF into `folder`, and the list of them, with their
    /// pages in the original and their fields, as `<source> units.csv` and `.json`. Files
    /// already there are not replaced: the new ones are numbered.
    pub fn export_units(&self, ids: &[UnitId], folder: &Path) -> Result<Vec<String>, EngineError> {
        let mut units = ids
            .iter()
            .map(|id| self.store.unit(*id))
            .collect::<Result<Vec<_>, _>>()?;
        units.sort_by_key(|u| (u.job_id, u.position));
        let Some(first) = units.first() else {
            return Err(UnitError::NoneChosen.into());
        };
        let job = self.store.job(first.job_id)?;
        let setup = job
            .setups
            .iter()
            .find(|s| s.id == first.setup_id)
            .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
        let fields = setup
            .prompt
            .schema
            .as_ref()
            .map(kalamu_core::contents::entry_fields)
            .unwrap_or_default();
        std::fs::create_dir_all(folder).map_err(|e| EngineError::Output(e.to_string()))?;
        let mut opened = Opened::new();
        let mut written = Vec::new();
        let mut rows = Vec::new();
        for unit in &units {
            let (bytes, source) = self.unit_pdf(unit, &mut opened)?;
            let name = file_stem(&stem(&source), unit.position, &unit.title);
            let path = free_path(folder, &name, "pdf");
            crate::output::write_atomic(&path, &bytes)?;
            rows.push(kalamu_export::units::Row {
                unit,
                pages: format_pages(&unit.pages),
                file_name: file_name(&path),
                original: source.file_name.clone(),
                original_sha256: source.sha256.clone(),
            });
            written.push(path.display().to_string());
        }
        let source_stem = job
            .sources
            .iter()
            .find(|s| s.id == first.source_id)
            .map(stem)
            .unwrap_or_default();
        let list = format!("{source_stem} units");
        for (extension, bytes) in [
            (
                "csv",
                kalamu_export::units::csv(&rows, &fields)
                    .map_err(|e| EngineError::Output(e.to_string()))?,
            ),
            (
                "json",
                kalamu_export::units::json(&rows)
                    .map_err(|e| EngineError::Output(e.to_string()))?,
            ),
        ] {
            let path = free_path(folder, &list, extension);
            crate::output::write_atomic(&path, &bytes)?;
            written.push(path.display().to_string());
        }
        Ok(written)
    }
}

/// A source's name without its extension.
fn stem(source: &SourceFile) -> String {
    Path::new(&source.file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.file_name.clone())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `folder/name.ext`, or `name (2).ext` and so on when that is taken.
fn free_path(folder: &Path, name: &str, extension: &str) -> PathBuf {
    let mut path = folder.join(format!("{name}.{extension}"));
    let mut n = 2;
    while path.exists() {
        path = folder.join(format!("{name} ({n}).{extension}"));
        n += 1;
    }
    path
}
