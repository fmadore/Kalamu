//! Error rates against the researcher's review, for any job, and how far the set-ups of a
//! comparison agree with each other. Error rates follow zmo-ai-pipelines' `zmo_evaluate.py`:
//! edit distance over characters (CER) and over words (WER), divided by the reference length,
//! here summed over every scored page rather than averaged page by page.

use kalamu_core::speakers::rename_text;
use kalamu_core::{CacheKey, Job, JobId, Setup, SetupId, SourceId, Task};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use ts_rs::TS;

use crate::{Engine, EngineError};

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

/// Edit distance between `hypothesis` and `reference`: the insertions, deletions and
/// substitutions that turn one into the other.
fn edits<T: PartialEq>(hypothesis: &[T], reference: &[T]) -> usize {
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
    prev[hypothesis.len()]
}

/// Edits divided by the reference's length; an empty reference is all wrong unless the
/// hypothesis is empty too.
fn rate(edits: usize, length: usize) -> f64 {
    if length == 0 {
        return if edits == 0 { 0.0 } else { 1.0 };
    }
    edits as f64 / length as f64
}

/// Edit distance between `hypothesis` and `reference`, divided by the reference length.
pub fn error_rate<T: PartialEq>(hypothesis: &[T], reference: &[T]) -> f64 {
    rate(edits(hypothesis, reference), reference.len())
}

/// Characters with runs of white space made one space: a line break the model kept or joined
/// is not a reading error.
fn characters(text: &str) -> Vec<char> {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .collect()
}

fn words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn cer(hypothesis: &str, reference: &str) -> f64 {
    error_rate(&characters(hypothesis), &characters(reference))
}

#[cfg(test)]
fn wer(hypothesis: &str, reference: &str) -> f64 {
    error_rate(&words(hypothesis), &words(reference))
}

/// Edits and lengths summed over the scored pages, so that the rates are taken over every
/// character and word: a short page with two errors weighs less than a full one.
#[derive(Debug, Clone, Copy, Default)]
struct Tally {
    char_edits: usize,
    chars: usize,
    word_edits: usize,
    words: usize,
    pages: u32,
}

impl Tally {
    fn add(&mut self, hypothesis: &str, reference: &str) {
        let (h, r) = (characters(hypothesis), characters(reference));
        self.char_edits += edits(&h, &r);
        self.chars += r.len();
        let (h, r) = (words(hypothesis), words(reference));
        self.word_edits += edits(&h, &r);
        self.words += r.len();
        self.pages += 1;
    }

    fn merge(&mut self, other: &Tally) {
        self.char_edits += other.char_edits;
        self.chars += other.chars;
        self.word_edits += other.word_edits;
        self.words += other.words;
        self.pages += other.pages;
    }

    fn score(&self, setup: &Setup, cost_usd: Option<f64>) -> SetupScore {
        let scored = self.pages > 0;
        SetupScore {
            setup_id: setup.id,
            label: setup.label.clone(),
            cer: scored.then(|| rate(self.char_edits, self.chars)),
            wer: scored.then(|| rate(self.word_edits, self.words)),
            scored_pages: self.pages,
            cost_usd,
        }
    }
}

/// How one set-up reads against the researcher's reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupScore {
    pub setup_id: SetupId,
    pub label: String,
    /// Character error rate over every scored page: edits divided by the reference's characters,
    /// runs of white space counting as one space. `None` when no page has a reference.
    pub cer: Option<f64>,
    /// Word error rate, likewise over words.
    pub wer: Option<f64>,
    /// Pages (or parts of a recording) with a reference that were scored.
    pub scored_pages: u32,
    pub cost_usd: Option<f64>,
}

/// A document's scores, one per set-up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceScores {
    pub source_id: SourceId,
    pub scores: Vec<SetupScore>,
}

/// A job's error rates against the pages the researcher reviewed or corrected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobScores {
    /// Over every document, one per set-up, in set-up order. Empty for tasks that are not
    /// scored (analyses, tables of contents).
    pub total: Vec<SetupScore>,
    /// The documents with at least one reference.
    pub sources: Vec<SourceScores>,
}

/// What scoring needs of one piece.
struct Scored {
    setup_id: SetupId,
    index: u32,
    ai_text: Option<String>,
    corrected_text: Option<String>,
    reviewed: bool,
    cost_usd: Option<f64>,
}

/// The text a page is scored against: the researcher's correction, or else a model's own text
/// that they marked as reviewed without changing it. Pieces are in set-up order, so with
/// several the first set-up's wins.
fn reference(pieces: &[Scored], index: u32) -> Option<&str> {
    let here = || pieces.iter().filter(move |p| p.index == index);
    here()
        .find_map(|p| p.corrected_text.as_deref())
        .or_else(|| {
            here()
                .filter(|p| p.reviewed)
                .find_map(|p| p.ai_text.as_deref())
        })
}

/// Extracted text, transcriptions and corrections have a text to score; analyses and tables
/// of contents answer in fields.
fn scored(task: Task) -> bool {
    matches!(
        task,
        Task::TextExtraction | Task::AudioTranscription | Task::TextCorrection
    )
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
    /// Each piece of a source with the texts scoring reads, in page and set-up order. Lighter
    /// than the review's pieces: no preview, timings or original text.
    fn scored_pieces(&self, job: &Job, source_id: SourceId) -> Result<Vec<Scored>, EngineError> {
        let corrections = self.store.corrections(job.id)?;
        let names = self.store.speaker_names(job.id)?;
        let mut out = Vec::new();
        for state in self.store.pieces(job.id)? {
            if state.source_id != source_id {
                continue;
            }
            let result = match &state.cache_key {
                Some(k) if state.status.is_finished() => {
                    self.store.cached_result(&CacheKey(k.clone()))?
                }
                _ => None,
            };
            let correction = corrections.iter().find(|c| {
                c.setup_id == state.setup_id && c.source_id == source_id && c.index == state.index
            });
            // As the review shows them: with the researcher's names for the speakers.
            let speaker_names = names
                .iter()
                .find(|n| n.setup_id == state.setup_id && n.source_id == source_id)
                .map(|n| n.names.clone())
                .unwrap_or_default();
            out.push(Scored {
                setup_id: state.setup_id,
                index: state.index,
                ai_text: result
                    .as_ref()
                    .map(|r| rename_text(&r.output.text, &speaker_names)),
                corrected_text: correction
                    .and_then(|c| c.text.as_deref())
                    .map(|t| rename_text(t, &speaker_names)),
                reviewed: correction.is_some_and(|c| c.reviewed),
                cost_usd: result.as_ref().and_then(|r| r.cost_usd),
            });
        }
        let order = |id: SetupId| job.setups.iter().position(|s| s.id == id);
        out.sort_by_key(|p| (p.index, order(p.setup_id)));
        Ok(out)
    }

    /// Each set-up's tally and cost on one source.
    fn tally(&self, job: &Job, pieces: &[Scored]) -> Vec<(Tally, Option<f64>)> {
        job.setups
            .iter()
            .map(|setup| {
                let mut tally = Tally::default();
                let mut cost: Option<f64> = None;
                for piece in pieces.iter().filter(|p| p.setup_id == setup.id) {
                    if let Some(c) = piece.cost_usd {
                        cost = Some(cost.unwrap_or(0.0) + c);
                    }
                    if !scored(job.task) {
                        continue;
                    }
                    if let (Some(reference), Some(ai)) =
                        (reference(pieces, piece.index), &piece.ai_text)
                    {
                        tally.add(ai, reference);
                    }
                }
                (tally, cost)
            })
            .collect()
    }

    /// Scores every set-up of a job on one source.
    ///
    /// A page counts as a reference when the researcher corrected it, or marked it as reviewed
    /// without changing it, in any set-up; that text is the reference for every set-up on the
    /// page.
    pub fn compare(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<ComparisonReport, EngineError> {
        let job = self.store.job(job_id)?;
        let pieces = self.scored_pieces(&job, source_id)?;
        let indices: std::collections::BTreeSet<u32> = pieces.iter().map(|p| p.index).collect();
        let text_of = |setup: SetupId, index: u32| -> Option<&Scored> {
            pieces
                .iter()
                .find(|p| p.setup_id == setup && p.index == index)
        };

        let scores = job
            .setups
            .iter()
            .zip(self.tally(&job, &pieces))
            .map(|(setup, (tally, cost))| tally.score(setup, cost))
            .collect();

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

    /// A job's error rates against the pages the researcher reviewed or corrected: for each
    /// document, and over all of them.
    pub fn job_scores(&self, job_id: JobId) -> Result<JobScores, EngineError> {
        let job = self.store.job(job_id)?;
        if !scored(job.task) {
            return Ok(JobScores {
                total: Vec::new(),
                sources: Vec::new(),
            });
        }
        let mut totals = vec![(Tally::default(), None::<f64>); job.setups.len()];
        let mut sources = Vec::new();
        for source in &job.sources {
            let pieces = self.scored_pieces(&job, source.id)?;
            let tallies = self.tally(&job, &pieces);
            for ((total, total_cost), (tally, cost)) in totals.iter_mut().zip(&tallies) {
                total.merge(tally);
                if let Some(c) = cost {
                    *total_cost = Some(total_cost.unwrap_or(0.0) + c);
                }
            }
            if tallies.iter().any(|(t, _)| t.pages > 0) {
                sources.push(SourceScores {
                    source_id: source.id,
                    scores: job
                        .setups
                        .iter()
                        .zip(&tallies)
                        .map(|(setup, (tally, cost))| tally.score(setup, *cost))
                        .collect(),
                });
            }
        }
        let total = job
            .setups
            .iter()
            .zip(&totals)
            .map(|(setup, (tally, cost))| tally.score(setup, *cost))
            .collect();
        Ok(JobScores { total, sources })
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
    fn rates_are_taken_over_every_character_not_averaged_by_page() {
        let mut tally = Tally::default();
        tally.add("Bamak0", "Bamako");
        tally.add(&"Ségou ".repeat(16), &"Ségou ".repeat(16));
        // One edit in 6 + 95 characters: 1%, where the mean of the pages' rates says 8%.
        assert_eq!(tally.chars, 101);
        assert!((rate(tally.char_edits, tally.chars) - 1.0 / 101.0).abs() < 1e-9);
        assert_eq!(tally.pages, 2);
        // A line break the model kept or joined is not an error.
        let mut lines = Tally::default();
        lines.add("le 3 mai\n1957", "le 3 mai 1957");
        assert_eq!((lines.char_edits, lines.word_edits), (0, 0));
    }

    fn piece(
        setup: SetupId,
        index: u32,
        ai: &str,
        corrected: Option<&str>,
        reviewed: bool,
    ) -> Scored {
        Scored {
            setup_id: setup,
            index,
            ai_text: Some(ai.into()),
            corrected_text: corrected.map(Into::into),
            reviewed,
            cost_usd: None,
        }
    }

    #[test]
    fn a_page_reviewed_as_it_stands_is_its_own_reference() {
        let setup = SetupId::new();
        let pieces = [
            piece(setup, 0, "Ségou", None, true),
            piece(setup, 1, "Bamak0", Some("Bamako"), true),
            piece(setup, 2, "Kayes", None, false),
        ];
        assert_eq!(reference(&pieces, 0), Some("Ségou"));
        assert_eq!(reference(&pieces, 1), Some("Bamako"));
        assert_eq!(reference(&pieces, 2), None, "not reviewed: no reference");
    }

    #[test]
    fn a_correction_wins_over_a_text_reviewed_as_it_stands() {
        let (a, b) = (SetupId::new(), SetupId::new());
        let pieces = [
            piece(a, 0, "Ségou", None, true),
            piece(b, 0, "Segou", Some("Ségou-ville"), false),
        ];
        assert_eq!(reference(&pieces, 0), Some("Ségou-ville"));
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
