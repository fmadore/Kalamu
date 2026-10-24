//! Comparison mode: how far set-ups agree with each other, and with a reference where the user
//! corrected a page. Error rates follow zmo-ai-pipelines' `zmo_evaluate.py`: edit distance over
//! characters (CER) and over words (WER), divided by the reference length.

use scriptorium_core::{JobId, SetupId, SourceId};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use ts_rs::TS;

use crate::{Engine, EngineError, PieceView};

/// A stretch of text in the side-by-side view, marked as shared or different.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DiffKind {
    Same,
    /// Only in the left text.
    Removed,
    /// Only in the right text.
    Added,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DiffSpan {
    pub kind: DiffKind,
    pub text: String,
}

/// Word-level differences, so names, numbers and dates that disagree stand out.
pub fn diff(left: &str, right: &str) -> Vec<DiffSpan> {
    let diff = TextDiff::from_words(left, right);
    let mut spans: Vec<DiffSpan> = Vec::new();
    for change in diff.iter_all_changes() {
        let kind = match change.tag() {
            ChangeTag::Equal => DiffKind::Same,
            ChangeTag::Delete => DiffKind::Removed,
            ChangeTag::Insert => DiffKind::Added,
        };
        match spans.last_mut() {
            Some(last) if last.kind == kind => last.text.push_str(change.value()),
            _ => spans.push(DiffSpan {
                kind,
                text: change.value().to_string(),
            }),
        }
    }
    spans
}

/// Edit distance between `hypothesis` and `reference`, divided by the reference length.
pub fn error_rate<T: PartialEq>(hypothesis: &[T], reference: &[T]) -> f64 {
    if reference.is_empty() {
        return if hypothesis.is_empty() { 0.0 } else { 1.0 };
    }
    let mut prev: Vec<usize> = (0..=hypothesis.len()).collect();
    let mut cur = vec![0; hypothesis.len() + 1];
    for (i, r) in reference.iter().enumerate() {
        cur[0] = i + 1;
        for (j, h) in hypothesis.iter().enumerate() {
            let cost = usize::from(r != h);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[hypothesis.len()] as f64 / reference.len() as f64
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cer(hypothesis: &str, reference: &str) -> f64 {
    let h: Vec<char> = normalize(hypothesis).chars().collect();
    let r: Vec<char> = normalize(reference).chars().collect();
    error_rate(&h, &r)
}

fn wer(hypothesis: &str, reference: &str) -> f64 {
    let h: Vec<&str> = hypothesis.split_whitespace().collect();
    let r: Vec<&str> = reference.split_whitespace().collect();
    error_rate(&h, &r)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupScore {
    pub setup_id: SetupId,
    pub label: String,
    /// Against the user's corrected pages; `None` when no page has a reference.
    pub cer: Option<f64>,
    pub wer: Option<f64>,
    /// Pages with a reference that were scored.
    pub scored_pages: u32,
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PageComparison {
    pub index: u32,
    /// Character error rate between each pair of set-ups, in set-up order: `(a, b, rate)`.
    pub agreement: Vec<(SetupId, SetupId, f64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComparisonReport {
    pub scores: Vec<SetupScore>,
    pub pages: Vec<PageComparison>,
}

impl Engine {
    /// Scores every set-up of a job on one source.
    ///
    /// A page counts as a reference when the user corrected it in any set-up; the corrected text
    /// is the reference for all set-ups on that page.
    pub fn compare(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<ComparisonReport, EngineError> {
        let job = self.store.job(job_id)?;
        let pieces = self.source_pieces(job_id, source_id)?;
        let indices: std::collections::BTreeSet<u32> = pieces.iter().map(|p| p.index).collect();
        let text_of = |setup: SetupId, index: u32| -> Option<&PieceView> {
            pieces
                .iter()
                .find(|p| p.setup_id == setup && p.index == index)
        };

        let mut scores = Vec::new();
        for setup in &job.setups {
            let (mut cer_sum, mut wer_sum, mut n) = (0.0, 0.0, 0u32);
            let mut cost: Option<f64> = None;
            for &index in &indices {
                let Some(view) = text_of(setup.id, index) else {
                    continue;
                };
                if let Some(c) = view.cost_usd {
                    cost = Some(cost.unwrap_or(0.0) + c);
                }
                let reference = pieces
                    .iter()
                    .find(|p| p.index == index && p.corrected_text.is_some())
                    .and_then(|p| p.corrected_text.clone());
                if let (Some(reference), Some(ai)) = (reference, &view.ai_text) {
                    cer_sum += cer(ai, &reference);
                    wer_sum += wer(ai, &reference);
                    n += 1;
                }
            }
            scores.push(SetupScore {
                setup_id: setup.id,
                label: setup.label.clone(),
                cer: (n > 0).then(|| cer_sum / f64::from(n)),
                wer: (n > 0).then(|| wer_sum / f64::from(n)),
                scored_pages: n,
                cost_usd: cost,
            });
        }

        let mut pages = Vec::new();
        for &index in &indices {
            let mut agreement = Vec::new();
            for (i, a) in job.setups.iter().enumerate() {
                for b in job.setups.iter().skip(i + 1) {
                    let (Some(ta), Some(tb)) = (
                        text_of(a.id, index).and_then(|v| v.ai_text.as_deref()),
                        text_of(b.id, index).and_then(|v| v.ai_text.as_deref()),
                    ) else {
                        continue;
                    };
                    agreement.push((a.id, b.id, cer(tb, ta)));
                }
            }
            pages.push(PageComparison { index, agreement });
        }
        Ok(ComparisonReport { scores, pages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_rates_count_edits_against_the_reference() {
        assert_eq!(cer("Bamako", "Bamako"), 0.0);
        assert!((cer("Bamak0", "Bamako") - 1.0 / 6.0).abs() < 1e-9);
        assert!((wer("le 3 mai 1957", "le 5 mai 1957") - 0.25).abs() < 1e-9);
        assert_eq!(cer("", ""), 0.0);
        assert_eq!(cer("x", ""), 1.0);
    }

    #[test]
    fn diffs_mark_words_that_disagree() {
        let spans = diff("le 3 mai 1957", "le 5 mai 1957");
        assert!(
            spans
                .iter()
                .any(|s| s.kind == DiffKind::Removed && s.text.contains('3'))
        );
        assert!(
            spans
                .iter()
                .any(|s| s.kind == DiffKind::Added && s.text.contains('5'))
        );
        let joined: String = spans
            .iter()
            .filter(|s| s.kind != DiffKind::Removed)
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(joined, "le 5 mai 1957");
    }
}
