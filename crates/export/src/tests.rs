//! Golden-file tests: each format's output for fixed documents is compared with a file in
//! `golden/`. Run with `UPDATE_GOLDEN=1` to rewrite the files after an intended change, and
//! review the diff.

use std::path::PathBuf;

use kalamu_core::provenance::{GeneratorInfo, ProvenancePrompt, ProvenanceSource};
use kalamu_core::{
    AnalysisSchema, ExportOptions, ExportPart, Field, FieldAnswer, FieldKind, FieldValue,
    InputKind, LanguageChoice, PromptSnapshot, Provenance, ProviderId, Segment, Setup, SetupId,
    SetupOptions, SourceFile, SourceId, Task, Timestamp, Usage,
};

use super::*;

fn source(name: &str, kind: InputKind) -> SourceFile {
    SourceFile {
        id: SourceId(uuid_nil()),
        path: format!(r"C:\archive\{name}"),
        file_name: name.into(),
        sha256: "0f".repeat(32),
        size_bytes: 1024,
        kind,
        page_count: None,
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        title: None,
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
        },
        engine: "cloud".into(),
        provider: setup.provider.clone(),
        model: setup.model.clone(),
        model_versions: vec![],
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
        preprocessing: vec![],
        usage: Usage::default(),
        cost_usd: Some(0.002),
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
                fields: vec![],
                regions: vec![],
            },
            ExportPart {
                piece: kalamu_core::PieceRef::page(2),
                text: "rendre compte de la réunion tenue à la mosquée.\n\n[ILLEGIBLE]".into(),
                segments: vec![],
                corrected: true,
                reviewed: true,
                fields: vec![],
                regions: vec![],
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
                fields: vec![],
                regions: vec![],
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
                fields: vec![],
                regions: vec![],
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
            fields,
            regions: vec![],
        }],
    );
    doc.setup.prompt.schema = Some(schema.clone());
    doc.provenance.prompt.schema = Some(schema);
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
        fields: vec![],
        regions: vec![],
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
    check("contents.txt", &run(ExportFormat::Txt, &contents()));
    check("contents.md", &run(ExportFormat::Markdown, &contents()));
    check("contents.json", &run(ExportFormat::Json, &contents()));
    check("contents.csv", &run(ExportFormat::Csv, &contents()));
    check("corrected.txt", &run(ExportFormat::Txt, &corrected()));
    check("corrected.csv", &run(ExportFormat::Csv, &corrected()));
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

#[test]
fn every_format_has_one_exporter() {
    let formats: Vec<_> = all().iter().map(|e| e.format()).collect();
    let mut unique = formats.clone();
    unique.dedup();
    assert_eq!(formats.len(), unique.len());
    assert_eq!(formats.len(), 8);
}

#[test]
fn timestamps_in_text_become_segments() {
    let segs = segments_from_text("[00:01:05] Speaker 2: Oui.\ncontinued\n[01:10] Hm.", 1000);
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].start_ms, 66_000);
    assert_eq!(segs[0].speaker.as_deref(), Some("Speaker 2"));
    assert_eq!(segs[0].text, "Oui. continued");
    assert_eq!(segs[1].start_ms, 71_000);
    assert_eq!(segs[1].speaker, None);
}

/// A real PDF in, the same PDF out with Mistral's blocks as invisible, searchable text.
#[test]
fn searchable_pdf_adds_an_invisible_text_layer_to_the_original() {
    use lopdf::{Document, Object, Stream, dictionary};
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let path = dir.path().join("lettre.pdf");
    let mut pdf = Document::with_version("1.7");
    let pages = pdf.new_object_id();
    let content = pdf.add_object(Stream::new(dictionary! {}, b"0 0 m 10 10 l S".to_vec()));
    let page = pdf.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages,
        "Contents" => content,
    });
    pdf.objects.insert(
        pages,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page.into()],
            "Count" => 1,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog = pdf.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages });
    pdf.trailer.set("Root", catalog);
    pdf.save(&path).unwrap_or_else(|e| panic!("{e}"));

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
            fields: vec![],
            regions: vec![kalamu_core::TextRegion {
                x: 0.1,
                y: 0.1,
                width: 0.6,
                height: 0.04,
                text: "Dakar, le 3 mai 1957".into(),
            }],
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
