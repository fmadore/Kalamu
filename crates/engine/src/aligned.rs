//! A translation of a transcript frozen with its timed lines (#167): planned in passages of
//! whole lines, each sent as numbered units and read back against the lines it sent
//! ([`kalamu_core::aligned`]), so every translated line keeps its line's time and speaker.
//!
//! What decides it is the source: a text frozen from a transcript whose every part had its
//! times (`DerivedFrom::timed`), its lines kept with the snapshot. Any other text, and every
//! translation made before lines were kept, is read in passages of text as before, with the
//! same cache keys.

use std::ops::Range;
use std::sync::Arc;

use kalamu_core::{
    AnalysisSchema, FieldAnswer, FieldValue, Job, PieceRef, PreparedInput, ProcessOutput, Segment,
    SourceFile, Task, aligned,
};

use crate::Engine;
use crate::check::AnswerCheck;

/// A transcript's timed lines, cut into the passages a translation sends.
pub(crate) struct Lines {
    lines: Vec<Segment>,
    passages: Vec<Range<usize>>,
    /// What the pieces' cache keys carry of the lines.
    fix: String,
}

impl Lines {
    fn new(lines: Vec<Segment>) -> Self {
        Self {
            passages: aligned::passages(&lines),
            fix: format!("timed_lines:{}", aligned::fingerprint(&lines)),
            lines,
        }
    }

    /// The pieces: one passage of whole lines each, numbered from 1, the first `sample` only
    /// when the researcher tries a sample.
    pub(crate) fn pieces(&self, sample: Option<u32>) -> Vec<PieceRef> {
        let count = sample.map_or(self.passages.len(), |n| self.passages.len().min(n as usize));
        (1..=count as u32)
            .map(|number| PieceRef::Passage { number })
            .collect()
    }

    /// Added to every piece's cache key, so that an answer read from other lines (or from the
    /// same text read as plain passages) is never taken for this one.
    pub(crate) fn fix(&self) -> &str {
        &self.fix
    }

    fn range(&self, piece: &PieceRef) -> Option<Range<usize>> {
        let PieceRef::Passage { number } = piece else {
            return None;
        };
        self.passages
            .get((*number as usize).checked_sub(1)?)
            .cloned()
    }

    /// A passage ready to send: its lines numbered, with neither times nor speakers.
    pub(crate) fn passage(self: &Arc<Self>, piece: &PieceRef, stem: &str) -> Option<Passage> {
        let range = self.range(piece)?;
        let input = PreparedInput {
            piece: piece.clone(),
            mime: "text/plain".into(),
            bytes: aligned::units(&self.lines, range.clone()).into_bytes(),
            file_name: format!("{stem}-{}.txt", piece.descriptor().replace(':', "-")),
        };
        Some(Passage {
            lines: self.clone(),
            range,
            input,
        })
    }

    /// A passage's lines as the review shows them beside the translation, with their times
    /// and speakers.
    pub(crate) fn source_text(&self, piece: &PieceRef) -> Option<String> {
        Some(aligned::source_text(&self.lines, self.range(piece)?))
    }
}

/// One passage of lines on its way to the model.
pub(crate) struct Passage {
    lines: Arc<Lines>,
    range: Range<usize>,
    pub(crate) input: PreparedInput,
}

/// A passage of lines answers in numbered lines, whatever the set-up's one field, asked to give
/// back every line by its number.
///
/// The answer is read against the lines sent: its text the translated lines, one paragraph
/// each, and its segments those lines with the times and speakers of the lines they translate.
/// An answer that does not give back every line once is refused as a broken answer, which the
/// engine asks again for and bills; a passage that never comes back whole fails with the
/// reason, and its subtitles are not written.
impl AnswerCheck for Passage {
    fn schema(&self) -> Option<AnalysisSchema> {
        Some(aligned::schema())
    }

    fn request(&self) -> Option<&str> {
        Some(aligned::REQUEST)
    }

    fn check(&self, mut output: ProcessOutput) -> Result<ProcessOutput, String> {
        match aligned::read(&output.fields, &self.lines.lines, self.range.clone()) {
            Ok(lines) => {
                let text = aligned::text(&lines);
                // The one field a translation's text is stored from.
                output.fields = vec![FieldAnswer {
                    key: kalamu_core::correction::TEXT.into(),
                    value: FieldValue::Text(text.clone()),
                    reason: None,
                }];
                output.text = text;
                output.segments = lines;
                Ok(output)
            }
            Err(reason) => Err(format!(
                "the translation did not give back its lines: {reason}"
            )),
        }
    }
}

impl Engine {
    /// The timed lines a translation reads from this source, when it is a transcript frozen
    /// with them; none for any other job or text. A source frozen with lines that can no longer
    /// be read is refused with the reason rather than translated without them.
    pub(crate) fn timed_lines(
        &self,
        job: &Job,
        source: &SourceFile,
    ) -> Result<Option<Arc<Lines>>, String> {
        let Some(from) = source.derived_from.as_ref().filter(|d| d.timed) else {
            return Ok(None);
        };
        if job.task != Task::Translation {
            return Ok(None);
        }
        match self.store.snapshot(&from.snapshot_id) {
            Ok(snapshot) if !snapshot.lines.is_empty() => {
                Ok(Some(Arc::new(Lines::new(snapshot.lines))))
            }
            _ => Err(
                "The timed lines of the transcript this text was taken from cannot be \
                      read. Send the transcript on again with Use this text in a new job."
                    .into(),
            ),
        }
    }
}
