#![allow(clippy::unwrap_used)]

use super::super::{Built, PartText, SnapshotPart, TimedPart, build, build_with_turns, locate};
use super::*;

fn seg(start_ms: u64, end_ms: u64, speaker: Option<&str>, text: &str) -> Segment {
    Segment {
        start_ms,
        end_ms,
        speaker: speaker.map(str::to_string),
        text: text.into(),
    }
}

struct Part {
    piece: PieceRef,
    text: String,
    segments: Vec<Segment>,
    failed: bool,
}

fn part(piece: PieceRef, text: &str, segments: Vec<Segment>) -> Part {
    Part {
        piece,
        text: text.into(),
        segments,
        failed: false,
    }
}

fn piece(index: u32, start_ms: u64, end_ms: u64) -> PieceRef {
    PieceRef::Segment {
        index,
        start_ms,
        end_ms,
    }
}

fn made(task: Task, parts: &[Part]) -> Built {
    let timed: Vec<TimedPart<'_>> = parts
        .iter()
        .map(|p| TimedPart {
            text: PartText {
                piece: &p.piece,
                text: &p.text,
                reviewed: false,
                corrected: false,
                failed: p.failed,
            },
            segments: &p.segments,
        })
        .collect();
    let built = build_with_turns(task, &timed);
    holds(&built);
    // The text and the parts are those `build` makes without turns.
    let plain: Vec<PartText<'_>> = timed.iter().map(|t| t.text).collect();
    assert_eq!(build(&plain), (built.text.clone(), built.parts.clone()));
    built
}

/// Every turn is non-empty, inside exactly one part, in order, overlapping none, and its
/// text is words with no white space at either end.
fn holds(built: &Built) {
    let mut last_end = 0;
    for turn in &built.turns {
        assert!(turn.start < turn.end, "{turn:?} is empty");
        assert!(turn.start >= last_end, "{turn:?} overlaps the one before");
        last_end = turn.end;
        let inside: Vec<&SnapshotPart> = built
            .parts
            .iter()
            .filter(|p| p.start <= turn.start && turn.end <= p.end)
            .collect();
        assert_eq!(inside.len(), 1, "{turn:?} is not inside one part");
        let words = turn_text(&built.text, turn).unwrap();
        assert_eq!(words, words.trim(), "{turn:?} has white space at an end");
    }
}

fn texts(built: &Built) -> Vec<&str> {
    built
        .turns
        .iter()
        .map(|t| turn_text(&built.text, t).unwrap())
        .collect()
}

fn speakers(built: &Built) -> Vec<Option<&str>> {
    built.turns.iter().map(|t| t.speaker.as_deref()).collect()
}

fn times(built: &Built) -> Vec<(Option<u64>, Option<u64>)> {
    built.turns.iter().map(|t| (t.start_ms, t.end_ms)).collect()
}

#[test]
fn each_segment_is_a_turn_where_its_words_stand() {
    // The provider's text spaces the words otherwise than its segments do.
    let built = made(
        Task::AudioTranscription,
        &[part(
            PieceRef::Whole,
            "Bonjour,  merci de me\nrecevoir.\nJe vous en prie.",
            vec![
                seg(0, 2_000, Some("Imam"), "Bonjour, merci de me recevoir."),
                seg(2_200, 4_000, Some("Chercheur"), "Je vous en prie."),
            ],
        )],
    );
    assert_eq!(
        texts(&built),
        ["Bonjour,  merci de me\nrecevoir.", "Je vous en prie."]
    );
    assert_eq!(speakers(&built), [Some("Imam"), Some("Chercheur")]);
    assert_eq!(
        times(&built),
        [(Some(0), Some(2_000)), (Some(2_200), Some(4_000))]
    );
}

#[test]
fn a_segment_that_cannot_be_placed_leaves_the_part_to_its_text() {
    let built = made(
        Task::AudioTranscription,
        &[part(
            piece(0, 30_000, 60_000),
            "[00:00:01] Speaker 1: Oui.\n[00:00:04] Speaker 2: Non.",
            vec![
                seg(31_000, 33_000, Some("Speaker 1"), "Oui."),
                seg(34_000, 36_000, Some("Speaker 2"), "Peut-être."),
            ],
        )],
    );
    assert_eq!(texts(&built), ["Oui.", "Non."]);
    assert_eq!(
        times(&built),
        [(Some(31_000), Some(34_000)), (Some(34_000), Some(60_000))]
    );
}

#[test]
fn a_corrected_transcript_is_read_from_its_stamps_on_the_recordings_clock() {
    // A correction empties the segments; the stamps are the piece's own.
    let built = made(
        Task::AudioTranscription,
        &[part(
            piece(1, 60_000, 120_000),
            "[00:00:05] Imam: Bismillah.\n  la suite du tour  \n\n[00:00:20] Chercheur:   Merci.\n\
             [00:00:30] Chercheur:\n[00:00:40] Rien qu'une phrase sans nom.",
            Vec::new(),
        )],
    );
    // The stamp and the label are outside each turn; a stamp with no words gives none.
    assert_eq!(
        texts(&built),
        [
            "Bismillah.\n  la suite du tour",
            "Merci.",
            "Rien qu'une phrase sans nom."
        ]
    );
    assert_eq!(speakers(&built), [Some("Imam"), Some("Chercheur"), None]);
    assert_eq!(
        times(&built),
        [
            (Some(65_000), Some(80_000)),
            (Some(80_000), Some(100_000)),
            (Some(100_000), Some(120_000))
        ]
    );
}

#[test]
fn words_before_the_first_stamp_are_a_turn_without_time() {
    let built = made(
        Task::AudioTranscription,
        &[part(
            PieceRef::Whole,
            "Entretien du 3 mai.\n[00:01:00] A: Bonjour.",
            Vec::new(),
        )],
    );
    assert_eq!(texts(&built), ["Entretien du 3 mai.", "Bonjour."]);
    assert_eq!(times(&built), [(None, None), (Some(60_000), None)]);
}

#[test]
fn speaker_lines_without_times_are_turns() {
    let built = made(
        Task::AudioTranscription,
        &[part(
            PieceRef::Whole,
            "Imam Traoré: Bonjour.\nChercheur: Merci, à 12:30 nous parlions.\nde la mosquée\n\
             Imam Traoré : Oui.",
            Vec::new(),
        )],
    );
    assert_eq!(
        texts(&built),
        [
            "Bonjour.",
            "Merci, à 12:30 nous parlions.\nde la mosquée",
            "Oui."
        ]
    );
    assert_eq!(
        speakers(&built),
        [Some("Imam Traoré"), Some("Chercheur"), Some("Imam Traoré")]
    );
    assert_eq!(times(&built), [(None, None); 3]);
}

#[test]
fn other_text_is_cut_into_paragraphs() {
    let pages = [PieceRef::page(1), PieceRef::page(2)];
    let built = made(
        Task::TextExtraction,
        &[
            part(
                pages[0].clone(),
                "Objet : demande de terrain.\nsuite du paragraphe.\n\n\n  Second paragraphe.  \n",
                Vec::new(),
            ),
            part(pages[1].clone(), "Dakar, le 3 mai 1957.", Vec::new()),
        ],
    );
    // A page's "Objet :" is not a speaker: only a transcription has speakers.
    assert_eq!(
        texts(&built),
        [
            "Objet : demande de terrain.\nsuite du paragraphe.",
            "Second paragraphe.",
            "Dakar, le 3 mai 1957."
        ]
    );
    assert_eq!(speakers(&built), [None, None, None]);
    // A transcription without labels or stamps has paragraphs too.
    let plain = made(
        Task::AudioTranscription,
        &[part(PieceRef::Whole, "Un.\n\nDeux.", Vec::new())],
    );
    assert_eq!(texts(&plain), ["Un.", "Deux."]);
}

#[test]
fn a_cut_recording_has_its_turns_on_the_recordings_clock_in_their_parts() {
    // The first piece has its provider's segments (already on the recording's clock, as the
    // export document gives them); the second was corrected, its stamps its own.
    let built = made(
        Task::AudioTranscription,
        &[
            part(
                piece(0, 0, 600_000),
                "Bonjour.\nMerci.",
                vec![
                    seg(1_000, 2_000, Some("A"), "Bonjour."),
                    seg(3_000, 4_000, Some("B"), "Merci."),
                ],
            ),
            part(
                piece(1, 600_000, 1_200_000),
                "[00:00:02] A: Merci à vous.\n[00:01:00] B: Au revoir.",
                Vec::new(),
            ),
        ],
    );
    assert_eq!(
        texts(&built),
        ["Bonjour.", "Merci.", "Merci à vous.", "Au revoir."]
    );
    assert_eq!(
        times(&built),
        [
            (Some(1_000), Some(2_000)),
            (Some(3_000), Some(4_000)),
            (Some(602_000), Some(660_000)),
            (Some(660_000), Some(1_200_000))
        ]
    );
    let part_of = |t: &Turn| {
        built
            .parts
            .iter()
            .position(|p| p.start <= t.start && t.end <= p.end)
            .unwrap()
    };
    let parts: Vec<usize> = built.turns.iter().map(part_of).collect();
    assert_eq!(parts, [0, 0, 1, 1]);
}

#[test]
fn arabic_turns_are_counted_in_characters() {
    let built = made(
        Task::AudioTranscription,
        &[
            part(
                piece(0, 0, 60_000),
                "قال الإمام: السلام عليكم.\nوعليكم السلام.",
                vec![
                    seg(0, 3_000, Some("الإمام"), "قال الإمام: السلام عليكم."),
                    seg(3_000, 6_000, Some("الباحث"), "وعليكم السلام."),
                ],
            ),
            part(
                piece(1, 60_000, 120_000),
                "[00:00:01] Imam: Le Père Éric écrit.",
                Vec::new(),
            ),
        ],
    );
    assert_eq!(
        texts(&built),
        [
            "قال الإمام: السلام عليكم.",
            "وعليكم السلام.",
            "Le Père Éric écrit."
        ]
    );
    // Offsets are characters, not bytes.
    let second = &built.turns[1];
    let chars: Vec<char> = built.text.chars().collect();
    let words: String = chars[second.start as usize..second.end as usize]
        .iter()
        .collect();
    assert_eq!(words, "وعليكم السلام.");
    assert!((built.text.len() as u32) > built.turns[2].end + 10);
}

#[test]
fn a_failed_piece_gives_no_turn() {
    let mut failed = part(piece(1, 60_000, 120_000), "", Vec::new());
    failed.failed = true;
    let built = made(
        Task::AudioTranscription,
        &[
            part(piece(0, 0, 60_000), "Un.", vec![seg(0, 1_000, None, "Un.")]),
            failed,
            part(
                piece(2, 120_000, 180_000),
                "Trois.",
                vec![seg(121_000, 122_000, None, "Trois.")],
            ),
        ],
    );
    assert_eq!(built.parts.len(), 2);
    assert_eq!(texts(&built), ["Un.", "Trois."]);
    assert_eq!(times(&built)[1], (Some(121_000), Some(122_000)));
}

#[test]
fn a_quotation_found_in_the_text_is_in_its_turn() {
    let built = made(
        Task::AudioTranscription,
        &[part(
            PieceRef::Whole,
            "[00:00:01] A: Nous avons construit la mosquée.\n[00:00:09] B: Avec qui ?",
            Vec::new(),
        )],
    );
    let found = locate(&built.text, &built.parts, "construit la mosquée");
    assert_eq!(found.len(), 1);
    assert_eq!(turn_of(&built.turns, &found[0]), Some(0));
    let across = Anchor {
        part: 0,
        start: built.turns[0].end - 3,
        end: built.turns[1].start + 3,
    };
    assert_eq!(turn_of(&built.turns, &across), None);
    // A label is in no turn.
    let label = locate(&built.text, &built.parts, "B:");
    assert_eq!(turn_of(&built.turns, &label[0]), None);
}

#[test]
fn a_snapshot_kept_before_turns_reads_with_none() {
    let built = made(
        Task::AudioTranscription,
        &[part(PieceRef::Whole, "A: Oui.", Vec::new())],
    );
    let snapshot = crate::Snapshot {
        id: "s".into(),
        job_id: crate::JobId::new(),
        setup_id: crate::SetupId::new(),
        source_id: crate::SourceId::new(),
        task: Task::AudioTranscription,
        original_path: "a.mp3".into(),
        sha256: "0".into(),
        parts: built.parts.clone(),
        missing: Vec::new(),
        created_at: crate::Timestamp(0),
        lines: Vec::new(),
        turns: built.turns.clone(),
    };
    let mut json = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(json["turns"][0]["speaker"], "A");
    json.as_object_mut().unwrap().remove("turns");
    let old: crate::Snapshot = serde_json::from_value(json).unwrap();
    assert!(old.turns.is_empty());
}
