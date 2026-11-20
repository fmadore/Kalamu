//! A cited analysis of a text too long for its model to read at once, read in parts of whole
//! turns ([`kalamu_core::analysis::cited::windows`]) and brought together by a second step
//! ([`crate::contents::SecondStep`], as a table of contents combines its pages).
//!
//! How a set-up reads such a text is settled when the job is created, from the catalogue's limit
//! for its model (`Setup::windows`, with the step that brings the parts together in
//! `Setup::merge`); whether a text is read in parts is decided when it is read, so a transcript
//! that fits is read whole, exactly as before, under the same key. Each part is a piece
//! (`PieceRef::Turns`), sent with its turns numbered as in the whole text, and its quotations
//! are found in the whole text like any. The step after them receives what the parts found, each
//! quotation by its number, and Kalamu puts the quotations back from their numbers
//! ([`kalamu_core::analysis::cited::consolidate`]): it can neither change nor add one. A part
//! that failed is named in the step's input and in its record, and the step runs again once it
//! is read.

use kalamu_catalog::ModelInfo;
use kalamu_core::analysis::cited::consolidate::{self, Findings, PartRead};
use kalamu_core::analysis::cited::{TurnWindows, windows};
use kalamu_core::{
    AnalysisSchema, PieceRef, PieceState, PreparedInput, ProcessOutput, RequestSetting, Setup,
    SourceFile, StoredResult,
};

use crate::check::AnswerCheck;
use crate::cited::Quotable;
use crate::contents::SecondStep;
use crate::piece::Input;

/// How a model reads a text too long for it, from its catalogue limit; `None` for a model the
/// catalogue gives no limit, whose texts are always read whole.
pub(crate) fn windows_for(model: &ModelInfo) -> Option<TurnWindows> {
    let most = model.most_chars()?;
    Some(TurnWindows::new(u32::try_from(most).unwrap_or(u32::MAX)))
}

/// The piece a review's change to piece `index` reaches under one set-up: for a text read in
/// parts, what brought them together, which holds its answers whatever the index; else the
/// piece at `index`. A set-up of a comparison may read in parts a text another reads whole.
pub(crate) fn answered(pieces: &[PieceState], index: u32) -> Option<&PieceState> {
    if pieces.iter().any(|p| p.piece.is_window()) {
        pieces.iter().find(|p| p.piece == PieceRef::Whole)
    } else {
        pieces.iter().find(|p| p.index == index)
    }
}

/// Added to the request of one part, after the request every cited analysis has.
pub(crate) fn part_request(first: u32, last: u32) -> String {
    format!(
        " The text is long and is read in parts: this part holds turns {first} to {last}. \
         Answer from this part alone; the parts are brought together afterwards."
    )
}

impl Quotable {
    /// Each turn's length as the model is sent it.
    pub(crate) fn sizes(&self) -> Vec<u32> {
        windows::line_sizes(&self.text, &self.turns)
    }

    /// The parts a set-up reads this text in; `None` when it reads it whole: the set-up reads
    /// no text in parts (a job made before), the text fits, or a turn alone does not (left to
    /// the provider's own refusal, as the job was refused for it when created).
    pub(crate) fn parts(&self, setup: &Setup) -> Option<Vec<PieceRef>> {
        let plan = windows::plan(&self.sizes(), setup.windows.as_ref()?).ok()??;
        Some(
            plan.into_iter()
                .enumerate()
                .map(|(i, (first, last))| PieceRef::Turns {
                    number: i as u32 + 1,
                    first,
                    last,
                })
                .collect(),
        )
    }

    /// When a turn starts on the recording, if the text has times.
    fn time_of(&self, turn: u32) -> Option<u64> {
        self.turns.get(turn.checked_sub(1)? as usize)?.start_ms
    }

    /// A part as a sentence names it: its turns, and its times when the text has them.
    pub(crate) fn part_name(&self, piece: &PieceRef) -> String {
        let PieceRef::Turns {
            number,
            first,
            last,
        } = piece
        else {
            return piece.label();
        };
        let start = self.time_of(*first);
        // A part ends where the turn after it starts.
        let end = self
            .time_of(last + 1)
            .or_else(|| self.turns.get(*last as usize - 1).and_then(|t| t.end_ms));
        match (start, end) {
            (Some(s), Some(e)) => format!(
                "part {number} (turns {first}–{last}, {} to {})",
                kalamu_core::result::clock(s),
                kalamu_core::result::clock(e)
            ),
            (Some(s), None) => format!(
                "part {number} (turns {first}–{last}, from {})",
                kalamu_core::result::clock(s)
            ),
            _ => format!("part {number} (turns {first}–{last})"),
        }
    }
}

/// The parts of a long text, brought together.
pub(crate) struct BringTogether<'a> {
    pub(crate) source: &'a SourceFile,
    pub(crate) setup: &'a Setup,
}

impl SecondStep for BringTogether<'_> {
    fn kind(&self) -> &'static str {
        "cited_parts"
    }

    fn waits_for_failed(&self) -> Option<&'static str> {
        None
    }

    fn nothing_read(&self) -> &'static str {
        "No part of the text could be read. Retry them, and they will be brought together."
    }

    fn input(
        &self,
        read: &[(PieceState, StoredResult)],
        failed: &[PieceState],
    ) -> Result<Input, String> {
        let schema = self
            .setup
            .prompt
            .schema
            .clone()
            .ok_or_else(|| "The analysis has no fields.".to_string())?;
        let parts: Vec<PartRead<'_>> = read
            .iter()
            .map(|(state, result)| PartRead {
                piece: &state.piece,
                answers: &result.output.fields,
            })
            .collect();
        let turns = |p: &PieceState| match p.piece {
            PieceRef::Turns { first, last, .. } => Some((first, last)),
            _ => None,
        };
        let unread: Vec<(u32, u32)> = failed.iter().filter_map(turns).collect();
        let findings = consolidate::findings(&schema.fields, &parts, &unread);
        let named = |states: &mut dyn Iterator<Item = &PieceState>| {
            states
                .filter_map(turns)
                .map(|(f, l)| format!("{f}-{l}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut settings = vec![RequestSetting::new(
            "turns_read",
            named(&mut read.iter().map(|(s, _)| s)),
        )];
        if !failed.is_empty() {
            settings.push(RequestSetting::new(
                "turns_not_read",
                named(&mut failed.iter()),
            ));
        }
        let stem = crate::source_stem(&self.source.file_name, "document");
        let input = PreparedInput {
            piece: PieceRef::Whole,
            mime: "text/plain".into(),
            bytes: findings.input.clone().into_bytes(),
            file_name: format!("{stem}-parts.json"),
        };
        Ok(Input::Checked {
            input,
            check: Box::new(Together {
                schema,
                findings,
                settings,
            }),
        })
    }
}

/// The step's answer read back in the set-up's own fields, each quotation put back from its
/// number; refused, and so asked again like any broken answer, when it gave numbers and not one
/// of them is a quotation the parts kept.
struct Together {
    schema: AnalysisSchema,
    findings: Findings,
    settings: Vec<RequestSetting>,
}

impl AnswerCheck for Together {
    fn check(&self, mut output: ProcessOutput) -> Result<ProcessOutput, String> {
        let counted = consolidate::resolve(&self.schema.fields, &mut output.fields, &self.findings);
        if counted.given > 0 && counted.found == 0 {
            return Err(format!(
                "none of the {} quotation numbers the answer gave is one the parts found",
                counted.given
            ));
        }
        output.text = self.schema.render(&output.fields);
        Ok(output)
    }

    fn settings(&self) -> Vec<RequestSetting> {
        self.settings.clone()
    }
}
