//! Writing results to disk: every chosen format plus `provenance.json`, next to the source or
//! into the chosen folder, from stored results with corrections applied.

use std::path::{Path, PathBuf};

use kalamu_core::{
    ExportError, ExportFormat, Job, JobId, OutputDestination, PieceState, PieceStatus,
    ProgressEvent, Setup, SetupId, SourceFile, SourceId, Task,
};

use crate::control::lock;
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
        let mut failed = false;
        for setup in setups {
            let mine = self.store.source_pieces(job.id, setup.id, source_id)?;
            let settled = |p: &PieceState| {
                p.status.is_finished()
                    || (p.status == PieceStatus::Failed && job.task != Task::Contents)
            };
            if mine.iter().all(settled) && mine.iter().any(|p| p.status.is_finished()) {
                match self.write_each_output(job, setup.id, source_id) {
                    Ok((files, problems)) => {
                        self.emit(ProgressEvent::SourceExported {
                            job_id: job.id,
                            setup_id: setup.id,
                            source_id,
                            files,
                        });
                        // A format that could not be written, the others written all the same.
                        for error in problems {
                            failed = true;
                            self.emit(ProgressEvent::SourceFailed {
                                job_id: job.id,
                                source_id,
                                error,
                            });
                        }
                    }
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

    /// Writes a source's outputs under one set-up and returns the files written; a format that
    /// could not be written fails the call, once the others are written.
    pub(crate) fn write_outputs(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<Vec<String>, EngineError> {
        let (files, problems) = self.write_each_output(job, setup_id, source_id)?;
        if problems.is_empty() {
            Ok(files)
        } else {
            Err(EngineError::Output(problems.join("; ")))
        }
    }

    /// Writes every chosen format a task has and `provenance.json`, and returns the files
    /// written and why any format could not be. A format the task does not have is skipped.
    /// Files marked out of date are no longer so once every format is written.
    pub(crate) fn write_each_output(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<(Vec<String>, Vec<String>), EngineError> {
        let _writing = lock(&self.writing);
        // The mark is read under the lock every write takes, before what is written is read:
        // a save after this raises it, and clearing this generation leaves that save's mark.
        let outdated = self
            .store
            .outdated_files(Some(job.id))?
            .into_iter()
            .find(|m| m.setup_id == setup_id && m.source_id == source_id);
        let (written, problems) = self.write_files(job, setup_id, source_id)?;
        if problems.is_empty()
            && let Some(mark) = outdated
        {
            self.store.clear_outdated(&mark)?;
        }
        Ok((written, problems))
    }

    /// [`Engine::write_each_output`]'s files, with the lock held.
    fn write_files(
        &self,
        job: &Job,
        setup_id: SetupId,
        source_id: SourceId,
    ) -> Result<(Vec<String>, Vec<String>), EngineError> {
        let document = self.export_document(job, setup_id, source_id)?;
        let (dir, base) = output_base(job, &document.source, &document.setup.label);
        std::fs::create_dir_all(&dir).map_err(|e| EngineError::Output(e.to_string()))?;

        let mut written = Vec::new();
        let mut problems = Vec::new();
        // A searchable PDF and ALTO need a model that reports where text sits: in a comparison
        // they are chosen for the set-ups whose model does, and skipped for the others.
        let layout = {
            let catalog = self.current_catalog();
            job.setups
                .iter()
                .find(|s| s.id == setup_id)
                .and_then(|s| catalog.model(&s.provider, &s.model))
                .is_none_or(|m| m.layout)
        };
        for format in &job.output.formats {
            if format.needs_layout() && !layout {
                continue;
            }
            let Some(exporter) = kalamu_export::exporter(*format) else {
                continue;
            };
            if !exporter.supports(job.task) {
                continue;
            }
            let bytes = match exporter.export(&document) {
                Ok(bytes) => bytes,
                Err(ExportError::Unsupported(_)) => continue,
                Err(e) => {
                    tracing::warn!(?format, error = %e, "export failed");
                    problems.push(e.to_string());
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
        Ok((written, problems))
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
        // Read and written under the lock the job's own writes take, so whichever comes last
        // writes from what is stored then.
        let _writing = lock(&self.writing);
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
        self.write_outdated_before_reading(job_id);
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
    /// run that wrote it and leaves out files moved or deleted since. Files out of date are
    /// written first, so what is opened from the list holds the last corrections.
    pub fn outputs(&self, job_id: JobId) -> Result<Vec<String>, EngineError> {
        self.write_outdated_before_reading(job_id);
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

impl Engine {
    /// A job's files out of date, written before they are listed or read. One that cannot be
    /// written is reported as the job's `SourceFailed` and listed as it is on disk.
    fn write_outdated_before_reading(&self, job_id: JobId) {
        if let Err(e) = self.write_outdated_of(Some(job_id)) {
            tracing::warn!(job = %job_id, error = %e, "files out of date not written");
        }
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
/// label in comparison mode, and `.analysis`, `.contents`, `.corrected` or `.translated.` and
/// the language's code after it so they never replace the transcription or extraction they
/// read, the text file corrected, or a translation of the same text into another language.
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
        Task::Translation => {
            base.push_str(".translated");
            // A code is letters and hyphens (`fr`, `zh-hans`); anything else is left out of a
            // file name.
            let code: String = job
                .settings
                .languages
                .codes()
                .first()
                .map(|c| c.as_str())
                .unwrap_or_default()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !code.is_empty() {
                base.push('.');
                base.push_str(&code);
            }
        }
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
    // A video's name is its title, and a document of pages its name, whose dots are not an
    // extension.
    let stem = if source.names_itself() {
        source.file_name.clone()
    } else {
        crate::source_stem(&source.file_name, "result")
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
