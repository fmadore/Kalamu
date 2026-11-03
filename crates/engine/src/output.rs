//! Writing results to disk: every chosen format plus `provenance.json`, next to the source or
//! into the chosen folder, from stored results with corrections applied.

use std::path::{Path, PathBuf};

use kalamu_core::{
    ExportFormat, InputKind, Job, JobId, OutputDestination, PieceState, PieceStatus, ProgressEvent,
    Setup, SetupId, SourceFile, SourceId, Task,
};

use crate::{Engine, EngineError};

impl Engine {
    /// Writes the outputs of each set-up whose pieces of a source are all settled, done or failed
    /// after their retries, with at least one done; true when one could not be written. A table
    /// of contents waits for all of them: without its combined entries it has nothing to write.
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
            let settled = |p: &&PieceState| {
                p.status.is_finished()
                    || (p.status == PieceStatus::Failed && job.task != Task::Contents)
            };
            if mine.iter().all(settled) && mine.iter().any(|p| p.status.is_finished()) {
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
            let Some(exporter) = kalamu_export::exporter(*format) else {
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
                path = dir.join(format!("{base}.kalamu.{}", format.extension()));
            }
            write_atomic(&path, &bytes)?;
            written.push(path.display().to_string());
        }
        written.push(write_provenance(&dir, &base, &document)?);
        Ok(written)
    }

    /// Writes again a source's `provenance.json`, and the `.json` result that holds it, under
    /// each set-up that wrote them: after something they record changed outside the job, such
    /// as the project documents the source is. The other files are left as they are, nothing is
    /// read again, and a running job is left to write its own.
    pub fn rewrite_provenance(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<Vec<String>, EngineError> {
        if self.is_running(job_id) {
            return Ok(Vec::new());
        }
        let job = self.store.job(job_id)?;
        let Some(source) = job.sources.iter().find(|s| s.id == source_id) else {
            return Ok(Vec::new());
        };
        let mut written = Vec::new();
        for setup in &job.setups {
            let (dir, base) = output_base(&job, source, &setup.label);
            if existing_output(&dir, &base, "provenance.json", &source.path).is_none() {
                continue;
            }
            let document = self.export_document(&job, setup.id, source_id)?;
            // Only a `.json` the job wrote: a file of the same name it did not choose is not
            // its own.
            let json = job
                .output
                .formats
                .contains(&ExportFormat::Json)
                .then(|| existing_output(&dir, &base, "json", &source.path))
                .flatten();
            if let (Some(path), Some(exporter)) =
                (json, kalamu_export::exporter(ExportFormat::Json))
            {
                let bytes = exporter
                    .export(&document)
                    .map_err(|e| EngineError::Output(e.to_string()))?;
                write_atomic(&path, &bytes)?;
                written.push(path.display().to_string());
            }
            written.push(write_provenance(&dir, &base, &document)?);
        }
        Ok(written)
    }

    /// The plain-text results a finished job wrote, for summarising them in a new job.
    pub fn text_outputs(&self, job_id: JobId) -> Result<Vec<String>, EngineError> {
        let job = self.store.job(job_id)?;
        let mut out = Vec::new();
        for source in &job.sources {
            for setup in &job.setups {
                let (dir, base) = output_base(&job, source, &setup.label);
                let path = ["txt", "md"]
                    .into_iter()
                    .find_map(|ext| existing_output(&dir, &base, ext, &source.path));
                if let Some(path) = path {
                    out.push(path.display().to_string());
                }
            }
        }
        Ok(out)
    }

    /// Every file a job has written that is still there: each chosen format, then
    /// `provenance.json`, for every source and set-up. Read from disk, so the list outlives the
    /// run that wrote it and leaves out files moved or deleted since.
    pub fn outputs(&self, job_id: JobId) -> Result<Vec<String>, EngineError> {
        let job = self.store.job(job_id)?;
        let extensions: Vec<&str> = job
            .output
            .formats
            .iter()
            .filter(|f| kalamu_export::exporter(**f).is_some_and(|e| e.supports(job.task)))
            .map(|f| f.extension())
            .chain(["provenance.json"])
            .collect();
        let mut out = Vec::new();
        for source in &job.sources {
            for setup in &job.setups {
                let (dir, base) = output_base(&job, source, &setup.label);
                out.extend(
                    extensions
                        .iter()
                        .filter_map(|ext| existing_output(&dir, &base, ext, &source.path))
                        .map(|p| p.display().to_string()),
                );
            }
        }
        Ok(out)
    }
}

/// Writes `{base}.provenance.json`; returns its path.
fn write_provenance(
    dir: &Path,
    base: &str,
    document: &kalamu_core::ExportDocument,
) -> Result<String, EngineError> {
    let provenance = serde_json::to_vec_pretty(&document.provenance)
        .map_err(|e| EngineError::Output(e.to_string()))?;
    let path = dir.join(format!("{base}.provenance.json"));
    write_atomic(&path, &provenance)?;
    Ok(path.display().to_string())
}

/// Where a source's results go and the name they share: the source's name, with the set-up's
/// label in comparison mode, and `.analysis`, `.contents` or `.corrected` after it so they never
/// replace the transcription or extraction they read, or the text file corrected.
///
/// Two sources that would share a name in one folder (`scan.pdf` and `scan.tif`, or two
/// `scan.pdf` from different folders sent to one) would write over each other's files: the later
/// ones in the job are numbered, `scan (2)`, as Windows names copies. Windows ignores case, so
/// the comparison does too.
fn output_base(job: &Job, source: &SourceFile, setup_label: &str) -> (PathBuf, String) {
    let (dir, mut stem) = place(job, source);
    let key = |dir: &Path, stem: &str| (dir.to_string_lossy().to_lowercase(), stem.to_lowercase());
    let mine = key(&dir, &stem);
    let earlier = job
        .sources
        .iter()
        .take_while(|s| s.id != source.id)
        .filter(|s| {
            let (d, s) = place(job, s);
            key(&d, &s) == mine
        })
        .count();
    if earlier > 0 {
        stem = format!("{stem} ({})", earlier + 1);
    }
    let mut base = if job.is_comparison() {
        format!("{stem}.{setup_label}")
    } else {
        stem
    };
    match job.task {
        Task::Analysis => base.push_str(".analysis"),
        Task::Contents => base.push_str(".contents"),
        Task::TextCorrection => base.push_str(".corrected"),
        Task::TextExtraction | Task::AudioTranscription => {}
    }
    (dir, base)
}

/// The folder a source's results go to, and the name they start from: the source's own.
fn place(job: &Job, source: &SourceFile) -> (PathBuf, String) {
    // Text frozen from an earlier result is kept in the app's data folder: its results go
    // beside the original it was read from.
    let beside = source
        .derived_from
        .as_ref()
        .map_or(source.path.as_str(), |d| d.original_path.as_str());
    let dir = match &job.output.destination {
        OutputDestination::BesideSources => Path::new(beside)
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
    (dir, stem)
}

/// The result with this extension, if it is on disk: `{base}.{ext}`, or the renamed copy written
/// when that name was the source itself, never the source.
fn existing_output(dir: &Path, base: &str, ext: &str, source: &str) -> Option<PathBuf> {
    [
        format!("{base}.{ext}"),
        format!("{base}.kalamu.{ext}"),
        // Written before the app was renamed from Scriptorium.
        format!("{base}.scriptorium.{ext}"),
    ]
    .into_iter()
    .map(|name| dir.join(name))
    .find(|p| p.is_file() && p != Path::new(source))
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

/// Write to a temporary file, then rename, so a crash never leaves half a file.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    let tmp = path.with_extension(format!(
        "{}.pending",
        path.extension().and_then(|e| e.to_str()).unwrap_or("out")
    ));
    std::fs::write(&tmp, bytes)
        .map_err(|e| EngineError::Output(format!("{}: {e}", path.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| EngineError::Output(format!("{}: {e}", path.display())))
}
