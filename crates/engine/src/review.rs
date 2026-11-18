//! What the review and comparison views read: each piece's AI output with the user's
//! correction on top.

use std::collections::BTreeMap;

use kalamu_core::list_items::{add_items, holds_item, rename_item, split_items};
use kalamu_core::speakers::{labels, rename_segments, rename_text};
use kalamu_core::timed::timed_lines;
use kalamu_core::{
    Correction, CutReason, FieldAnswer, FieldKind, JobId, PieceRef, PieceStatus, RowEdit, SetupId,
    SourceId, Task, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// One piece as the review view shows it: the AI output, the correction on top, and status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PieceView {
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub index: u32,
    pub piece: PieceRef,
    pub status: PieceStatus,
    pub error: Option<String>,
    /// What the model returned, never edited, with the researcher's speaker names applied.
    pub ai_text: Option<String>,
    /// The user's corrected text, if any, with speaker names applied.
    pub corrected_text: Option<String>,
    pub reviewed: bool,
    pub model_version: Option<String>,
    pub cost_usd: Option<f64>,
    /// Timed lines of a transcription, relative to the start of the piece.
    pub segments: Vec<kalamu_core::Segment>,
    /// An analysis's answers from the model, never edited.
    pub ai_fields: Vec<FieldAnswer>,
    /// The researcher's corrected answers, if any.
    pub corrected_fields: Option<Vec<FieldAnswer>>,
    /// The model's speaker labels in this piece, when the set-up asked for them.
    pub speakers: Vec<String>,
    /// The names the researcher gave this set-up's speakers in this recording.
    pub speaker_names: BTreeMap<String, String>,
    /// For a correction or a translation, the text as it was before: the PDF page's text layer
    /// or the text file's page or passage, which the review compares with the correction and
    /// shows beside the translation.
    pub source_text: Option<String>,
    /// The model's answer stopped before its end: the end of the piece may be missing.
    pub truncated: bool,
    /// Why it stopped, when `truncated`: the output limit, a filter, or another reason.
    #[serde(default, skip_serializing_if = "CutReason::is_limit")]
    pub cut_reason: CutReason,
    /// For a page of a document of images, the original file it is, where it lies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub original_file: Option<String>,
}

/// What adding items to a list answer did: the set-ups' corrections as they stood, to put back
/// with [`Engine::save_corrections`], and what became of each item asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ListedAdded {
    pub before: Vec<Correction>,
    /// Items added to at least one set-up's answer, as written.
    pub added: Vec<String>,
    /// Items every set-up's answer already held: nothing to add.
    pub already: Vec<String>,
    /// Items a field of choices does not offer: not added.
    pub refused: Vec<String>,
}

impl Engine {
    /// Every piece of one source, for the review and comparison views.
    pub fn source_pieces(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<Vec<PieceView>, EngineError> {
        let job = self.store.job(job_id)?;
        let corrections = self.store.corrections(job_id)?;
        let speakers = self.store.speakers(job_id)?;
        // A correction's or a translation's source is read once, for the text each piece had
        // before.
        let source = job.sources.iter().find(|s| s.id == source_id);
        let opened = match source {
            Some(source) if job.task.rewrites_text() => {
                kalamu_media::OpenSource::open(source, self.config.ffmpeg.as_ref()).ok()
            }
            _ => None,
        };
        // A translation of a transcript's timed lines shows each passage's lines, with their
        // times and speakers, rather than the frozen text cut otherwise.
        let lines = source.and_then(|s| self.timed_lines(&job, s).ok().flatten());
        let mut out = Vec::new();
        for state in self.store.pieces(job_id)? {
            if state.source_id != source_id {
                continue;
            }
            let result = self.store.finished_result(&state)?;
            let correction = Correction::of(&corrections, &state);
            let diarized = job
                .setups
                .iter()
                .any(|s| s.id == state.setup_id && s.options.diarize);
            let speaker_names = speakers.of(state.setup_id, source_id).clone();
            let speakers = match &result {
                Some(r) if diarized => labels(&r.output.text, &r.output.segments),
                _ => Vec::new(),
            };
            let truncated = result.as_ref().is_some_and(|r| r.output.truncated);
            let cut_reason = result
                .as_ref()
                .map(|r| r.output.cut_reason)
                .unwrap_or_default();
            out.push(PieceView {
                setup_id: state.setup_id,
                source_id,
                index: state.index,
                piece: state.piece.clone(),
                status: state.status,
                error: state.error.clone(),
                ai_text: result
                    .as_ref()
                    .map(|r| rename_text(&r.output.text, &speaker_names)),
                corrected_text: correction
                    .and_then(|c| c.text.as_deref())
                    .map(|t| rename_text(t, &speaker_names)),
                reviewed: correction.is_some_and(|c| c.reviewed),
                model_version: result.as_ref().and_then(|r| r.output.model_version.clone()),
                cost_usd: result.as_ref().and_then(|r| r.cost_usd),
                ai_fields: result
                    .as_ref()
                    .map(|r| r.output.fields.clone())
                    .unwrap_or_default(),
                corrected_fields: correction.and_then(|c| c.fields.clone()),
                // Models that write `[hh:mm:ss]` into their text (Gemini, the interview
                // prompts) give their timed lines that way.
                segments: match result {
                    Some(r)
                        if r.output.segments.is_empty() && job.task == Task::AudioTranscription =>
                    {
                        timed_lines(&rename_text(&r.output.text, &speaker_names))
                    }
                    Some(r) => rename_segments(&r.output.segments, &speaker_names),
                    None => Vec::new(),
                },
                speakers,
                speaker_names,
                source_text: match &lines {
                    Some(lines) => lines.source_text(&state.piece),
                    None => opened.as_ref().and_then(|o| o.text_of(&state.piece).ok()),
                },
                truncated,
                cut_reason,
                original_file: match (&state.piece, source) {
                    (PieceRef::Page { number, .. }, Some(source)) => {
                        source.page_file(*number).map(|(page, _)| page.path.clone())
                    }
                    _ => None,
                },
            });
        }
        out.sort_by_key(|p| (p.index, p.setup_id));
        Ok(out)
    }

    /// Renames a name an analysis listed in field `key`, or takes it out when `to` is none or
    /// blank, in the answers of every set-up that found it in one piece, each kept reviewed or
    /// not as it was. All the set-ups change or none does. Gives back their corrections as they
    /// stood before (a set-up never corrected as an empty one), for
    /// [`Engine::save_corrections`] to put back; none when no set-up held the name.
    pub fn rename_listed(
        &self,
        job_id: JobId,
        source_id: SourceId,
        index: u32,
        key: &str,
        from: &str,
        to: Option<&str>,
    ) -> Result<Vec<Correction>, EngineError> {
        let job = self.store.job(job_id)?;
        let corrections = self.store.corrections(job_id)?;
        let now = Timestamp::now();
        let (mut before, mut after) = (Vec::new(), Vec::new());
        for setup in &job.setups {
            let pieces = self.store.source_pieces(job_id, setup.id, source_id)?;
            let Some(piece) = pieces.iter().find(|p| p.index == index) else {
                continue;
            };
            if piece.status == PieceStatus::Failed {
                continue;
            }
            let Some(answers) = self.store.answers(piece, &corrections)? else {
                continue;
            };
            if !holds_item(&answers, key, from) {
                continue;
            }
            let was = Correction::of(&corrections, piece)
                .cloned()
                .unwrap_or(Correction {
                    job_id,
                    setup_id: setup.id,
                    source_id,
                    index,
                    text: None,
                    fields: None,
                    reviewed: false,
                    updated_at: now,
                });
            after.push(Correction {
                fields: Some(rename_item(&answers, key, from, to)),
                updated_at: now,
                ..was.clone()
            });
            before.push(was);
        }
        if !after.is_empty() {
            self.store.put_corrections(&after)?;
            self.write_later(job_id);
        }
        Ok(before)
    }

    /// Adds the items the researcher typed (one per line or between semicolons) to list field
    /// `key` in one piece, in the corrected answers of every set-up that read it and asks for
    /// that field, each kept reviewed or not as it was: the document names them, whichever model
    /// missed them. An item a set-up's answer already holds is not added to it twice; a field of
    /// choices takes only the choices it offers, as the schema spells them. All the set-ups
    /// change or none does. Text, single-choice and entries fields take no items.
    pub fn add_listed(
        &self,
        job_id: JobId,
        source_id: SourceId,
        index: u32,
        key: &str,
        items: &[String],
    ) -> Result<ListedAdded, EngineError> {
        let wanted: Vec<String> = split_items(&items.join(
            "
",
        ));
        let job = self.store.job(job_id)?;
        let corrections = self.store.corrections(job_id)?;
        let now = Timestamp::now();
        let (mut before, mut after) = (Vec::new(), Vec::new());
        let (mut added, mut offered, mut refused) = (
            Vec::<String>::new(),
            Vec::<String>::new(),
            Vec::<String>::new(),
        );
        let note = |list: &mut Vec<String>, item: &str| {
            if !list.iter().any(|i| i.to_lowercase() == item.to_lowercase()) {
                list.push(item.to_string());
            }
        };
        for setup in &job.setups {
            let Some(field) = setup
                .prompt
                .schema
                .as_ref()
                .and_then(|s| s.fields.iter().find(|f| f.key == key))
            else {
                continue;
            };
            // A field of choices takes the choices it offers, spelled as offered.
            let accepted: Vec<String> = match &field.kind {
                FieldKind::List { .. } => wanted.clone(),
                FieldKind::Choices { values } => wanted
                    .iter()
                    .filter_map(|item| {
                        let choice = values
                            .iter()
                            .find(|v| v.trim().to_lowercase() == item.to_lowercase());
                        if choice.is_none() {
                            note(&mut refused, item);
                        }
                        choice.cloned()
                    })
                    .collect(),
                _ => continue,
            };
            let pieces = self.store.source_pieces(job_id, setup.id, source_id)?;
            let Some(piece) = pieces.iter().find(|p| p.index == index) else {
                continue;
            };
            if piece.status == PieceStatus::Failed {
                continue;
            }
            let Some(answers) = self.store.answers(piece, &corrections)? else {
                continue;
            };
            for item in &accepted {
                note(&mut offered, item);
            }
            let (next, here) = add_items(&answers, key, &accepted);
            if here.is_empty() {
                continue;
            }
            for item in &here {
                note(&mut added, item);
            }
            let was = Correction::of(&corrections, piece)
                .cloned()
                .unwrap_or(Correction {
                    job_id,
                    setup_id: setup.id,
                    source_id,
                    index,
                    text: None,
                    fields: None,
                    reviewed: false,
                    updated_at: now,
                });
            after.push(Correction {
                fields: Some(next),
                updated_at: now,
                ..was.clone()
            });
            before.push(was);
        }
        if !after.is_empty() {
            self.store.put_corrections(&after)?;
            self.write_later(job_id);
        }
        // Each said in the order asked for, whichever set-up took it first.
        let (added, already) = offered
            .into_iter()
            .partition(|o| added.iter().any(|a| a.to_lowercase() == o.to_lowercase()));
        Ok(ListedAdded {
            before,
            added,
            already,
            refused,
        })
    }

    /// Makes one change to a page's rows in a records analysis (a cell typed, a row added or
    /// removed), on the page's answers as the researcher left them: the model's stay as they
    /// are, the page's corrected answers hold the change, and the page stays reviewed or not as
    /// it was. A page not read has no rows to change.
    pub fn edit_rows(
        &self,
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
        index: u32,
        change: &RowEdit,
    ) -> Result<(), EngineError> {
        let job = self.store.job(job_id)?;
        let schema = job
            .setups
            .iter()
            .find(|s| s.id == setup_id)
            .and_then(|s| s.prompt.schema.as_ref())
            .ok_or(EngineError::Rows(kalamu_core::RowError::NotRecords))?;
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
        let fields = kalamu_core::records::edit(schema, &answers, change)?;
        let reviewed = Correction::of(&corrections, piece).is_some_and(|c| c.reviewed);
        self.store.put_correction(&Correction {
            job_id,
            setup_id,
            source_id,
            index,
            text: None,
            fields: Some(fields),
            reviewed,
            updated_at: Timestamp::now(),
        })?;
        self.write_later(job_id);
        Ok(())
    }

    /// Saves several corrections or review marks at once, all or none, as
    /// [`Engine::save_correction`] saves one: what [`Engine::rename_listed`] changed, put back.
    pub fn save_corrections(&self, corrections: Vec<Correction>) -> Result<(), EngineError> {
        let now = Timestamp::now();
        let corrections: Vec<Correction> = corrections
            .into_iter()
            .map(|c| Correction {
                updated_at: now,
                ..c
            })
            .collect();
        self.store.put_corrections(&corrections)?;
        let mut jobs: Vec<JobId> = corrections.iter().map(|c| c.job_id).collect();
        jobs.sort_unstable();
        jobs.dedup();
        for job in jobs {
            self.write_later(job);
        }
        Ok(())
    }
}
