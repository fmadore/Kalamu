//! How far the set-ups of an analysis agree over a whole job, field by field and pair by pair.
//!
//! Comparison mode shows each document's answers side by side; this counts them over every
//! document both set-ups of a pair answered. A choice from a list is measured by exact agreement
//! (the share of documents given the same choice) and by Cohen's kappa, which discounts the
//! agreement two set-ups would reach by chance given how often each picks each value. A list of
//! names or themes, or several choices from a list, is measured by its overlap: the items both
//! listed over the items either listed (Jaccard), averaged over the documents. Items and choices
//! are compared folded ([`crate::list_items::folded`]): capitals, accents and punctuation aside.
//!
//! Free text is left out: two models rarely word a summary alike, and the review's side-by-side
//! view says more about it than a share of identical answers. Repeating groups are left out too:
//! their entries are rows, not one answer per document.
//!
//! The counts read the models' own answers, never the researcher's corrections: a correction
//! made to one set-up's answer is the researcher's reading, not that model's, and would make the
//! two look closer or further apart than they were. Nothing here reads the store; the engine
//! feeds [`AgreementTally`] one document at a time.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::list_items::folded;
use crate::{FieldAnswer, FieldKind, FieldValue, Setup, SetupId};

/// How a field's answers are compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AgreementMeasure {
    /// One value from a fixed set: exact agreement and Cohen's kappa.
    Choice,
    /// Names, terms or several choices: the overlap of the items, averaged over documents.
    Overlap,
}

/// One pair of set-ups on one field, over the pieces both answered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PairAgreement {
    /// The earlier of the two in the job's order of set-ups.
    pub first: SetupId,
    pub second: SetupId,
    /// The pieces (documents, or pages for an analysis read page by page) both set-ups
    /// answered with this field. A piece one of them failed or has not reached is not counted.
    pub pieces: u32,
    /// For a choice, the share of pieces given the same value; for a list, the mean overlap.
    /// Between 0 and 1; `None` when no piece was counted.
    pub agreement: Option<f64>,
    /// For a choice, Cohen's kappa: 1 when the two always agree, 0 when they agree no more than
    /// chance would, below 0 when less. `None` for a list, when no piece was counted, and when
    /// it cannot be computed: both set-ups gave every piece one and the same value, so chance
    /// alone would agree every time.
    pub kappa: Option<f64>,
}

/// One field of the analysis, compared between every pair of set-ups that both ask it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldAgreement {
    pub key: String,
    /// The field's name as the first set-up that asks it calls it.
    pub label: String,
    pub measure: AgreementMeasure,
    /// In the job's order of pairs: the first set-up with each later one, then the second with
    /// each after it, and so on.
    pub pairs: Vec<PairAgreement>,
}

/// A job's agreement between set-ups, one row per field compared. Empty for a job with one
/// set-up, a task other than an analysis, or fields none of which can be compared.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobAgreement {
    pub fields: Vec<FieldAgreement>,
}

/// The overlap of two lists: the items both hold over the items either holds (Jaccard),
/// compared folded and each counted once. Two empty lists agree, at 1: both found nothing.
pub fn overlap(first: &[String], second: &[String]) -> f64 {
    let items = |list: &[String]| -> BTreeSet<String> {
        list.iter()
            .map(|item| folded(item))
            .filter(|item| !item.is_empty())
            .collect()
    };
    let (a, b) = (items(first), items(second));
    let either = a.union(&b).count();
    if either == 0 {
        return 1.0;
    }
    a.intersection(&b).count() as f64 / either as f64
}

/// Exact agreement and Cohen's kappa over pairs of choices, one pair per piece, compared
/// folded: `(agreement, kappa)`, each as [`PairAgreement`] describes it.
pub fn choice_agreement<'a>(
    choices: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> (Option<f64>, Option<f64>) {
    let mut counts = ChoiceCounts::default();
    for (a, b) in choices {
        counts.add(a, b);
    }
    (counts.agreement(), counts.kappa())
}

/// What a choice's agreement needs of the pieces counted: how many, how many agreed, and how
/// often each set-up gave each value (the margins chance agreement is computed from).
#[derive(Debug, Default)]
struct ChoiceCounts {
    pieces: u64,
    same: u64,
    first: HashMap<String, u64>,
    second: HashMap<String, u64>,
}

impl ChoiceCounts {
    fn add(&mut self, a: &str, b: &str) {
        let (a, b) = (folded(a), folded(b));
        self.pieces += 1;
        if a == b {
            self.same += 1;
        }
        *self.first.entry(a).or_default() += 1;
        *self.second.entry(b).or_default() += 1;
    }

    fn agreement(&self) -> Option<f64> {
        (self.pieces > 0).then(|| self.same as f64 / self.pieces as f64)
    }

    /// κ = (pₒ − pₑ) / (1 − pₑ), with pₒ = same / n and pₑ = Σ firstₖ · secondₖ / n². Multiplied
    /// through by n², whole numbers throughout: (same · n − Σ) / (n² − Σ), and undefined exactly
    /// when Σ = n², every piece given one value by both.
    fn kappa(&self) -> Option<f64> {
        let n = u128::from(self.pieces);
        if n == 0 {
            return None;
        }
        let chance: u128 = self
            .first
            .iter()
            .map(|(value, a)| {
                u128::from(*a) * u128::from(self.second.get(value).copied().unwrap_or(0))
            })
            .sum();
        let all = n * n;
        if chance >= all {
            return None;
        }
        let observed = u128::from(self.same) * n;
        Some((observed as f64 - chance as f64) / (all - chance) as f64)
    }
}

#[derive(Debug)]
enum Counts {
    Choice(ChoiceCounts),
    Overlap { pieces: u64, sum: f64 },
}

#[derive(Debug)]
struct PairTally {
    /// The two set-ups' positions in the job, and their ids.
    first: usize,
    second: usize,
    ids: (SetupId, SetupId),
    counts: Counts,
}

#[derive(Debug)]
struct FieldTally {
    key: String,
    label: String,
    measure: AgreementMeasure,
    pairs: Vec<PairTally>,
}

/// A field's measure, or none for one that is not compared (free text, repeating groups).
fn measure(kind: &FieldKind) -> Option<AgreementMeasure> {
    match kind {
        FieldKind::Choice { .. } => Some(AgreementMeasure::Choice),
        FieldKind::List { .. } | FieldKind::Choices { .. } => Some(AgreementMeasure::Overlap),
        FieldKind::Text | FieldKind::Entries { .. } | FieldKind::Cited { .. } => None,
    }
}

/// A set-up's measure for field `key`, when its analysis asks it and it can be compared.
fn measure_of(setup: &Setup, key: &str) -> Option<AgreementMeasure> {
    setup
        .prompt
        .schema
        .as_ref()?
        .fields
        .iter()
        .find(|f| f.key == key)
        .and_then(|f| measure(&f.kind))
}

/// One answer as a choice: its text; none for a missing or blank answer.
fn choice_of<'a>(answers: &'a [FieldAnswer], key: &str) -> Option<&'a str> {
    match &answers.iter().find(|a| a.key == key)?.value {
        FieldValue::Text(text) if !text.trim().is_empty() => Some(text),
        _ => None,
    }
}

/// One answer as a list of items; none for a missing answer or one that is not a list.
fn list_of<'a>(answers: &'a [FieldAnswer], key: &str) -> Option<&'a [String]> {
    match &answers.iter().find(|a| a.key == key)?.value {
        FieldValue::List(items) => Some(items),
        _ => None,
    }
}

/// The agreement of a job's set-ups, counted one piece at a time so that a job of thousands of
/// documents never holds all their answers at once.
#[derive(Debug)]
pub struct AgreementTally {
    fields: Vec<FieldTally>,
}

impl AgreementTally {
    /// The fields to compare: every choice and list field of the set-ups' analyses, in the order
    /// the first set-up asking each lists them, each with the pairs of set-ups that both ask it
    /// with the same measure.
    pub fn new(setups: &[Setup]) -> Self {
        let mut fields: Vec<FieldTally> = Vec::new();
        let asked = setups
            .iter()
            .filter_map(|s| s.prompt.schema.as_ref())
            .flat_map(|schema| &schema.fields);
        for field in asked {
            let Some(kind) = measure(&field.kind) else {
                continue;
            };
            if fields.iter().any(|f| f.key == field.key) {
                continue;
            }
            let mut pairs = Vec::new();
            for (i, a) in setups.iter().enumerate() {
                for (j, b) in setups.iter().enumerate().skip(i + 1) {
                    if measure_of(a, &field.key) != Some(kind)
                        || measure_of(b, &field.key) != Some(kind)
                    {
                        continue;
                    }
                    let counts = match kind {
                        AgreementMeasure::Choice => Counts::Choice(ChoiceCounts::default()),
                        AgreementMeasure::Overlap => Counts::Overlap {
                            pieces: 0,
                            sum: 0.0,
                        },
                    };
                    pairs.push(PairTally {
                        first: i,
                        second: j,
                        ids: (a.id, b.id),
                        counts,
                    });
                }
            }
            if !pairs.is_empty() {
                fields.push(FieldTally {
                    key: field.key.clone(),
                    label: field.label.clone(),
                    measure: kind,
                    pairs,
                });
            }
        }
        Self { fields }
    }

    /// Whether no field can be compared, so there is nothing to read.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Counts one piece: each set-up's answers to it, in the job's order of set-ups, `None` for
    /// a set-up with no answer (failed, not yet run). A pair is counted on a field only when both
    /// answered it.
    pub fn add(&mut self, answers: &[Option<&[FieldAnswer]>]) {
        let of = |i: usize| answers.get(i).copied().flatten();
        for field in &mut self.fields {
            for pair in &mut field.pairs {
                let (Some(a), Some(b)) = (of(pair.first), of(pair.second)) else {
                    continue;
                };
                match &mut pair.counts {
                    Counts::Choice(counts) => {
                        if let (Some(a), Some(b)) =
                            (choice_of(a, &field.key), choice_of(b, &field.key))
                        {
                            counts.add(a, b);
                        }
                    }
                    Counts::Overlap { pieces, sum } => {
                        if let (Some(a), Some(b)) = (list_of(a, &field.key), list_of(b, &field.key))
                        {
                            *pieces += 1;
                            *sum += overlap(a, b);
                        }
                    }
                }
            }
        }
    }

    /// The agreement over every piece counted.
    pub fn finish(&self) -> JobAgreement {
        JobAgreement {
            fields: self
                .fields
                .iter()
                .map(|field| FieldAgreement {
                    key: field.key.clone(),
                    label: field.label.clone(),
                    measure: field.measure,
                    pairs: field
                        .pairs
                        .iter()
                        .map(|pair| {
                            let (pieces, agreement, kappa) = match &pair.counts {
                                Counts::Choice(c) => (c.pieces, c.agreement(), c.kappa()),
                                Counts::Overlap { pieces, sum } => {
                                    (*pieces, (*pieces > 0).then(|| sum / *pieces as f64), None)
                                }
                            };
                            PairAgreement {
                                first: pair.ids.0,
                                second: pair.ids.1,
                                pieces: u32::try_from(pieces).unwrap_or(u32::MAX),
                                agreement,
                                kappa,
                            }
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests;
