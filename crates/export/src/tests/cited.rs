//! An interview analysis whose fields cite the transcript: the Markdown memo with each
//! quotation's speaker and time, the JSON result with anchors, turns and the snapshot read,
//! one row per quotation in `.quotations.csv` beside the analysis's own CSV, and the text and
//! Word files from the answers set out as text. The fixture is an interview in French with two
//! speakers, a phrase said twice, a quotation the model gave that is not in the transcript, and
//! a theme the researcher removed.

use kalamu_core::analysis::cited::{self, CitedEdit, CitedText};
use kalamu_core::snapshot::{PartText, TimedPart, build_with_turns};
use kalamu_core::{DerivedFrom, JobId, PieceRef};

use super::*;

const TRANSCRIPT: &str = "[00:00:05] Interviewer: Comment avez-vous commencé au port ?\n\
[00:00:12] Awa: J’ai commencé en 1998. Au début, on ne savait pas.\n\
[00:01:10] Interviewer: Et ensuite ?\n\
[00:01:15] Awa: Ensuite, «\u{a0}on ne savait pas\u{a0}» est devenu notre devise.";

fn field(label: &str, instructions: &str, kind: FieldKind) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: instructions.into(),
        kind,
        reason: false,
    }
}

fn schema() -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![
            field("Summary", "", FieldKind::Text),
            field(
                "Themes",
                "",
                FieldKind::Cited {
                    fields: vec![
                        field("Name", "", FieldKind::Text),
                        field("Description", "", FieldKind::Text),
                        field(
                            "Origin",
                            "",
                            FieldKind::Choice {
                                values: vec![
                                    "The researcher's guide".into(),
                                    "The interview".into(),
                                ],
                            },
                        ),
                    ],
                    max: Some(8),
                },
            ),
            field("Guide themes not found", "", FieldKind::List { max: None }),
            field("Keywords", "", FieldKind::List { max: Some(12) }),
            field(
                "Quotations",
                "",
                FieldKind::Cited {
                    fields: vec![field("Why it matters", "", FieldKind::Text)],
                    max: None,
                },
            ),
        ],
    };
    schema.assign_keys();
    schema
}

/// The model's answer checked against the transcript, then the researcher's removal of the
/// third theme: the model's fields and the corrected ones.
fn answers(schema: &AnalysisSchema) -> (Vec<FieldAnswer>, Vec<FieldAnswer>) {
    let piece = PieceRef::Whole;
    let built = build_with_turns(
        Task::AudioTranscription,
        &[TimedPart {
            text: PartText {
                piece: &piece,
                text: TRANSCRIPT,
                reviewed: true,
                corrected: true,
                failed: false,
            },
            segments: &[],
        }],
    );
    let mut model = schema
        .read_answer(&serde_json::json!({
            "summary": "Awa raconte ses débuts au port de pêche en 1998 et une phrase devenue la devise de son équipe.",
            "themes": [
                {"name": "Les débuts", "description": "Elle commence au port sans savoir ce qui l’attend.",
                 "origin": "The researcher's guide",
                 "quotes": [{"turn": 2, "words": "J'ai commencé en 1998."},
                            {"turn": 2, "words": "On nous a tout appris sur place."}]},
                {"name": "Une devise", "description": "Une phrase reprise par toute l’équipe.",
                 "origin": "The interview",
                 "quotes": [{"turn": 4, "words": "« on ne savait pas » est devenu notre devise"},
                            {"turn": 2, "words": "Au début, on ne savait pas."}]},
                {"name": "Le port", "description": "Le lieu de travail.",
                 "origin": "The interview",
                 "quotes": [{"turn": 1, "words": "Comment avez-vous commencé au port ?"}]},
            ],
            "guide_themes_not_found": ["La famille"],
            "keywords": ["port de pêche", "1998", "devise"],
            "quotations": [
                {"why_it_matters": "La phrase résume une expérience collective.",
                 "quotes": [{"turn": 4, "words": "est devenu notre devise."}]},
            ],
        }))
        .unwrap_or_else(|e| panic!("{e}"));
    cited::cite(
        &schema.fields,
        &mut model,
        CitedText {
            text: &built.text,
            parts: &built.parts,
            turns: &built.turns,
        },
    );
    let removed = cited::edit(
        &schema.fields,
        &model,
        "themes",
        &CitedEdit::RemoveEntry { entry: 2 },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    // And a passage the researcher selected, added to the first theme as their own quotation.
    let words = "Au début, on ne savait pas.";
    let at = built.text.find(words).unwrap_or_default();
    let start = built.text[..at].chars().count() as u32;
    let corrected = cited::edit_in(
        &schema.fields,
        &removed,
        "themes",
        &CitedEdit::AddQuotation {
            entry: 0,
            start,
            end: start + words.chars().count() as u32,
        },
        Some(CitedText {
            text: &built.text,
            parts: &built.parts,
            turns: &built.turns,
        }),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    (model, corrected)
}

fn cited_interview() -> ExportDocument {
    let schema = schema();
    let (model, corrected) = answers(&schema);
    let mut source = source("entretien-awa.txt", InputKind::Text);
    source.derived_from = Some(DerivedFrom {
        snapshot_id: "5f0c7a2e-0000-4000-8000-000000000001".into(),
        job_id: JobId(uuid_nil()),
        setup_id: SetupId(uuid_nil()),
        source_id: SourceId(uuid_nil()),
        original_path: r"C:\terrain\entretien-awa.m4a".into(),
        unit: None,
        timed: false,
    });
    let mut doc = document(
        Task::Analysis,
        source,
        vec![ExportPart {
            piece: PieceRef::Whole,
            text: schema.render(&corrected),
            segments: vec![],
            corrected: true,
            reviewed: true,
            rows_corrected: Vec::new(),
            fields: corrected,
            regions: vec![],
            failed: None,
            drawn_dpi: None,
            read_at: None,
            corrected_at: None,
            researcher: Default::default(),
        }],
    );
    doc.setup.prompt.preset_id = Some("analysis.interview".into());
    doc.setup.prompt.preset_name = "Interview: themes and quotations".into();
    doc.setup.prompt.schema = Some(schema.clone());
    doc.provenance.prompt.schema = Some(schema.clone());
    doc.provenance.corrected = vec![1];
    doc.provenance.cited = cited::counts(&schema.fields, &model)
        .into_iter()
        .map(
            |(field, checked, left_out, dropped)| kalamu_core::provenance::ProvenanceCited {
                part: 1,
                field,
                checked,
                left_out,
                entries_left_out: dropped,
            },
        )
        .collect();
    doc
}

fn files(format: ExportFormat, doc: &ExportDocument) -> Vec<kalamu_core::ExportFile> {
    exporter(format)
        .map(|e| e.export_files(doc))
        .and_then(Result::ok)
        .unwrap_or_default()
}

#[test]
fn an_interview_analysis_matches_golden_files() {
    let doc = cited_interview();
    check("interview-analysis.md", &run(ExportFormat::Markdown, &doc));
    check("interview-analysis.txt", &run(ExportFormat::Txt, &doc));
    check("interview-analysis.json", &run(ExportFormat::Json, &doc));
    let csv = files(ExportFormat::Csv, &doc);
    let suffixes: Vec<&str> = csv.iter().map(|f| f.suffix.as_str()).collect();
    assert_eq!(suffixes, ["csv", "quotations.csv"]);
    check("interview-analysis.csv", &csv[0].bytes);
    check("interview-analysis.quotations.csv", &csv[1].bytes);
    // The workbook writes the same two tables.
    let xlsx = files(ExportFormat::Xlsx, &doc);
    let suffixes: Vec<&str> = xlsx.iter().map(|f| f.suffix.as_str()).collect();
    assert_eq!(suffixes, ["xlsx", "quotations.xlsx"]);
    super::check_workbook("interview-analysis.xlsx.json", &xlsx[0].bytes);
    super::check_workbook("interview-analysis.quotations.xlsx.json", &xlsx[1].bytes);
    assert!(run(ExportFormat::Docx, &doc).starts_with(b"PK"));
}

#[test]
fn the_files_hold_only_what_the_transcript_says_and_the_researcher_kept() {
    let doc = cited_interview();
    let memo = String::from_utf8(run(ExportFormat::Markdown, &doc)).unwrap_or_default();
    // The transcript's own words, apostrophe and guillemets, with speaker and time.
    assert!(
        memo.contains("> J’ai commencé en 1998.\n>\n> — Awa, [00:00:12]"),
        "{memo}"
    );
    assert!(memo.contains("«\u{a0}on ne savait pas\u{a0}» est devenu notre devise"));
    // Neither the words found nowhere, nor the theme the researcher removed.
    assert!(!memo.contains("tout appris"));
    assert!(!memo.contains("Le port"));
    assert!(memo.contains("*1 quotation could not be found in the text and was left out.*"));
    // Where each theme comes from, and the guide's theme the interview does not take up.
    assert!(memo.contains("### Les débuts\n\nElle commence au port sans savoir ce qui l’attend.\n\n**Origin:** The researcher's guide"), "{memo}");
    assert!(
        memo.contains("## Guide themes not found\n\nLa famille"),
        "{memo}"
    );

    let rows =
        String::from_utf8(files(ExportFormat::Csv, &doc)[1].bytes.clone()).unwrap_or_default();
    assert_eq!(
        rows.lines().count(),
        1 + 5,
        "a heading and one row per quotation kept"
    );
    // The quotation the researcher added, marked as theirs in every file.
    assert!(
        memo.contains(
            "> Au début, on ne savait pas.\n>\n> — Awa, [00:00:12] · added by the researcher"
        ),
        "{memo}"
    );
    assert!(
        rows.contains(",\"Au début, on ne savait pas.\",2,,researcher"),
        "{rows}"
    );
    assert_eq!(rows.matches(",model").count(), 4, "{rows}");
    let json: serde_json::Value =
        serde_json::from_slice(&run(ExportFormat::Json, &doc)).unwrap_or_default();
    assert_eq!(
        json["snapshot"]["id"],
        "5f0c7a2e-0000-4000-8000-000000000001"
    );
    let devise = &json["parts"][0]["fields"]["themes"]["entries"][1]["evidence"];
    assert_eq!(devise[0]["turn"], 4);
    assert_eq!(devise[0]["start_ms"], 75_000);
    assert_eq!(
        devise[1]["turn"], 2,
        "the phrase said twice, where each was named"
    );
    assert_eq!(json["provenance"]["cited"][0]["left_out"], 1);
}

/// Without a guide, the model lists no guide theme left out: the memo prints no heading with
/// nothing under it (the 1.10.0-rc2 test pass found an empty "Guide themes not found"), and
/// keeps the other fields in their order.
#[test]
fn a_field_with_nothing_to_say_has_no_heading_in_the_memo() {
    let mut doc = cited_interview();
    for answer in &mut doc.parts[0].fields {
        if answer.key == "guide_themes_not_found" {
            answer.value = FieldValue::List(Vec::new());
        }
    }
    let memo = run(ExportFormat::Markdown, &doc);
    check("interview-analysis-no-guide.md", &memo);
    let memo = String::from_utf8(memo).unwrap_or_default();
    assert!(!memo.contains("Guide themes not found"), "{memo}");
    assert!(
        memo.contains("left out.*\n\n## Keywords\n\nport de pêche"),
        "{memo}"
    );
}

#[test]
fn an_analysis_without_cited_fields_writes_one_csv_and_no_snapshot() {
    let doc = analysis();
    let csv = files(ExportFormat::Csv, &doc);
    assert_eq!(csv.len(), 1);
    let json: serde_json::Value =
        serde_json::from_slice(&run(ExportFormat::Json, &doc)).unwrap_or_default();
    assert!(json.get("snapshot").is_none());
}
