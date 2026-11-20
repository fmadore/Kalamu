//! A document read against a research question (`analysis.screening`): its answer, the reason
//! and the passages found word for word in a text read from two scanned pages. The memo names
//! each passage's page, and the quotations table gives it a column of its own; a passage the
//! model gave that is not in the text is left out and said.

use kalamu_core::analysis::cited::{self, CitedText};
use kalamu_core::snapshot::{PartText, TimedPart, build_with_turns};
use kalamu_core::{DerivedFrom, JobId, PieceRef};

use super::*;

fn field(label: &str, kind: FieldKind) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn schema() -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![
            field(
                "Relevance",
                FieldKind::Choice {
                    values: ["Relevant", "Partly relevant", "Not relevant", "Unsure"]
                        .map(String::from)
                        .to_vec(),
                },
            ),
            field("Reason", FieldKind::Text),
            field(
                "Passages",
                FieldKind::Cited {
                    fields: vec![],
                    max: Some(3),
                },
            ),
        ],
    };
    schema.assign_keys();
    schema
}

fn screened() -> ExportDocument {
    let schema = schema();
    let (one, two) = (PieceRef::page(1), PieceRef::page(2));
    let part = |piece, text| TimedPart {
        text: PartText {
            piece,
            text,
            reviewed: false,
            corrected: false,
            failed: false,
        },
        segments: &[],
    };
    let built = build_with_turns(
        Task::TextExtraction,
        &[
            part(
                &one,
                "Le conseil municipal s’est réuni le 3 mars.\n\nIl a voté le budget du marché.",
            ),
            part(
                &two,
                "Les commerçantes du marché demandent un abri contre la pluie.",
            ),
        ],
    );
    let mut fields = schema
        .read_answer(&serde_json::json!({
            "relevance": "Relevant",
            "reason": "Le procès-verbal rapporte la demande des commerçantes du marché.",
            "passages": [
                {"quotes": [{"turn": 3, "words": "Les commerçantes du marché demandent un abri contre la pluie."}]},
                {"quotes": [{"turn": 2, "words": "Il a voté le budget du marché."}]},
                {"quotes": [{"turn": 2, "words": "Les femmes du marché ont manifesté."}]},
            ],
        }))
        .unwrap_or_else(|e| panic!("{e}"));
    cited::cite(
        &schema.fields,
        &mut fields,
        CitedText {
            text: &built.text,
            parts: &built.parts,
            turns: &built.turns,
        },
    );
    let mut source = source("conseil-1957.txt", InputKind::Text);
    source.derived_from = Some(DerivedFrom {
        snapshot_id: "5f0c7a2e-0000-4000-8000-000000000002".into(),
        job_id: JobId(uuid_nil()),
        setup_id: SetupId(uuid_nil()),
        source_id: SourceId(uuid_nil()),
        original_path: r"C:\archives\conseil-1957.pdf".into(),
        unit: None,
        timed: false,
    });
    let mut doc = document(
        Task::Analysis,
        source,
        vec![ExportPart {
            piece: PieceRef::Whole,
            text: schema.render(&fields),
            segments: vec![],
            corrected: false,
            reviewed: false,
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
    doc.setup.prompt.preset_id = Some("analysis.screening".into());
    doc.setup.prompt.preset_name = "Does it speak of my question?".into();
    doc.setup.prompt.schema = Some(schema.clone());
    doc.provenance.prompt.schema = Some(schema);
    doc
}

#[test]
fn a_screened_document_matches_golden_files() {
    let doc = screened();
    check("screening-analysis.md", &run(ExportFormat::Markdown, &doc));
    let csv = exporter(ExportFormat::Csv)
        .map(|e| e.export_files(&doc))
        .and_then(Result::ok)
        .unwrap_or_default();
    let suffixes: Vec<&str> = csv.iter().map(|f| f.suffix.as_str()).collect();
    assert_eq!(suffixes, ["csv", "quotations.csv"]);
    check("screening-analysis.csv", &csv[0].bytes);
    check("screening-analysis.quotations.csv", &csv[1].bytes);
}

#[test]
fn each_passage_names_its_page_and_one_not_in_the_text_is_left_out() {
    let doc = screened();
    let memo = String::from_utf8(run(ExportFormat::Markdown, &doc)).unwrap_or_default();
    assert!(
        memo.contains(
            "> Les commerçantes du marché demandent un abri contre la pluie.\n>\n> — page 2"
        ),
        "{memo}"
    );
    assert!(!memo.contains("ont manifesté"));
    assert!(memo.contains("1 quotation could not be found in the text"));
}
