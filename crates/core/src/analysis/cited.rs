//! Entries that cite the document (`FieldKind::Cited`): a theme with its description, a
//! passage worth quoting with why it matters, each with one or more quotations of the text.
//!
//! The model is sent the text with its turns numbered ([`numbered`]) and answers, for each
//! quotation, the number of the turn and the words ([`read`] keeps them as [`Claim`]s, never
//! stored). [`cite`] then looks for the words in that turn, then in the turns nearest it, then in
//! any turn, forgiving only what a model changes when it copies ([`crate::snapshot::forgiving`]),
//! and keeps each one found as [`Evidence`]: where it stands in the text (an [`Anchor`]), the
//! text's own characters there, and the turn's speaker and times, copied from the turn and never
//! from the model. A quotation found nowhere is left out and counted, and an entry left with no
//! quotation is left out too: what is shown as someone's words is always theirs.
//!
//! The researcher's corrections are a layer over the model's answer, as for any field: an entry
//! edited or removed, a quotation removed ([`edit`]), never one added or changed
//! ([`keeps_evidence`]).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::{Field, FieldAnswer, FieldKind, FieldValue, SchemaError, read_fields, render_fields};
use crate::snapshot::{Anchor, SnapshotPart, Turn, forgiving, slice, turn_text};

mod edit;
pub use edit::{CitedEdit, CitedError, edit, keeps_evidence};

/// The property that holds an entry's quotations in the model's answer.
pub const QUOTES: &str = "quotes";
/// A quotation's turn number, as sent.
pub const TURN: &str = "turn";
/// A quotation's words.
pub const WORDS: &str = "words";

/// Added to the request of an analysis with a field that cites the document, after the
/// prompt's own.
pub const REQUEST: &str = "The text is sent one numbered turn at a time: the turn's number in \
brackets, such as [12], then its time and its speaker when they are known, then what was said. \
For every quotation, give the number of the turn it comes from and the words exactly as they \
stand in that turn, copied character for character: never correct, translate, shorten or join \
words from two turns, and never quote words that are not in the text. Give only the words said, \
without the number, the time or the speaker.";

/// A field's entries with their quotations, as kept.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CitedValue {
    pub entries: Vec<CitedEntry>,
    /// The model's quotations that could not be found in the text, left out.
    #[serde(default)]
    pub left_out: u32,
    /// The model's entries left out because none of their quotations could be found (their
    /// quotations are counted in `left_out`).
    #[serde(default)]
    pub dropped: u32,
}

/// One entry: its own fields' answers and its quotations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CitedEntry {
    pub answers: Vec<FieldAnswer>,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    /// The model's quotations before they are looked for in the text ([`cite`]). Never stored:
    /// an answer kept without being checked keeps no quotation at all.
    #[serde(skip)]
    #[ts(skip)]
    pub claims: Vec<Claim>,
}

/// A quotation as the model gave it: the turn it named and its words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// The turn's number as sent, from 1; `None` when the model gave none that reads as one.
    pub turn: Option<u32>,
    pub words: String,
}

/// A quotation found in the text. `CitedEvidence` in the interface, where the dictionary's
/// `Evidence` (the results that listed a form) has the name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CitedEvidence")]
pub struct Evidence {
    /// Where it stands in the text read (the snapshot's, or the text file's), in characters.
    pub anchor: Anchor,
    /// The text's own characters between the anchor's ends.
    pub quote: String,
    /// The turn it lies in, numbered from 1 as the text was sent: the snapshot's
    /// `turns[turn - 1]`.
    pub turn: u32,
    /// The turn's speaker, as the researcher named them when the text was frozen.
    pub speaker: Option<String>,
    /// When the turn starts and ends on the recording's clock, when the text has times.
    #[ts(type = "number | null")]
    pub start_ms: Option<u64>,
    #[ts(type = "number | null")]
    pub end_ms: Option<u64>,
}

/// Checks a cited field's own fields: text, lists and choices, none named as the quotations
/// are. A cited field may have none, its entries then quotations alone.
pub(super) fn validate(fields: &[Field]) -> Result<(), SchemaError> {
    if fields.is_empty() {
        return Ok(());
    }
    if let Some(reserved) = fields.iter().find(|f| f.key == QUOTES) {
        return Err(SchemaError::ReservedName(reserved.label.trim().to_string()));
    }
    super::validate_fields(fields, true)
}

/// Reads the model's entries: each entry's fields as any fields are read, and its quotations
/// as claims, to be looked for in the text. Entries past the maximum are left out.
pub(super) fn read(
    raw: &Value,
    field: &Field,
    fields: &[Field],
    max: Option<u32>,
) -> Result<CitedValue, String> {
    let entries = raw
        .as_array()
        .ok_or_else(|| format!("\"{}\" is not a list", field.label))?;
    let mut out = Vec::new();
    for entry in entries {
        let answers = if fields.is_empty() {
            Vec::new()
        } else {
            read_fields(fields, entry)?
        };
        let quotes = entry
            .get(QUOTES)
            .and_then(Value::as_array)
            .ok_or_else(|| format!("an entry of \"{}\" has no quotations", field.label))?;
        let claims = quotes
            .iter()
            .filter_map(|q| {
                let words = q.get(WORDS).and_then(Value::as_str)?.trim().to_string();
                let turn = match q.get(TURN) {
                    Some(Value::Number(n)) => n.as_u64().and_then(|n| u32::try_from(n).ok()),
                    Some(Value::String(s)) => s
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .trim()
                        .parse()
                        .ok(),
                    _ => None,
                };
                (!words.is_empty()).then_some(Claim { turn, words })
            })
            .collect();
        out.push(CitedEntry {
            answers,
            evidence: Vec::new(),
            claims,
        });
    }
    if let Some(max) = max {
        out.truncate(max as usize);
    }
    Ok(CitedValue {
        entries: out,
        left_out: 0,
        dropped: 0,
    })
}

/// The text read, with its parts and turns, in which quotations are looked for.
#[derive(Clone, Copy)]
pub struct CitedText<'a> {
    pub text: &'a str,
    pub parts: &'a [SnapshotPart],
    pub turns: &'a [Turn],
}

/// How many quotations the model gave, and how many were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cited {
    pub given: u32,
    pub found: u32,
}

/// Looks for every claimed quotation of the cited fields in the text, keeps those found as
/// evidence and counts the others as left out; an entry left with no quotation is left out
/// too. The claims are used up.
pub fn cite(fields: &[Field], answers: &mut [FieldAnswer], doc: CitedText<'_>) -> Cited {
    let mut counted = Cited::default();
    for field in fields {
        if !matches!(field.kind, FieldKind::Cited { .. }) {
            continue;
        }
        let Some(FieldValue::Cited(value)) = answers
            .iter_mut()
            .find(|a| a.key == field.key)
            .map(|a| &mut a.value)
        else {
            continue;
        };
        let mut kept = Vec::new();
        for mut entry in std::mem::take(&mut value.entries) {
            for claim in std::mem::take(&mut entry.claims) {
                counted.given += 1;
                match place(doc, &claim) {
                    Some(evidence) => {
                        counted.found += 1;
                        if !entry.evidence.contains(&evidence) {
                            entry.evidence.push(evidence);
                        }
                    }
                    None => value.left_out += 1,
                }
            }
            if entry.evidence.is_empty() {
                value.dropped += 1;
            } else {
                kept.push(entry);
            }
        }
        value.entries = kept;
    }
    counted
}

/// Where one claimed quotation stands: in the turn named, else in the turn nearest it that
/// holds it (the earlier of two as near), else in the first that does when no turn was named.
/// Always inside one turn: words running from one turn into the next are found in neither.
pub fn place(doc: CitedText<'_>, claim: &Claim) -> Option<Evidence> {
    let named = claim
        .turn
        .and_then(|n| n.checked_sub(1))
        .map(|n| n as usize)
        .filter(|&n| n < doc.turns.len());
    let mut order: Vec<usize> = (0..doc.turns.len()).collect();
    if let Some(n) = named {
        order.sort_by_key(|&i| (i.abs_diff(n), i));
    }
    for i in order {
        let turn = &doc.turns[i];
        let Some(words) = turn_text(doc.text, turn) else {
            continue;
        };
        let Some(&(from, to)) = forgiving::find(words, &claim.words).first() else {
            continue;
        };
        let (start, end) = (turn.start + from, turn.start + to);
        let part = doc
            .parts
            .iter()
            .position(|p| p.start <= start && end <= p.end)?;
        let quote = slice(doc.text, start, end)?.to_string();
        return Some(Evidence {
            anchor: Anchor {
                part: part as u32,
                start,
                end,
            },
            quote,
            turn: i as u32 + 1,
            speaker: turn.speaker.clone(),
            start_ms: turn.start_ms,
            end_ms: turn.end_ms,
        });
    }
    None
}

/// The text as the model reads it: each turn on one line after its number in brackets
/// (counted from 1, as a translation's lines are), with its time and its speaker when known:
/// `[3] [00:04:12] Awa: what she said`.
pub fn numbered(text: &str, turns: &[Turn]) -> String {
    turns
        .iter()
        .enumerate()
        .filter_map(|(i, turn)| {
            // On one line, its own spaces (no-break ones included) kept within each line.
            let words = turn_text(text, turn)?
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            let mut line = format!("[{}] ", i + 1);
            if let Some(ms) = turn.start_ms {
                line.push_str(&format!("[{}] ", crate::result::clock(ms)));
            }
            if let Some(speaker) = &turn.speaker {
                line.push_str(&format!("{speaker}: "));
            }
            line.push_str(&words);
            Some(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether any of these fields cites the document.
pub fn has_cited(fields: &[Field]) -> bool {
    fields
        .iter()
        .any(|f| matches!(f.kind, FieldKind::Cited { .. }))
}

/// Who said a quotation and when, as the files show it: `Awa, 00:04:12`; empty when neither is
/// known.
pub fn said_by(evidence: &Evidence) -> String {
    let mut out = evidence.speaker.clone().unwrap_or_default();
    if let Some(ms) = evidence.start_ms {
        if !out.is_empty() {
            out.push_str(", ");
        }
        out.push_str(&crate::result::clock(ms));
    }
    out
}

/// What was left out of a field's answer, as the files say it; `None` when nothing was.
pub fn left_out_note(value: &CitedValue) -> Option<String> {
    let mut notes = Vec::new();
    match value.left_out {
        0 => {}
        1 => notes.push("1 quotation could not be found in the text and was left out.".into()),
        n => notes.push(format!(
            "{n} quotations could not be found in the text and were left out."
        )),
    }
    match value.dropped {
        0 => {}
        1 => notes.push("1 entry was left out because none of its quotations was found.".into()),
        n => notes.push(format!(
            "{n} entries were left out because none of their quotations was found."
        )),
    }
    (!notes.is_empty()).then(|| notes.join(" "))
}

/// A cited field as plain text: its label, then each entry's fields and its quotations, one per
/// line, and what was left out.
pub(super) fn render(field: &Field, fields: &[Field], value: &FieldValue) -> String {
    let empty = CitedValue::default();
    let value = value.cited().unwrap_or(&empty);
    let mut blocks = vec![field.label.clone()];
    for entry in &value.entries {
        let mut lines = Vec::new();
        let own = render_fields(fields, &entry.answers);
        if !own.is_empty() {
            lines.push(own);
        }
        for evidence in &entry.evidence {
            let by = said_by(evidence);
            lines.push(if by.is_empty() {
                format!("\"{}\"", evidence.quote)
            } else {
                format!("\"{}\" ({by})", evidence.quote)
            });
        }
        blocks.push(lines.join("\n"));
    }
    if let Some(note) = left_out_note(value) {
        blocks.push(note);
    }
    blocks.join("\n\n")
}

/// Per cited field of a model's answer: its key, how many quotations the model gave (checked
/// against the text), how many were left out, and how many entries.
pub fn counts(fields: &[Field], answers: &[FieldAnswer]) -> Vec<(String, u32, u32, u32)> {
    fields
        .iter()
        .filter(|f| matches!(f.kind, FieldKind::Cited { .. }))
        .filter_map(|f| {
            let value = answers.iter().find(|a| a.key == f.key)?.value.cited()?;
            let kept: u32 = value.entries.iter().map(|e| e.evidence.len() as u32).sum();
            Some((
                f.key.clone(),
                kept + value.left_out,
                value.left_out,
                value.dropped,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests;
