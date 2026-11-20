//! Bringing together the answers of a text read in parts ([`super::windows`]): one call over
//! what every part found, which writes the answer for the whole text.
//!
//! The parts' quotations are already found in the text and kept as [`Evidence`]. They are sent
//! to the model numbered once (`Q1`, `Q2`…), the same quotation found by two parts that overlap
//! numbered once, and the model answers each entry with the numbers of its quotations, never
//! with words ([`schema`]). Kalamu then puts back each number's evidence as the part kept it
//! ([`resolve`]): the step can group, order and leave out quotations, never change or add one. A
//! number it does not know (words written in its place, a number past the last) is left out and
//! counted, as a quotation not found in the text is.

use serde_json::{Map, Value, json};

use super::{CitedEntry, CitedValue, Evidence, QUOTES};
use crate::{AnalysisSchema, Field, FieldAnswer, FieldKind, FieldValue, PieceRef};

/// The label of an entry's quotations in the step's answer.
const QUOTES_LABEL: &str = "Quotations";

/// The fields the step answers in: the set-up's own, each field of entries that cite the text
/// answered with its entries' fields and the numbers of their quotations in `quotes`.
pub fn schema(fields: &AnalysisSchema) -> AnalysisSchema {
    AnalysisSchema {
        fields: fields
            .fields
            .iter()
            .map(|field| match &field.kind {
                FieldKind::Cited { fields: own, max } => {
                    let mut own = own.clone();
                    own.push(Field {
                        key: QUOTES.into(),
                        label: QUOTES_LABEL.into(),
                        instructions: "The numbers of the quotations that show this entry, such \
                                       as \"Q12\", taken from the parts' quotations: only numbers, \
                                       never words."
                            .into(),
                        kind: FieldKind::List { max: None },
                        reason: false,
                    });
                    let most = max
                        .map(|m| format!(" At most {m} entries, the most important first."))
                        .unwrap_or_default();
                    Field {
                        kind: FieldKind::Entries { fields: own },
                        instructions: format!("{}{most}", field.instructions.trim()),
                        ..field.clone()
                    }
                }
                _ => field.clone(),
            })
            .collect(),
    }
}

/// What the parts found, as the step reads it, and each quotation by its number.
pub struct Findings {
    /// The parts' answers and the quotations, as JSON text.
    pub input: String,
    /// The quotations in the order numbered: `Q1` is the first.
    pub evidence: Vec<Evidence>,
    /// Per field of entries that cite the text, by key: how many quotations and entries the
    /// parts left out, carried into the whole answer's counts.
    pub carried: Vec<(String, u32, u32)>,
}

/// One part read: its piece (its turns) and its answers.
pub struct PartRead<'a> {
    pub piece: &'a PieceRef,
    pub answers: &'a [FieldAnswer],
}

/// The step's input: each part's turns and answers, its quotations by number, the turns no part
/// could read, and the quotations themselves with their turn, speaker and time.
pub fn findings(fields: &[Field], parts: &[PartRead<'_>], unread: &[(u32, u32)]) -> Findings {
    let mut evidence: Vec<Evidence> = Vec::new();
    let mut carried: Vec<(String, u32, u32)> = Vec::new();
    let mut read = Vec::new();
    for (n, part) in parts.iter().enumerate() {
        let mut answers = Map::new();
        for field in fields {
            let Some(answer) = part.answers.iter().find(|a| a.key == field.key) else {
                continue;
            };
            let value = match (&field.kind, answer.value.cited()) {
                (FieldKind::Cited { .. }, Some(cited)) => {
                    match carried.iter_mut().find(|(k, _, _)| k == &field.key) {
                        Some(c) => {
                            c.1 += cited.left_out;
                            c.2 += cited.dropped;
                        }
                        None => carried.push((field.key.clone(), cited.left_out, cited.dropped)),
                    }
                    Value::Array(
                        cited
                            .entries
                            .iter()
                            .map(|entry| entry_value(entry, &mut evidence))
                            .collect(),
                    )
                }
                _ => serde_json::to_value(&answer.value).unwrap_or(Value::Null),
            };
            answers.insert(field.key.clone(), value);
        }
        let turns = match part.piece {
            PieceRef::Turns { first, last, .. } => format!("{first}-{last}"),
            other => other.label(),
        };
        read.push(json!({ "part": n + 1, "turns": turns, "answers": answers }));
    }
    let quotations: Map<String, Value> = evidence
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut q = Map::new();
            q.insert("turn".into(), json!(e.turn));
            if let Some(speaker) = &e.speaker {
                q.insert("speaker".into(), json!(speaker));
            }
            if let Some(ms) = e.start_ms {
                q.insert("time".into(), json!(crate::result::clock(ms)));
            }
            q.insert("words".into(), json!(e.quote));
            (id(i), Value::Object(q))
        })
        .collect();
    let mut input = Map::new();
    input.insert("parts".into(), Value::Array(read));
    input.insert("quotations".into(), Value::Object(quotations));
    if !unread.is_empty() {
        input.insert(
            "turns_not_read".into(),
            json!(
                unread
                    .iter()
                    .map(|(f, l)| format!("{f}-{l}"))
                    .collect::<Vec<_>>()
            ),
        );
    }
    Findings {
        input: serde_json::to_string_pretty(&Value::Object(input)).unwrap_or_default(),
        evidence,
        carried,
    }
}

/// An entry as the step reads it: its own answers, and its quotations by number, each
/// quotation numbered the first time it is met.
fn entry_value(entry: &CitedEntry, evidence: &mut Vec<Evidence>) -> Value {
    let mut out = Map::new();
    for answer in &entry.answers {
        out.insert(
            answer.key.clone(),
            serde_json::to_value(&answer.value).unwrap_or(Value::Null),
        );
    }
    let ids: Vec<Value> = entry
        .evidence
        .iter()
        .map(|e| {
            let at = match evidence.iter().position(|known| known.anchor == e.anchor) {
                Some(at) => at,
                None => {
                    evidence.push(e.clone());
                    evidence.len() - 1
                }
            };
            json!(id(at))
        })
        .collect();
    out.insert(QUOTES.into(), Value::Array(ids));
    Value::Object(out)
}

fn id(index: usize) -> String {
    format!("Q{}", index + 1)
}

/// The place of a quotation's number in the list (`Q12`, `q12`, `12`, `[Q12]` → 11); `None`
/// for anything else, words included.
fn index_of(id: &str) -> Option<usize> {
    let id = id
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    let digits = id.strip_prefix(['Q', 'q']).unwrap_or(id);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse::<usize>().ok()?.checked_sub(1)
}

/// How many quotation numbers the step gave, and how many it gave that Kalamu knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Resolved {
    pub given: u32,
    pub found: u32,
}

/// The step's answer in the set-up's own fields: each entry's quotations put back from their
/// numbers as the parts kept them, a number not known left out and counted, a quotation lying
/// inside another of the same entry kept once (the longer), and an entry left with none left
/// out and counted. Every other field is kept as the step answered it.
pub fn resolve(fields: &[Field], answers: &mut [FieldAnswer], findings: &Findings) -> Resolved {
    let mut counted = Resolved::default();
    for field in fields {
        let FieldKind::Cited { max, .. } = &field.kind else {
            continue;
        };
        let Some(answer) = answers.iter_mut().find(|a| a.key == field.key) else {
            continue;
        };
        let (left_out, dropped) = findings
            .carried
            .iter()
            .find(|(k, _, _)| k == &field.key)
            .map_or((0, 0), |(_, l, d)| (*l, *d));
        let mut value = CitedValue {
            entries: Vec::new(),
            left_out,
            dropped,
        };
        let entries: Vec<Vec<FieldAnswer>> = answer.value.entries().to_vec();
        for mut own in entries {
            let ids: Vec<String> = match own.iter().position(|a| a.key == QUOTES) {
                Some(at) => match own.remove(at).value {
                    FieldValue::List(ids) => ids,
                    FieldValue::Text(id) => vec![id],
                    _ => Vec::new(),
                },
                None => Vec::new(),
            };
            let mut kept: Vec<Evidence> = Vec::new();
            for id in ids {
                counted.given += 1;
                let Some(evidence) = index_of(&id).and_then(|i| findings.evidence.get(i)) else {
                    value.left_out += 1;
                    continue;
                };
                counted.found += 1;
                let within = |outer: &Evidence, inner: &Evidence| {
                    outer.anchor.start <= inner.anchor.start && inner.anchor.end <= outer.anchor.end
                };
                if kept.iter().any(|k| within(k, evidence)) {
                    continue;
                }
                kept.retain(|k| !within(evidence, k));
                kept.push(evidence.clone());
            }
            if kept.is_empty() {
                value.dropped += 1;
                continue;
            }
            value.entries.push(CitedEntry {
                answers: own,
                evidence: kept,
                claims: Vec::new(),
            });
        }
        if let Some(max) = max {
            value.entries.truncate(*max as usize);
        }
        answer.value = FieldValue::Cited(value);
    }
    counted
}

#[cfg(test)]
mod tests;
