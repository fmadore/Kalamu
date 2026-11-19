//! An analysis whose fields cite the text (`FieldKind::Cited`): its source sent with its turns
//! numbered, and its answer checked against the text before it is kept
//! ([`kalamu_core::analysis::cited`]).
//!
//! The text is the snapshot's when the source was frozen from a result, with the snapshot's
//! turns, or the turns read again from its text when it was frozen before turns were kept; a
//! plain text file is read in paragraphs. A scan or a recording given directly has no text to
//! quote, and such a job is refused before anything is sent ([`Engine::check_cited`]).
//!
//! The numbered form of the text enters the pieces' cache keys as `cited_turns:<sha256>`, only
//! for set-ups with a cited field: every other key is unchanged.

use std::sync::Arc;

use kalamu_core::analysis::cited::{self, CitedEdit, CitedError, CitedText};
use kalamu_core::{
    AnalysisSchema, Correction, InputKind, Job, JobId, PieceRef, PreparedInput, ProcessOutput,
    Setup, SetupId, SnapshotPart, SourceFile, SourceId, Task, Timestamp, Turn,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::check::AnswerCheck;
use crate::setup::NewJob;
use crate::{Engine, EngineError};

/// A source's text with its parts and turns, and the form the model reads it in.
pub(crate) struct Quotable {
    text: String,
    parts: Vec<SnapshotPart>,
    turns: Vec<Turn>,
    numbered: String,
    fix: String,
}

impl Quotable {
    fn new(text: String, parts: Vec<SnapshotPart>, turns: Vec<Turn>) -> Self {
        let numbered = cited::numbered(&text, &turns);
        let fix = format!(
            "cited_turns:{}",
            kalamu_core::fingerprint::sha256_hex(numbered.as_bytes())
        );
        Self {
            text,
            parts,
            turns,
            numbered,
            fix,
        }
    }

    /// Added to the cache key of a set-up with a cited field, so an answer read from other
    /// turns (or from the same text sent whole) is never taken for this one.
    pub(crate) fn fix(&self) -> &str {
        &self.fix
    }

    /// The input a cited set-up sends for the whole document, and the check its answer must
    /// pass.
    pub(crate) fn call(self: &Arc<Self>, setup: &Setup, stem: &str) -> Option<Cite> {
        let schema = setup
            .prompt
            .schema
            .clone()
            .filter(|s| cited::has_cited(&s.fields))?;
        Some(Cite {
            text: self.clone(),
            schema,
            input: PreparedInput {
                piece: PieceRef::Whole,
                mime: "text/plain".into(),
                bytes: self.numbered.clone().into_bytes(),
                file_name: format!("{stem}-turns.txt"),
            },
        })
    }
}

/// Whether a set-up's fields cite the text.
pub(crate) fn cites(setup: &Setup) -> bool {
    setup
        .prompt
        .schema
        .as_ref()
        .is_some_and(|s| cited::has_cited(&s.fields))
}

/// One cited set-up's call: its input, and the check of its answer.
pub(crate) struct Cite {
    text: Arc<Quotable>,
    schema: AnalysisSchema,
    pub(crate) input: PreparedInput,
}

/// Every quotation looked for in the text: those found kept with their place, speaker and
/// times, the others left out and counted. Refused, and so asked again and billed as any broken
/// answer, only when the model gave quotations and not one of them could be found.
impl AnswerCheck for Cite {
    fn request(&self) -> Option<&str> {
        Some(cited::REQUEST)
    }

    fn check(&self, mut output: ProcessOutput) -> Result<ProcessOutput, String> {
        let doc = CitedText {
            text: &self.text.text,
            parts: &self.text.parts,
            turns: &self.text.turns,
        };
        let counted = cited::cite(&self.schema.fields, &mut output.fields, doc);
        if counted.given > 0 && counted.found == 0 {
            return Err(format!(
                "none of the {} quotations the answer gave could be found in the text",
                counted.given
            ));
        }
        output.text = self.schema.render(&output.fields);
        Ok(output)
    }
}

/// The sentence a researcher reads when a cited analysis is given a source with no text to
/// quote.
pub(crate) fn not_text(file: &str) -> String {
    format!(
        "{file} is not a text: quotations can only be found in a transcript or a text. \
         Transcribe or read it first, then analyse the text with Use this text."
    )
}

impl Engine {
    /// The text a cited analysis of this source quotes, when one of the job's set-ups has a
    /// cited field; none for any other job. A source with no text, or a snapshot that can no
    /// longer be read, is refused with the reason.
    pub(crate) fn quotable(
        &self,
        job: &Job,
        source: &SourceFile,
    ) -> Result<Option<Arc<Quotable>>, String> {
        if job.task != Task::Analysis || !job.setups.iter().any(cites) {
            return Ok(None);
        }
        self.quotable_text(source).map(|q| Some(Arc::new(q)))
    }

    /// A text source's text with its parts and turns.
    pub(crate) fn quotable_text(&self, source: &SourceFile) -> Result<Quotable, String> {
        if source.kind != InputKind::Text {
            return Err(not_text(&source.file_name));
        }
        let read = kalamu_media::TextSource::open(std::path::Path::new(&source.path))
            .and_then(|t| t.text())
            .map_err(|e| e.to_string())?;
        if let Some(from) = &source.derived_from {
            let snapshot = self.store.snapshot(&from.snapshot_id).map_err(|_| {
                "The transcript this text was frozen from can no longer be read. Send it on \
                 again with Use this text in a new job."
                    .to_string()
            })?;
            let turns = if snapshot.turns.is_empty() {
                kalamu_core::snapshot::turns_of(snapshot.task, &read, &snapshot.parts)
            } else {
                snapshot.turns
            };
            return Ok(Quotable::new(read, snapshot.parts, turns));
        }
        let (text, parts) = kalamu_core::snapshot::plain(&read);
        let turns = kalamu_core::snapshot::turns_of(Task::Analysis, &text, &parts);
        Ok(Quotable::new(text, parts, turns))
    }

    /// Refuses a new analysis with a cited field whose sources are not all texts: a scan or a
    /// recording has no words to find a quotation in.
    pub(crate) fn check_cited(&self, new: &NewJob) -> Result<(), EngineError> {
        if new.task != Task::Analysis {
            return Ok(());
        }
        let mut cited = false;
        for request in &new.setups {
            let preset = match request.preset_id.as_deref() {
                Some(id) => self.with_prompts(|p| p.get(id))?,
                None => self.with_prompts(|p| p.default_for(new.task)),
            };
            cited |= preset
                .schema
                .as_ref()
                .is_some_and(|s| cited::has_cited(&s.fields));
        }
        match new.sources.iter().find(|s| s.kind != InputKind::Text) {
            Some(source) if cited => Err(EngineError::CitedNeedsText(source.file_name.clone())),
            _ => Ok(()),
        }
    }
}

/// The text a cited analysis read, for the review to show its quotations in place: the
/// snapshot it was frozen in (none for a text file given directly), the text's hash, the text,
/// and its turns as the quotations number them (`turns[evidence.turn - 1]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CitedSource {
    pub snapshot_id: Option<String>,
    pub sha256: String,
    pub text: String,
    pub turns: Vec<Turn>,
}

impl Engine {
    /// The text one source of a cited analysis read, with its turns.
    pub fn cited_source(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<CitedSource, EngineError> {
        let job = self.store.job(job_id)?;
        let source = job
            .sources
            .iter()
            .find(|s| s.id == source_id)
            .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
        let quotable = self
            .quotable_text(source)
            .map_err(|_| EngineError::CitedNeedsText(source.file_name.clone()))?;
        Ok(CitedSource {
            snapshot_id: source.derived_from.as_ref().map(|d| d.snapshot_id.clone()),
            sha256: source.sha256.clone(),
            text: quotable.text,
            turns: quotable.turns,
        })
    }

    /// Makes one change to a cited field's entries in one piece (an entry's field typed, an
    /// entry or one of its quotations removed), on the answers as the researcher left them:
    /// the model's stay as they are, the piece's corrected answers hold the change, and the
    /// piece stays reviewed or not as it was.
    #[allow(clippy::too_many_arguments)]
    pub fn edit_cited(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
        index: u32,
        key: &str,
        change: &CitedEdit,
    ) -> Result<(), EngineError> {
        let job = self.store.job(job_id)?;
        let schema = job
            .setups
            .iter()
            .find(|s| s.id == setup_id)
            .and_then(|s| s.prompt.schema.as_ref())
            .ok_or_else(|| EngineError::Cited(CitedError::NotCited(key.to_string())))?;
        let pieces = self.store.source_pieces(job_id, setup_id, source_id)?;
        let piece = pieces
            .iter()
            .find(|p| p.index == index)
            .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
        let corrections = self.store.source_corrections(job_id, setup_id, source_id)?;
        let answers = self
            .store
            .answers(piece, &corrections)?
            .ok_or(EngineError::NothingRead)?;
        let fields =
            cited::edit(&schema.fields, &answers, key, change).map_err(EngineError::Cited)?;
        let reviewed = Correction::of(&corrections, piece).is_some_and(|c| c.reviewed);
        let text = Correction::of(&corrections, piece).and_then(|c| c.text.clone());
        self.store.put_correction(&Correction {
            job_id,
            setup_id,
            source_id,
            index,
            text,
            fields: Some(fields),
            reviewed,
            updated_at: Timestamp::now(),
        })?;
        self.write_later(job_id);
        Ok(())
    }

    /// Checks a correction's cited fields before it is kept: every quotation one the model's
    /// answer holds, and how many were left out the model's count ([`cited::keeps_evidence`]).
    /// A correction of anything else is kept as it is.
    pub(crate) fn settle_cited(&self, correction: &mut Correction) -> Result<(), EngineError> {
        let Some(fields) = correction.fields.as_mut() else {
            return Ok(());
        };
        let job = self.store.job(correction.job_id)?;
        let Some(schema) = job
            .setups
            .iter()
            .find(|s| s.id == correction.setup_id)
            .and_then(|s| s.prompt.schema.as_ref())
            .filter(|s| cited::has_cited(&s.fields))
        else {
            return Ok(());
        };
        let pieces = self.store.source_pieces(
            correction.job_id,
            correction.setup_id,
            correction.source_id,
        )?;
        let model = match pieces.iter().find(|p| p.index == correction.index) {
            Some(piece) => self
                .store
                .finished_result(piece)?
                .map(|r| r.output.fields)
                .unwrap_or_default(),
            None => Vec::new(),
        };
        cited::keeps_evidence(&schema.fields, &model, fields).map_err(EngineError::Cited)
    }
}
