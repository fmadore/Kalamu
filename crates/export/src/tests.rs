//! Golden-file tests: each format's output for fixed documents is compared with a file in
//! `golden/`. Run with `UPDATE_GOLDEN=1` to rewrite the files after an intended change, and
//! review the diff.

use std::path::PathBuf;

use kalamu_core::provenance::{GeneratorInfo, ProvenancePrompt, ProvenanceSource};
use kalamu_core::{
    AnalysisSchema, ExportOptions, ExportPart, Field, FieldAnswer, FieldKind, FieldValue,
    InputKind, LanguageChoice, PromptSnapshot, Provenance, ProviderId, RequestSetting, Segment,
    Setup, SetupId, SetupOptions, SourceFile, SourceId, Task, Timestamp, Usage,
};

use super::*;

mod cited;
mod page_xml;
mod records;
mod researcher;
mod screening;

fn source(name: &str, kind: InputKind) -> SourceFile {
    SourceFile {
        id: SourceId(uuid_nil()),
        path: format!(r"C:\archive\{name}"),
        file_name: name.into(),
        sha256: "0f".repeat(32),
        size_bytes: 1024,
        modified_ms: None,
        kind,
        page_count: None,
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        dense_pages: None,
        title: None,
        derived_from: None,
        pages: Vec::new(),
    }
}

fn uuid_nil() -> uuid::Uuid {
    uuid::Uuid::nil()
}

fn document(task: Task, source: SourceFile, parts: Vec<ExportPart>) -> ExportDocument {
    let setup = Setup {
        id: SetupId(uuid_nil()),
        label: "gemini-flash".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        prompt: PromptSnapshot {
            preset_id: Some("text_extraction.printed_diplomatic".into()),
            preset_name: "Printed document, diplomatic".into(),
            template: "Transcribe.".into(),
            text: "Transcribe.".into(),
            user_prompt: "Transcribe this page.".into(),
            schema: None,
        },
        options: SetupOptions::default(),
        merge: None,
        pages_cap: None,
        windows: None,
    };
    let provenance = Provenance {
        schema_version: 1,
        generator: GeneratorInfo {
            name: "Kalamu".into(),
            version: "0.1.0".into(),
        },
        created_at: Timestamp(0),
        task,
        source: ProvenanceSource {
            file_name: source.file_name.clone(),
            sha256: source.sha256.clone(),
            size_bytes: source.size_bytes,
            url: None,
            title: None,
            read_at: None,
            derived_from: None,
            pages: kalamu_core::provenance::ProvenancePage::of(&source.pages),
        },
        documents: vec![],
        engine: "cloud".into(),
        provider: setup.provider.clone(),
        server: None,
        model: setup.model.clone(),
        model_versions: vec![],
        request: vec![
            RequestSetting::new("generationConfig.thinkingConfig.thinkingLevel", "low"),
            RequestSetting::new("generationConfig.maxOutputTokens", 65535),
        ],
        prompt: ProvenancePrompt {
            preset_id: setup.prompt.preset_id.clone(),
            preset_name: setup.prompt.preset_name.clone(),
            text: setup.prompt.text.clone(),
            user_prompt: setup.prompt.user_prompt.clone(),
            sha256: "ab".repeat(32),
            schema: setup.prompt.schema.clone(),
        },
        merge: None,
        languages: LanguageChoice::Auto,
        detected_languages: vec![],
        options: SetupOptions::default(),
        pieces: parts.len() as u32,
        corrected: vec![2],
        reviewed: vec![1, 2],
        truncated: vec![],
        cut_by_filter: vec![],
        failed: vec![],
        lines_changed: vec![],
        preprocessing: vec![],
        renamed_speakers: vec![],
        listed_by_hand: vec![],
        cited: vec![],
        passages: vec![],
        regions: vec![],
        region_readings: vec![],
        usage: Usage::default(),
        cost_usd: Some(0.002),
        refused: None,
        workflow: None,
    };
    ExportDocument {
        task,
        source,
        setup,
        parts,
        provenance,
        options: ExportOptions::default(),
    }
}

fn letter() -> ExportDocument {
    document(
        Task::TextExtraction,
        source("lettre.pdf", InputKind::Pdf),
        vec![
            ExportPart {
                piece: kalamu_core::PieceRef::page(1),
                text: "Bamako, le 3 mai 1957\n\nMonsieur le Gouverneur,\nj'ai l'honneur de vous "
                    .into(),
                segments: vec![],
                corrected: false,
                reviewed: true,
                rows_corrected: Vec::new(),
                fields: vec![],
                regions: vec![],
                failed: None,
                drawn_dpi: None,
                read_at: None,
                corrected_at: None,
                researcher: Default::default(),
            },
            ExportPart {
                piece: kalamu_core::PieceRef::page(2),
                text: "rendre compte de la réunion tenue à la mosquée.\n\n[ILLEGIBLE]".into(),
                segments: vec![],
                corrected: true,
                reviewed: true,
                rows_corrected: Vec::new(),
                fields: vec![],
                regions: vec![],
                failed: None,
                drawn_dpi: None,
                read_at: None,
                corrected_at: None,
                researcher: Default::default(),
            },
            // Refused every time: the letter is still written, the page marked in its place.
            ExportPart {
                piece: kalamu_core::PieceRef::page(3),
                text: String::new(),
                segments: vec![],
                corrected: false,
                reviewed: false,
                rows_corrected: Vec::new(),
                fields: vec![],
                regions: vec![],
                failed: Some("Gemini's copyright filter stopped this page (RECITATION)".into()),
                drawn_dpi: None,
                read_at: None,
                corrected_at: None,
                researcher: Default::default(),
            },
        ],
    )
}

fn interview() -> ExportDocument {
    document(
        Task::AudioTranscription,
        source("entretien.m4a", InputKind::Audio),
        vec![
            ExportPart {
                piece: kalamu_core::PieceRef::Segment {
                    index: 0,
                    start_ms: 0,
                    end_ms: 1_200_000,
                },
                text: String::new(),
                segments: vec![
                    Segment {
                        start_ms: 1_000,
                        end_ms: 5_000,
                        speaker: Some("Speaker 1".into()),
                        text: "Bonjour, merci de nous recevoir.".into(),
                    },
                    Segment {
                        start_ms: 5_500,
                        end_ms: 9_000,
                        speaker: Some("Speaker 2".into()),
                        text: "Je vous en prie.".into(),
                    },
                ],
                corrected: false,
                reviewed: false,
                rows_corrected: Vec::new(),
                fields: vec![],
                regions: vec![],
                failed: None,
                drawn_dpi: None,
                read_at: None,
                corrected_at: None,
                researcher: Default::default(),
            },
            ExportPart {
                // A prompt-based model that wrote timestamps into its text.
                piece: kalamu_core::PieceRef::Segment {
                    index: 1,
                    start_ms: 1_200_000,
                    end_ms: 2_400_000,
                },
                text: "[00:00:02] Speaker 1: Parlons de la medersa.\n[00:00:06] Speaker 2: Elle a ouvert en 1962.".into(),
                segments: vec![],
                corrected: false,
                reviewed: false,
                rows_corrected: Vec::new(),
                fields: vec![],
                regions: vec![],
                failed: None,
                drawn_dpi: None,
                read_at: None,
                corrected_at: None,
                researcher: Default::default(),
            },
        ],
    )
}

/// An analysis of one article: a summary, the places it names and its tone with a reason.
fn analysis() -> ExportDocument {
    let mut schema = AnalysisSchema {
        fields: vec![
            Field {
                key: String::new(),
                label: "Summary".into(),
                instructions: String::new(),
                kind: FieldKind::Text,
                reason: false,
            },
            Field {
                key: String::new(),
                label: "Places".into(),
                instructions: String::new(),
                kind: FieldKind::List { max: None },
                reason: false,
            },
            Field {
                key: String::new(),
                label: "Tone".into(),
                instructions: String::new(),
                kind: FieldKind::Choice {
                    values: vec!["Positive".into(), "Neutral".into(), "Negative".into()],
                },
                reason: true,
            },
        ],
    };
    schema.assign_keys();
    let fields = vec![
        FieldAnswer {
            key: "summary".into(),
            value: FieldValue::Text("Réunion de deux cents fidèles à la mosquée de Bamako.".into()),
            reason: None,
        },
        FieldAnswer {
            key: "places".into(),
            value: FieldValue::List(vec!["Bamako".into(), "Ségou".into()]),
            reason: None,
        },
        FieldAnswer {
            key: "tone".into(),
            value: FieldValue::Text("Neutral".into()),
            reason: Some("A factual report of the meeting.".into()),
        },
    ];
    let mut doc = document(
        Task::Analysis,
        source("bulletin.txt", InputKind::Text),
        vec![ExportPart {
            piece: kalamu_core::PieceRef::Whole,
            text: schema.render(&fields),
            segments: vec![],
            corrected: true,
            reviewed: true,
            rows_corrected: Vec::new(),
            fields,
            regions: vec![],
            failed: None,
            drawn_dpi: None,
            read_at: None,
            corrected_at: None,
            researcher: Default::default(),
        }],
    );
    doc.setup.prompt.schema = Some(schema.clone());
    doc.provenance.prompt.schema = Some(schema);
    doc
}

/// The same analysis, a place the model missed added by the researcher: in the answers as
/// any other, and named in the provenance as written by hand.
fn analysis_with_a_place_added() -> ExportDocument {
    let mut doc = analysis();
    let part = &mut doc.parts[0];
    let (fields, _) =
        kalamu_core::list_items::add_items(&part.fields, "places", &["Tombouctou".into()]);
    if let Some(schema) = &doc.setup.prompt.schema {
        part.text = schema.render(&fields);
    }
    part.fields = fields;
    doc.provenance.corrected = vec![1];
    doc.provenance.listed_by_hand = vec![kalamu_core::provenance::ProvenanceListed {
        part: 1,
        field: "places".into(),
        items: vec!["Tombouctou".into()],
    }];
    doc
}

/// A magazine issue's table of contents: the combined entries, one corrected by the researcher.
fn contents() -> ExportDocument {
    use kalamu_core::contents::{answers, merge_schema};
    let entry = |title: &str, pages: &str, authors: &[&str], summary: &str| {
        vec![
            FieldAnswer {
                key: "title".into(),
                value: FieldValue::Text(title.into()),
                reason: None,
            },
            FieldAnswer {
                key: "pages".into(),
                value: FieldValue::Text(pages.into()),
                reason: None,
            },
            FieldAnswer {
                key: "authors".into(),
                value: FieldValue::List(authors.iter().map(|a| (*a).to_string()).collect()),
                reason: None,
            },
            FieldAnswer {
                key: "summary".into(),
                value: FieldValue::Text(summary.into()),
                reason: None,
            },
        ]
    };
    let mut fields = AnalysisSchema {
        fields: vec![
            Field {
                key: String::new(),
                label: "Authors".into(),
                instructions: String::new(),
                kind: FieldKind::List { max: None },
                reason: false,
            },
            Field {
                key: String::new(),
                label: "Summary".into(),
                instructions: String::new(),
                kind: FieldKind::Text,
                reason: false,
            },
        ],
    };
    fields.assign_keys();
    let mut doc = document(
        Task::Contents,
        source("an-nasr-12.pdf", InputKind::Pdf),
        vec![ExportPart {
            piece: kalamu_core::PieceRef::Whole,
            text: String::new(),
            segments: vec![],
            corrected: true,
            reviewed: false,
            rows_corrected: Vec::new(),
            fields: answers(vec![
                entry(
                    "[Éditorial] Le Hajj, pilier de l'islam",
                    "1, 3-4",
                    &["La Rédaction"],
                    "L'éditorial rappelle les conditions du pèlerinage de 1985.",
                ),
                entry(
                    "Fatwa sur le jeûne",
                    "5",
                    &[],
                    "Réponses de l'imam aux lecteurs.",
                ),
            ]),
            regions: vec![],
            failed: None,
            drawn_dpi: None,
            read_at: None,
            corrected_at: None,
            researcher: Default::default(),
        }],
    );
    doc.setup.merge = Some(kalamu_core::MergeStep {
        provider: ProviderId::new("mistral"),
        model: "mistral-small-2603".into(),
        prompt: PromptSnapshot {
            preset_id: Some("contents.periodical".into()),
            preset_name: "Newspaper or magazine issue".into(),
            template: "Combine.".into(),
            text: "Combine.".into(),
            user_prompt: "Combine the pages.".into(),
            schema: Some(merge_schema(&fields)),
        },
    });
    doc
}

/// A long text file corrected in passages, which carry no markers.
fn corrected() -> ExportDocument {
    let part = |number: u32, text: &str| ExportPart {
        piece: kalamu_core::PieceRef::Passage { number },
        text: text.into(),
        segments: vec![],
        corrected: false,
        reviewed: false,
        rows_corrected: Vec::new(),
        fields: vec![],
        regions: vec![],
        failed: None,
        drawn_dpi: None,
        read_at: None,
        corrected_at: None,
        researcher: Default::default(),
    };
    document(
        Task::TextCorrection,
        source("gallica.txt", InputKind::Text),
        vec![
            part(1, "Le gouvernement a décidé."),
            part(2, "La séance est levée."),
        ],
    )
}

/// An earlier result's text translated page by page, the second page corrected in review.
fn translated() -> ExportDocument {
    let part = |number: u32, text: &str, corrected: bool| ExportPart {
        piece: kalamu_core::PieceRef::Page { number, half: None },
        text: text.into(),
        segments: vec![],
        corrected,
        reviewed: corrected,
        rows_corrected: Vec::new(),
        fields: vec![],
        regions: vec![],
        failed: None,
        drawn_dpi: None,
        read_at: None,
        corrected_at: None,
        researcher: Default::default(),
    };
    document(
        Task::Translation,
        source("lettre.txt", InputKind::Text),
        vec![
            part(1, "The government has decided.", false),
            part(2, "[00:01:15] Speaker 2: The meeting is closed.", true),
        ],
    )
}

/// The interview's transcript translated line by line (#167): each passage's lines keep the
/// times and speakers of the lines they translate, on the recording's clock; the second passage
/// was corrected in review, one paragraph per line, so its lines carry the corrected words.
fn translated_interview() -> ExportDocument {
    let line = |start_ms: u64, end_ms: u64, speaker: &str, text: &str| Segment {
        start_ms,
        end_ms,
        speaker: Some(speaker.into()),
        text: text.into(),
    };
    let part = |number: u32, lines: Vec<Segment>, corrected: bool| ExportPart {
        piece: kalamu_core::PieceRef::Passage { number },
        text: kalamu_core::aligned::text(&lines),
        segments: lines,
        corrected,
        reviewed: corrected,
        rows_corrected: Vec::new(),
        fields: vec![],
        regions: vec![],
        failed: None,
        drawn_dpi: None,
        read_at: None,
        corrected_at: None,
        researcher: Default::default(),
    };
    let mut doc = document(
        Task::Translation,
        source("entretien.txt", InputKind::Text),
        vec![
            part(
                1,
                vec![
                    line(1_000, 5_000, "Speaker 1", "Hello, thank you for having us."),
                    line(5_500, 9_000, "Speaker 2", "You are welcome."),
                ],
                false,
            ),
            part(
                2,
                vec![
                    line(
                        1_202_000,
                        1_206_000,
                        "Speaker 1",
                        "Let us talk about the madrasa.",
                    ),
                    line(
                        1_206_000,
                        1_210_000,
                        "Speaker 2",
                        "It opened in 1962, after the community had gathered the money for its \
                         first classrooms over several years.",
                    ),
                ],
                true,
            ),
        ],
    );
    doc.provenance.corrected = vec![2];
    doc.provenance.reviewed = vec![2];
    doc
}

/// The letter read from three photographs made one document (#52): written as one result,
/// page by page as a PDF's, its provenance recording each page's file in the order read.
fn photographed_letter() -> ExportDocument {
    let page = |name: &str, sha: &str, label: Option<&str>| kalamu_core::PageFile {
        id: kalamu_core::ordered::page_id(&format!(r"C:\archive\Box 3\{name}"), sha),
        path: format!(r"C:\archive\Box 3\{name}"),
        file_name: name.into(),
        sha256: sha.repeat(32),
        size_bytes: 2048,
        modified_ms: None,
        frames: 1,
        label: label.map(str::to_string),
    };
    let source = SourceFile::image_document(
        "Lettre au Gouverneur",
        vec![
            page("IMG_0002.jpg", "a1", Some("f. 1r")),
            page("IMG_0010.jpg", "b2", None),
            page("IMG_0011.jpg", "c3", None),
        ],
    );
    let mut doc = letter();
    doc.provenance.source = ProvenanceSource {
        file_name: source.file_name.clone(),
        sha256: source.sha256.clone(),
        size_bytes: source.size_bytes,
        url: None,
        title: None,
        read_at: None,
        derived_from: None,
        pages: kalamu_core::provenance::ProvenancePage::of(&source.pages),
    };
    doc.source = source;
    doc
}

#[test]
fn a_document_of_images_matches_golden_files() {
    let doc = photographed_letter();
    check("photographed.txt", &run(ExportFormat::Txt, &doc));
    check("photographed.md", &run(ExportFormat::Markdown, &doc));
    check("photographed.json", &run(ExportFormat::Json, &doc));
    // Pages are numbered as the document's, like a PDF's: the same text as the PDF letter.
    assert_eq!(
        run(ExportFormat::Txt, &doc),
        run(ExportFormat::Txt, &letter())
    );
    // No PDF to lay a text layer over: the format is not written for images.
    assert!(exporter(ExportFormat::SearchablePdf).is_some_and(|e| e.export(&doc).is_err()));
}

fn check(name: &str, actual: &[u8]) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("golden")
        .join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(&path));
        let _ = std::fs::write(&path, actual);
        return;
    }
    let expected = std::fs::read(&path).unwrap_or_default();
    assert_eq!(
        String::from_utf8_lossy(actual),
        String::from_utf8_lossy(&expected),
        "{name} differs from its golden file; rerun with UPDATE_GOLDEN=1 if intended"
    );
}

fn run(format: ExportFormat, doc: &ExportDocument) -> Vec<u8> {
    exporter(format)
        .map(|e| e.export(doc))
        .and_then(Result::ok)
        .unwrap_or_default()
}

#[test]
fn text_formats_match_golden_files() {
    check("letter.txt", &run(ExportFormat::Txt, &letter()));
    check("letter.md", &run(ExportFormat::Markdown, &letter()));
    check("letter.json", &run(ExportFormat::Json, &letter()));
    check("letter.csv", &run(ExportFormat::Csv, &letter()));
    check("interview.txt", &run(ExportFormat::Txt, &interview()));
    check("interview.srt", &run(ExportFormat::Srt, &interview()));
    check("interview.vtt", &run(ExportFormat::Vtt, &interview()));
    check("interview.csv", &run(ExportFormat::Csv, &interview()));
    check("analysis.txt", &run(ExportFormat::Txt, &analysis()));
    check("analysis.json", &run(ExportFormat::Json, &analysis()));
    check("analysis.csv", &run(ExportFormat::Csv, &analysis()));
    let added = analysis_with_a_place_added();
    check("analysis-added.json", &run(ExportFormat::Json, &added));
    check("analysis-added.csv", &run(ExportFormat::Csv, &added));
    check("contents.txt", &run(ExportFormat::Txt, &contents()));
    check("contents.md", &run(ExportFormat::Markdown, &contents()));
    check("contents.json", &run(ExportFormat::Json, &contents()));
    check("contents.csv", &run(ExportFormat::Csv, &contents()));
    check("corrected.txt", &run(ExportFormat::Txt, &corrected()));
    check("corrected.csv", &run(ExportFormat::Csv, &corrected()));
    check("translated.txt", &run(ExportFormat::Txt, &translated()));
    check("translated.csv", &run(ExportFormat::Csv, &translated()));
}

/// A workbook's cells read back, one row of JSON strings per line, compared with
/// `golden/<name>`.
fn check_workbook(name: &str, bytes: &[u8]) {
    let rows: Vec<String> = crate::table::tests::workbook_cells(bytes)
        .iter()
        .map(|row| serde_json::to_string(row).unwrap_or_default())
        .collect();
    check(name, format!("[\n{}\n]\n", rows.join(",\n")).as_bytes());
}

/// Each task's workbook holds the same table as its CSV, every cell text, none guarded.
#[test]
fn workbooks_match_golden_files() {
    let added = analysis_with_a_place_added();
    for (name, doc) in [
        ("letter", letter()),
        ("interview", interview()),
        ("analysis", analysis()),
        ("analysis-added", added),
        ("contents", contents()),
        ("corrected", corrected()),
        ("translated", translated()),
    ] {
        check_workbook(&format!("{name}.xlsx.json"), &run(ExportFormat::Xlsx, &doc));
    }
}

#[test]
fn a_workbook_cell_a_spreadsheet_would_compute_is_text_as_written() {
    let mut doc = interview();
    doc.parts[0].segments[0].text = "- Oui, au +223 20 22 33 44.".into();
    doc.parts[0].segments[1].text = "=1+1".into();
    let rows = crate::table::tests::workbook_cells(&run(ExportFormat::Xlsx, &doc));
    assert_eq!(rows[1][3], "- Oui, au +223 20 22 33 44.");
    assert_eq!(rows[2][3], "=1+1");
}

/// A transcription's CSV is for a spreadsheet: a line a spreadsheet would compute is shown as
/// text, while the JSON keeps it exactly.
#[test]
fn a_csv_cell_a_spreadsheet_would_compute_is_written_as_text() {
    let mut doc = interview();
    doc.parts[0].segments[0].text = "- Oui, au +223 20 22 33 44.".into();
    doc.parts[0].segments[1].text = "=1+1".into();
    let csv = String::from_utf8(run(ExportFormat::Csv, &doc)).unwrap_or_default();
    assert!(
        csv.contains("Speaker 1,\"'- Oui, au +223 20 22 33 44.\""),
        "{csv}"
    );
    assert!(csv.contains("Speaker 2,'=1+1"), "{csv}");
    let json = String::from_utf8(run(ExportFormat::Json, &doc)).unwrap_or_default();
    assert!(json.contains("\"=1+1\""), "{json}");
}

#[test]
fn word_files_are_valid_documents_with_the_text() {
    let bytes = run(ExportFormat::Docx, &letter());
    assert!(bytes.starts_with(b"PK"), "a .docx is a zip archive");
    // The document part is deflated, so check that it unpacks to something holding the text.
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("word/document.xml"));
    assert!(run(ExportFormat::Docx, &contents()).starts_with(b"PK"));
}

#[test]
fn subtitles_need_timestamps() {
    let mut doc = letter();
    doc.task = Task::AudioTranscription;
    let srt = exporter(ExportFormat::Srt).map(|e| e.export(&doc));
    assert!(matches!(srt, Some(Err(_))));
}

/// A translated transcript's subtitles keep the times and speakers of the lines they translate,
/// the corrected passage's words included; its text keeps them too.
#[test]
fn a_translated_transcript_matches_golden_files() {
    let doc = translated_interview();
    check("translated-interview.srt", &run(ExportFormat::Srt, &doc));
    check("translated-interview.vtt", &run(ExportFormat::Vtt, &doc));
    check("translated-interview.txt", &run(ExportFormat::Txt, &doc));
    check("translated-interview.json", &run(ExportFormat::Json, &doc));
    let srt = String::from_utf8(run(ExportFormat::Srt, &doc)).unwrap_or_default();
    assert!(
        srt.contains("00:20:02,000 --> 00:20:06,000\nSpeaker 1: Let us talk about the madrasa."),
        "{srt}"
    );
}

/// The Ouagadougou transcript translated in the 1.10.0-rc2 test pass: its `.srt` ended a line
/// on a cue of 0.27 s holding "old.", where its `.vtt` cut the line in two. A line is now cut
/// into cues of even length, the name's room kept in both files, so they show the same cues and
/// none is too short to read; a line too short for its cues at a second each is cut in fewer.
#[test]
fn subtitles_cut_a_line_into_even_cues_the_same_in_both_files() {
    let line = |start_ms: u64, end_ms: u64, speaker: &str, text: &str| Segment {
        start_ms,
        end_ms,
        speaker: Some(speaker.into()),
        text: text.into(),
    };
    let lines = vec![
        line(
            100,
            7_600,
            "Mariam Kaboré",
            "Hello and thank you for having me. To begin, could you introduce yourself and tell \
             me how you came to work at the newspaper?",
        ),
        line(
            9_100,
            20_400,
            "Adama Sawadogo",
            "Hello. My name is Adama Sawadogo. I joined the newspaper Le Réveil (The Awakening) \
             in 1982 in Ouagadougou as an apprentice typographer. I was 17 years old.",
        ),
        // Said quickly: two cues of 84 characters would last under a second each.
        line(
            21_000,
            22_500,
            "Mariam Kaboré",
            "So you learned the trade at the paper itself, from the older typographers who were \
             already working there before you?",
        ),
    ];
    let mut doc = translated_interview();
    doc.parts = vec![ExportPart {
        piece: kalamu_core::PieceRef::Passage { number: 1 },
        text: kalamu_core::aligned::text(&lines),
        segments: lines,
        ..doc.parts[0].clone()
    }];
    let srt = run(ExportFormat::Srt, &doc);
    let vtt = run(ExportFormat::Vtt, &doc);
    check("even-cues.srt", &srt);
    check("even-cues.vtt", &vtt);
    let cues = crate::cues(&doc.parts[0].segments, true);
    // The same times in both files.
    let srt = String::from_utf8(srt).unwrap_or_default();
    let vtt = String::from_utf8(vtt).unwrap_or_default();
    let times = |text: &str, separator: char| -> Vec<String> {
        text.lines()
            .filter(|l| l.contains(" --> "))
            .map(|l| l.replace(separator, "."))
            .collect()
    };
    assert_eq!(times(&srt, ','), times(&vtt, '.'));
    assert_eq!(times(&srt, ',').len(), cues.len());
    // No sliver: every cue lasts a second at least, the last ends with its line.
    for cue in &cues {
        assert!(cue.end_ms - cue.start_ms >= 1_000, "{cue:?}");
    }
    assert!(srt.contains("I was 17 years old.\n"), "{srt}");
    assert!(!srt.contains("\nold.\n"), "{srt}");
    assert_eq!(cues.iter().filter(|c| c.end_ms == 20_400).count(), 1);
    // The quick line in one cue, whole.
    let quick: Vec<_> = cues.iter().filter(|c| c.start_ms >= 21_000).collect();
    assert_eq!(quick.len(), 1, "{quick:?}");
    // Each cue about as long as the others of its line.
    let adama: Vec<usize> = cues
        .iter()
        .filter(|c| c.start_ms >= 9_100 && c.end_ms <= 20_400)
        .map(|c| c.text.chars().count() + if c.opens_line { 16 } else { 0 })
        .collect();
    let (short, long) = (
        adama.iter().min().copied().unwrap_or_default(),
        adama.iter().max().copied().unwrap_or_default(),
    );
    assert!(long - short < 20, "{adama:?}");
}

#[test]
fn a_translation_without_times_or_whose_correction_changed_its_lines_has_no_subtitles() {
    let refusal = |doc: &ExportDocument, format: ExportFormat| {
        exporter(format)
            .map(|e| e.export(doc))
            .and_then(Result::err)
            .map(|e| e.to_string())
            .unwrap_or_default()
    };
    // Made from a text without times, or before lines were kept: its markers are not read.
    let old = translated();
    for format in [ExportFormat::Srt, ExportFormat::Vtt] {
        assert!(refusal(&old, format).contains("no times"), "{format:?}");
    }
    // A passage whose correction joined two lines: refused, never shifted.
    let mut joined = translated_interview();
    joined.parts[1].segments.clear();
    joined.parts[1].text = "Let us talk about the madrasa. It opened in 1962.".into();
    joined.provenance.lines_changed = vec![2];
    let message = refusal(&joined, ExportFormat::Srt);
    assert!(
        message.contains("passage 2 no longer has one paragraph per line"),
        "{message}"
    );
    joined.provenance.lines_changed = vec![1, 2];
    assert!(refusal(&joined, ExportFormat::Vtt).contains("passages 1, 2 no longer have"));
    // Its other formats are written all the same, the corrected text as left.
    let txt = String::from_utf8(run(ExportFormat::Txt, &joined)).unwrap_or_default();
    assert!(txt.contains("madrasa. It opened in 1962."), "{txt}");
}

#[test]
fn every_format_has_one_exporter() {
    let formats: Vec<_> = all().iter().map(|e| e.format()).collect();
    let mut unique = formats.clone();
    unique.dedup();
    assert_eq!(formats.len(), unique.len());
    assert_eq!(formats.len(), 11);
}

/// A one-page A4 PDF at `path`.
fn a4_pdf(path: &std::path::Path) {
    pdf_of_pages(path, &[(595, 842)]);
}

/// A PDF whose pages have these sizes, in points.
fn pdf_of_pages(path: &std::path::Path, sizes: &[(i64, i64)]) {
    use lopdf::{Document, Object, Stream, dictionary};
    let mut pdf = Document::with_version("1.7");
    let pages = pdf.new_object_id();
    let content = pdf.add_object(Stream::new(dictionary! {}, b"0 0 m 10 10 l S".to_vec()));
    let kids: Vec<Object> = sizes
        .iter()
        .map(|&(w, h)| {
            pdf.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages,
                "Contents" => content,
                "MediaBox" => vec![0.into(), 0.into(), w.into(), h.into()],
            })
            .into()
        })
        .collect();
    pdf.objects.insert(
        pages,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => i64::try_from(sizes.len()).unwrap_or(0),
        }),
    );
    let catalog = pdf.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages });
    pdf.trailer.set("Root", catalog);
    pdf.save(path).unwrap_or_else(|e| panic!("{e}"));
}

/// A real PDF in, the same PDF out with Mistral's blocks as invisible, searchable text.
#[test]
fn searchable_pdf_adds_an_invisible_text_layer_to_the_original() {
    use lopdf::Document;
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let path = dir.path().join("lettre.pdf");
    a4_pdf(&path);

    let mut src = source("lettre.pdf", InputKind::Pdf);
    src.path = path.display().to_string();
    let doc = document(
        Task::TextExtraction,
        src,
        vec![ExportPart {
            piece: kalamu_core::PieceRef::page(1),
            text: "Dakar, le 3 mai 1957".into(),
            segments: vec![],
            corrected: false,
            reviewed: false,
            rows_corrected: Vec::new(),
            fields: vec![],
            regions: vec![kalamu_core::TextRegion::block(
                0.1,
                0.1,
                0.6,
                0.04,
                "Dakar, le 3 mai 1957",
            )],
            failed: None,
            drawn_dpi: None,
            read_at: None,
            corrected_at: None,
            researcher: Default::default(),
        }],
    );
    let bytes = exporter(ExportFormat::SearchablePdf)
        .unwrap_or_else(|| panic!("no exporter"))
        .export(&doc)
        .unwrap_or_else(|e| panic!("{e}"));
    let out = Document::load_mem(&bytes).unwrap_or_else(|e| panic!("{e}"));
    let text = out.extract_text(&[1]).unwrap_or_default();
    assert!(text.contains("Dakar"), "extracted: {text:?}");

    // A model without positions gives no searchable PDF rather than an empty layer.
    let mut plain = doc.clone();
    plain.parts[0].regions.clear();
    assert!(exporter(ExportFormat::SearchablePdf).is_some_and(|e| e.export(&plain).is_err()));
}

/// A word as Tesseract places it, its confidence given or not.
fn word(x: f32, y: f32, width: f32, text: &str, confidence: Option<f32>) -> kalamu_core::TextWord {
    kalamu_core::TextWord {
        x,
        y,
        width,
        height: 0.02,
        text: text.into(),
        confidence,
    }
}

/// ALTO from a PDF: Tesseract's words where it put them (a corrected word without its
/// confidence), and a block placed whole whose lines and words are estimated.
#[test]
fn alto_places_each_word_and_estimates_blocks_placed_whole() {
    use kalamu_core::{TextLine, TextRegion};
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let path = dir.path().join("lettre.pdf");
    a4_pdf(&path);
    let mut src = source("lettre.pdf", InputKind::Pdf);
    src.path = path.display().to_string();
    let placed = TextRegion {
        lines: vec![TextLine {
            x: 0.1,
            y: 0.1,
            width: 0.4,
            height: 0.02,
            words: vec![
                word(0.1, 0.1, 0.12, "Dakar,", Some(0.96)),
                word(0.24, 0.1, 0.04, "le", Some(0.91)),
                word(0.3, 0.1, 0.02, "3", Some(0.88)),
                word(0.34, 0.1, 0.06, "mai", None),
            ],
        }],
        ..TextRegion::block(0.1, 0.1, 0.4, 0.02, "Dakar, le 3 mai")
    };
    let whole = TextRegion::block(
        0.1,
        0.2,
        0.8,
        0.04,
        "Le Gouverneur & les délégués\n<réunis>",
    );
    let doc = document(
        Task::TextExtraction,
        src,
        vec![ExportPart {
            piece: kalamu_core::PieceRef::page(1),
            text: "Dakar, le 3 mai\n\nLe Gouverneur & les délégués\n<réunis>".into(),
            segments: vec![],
            corrected: true,
            reviewed: false,
            rows_corrected: Vec::new(),
            fields: vec![],
            regions: vec![placed.clone(), whole],
            failed: None,
            drawn_dpi: None,
            read_at: None,
            corrected_at: None,
            researcher: Default::default(),
        }],
    );
    check("letter.alto.xml", &run(ExportFormat::Alto, &doc));

    // The right half of a spread is put back on the right of the whole page.
    let mut spread = doc.clone();
    spread.parts[0].piece = kalamu_core::PieceRef::Page {
        number: 1,
        half: Some(kalamu_core::PageHalf::Right),
    };
    spread.parts[0].regions = vec![placed];
    let xml = String::from_utf8(run(ExportFormat::Alto, &spread)).unwrap_or_default();
    // 0.5 + 0.1 / 2 of 2479 pixels.
    assert!(xml.contains("CONTENT=\"Dakar,\" HPOS=\"1363\""), "{xml}");

    // Without positions there is no ALTO.
    let mut plain = doc;
    plain.parts[0].regions.clear();
    assert!(exporter(ExportFormat::Alto).is_some_and(|e| e.export(&plain).is_err()));
}

/// ALTO from Mistral OCR's paragraph, written on one line: the lines its block holds on the
/// page, as the review draws them, not one line the block's height.
#[test]
fn alto_wraps_a_paragraph_placed_whole_into_the_lines_its_block_holds() {
    use kalamu_core::TextRegion;
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let path = dir.path().join("journal.pdf");
    a4_pdf(&path);
    let mut src = source("journal.pdf", InputKind::Pdf);
    src.path = path.display().to_string();
    let text = ["Le cercle de Kita a reçu les notables"; 10].join(" ");
    let block = TextRegion::block(0.1, 0.2, 0.3, 0.4, text.clone());
    let doc = document(
        Task::TextExtraction,
        src,
        vec![ExportPart {
            piece: kalamu_core::PieceRef::page(1),
            text,
            segments: vec![],
            corrected: false,
            reviewed: false,
            rows_corrected: Vec::new(),
            fields: vec![],
            regions: vec![block.clone()],
            failed: None,
            drawn_dpi: None,
            read_at: None,
            corrected_at: None,
            researcher: Default::default(),
        }],
    );
    let xml = String::from_utf8(run(ExportFormat::Alto, &doc)).unwrap_or_default();
    // A4 drawn at 300 dpi: 2479 by 3508 pixels.
    let expected = block.estimated_lines(3508.0 / 2479.0).len();
    assert!(expected > 10, "{expected}");
    assert_eq!(xml.matches("<TextLine ").count(), expected, "{xml}");
    assert_eq!(xml.matches("<String ").count(), 80);
}

/// ALTO from a PDF whose second page, a poster, was drawn below 300 dpi for Tesseract: each
/// page measured at the resolution its result records, and the file says which.
#[test]
fn alto_measures_each_pdf_page_at_the_resolution_it_was_drawn_at() {
    use kalamu_core::TextRegion;
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let path = dir.path().join("affiche.pdf");
    // A4, then 2,000 by 1,000 points, drawn 7,000 pixels wide: 252 dpi.
    pdf_of_pages(&path, &[(595, 842), (2000, 1000)]);
    let mut src = source("affiche.pdf", InputKind::Pdf);
    src.path = path.display().to_string();
    let page = |number: u32, text: &str, drawn_dpi: f32| ExportPart {
        piece: kalamu_core::PieceRef::page(number),
        text: text.into(),
        segments: vec![],
        corrected: false,
        reviewed: false,
        rows_corrected: Vec::new(),
        fields: vec![],
        regions: vec![TextRegion {
            lines: vec![kalamu_core::TextLine {
                x: 0.1,
                y: 0.1,
                width: 0.5,
                height: 0.02,
                words: vec![word(0.1, 0.1, 0.5, text, Some(0.9))],
            }],
            ..TextRegion::block(0.1, 0.1, 0.5, 0.02, text)
        }],
        failed: None,
        drawn_dpi: Some(drawn_dpi),
        read_at: None,
        corrected_at: None,
        researcher: Default::default(),
    };
    let doc = document(
        Task::TextExtraction,
        src,
        vec![page(1, "Dakar", 300.0), page(2, "Bamako", 252.0)],
    );
    let xml = run(ExportFormat::Alto, &doc);
    check("poster.alto.xml", &xml);
    let xml = String::from_utf8(xml).unwrap_or_default();
    assert!(
        xml.contains("PDF pages measured at 300 dpi, page 2 at 252 dpi"),
        "{xml}"
    );
    // 2,000 points at 252 dpi.
    assert!(
        xml.contains("PHYSICAL_IMG_NR=\"2\" WIDTH=\"7000\" HEIGHT=\"3500\""),
        "{xml}"
    );
}

/// ALTO from a document of images: each page measured in its own file.
#[test]
fn alto_of_a_document_of_images_measures_each_page_in_its_file() {
    use kalamu_core::TextRegion;
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut pages = Vec::new();
    for (name, width) in [("p1.png", 1000), ("p2.png", 2000)] {
        let path = dir.path().join(name);
        image::DynamicImage::ImageRgb8(image::RgbImage::new(width, 1500))
            .save(&path)
            .unwrap_or_else(|e| panic!("{e}"));
        pages.push(kalamu_core::PageFile {
            id: name.into(),
            path: path.display().to_string(),
            file_name: name.into(),
            sha256: "00".repeat(32),
            size_bytes: 1,
            modified_ms: None,
            frames: 1,
            label: None,
        });
    }
    let source = SourceFile::image_document("Pages", pages);
    let part = |n: u32| ExportPart {
        piece: kalamu_core::PieceRef::page(n),
        text: "Mot".into(),
        segments: vec![],
        corrected: false,
        reviewed: false,
        rows_corrected: Vec::new(),
        fields: vec![],
        regions: vec![TextRegion::block(0.5, 0.1, 0.2, 0.02, "Mot")],
        failed: None,
        drawn_dpi: None,
        read_at: None,
        corrected_at: None,
        researcher: Default::default(),
    };
    let doc = document(Task::TextExtraction, source, vec![part(1), part(2)]);
    let xml = String::from_utf8(run(ExportFormat::Alto, &doc)).unwrap_or_default();
    assert!(xml.contains("WIDTH=\"1000\""), "{xml}");
    assert!(xml.contains("WIDTH=\"2000\""), "{xml}");
}

#[test]
fn a_table_of_contents_items_are_listed_as_golden_files() {
    let ids = (
        kalamu_core::JobId(uuid_nil()),
        SetupId(uuid_nil()),
        SourceId(uuid_nil()),
    );
    let mut fatwa = kalamu_core::DocumentUnit::new(ids.0, ids.1, ids.2, 1);
    fatwa.id = kalamu_core::UnitId(uuid_nil());
    fatwa.title = "Fatwa sur le jeûne, \"en voyage\"".into();
    fatwa.pages = vec![
        kalamu_core::PieceRef::page(3),
        kalamu_core::PieceRef::Page {
            number: 8,
            half: Some(kalamu_core::PageHalf::Left),
        },
    ];
    fatwa.entry = Some(2);
    fatwa.fields = vec![FieldAnswer {
        key: "authors".into(),
        value: FieldValue::List(vec!["La Rédaction".into(), "A. Diallo".into()]),
        reason: None,
    }];
    let fields = vec![
        Field {
            key: "authors".into(),
            label: "Authors".into(),
            instructions: String::new(),
            kind: FieldKind::List { max: None },
            reason: false,
        },
        Field {
            key: "summary".into(),
            label: "Summary".into(),
            instructions: String::new(),
            kind: FieldKind::Text,
            reason: false,
        },
    ];
    let rows = [units::Row {
        unit: &fatwa,
        pages: "3, 8 (left)".into(),
        file_name: "an-nasr-12 02 Fatwa sur le jeûne, en voyage.pdf".into(),
        original: "an-nasr-12.pdf".into(),
        original_sha256: "0f".repeat(32),
    }];
    check("units.csv", &units::csv(&rows, &fields).unwrap_or_default());
    check_workbook(
        "units.xlsx.json",
        &units::xlsx(&rows, &fields).unwrap_or_default(),
    );
    check("units.json", &units::json(&rows).unwrap_or_default());
}
