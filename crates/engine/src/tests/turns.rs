//! A snapshot knows its turns: where each speaker's words sit in the frozen text, with their
//! times on the recording's clock, read with the text so the two never disagree.

use kalamu_core::snapshot::turn_text;
use kalamu_core::{Snapshot, SpeakerNames};

use super::subtitles::{Subtitles, subtitles};
use super::*;

/// The snapshot a frozen source was read from, with its text as written to disk.
fn frozen(t: &Subtitles, job: &Job) -> (Snapshot, String) {
    let sources = t
        .engine
        .snapshot(job.id, job.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let from = sources[0]
        .derived_from
        .clone()
        .unwrap_or_else(|| panic!("lineage"));
    let snapshot = t
        .engine
        .store
        .snapshot(&from.snapshot_id)
        .unwrap_or_else(|e| panic!("{e}"));
    let text = std::fs::read_to_string(&sources[0].path).unwrap_or_else(|e| panic!("{e}"));
    (snapshot, text)
}

/// A turn's words, speaker, start and end.
type Read = (String, Option<String>, Option<u64>, Option<u64>);

/// Each turn's words, speaker and times, every turn checked to lie in one part of the text.
fn turns(snapshot: &Snapshot, text: &str) -> Vec<Read> {
    snapshot
        .turns
        .iter()
        .map(|turn| {
            assert!(
                snapshot
                    .parts
                    .iter()
                    .any(|p| p.start <= turn.start && turn.end <= p.end),
                "{turn:?} is not inside one part"
            );
            let words =
                turn_text(text, turn).unwrap_or_else(|| panic!("{turn:?} is past the text"));
            (
                words.to_string(),
                turn.speaker.clone(),
                turn.start_ms,
                turn.end_ms,
            )
        })
        .collect()
}

fn turn(words: &str, speaker: &str, start_ms: u64, end_ms: Option<u64>) -> Read {
    (words.into(), Some(speaker.into()), Some(start_ms), end_ms)
}

#[tokio::test]
async fn a_frozen_transcript_has_its_turns_in_its_text() {
    let t = subtitles();
    let transcript = t.transcript().await;
    let (snapshot, text) = frozen(&t, &transcript);
    assert_eq!(
        text,
        "[00:00:05] Speaker 1: Bonjour.\n[00:01:10] Speaker 2: Merci."
    );
    // The stamp and the speaker are outside each turn: a turn is the words said.
    assert_eq!(
        turns(&snapshot, &text),
        [
            turn("Bonjour.", "Speaker 1", 5_000, Some(70_000)),
            turn("Merci.", "Speaker 2", 70_000, None),
        ]
    );

    // Corrected and its speakers named: the turns follow the text as the researcher left it.
    t.correct(
        &transcript,
        0,
        "[00:00:05] Speaker 1: Bonjour à tous,\nsoyez les bienvenus.\n[00:01:10] Speaker 2: Merci.",
    );
    t.engine
        .save_speaker_names(
            transcript.id,
            SpeakerNames {
                setup_id: transcript.setups[0].id,
                source_id: transcript.sources[0].id,
                names: [("Speaker 1".to_string(), "Imam Traoré".to_string())].into(),
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let (snapshot, text) = frozen(&t, &transcript);
    assert_eq!(
        turns(&snapshot, &text),
        [
            turn(
                "Bonjour à tous,\nsoyez les bienvenus.",
                "Imam Traoré",
                5_000,
                Some(70_000)
            ),
            turn("Merci.", "Speaker 2", 70_000, None),
        ]
    );
}

#[tokio::test]
async fn a_frozen_translation_has_its_lines_as_turns() {
    let t = subtitles();
    let transcript = t.transcript().await;
    let sources = t
        .engine
        .snapshot(transcript.id, transcript.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let translation = t.translation(sources);
    assert_eq!(t.run(translation.id).await, JobStatus::Completed);
    // Read from the translation's segments, each one line with the time of the line it
    // translates.
    let (snapshot, text) = frozen(&t, &translation);
    assert_eq!(text, "EN: Bonjour.\n\nEN: Merci.");
    let read = turns(&snapshot, &text);
    assert_eq!(read.len(), 2);
    assert_eq!(
        (read[0].0.as_str(), read[0].1.as_deref(), read[0].2),
        ("EN: Bonjour.", Some("Speaker 1"), Some(5_000))
    );
    assert_eq!(
        (read[1].0.as_str(), read[1].1.as_deref(), read[1].2),
        ("EN: Merci.", Some("Speaker 2"), Some(70_000))
    );
}
