//! Speaker labels in transcriptions: finding them, and the names a researcher gives them.
//!
//! A rename is a layer over the model's output, like a correction: the review and every export
//! apply it, and the stored output keeps the model's own labels.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Segment, SetupId, SourceId};

/// The names given to one set-up's speakers in one recording: model label → name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpeakerNames {
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub names: BTreeMap<String, String>,
}

/// Whether the text before a colon reads as a speaker label (`Speaker 2`, `Imam Traoré`, `A`)
/// rather than a clause or a time.
pub fn looks_like_label(label: &str) -> bool {
    !label.is_empty()
        && label.chars().count() <= 40
        && label.split_whitespace().count() <= 4
        && !label.contains('[')
        && label.chars().any(char::is_alphabetic)
}

/// Where the label sits in a transcript line, after an optional `[hh:mm:ss]` stamp: its byte
/// range, ending before the colon that follows it.
fn label_span(line: &str) -> Option<(usize, usize)> {
    let mut at = line.len() - line.trim_start().len();
    if line[at..].starts_with('[') {
        at += line[at..].find(']')? + 1;
        at += line[at..].len() - line[at..].trim_start().len();
    }
    let colon = at + line[at..].find(':')?;
    let label = line[at..colon].trim_end();
    looks_like_label(label).then_some((at, at + label.len()))
}

/// The distinct speaker labels of a transcription, in order of first appearance: from its timed
/// segments when it has any, else from the start of its lines.
pub fn labels(text: &str, segments: &[Segment]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let found: Vec<String> = if segments.iter().any(|s| s.speaker.is_some()) {
        segments.iter().filter_map(|s| s.speaker.clone()).collect()
    } else {
        text.lines()
            .filter_map(|line| label_span(line).map(|(a, b)| line[a..b].to_string()))
            .collect()
    };
    for label in found {
        if !out.contains(&label) {
            out.push(label);
        }
    }
    out
}

/// The names that actually rename something: blank names and unchanged labels are dropped.
fn effective(names: &BTreeMap<String, String>) -> BTreeMap<&str, &str> {
    names
        .iter()
        .map(|(label, name)| (label.as_str(), name.trim()))
        .filter(|(label, name)| !name.is_empty() && label != name)
        .collect()
}

/// The text with each renamed label replaced where it opens a line.
pub fn rename_text(text: &str, names: &BTreeMap<String, String>) -> String {
    let names = effective(names);
    if names.is_empty() {
        return text.to_string();
    }
    text.split('\n')
        .map(|line| match label_span(line) {
            Some((a, b)) => match names.get(&line[a..b]) {
                Some(name) => format!("{}{name}{}", &line[..a], &line[b..]),
                None => line.to_string(),
            },
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The segments with their speakers renamed.
pub fn rename_segments(segments: &[Segment], names: &BTreeMap<String, String>) -> Vec<Segment> {
    let names = effective(names);
    segments
        .iter()
        .map(|s| {
            let mut s = s.clone();
            if let Some(name) = s.speaker.as_deref().and_then(|l| names.get(l)) {
                s.speaker = Some((*name).to_string());
            }
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
            .collect()
    }

    #[test]
    fn finds_labels_after_a_stamp_or_at_the_start() {
        let text = "[00:00:01] Speaker 1: Bonjour.\n[00:00:04] Speaker 2: Merci.\nSpeaker 1: Oui.\n\
                    Dakar, le 3 mai 1957, au cours de la séance du conseil municipal: rien.";
        assert_eq!(labels(text, &[]), ["Speaker 1", "Speaker 2"]);
        assert!(labels("12:30 the meeting", &[]).is_empty());
    }

    #[test]
    fn segments_win_over_the_text() {
        let segments = [Segment {
            start_ms: 0,
            end_ms: 1,
            speaker: Some("A".into()),
            text: "Bonjour".into(),
        }];
        assert_eq!(labels("Speaker 1: Bonjour", &segments), ["A"]);
        let renamed = rename_segments(&segments, &names(&[("A", "Imam Traoré")]));
        assert_eq!(renamed[0].speaker.as_deref(), Some("Imam Traoré"));
    }

    #[test]
    fn renames_only_labels_opening_a_line() {
        let text = "[00:00:01] Speaker 1: Speaker 1: is me.\nSpeaker 2: Merci.\n\nSpeaker 10: Non.";
        let map = names(&[
            ("Speaker 1", "Imam Traoré"),
            ("Speaker 2", " "),
            ("Speaker 10", ""),
        ]);
        assert_eq!(
            rename_text(text, &map),
            "[00:00:01] Imam Traoré: Speaker 1: is me.\nSpeaker 2: Merci.\n\nSpeaker 10: Non."
        );
    }
}
