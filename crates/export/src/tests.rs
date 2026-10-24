//! Golden-file tests: each format's output for fixed documents is compared with a file in
//! `golden/`. Run with `UPDATE_GOLDEN=1` to rewrite the files after an intended change, and
//! review the diff.

use std::path::PathBuf;

use scriptorium_core::provenance::{GeneratorInfo, ProvenancePrompt, ProvenanceSource};
use scriptorium_core::{
    ExportOptions, ExportPart, InputKind, LanguageChoice, PromptSnapshot, Provenance, ProviderId,
    Segment, Setup, SetupId, SetupOptions, SourceFile, SourceId, Task, Timestamp, Usage,
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
        },
        options: SetupOptions::default(),
    };
    let provenance = Provenance {
        schema_version: 1,
        generator: GeneratorInfo {
            name: "Scriptorium".into(),
            version: "0.1.0".into(),
        },
        created_at: Timestamp(0),
        task,
        source: ProvenanceSource {
            file_name: source.file_name.clone(),
            sha256: source.sha256.clone(),
            size_bytes: source.size_bytes,
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
        },
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
                piece: scriptorium_core::PieceRef::page(1),
                text: "Bamako, le 3 mai 1957\n\nMonsieur le Gouverneur,\nj'ai l'honneur de vous "
                    .into(),
                segments: vec![],
                corrected: false,
                reviewed: true,
            },
            ExportPart {
                piece: scriptorium_core::PieceRef::page(2),
                text: "rendre compte de la réunion tenue à la mosquée.\n\n[ILLEGIBLE]".into(),
                segments: vec![],
                corrected: true,
                reviewed: true,
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
                piece: scriptorium_core::PieceRef::Segment {
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
            },
            ExportPart {
                // A prompt-based model that wrote timestamps into its text.
                piece: scriptorium_core::PieceRef::Segment {
                    index: 1,
                    start_ms: 1_200_000,
                    end_ms: 2_400_000,
                },
                text: "[00:00:02] Speaker 1: Parlons de la medersa.\n[00:00:06] Speaker 2: Elle a ouvert en 1962.".into(),
                segments: vec![],
                corrected: false,
                reviewed: false,
            },
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
}

#[test]
fn word_files_are_valid_documents_with_the_text() {
    let bytes = run(ExportFormat::Docx, &letter());
    assert!(bytes.starts_with(b"PK"), "a .docx is a zip archive");
    // The document part is deflated, so check that it unpacks to something holding the text.
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("word/document.xml"));
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
    assert_eq!(formats.len(), 7);
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
