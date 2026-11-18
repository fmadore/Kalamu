use super::*;
use crate::FieldValue;

fn line(start_ms: u64, end_ms: u64, speaker: Option<&str>, text: &str) -> Segment {
    Segment {
        start_ms,
        end_ms,
        speaker: speaker.map(str::to_string),
        text: text.into(),
    }
}

fn part(piece: PieceRef, text: &str, segments: Vec<Segment>) -> ExportPart {
    ExportPart {
        piece,
        text: text.into(),
        segments,
        corrected: false,
        reviewed: false,
        fields: Vec::new(),
        regions: Vec::new(),
        failed: None,
        drawn_dpi: None,
        read_at: None,
        corrected_at: None,
    }
}

fn interview() -> Vec<Segment> {
    vec![
        line(2_000, 6_500, Some("Imam"), "Bismillah. Nous commençons."),
        line(7_000, 9_000, Some("Chercheur"), "Merci d'être venu."),
        line(9_500, 12_000, Some("Imam"), "C'est un plaisir."),
    ]
}

fn answer(entries: &[(&str, &str)]) -> Vec<FieldAnswer> {
    let entry = |id: &str, text: &str| {
        vec![
            FieldAnswer {
                key: ID.into(),
                value: FieldValue::Text(id.into()),
                reason: None,
            },
            FieldAnswer {
                key: TEXT.into(),
                value: FieldValue::Text(text.into()),
                reason: None,
            },
        ]
    };
    vec![FieldAnswer {
        key: LINES.into(),
        value: FieldValue::Entries(entries.iter().map(|(i, t)| entry(i, t)).collect()),
        reason: None,
    }]
}

#[test]
fn a_transcripts_lines_are_kept_on_the_recordings_clock_when_every_part_has_times() {
    let segment = |index, start_ms, end_ms| PieceRef::Segment {
        index,
        start_ms,
        end_ms,
    };
    // Timed segments (already on the recording's clock), then markers in a later piece's text.
    let parts = vec![
        part(
            segment(0, 0, 600_000),
            "Bismillah. Nous commençons.",
            vec![line(
                2_000,
                6_500,
                Some("Imam"),
                "Bismillah.\n Nous   commençons.",
            )],
        ),
        part(
            segment(1, 600_000, 1_200_000),
            "[00:00:05] Chercheur: Merci.\n[00:00:09] Imam: Avec plaisir.",
            Vec::new(),
        ),
    ];
    let lines = lines_of(Task::AudioTranscription, &parts).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0].text, "Bismillah. Nous commençons.",
        "one line each"
    );
    assert_eq!(
        lines[1].start_ms, 605_000,
        "markers read on the piece's clock"
    );
    assert_eq!(lines[1].speaker.as_deref(), Some("Chercheur"));
    assert_eq!(
        lines[1].end_ms, 609_000,
        "a marker's line ends where the next begins"
    );

    // A part corrected without its times: no lines at all rather than some.
    let mut corrected = parts.clone();
    corrected[1] = part(
        segment(1, 600_000, 1_200_000),
        "Merci. Avec plaisir.",
        Vec::new(),
    );
    assert_eq!(lines_of(Task::AudioTranscription, &corrected), None);

    // A part that failed is left out, as from the text; an empty one has nothing to time.
    let mut failed = parts.clone();
    failed[1].failed = Some("timeout".into());
    failed.push(part(segment(2, 1_200_000, 1_800_000), "  ", Vec::new()));
    assert_eq!(
        lines_of(Task::AudioTranscription, &failed).map(|l| l.len()),
        Some(1)
    );

    // Only a transcript or a translation of one has lines.
    assert_eq!(lines_of(Task::TextExtraction, &parts), None);
    let translated = vec![part(
        PieceRef::Passage { number: 1 },
        "In the name of God.",
        vec![line(2_000, 6_500, Some("Imam"), "In the name of God.")],
    )];
    assert_eq!(
        lines_of(Task::Translation, &translated).map(|l| l.len()),
        Some(1)
    );
    // A translation's markers are not read: it never wrote times of its own.
    let untimed = vec![part(
        PieceRef::Passage { number: 1 },
        "[00:00:05] Chercheur: Thank you.",
        Vec::new(),
    )];
    assert_eq!(lines_of(Task::Translation, &untimed), None);
}

#[test]
fn passages_never_split_a_line_and_stay_short() {
    let long = "mot ".repeat(400);
    let lines: Vec<Segment> = (0..10u64)
        .map(|i| line(i * 1_000, i * 1_000 + 900, None, long.trim()))
        .collect();
    let cut = passages(&lines);
    assert!(cut.len() > 1);
    assert_eq!(cut.first().map(|r| r.start), Some(0));
    assert_eq!(cut.last().map(|r| r.end), Some(10));
    assert!(cut.windows(2).all(|w| w[0].end == w[1].start));
    let many: Vec<Segment> = (0..300u64).map(|i| line(i, i + 1, None, "Oui.")).collect();
    assert!(passages(&many).iter().all(|r| r.len() <= PASSAGE_LINES));
    assert!(passages(&[]).is_empty());
}

#[test]
fn a_passage_is_sent_as_numbered_lines_without_times_or_speakers() {
    let lines = interview();
    assert_eq!(
        units(&lines, 1..3),
        "[2] Merci d'être venu.\n[3] C'est un plaisir."
    );
    assert_eq!(
        source_text(&lines, 0..2),
        "[00:00:02] Imam: Bismillah. Nous commençons.\n\n[00:00:07] Chercheur: Merci d'être venu."
    );
}

#[test]
fn an_answer_gives_each_line_its_own_time_and_speaker_whatever_its_order() {
    let lines = interview();
    let read = read(
        &answer(&[
            ("[3]", "It is a pleasure."),
            ("2", "Thank you for\ncoming."),
            (" 1 ", "In the name of God. We begin."),
        ]),
        &lines,
        0..3,
    )
    .unwrap();
    assert_eq!(read[1].text, "Thank you for coming.");
    assert_eq!((read[1].start_ms, read[1].end_ms), (7_000, 9_000));
    assert_eq!(read[1].speaker.as_deref(), Some("Chercheur"));
    assert_eq!(read[2].text, "It is a pleasure.");
    assert_eq!(
        text(&read).matches("\n\n").count(),
        2,
        "one paragraph per line"
    );

    // A later passage is numbered over the whole transcript.
    let second = super::read(
        &answer(&[("2", "Thanks."), ("3", "Pleasure.")]),
        &lines,
        1..3,
    )
    .unwrap();
    assert_eq!(second[0].start_ms, 7_000);
}

#[test]
fn an_answer_that_does_not_give_back_every_line_once_is_refused() {
    let lines = interview();
    let refused = |entries: &[(&str, &str)]| read(&answer(entries), &lines, 0..3).unwrap_err();
    let missing = refused(&[("1", "a"), ("3", "c")]);
    assert!(
        missing.contains("1 of 3 lines") && missing.ends_with(": 2"),
        "{missing}"
    );
    assert!(refused(&[("1", "a"), ("2", "b"), ("2", "b"), ("3", "c")]).contains("twice"));
    assert!(refused(&[("1", "a"), ("2", "b"), ("3", "c"), ("4", "d")]).contains("\"4\""));
    assert!(refused(&[("1", "a"), ("2", " "), ("3", "c")]).contains("empty"));
    assert!(refused(&[("un", "a"), ("2", "b"), ("3", "c")]).contains("\"un\""));
    assert!(read(&[], &lines, 0..3).is_err());
    // A line of another passage is not this one's.
    assert!(read(&answer(&[("1", "a"), ("2", "b")]), &lines, 1..3).is_err());
}

#[test]
fn a_correction_keeps_the_times_only_while_it_keeps_one_paragraph_per_line() {
    let lines = interview();
    let kept = as_corrected(
        &lines,
        "In God's name. We begin.\r\n\r\nThanks for coming.\n\n\n\nA pleasure,\nreally.",
    )
    .unwrap();
    assert_eq!(kept[0].text, "In God's name. We begin.");
    assert_eq!(
        kept[2].text, "A pleasure, really.",
        "a line broken inside stays one cue"
    );
    assert_eq!(kept[2].start_ms, 9_500);
    // Two lines joined: three lines, two paragraphs.
    assert_eq!(
        as_corrected(&lines, "In God's name. Thanks for coming.\n\nA pleasure."),
        Err(2)
    );
}

#[test]
fn the_fingerprint_follows_the_times_as_well_as_the_words() {
    let lines = interview();
    let mut moved = lines.clone();
    moved[1].start_ms += 1_000;
    assert_ne!(fingerprint(&lines), fingerprint(&moved));
    assert_eq!(fingerprint(&lines), fingerprint(&interview()));
}

#[test]
fn the_schema_is_one_field_of_numbered_entries() {
    let schema = schema();
    assert!(schema.validate_analysis().is_ok());
    let json = crate::structured::schema(&schema);
    assert_eq!(
        json["properties"]["lines"]["items"]["required"],
        serde_json::json!(["id", "text"])
    );
}
