//! Descriptions written through an export profile: its columns, headings and order, lists,
//! missing values and line breaks as it says, from what is stored only.

use kalamu_core::{
    DescriptionField as F, DocumentId, Entry, EntryId, ExportProfile, Labels, ProfileColumn,
    ProfileStarter, ProfileValue, SourceDescription, Timestamp,
};

use super::*;

fn column(heading: &str, value: ProfileValue) -> ProfileColumn {
    ProfileColumn {
        heading: heading.into(),
        value,
    }
}

fn omeka() -> ExportProfile {
    ExportProfile {
        id: String::new(),
        name: "Omeka S".into(),
        columns: vec![
            column(
                "dcterms:identifier",
                ProfileValue::Part {
                    field: F::Shelfmark,
                },
            ),
            column("dcterms:title", ProfileValue::Title),
            column(
                "Dépôt",
                ProfileValue::Part {
                    field: F::Repository,
                },
            ),
            column(
                "Dépôt (origine)",
                ProfileValue::From {
                    field: F::Repository,
                },
            ),
            column(
                "dcterms:bibliographicCitation",
                ProfileValue::Part { field: F::Citation },
            ),
            column("dcterms:subject", ProfileValue::Tags),
            column(
                "o:item_set",
                ProfileValue::Fixed {
                    text: "Fonds AOF".into(),
                },
            ),
        ],
        list_separator: "|".into(),
        missing: "n.d.".into(),
        join_lines: false,
        updated_at: Timestamp(0),
    }
}

/// Two documents: the first described, tagged and in a collection; the second bare.
fn described_project(f: &Fixture) -> (ProjectId, Vec<DocumentId>, CollectionId) {
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a"), f.file("b.pdf", "b")], None)
        .unwrap();
    let letters = f.projects.create_collection(p.id, "Lettres").unwrap();
    f.projects
        .add_to_collection(p.id, letters.id, &docs[..1])
        .unwrap();
    f.projects.tag_documents(p.id, &docs[..1], "Dakar").unwrap();
    f.projects
        .tag_documents(p.id, &docs[..1], "Islam, 1957")
        .unwrap();
    f.projects
        .set_source_defaults(
            p.id,
            [(F::Repository, "ANS".to_string())].into_iter().collect(),
        )
        .unwrap();
    let own: SourceDescription = [
        (F::Shelfmark, "19G 1".to_string()),
        (F::Citation, "« Lettre », 1957,\nf. 2".to_string()),
    ]
    .into_iter()
    .collect();
    f.projects.describe_document(p.id, docs[0], own).unwrap();
    (p.id, docs, letters.id)
}

#[test]
fn a_profile_writes_its_columns_under_its_headings() {
    let f = fixture();
    let (p, _, letters) = described_project(&f);
    let csv = csv_body(
        f.projects
            .export_profiled_csv(p, None, &omeka(), &Labels::default())
            .unwrap(),
    );
    assert_eq!(
        csv,
        "dcterms:identifier,dcterms:title,Dépôt,Dépôt (origine),dcterms:bibliographicCitation,\
         dcterms:subject,o:item_set\n\
         19G 1,a,ANS,project,\"« Lettre », 1957,\nf. 2\",\"Dakar|Islam, 1957\",Fonds AOF\n\
         n.d.,b,ANS,project,n.d.,n.d.,Fonds AOF\n"
    );

    // Line breaks made spaces, for tools that read a row per line.
    let mut one_line = omeka();
    one_line.join_lines = true;
    let csv = f
        .projects
        .export_profiled_csv(p, Some(letters), &one_line, &Labels::default())
        .unwrap();
    assert!(csv.contains("\"« Lettre », 1957, f. 2\""), "{csv}");
    assert!(!csv.contains(",b,"), "a collection keeps to its documents");
}

#[test]
fn a_profiles_json_keeps_its_order_lists_and_missing_values() {
    let f = fixture();
    let (p, _, _) = described_project(&f);
    let json = f
        .projects
        .export_profiled_json(p, None, &omeka(), &Labels::default())
        .unwrap();
    let identifier = json.find("\"dcterms:identifier\": \"19G 1\"").unwrap();
    let title = json.find("\"dcterms:title\": \"a\"").unwrap();
    assert!(
        identifier < title,
        "the profile's order, not the alphabet's"
    );
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["format"], "kalamu-profiled-descriptions");
    assert_eq!(value["profile"], "Omeka S");
    assert_eq!(value["columns"][2], "Dépôt");
    let rows = value["rows"].as_array().unwrap();
    assert_eq!(rows[0]["dcterms:subject"][1], "Islam, 1957");
    assert_eq!(
        rows[0]["dcterms:bibliographicCitation"],
        "« Lettre », 1957,\nf. 2"
    );
    assert!(rows[1]["dcterms:identifier"].is_null());
    assert!(rows[1]["dcterms:subject"].is_null());
}

#[test]
fn every_column_writes_what_the_default_csv_does() {
    let f = fixture();
    let (p, _, _) = described_project(&f);
    let every = ExportProfile {
        columns: ProfileStarter::Every.columns(),
        list_separator: "; ".into(),
        missing: String::new(),
        ..omeka()
    };
    assert_eq!(
        f.projects
            .export_profiled_csv(p, None, &every, &Labels::default())
            .unwrap(),
        f.projects.export_descriptions_csv(p, None).unwrap()
    );
}

/// A correction of one piece of a job's only set-up over a source.
fn correct(
    f: &Fixture,
    job: &Job,
    source: SourceId,
    index: u32,
    text: Option<&str>,
    fields: Option<Vec<kalamu_core::FieldAnswer>>,
    reviewed: bool,
) {
    f.store
        .put_correction(&Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: source,
            index,
            text: text.map(str::to_string),
            fields,
            reviewed,
            updated_at: Timestamp::now(),
        })
        .unwrap();
}

#[test]
fn result_columns_read_the_preferred_or_only_result_as_corrected() {
    use kalamu_core::{FieldAnswer, FieldValue};
    let f = fixture();
    let p = f.projects.create("Presse", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a"), f.file("b.pdf", "b")], None)
        .unwrap();
    let sources: Vec<SourceFile> = docs
        .iter()
        .map(|id| f.store.document(*id).unwrap().source)
        .collect();

    // Both documents read once; the second read again by another job.
    let first = f.job(Task::TextExtraction, sources.clone());
    correct(
        &f,
        &first,
        sources[0].id,
        0,
        Some("Page une.\n"),
        None,
        true,
    );
    correct(
        &f,
        &first,
        sources[0].id,
        1,
        Some("Page deux."),
        None,
        false,
    );
    let again = f.job(Task::TextExtraction, vec![sources[1].clone()]);
    let analysis = f.job(Task::Analysis, vec![sources[0].clone()]);
    let answer = |key: &str, value| FieldAnswer {
        key: key.into(),
        value,
        reason: None,
    };
    correct(
        &f,
        &analysis,
        sources[0].id,
        0,
        None,
        Some(vec![
            answer(
                "people",
                FieldValue::List(vec!["Amadou".into(), "Fatou".into()]),
            ),
            answer("summary", FieldValue::Text("Une lettre.".into())),
        ]),
        true,
    );
    for job in [&first, &again, &analysis] {
        f.projects.link_job(p.id, job.id, None).unwrap();
    }

    let profile = ExportProfile {
        columns: vec![
            column("title", ProfileValue::Title),
            column(
                "text",
                ProfileValue::Text {
                    task: Task::TextExtraction,
                },
            ),
            column(
                "status",
                ProfileValue::Status {
                    task: Task::TextExtraction,
                },
            ),
            column(
                "model",
                ProfileValue::Model {
                    task: Task::TextExtraction,
                },
            ),
            column(
                "people",
                ProfileValue::Answer {
                    key: "people".into(),
                },
            ),
            column(
                "summary",
                ProfileValue::Answer {
                    key: "summary".into(),
                },
            ),
            column(
                "people_labelled",
                ProfileValue::Labelled {
                    key: "people".into(),
                },
            ),
            column(
                "analysis_status",
                ProfileValue::Status {
                    task: Task::Analysis,
                },
            ),
        ],
        missing: "—".into(),
        join_lines: false,
        ..omeka()
    };
    let calls_before = f.store.pieces(first.id).unwrap();
    // The researcher's dictionary: Fatou is Fatou Sow; an Amadou among places is not this one.
    let entry = |namespace: &str, label: &str, form: &str| Entry {
        id: EntryId::new(),
        project_id: p.id,
        namespace: namespace.into(),
        label: label.into(),
        forms: vec![form.into()],
        reviewed: true,
        created_at: Timestamp(0),
        updated_at: Timestamp(0),
    };
    let labels = Labels::of(&[
        entry("people", "Fatou Sow", "Fatou"),
        entry("places", "Amadou (village)", "Amadou"),
    ]);
    let json: serde_json::Value = serde_json::from_str(
        &f.projects
            .export_profiled_json(p.id, None, &profile, &labels)
            .unwrap(),
    )
    .unwrap();
    let a = &json["rows"][0];
    assert_eq!(a["text"], "Page une.\n\nPage deux.");
    assert_eq!(a["status"], "partly_reviewed");
    assert_eq!(a["model"], "gemini-3.8-flash");
    assert_eq!(a["people"], serde_json::json!(["Amadou", "Fatou"]));
    assert_eq!(a["summary"], "Une lettre.");
    assert_eq!(
        a["people_labelled"],
        serde_json::json!(["Amadou", "Fatou Sow"]),
        "the answer itself is unchanged; only this column writes the labels"
    );
    // The fixture marks the first of two pieces reviewed.
    assert_eq!(a["analysis_status"], "partly_reviewed");
    // Two readings of b and none preferred: the app does not choose.
    let b = &json["rows"][1];
    assert!(b["status"].is_null() && b["model"].is_null());
    assert!(b["people"].is_null());

    // Once the researcher prefers one, it is read.
    f.projects
        .set_preferred(
            p.id,
            docs[1],
            Task::TextExtraction,
            Some((again.id, again.setups[0].id)),
        )
        .unwrap();
    let csv = csv_body(
        f.projects
            .export_profiled_csv(p.id, None, &profile, &Labels::default())
            .unwrap(),
    );
    let rows: Vec<&str> = csv.lines().collect();
    assert_eq!(
        rows[0],
        "title,text,status,model,people,summary,people_labelled,analysis_status"
    );
    assert!(
        rows.last()
            .unwrap()
            .ends_with(",partly_reviewed,gemini-3.8-flash,—,—,—,—"),
        "{csv}"
    );
    // Nothing was read again.
    assert_eq!(f.store.pieces(first.id).unwrap(), calls_before);
}

#[test]
fn the_fields_a_projects_analyses_asked_are_offered_by_key() {
    use kalamu_core::{AnalysisSchema, Field, FieldKind};
    let f = fixture();
    let p = f.projects.create("Presse", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    let source = f.store.document(docs[0]).unwrap().source;
    let mut job = f.job(Task::Analysis, vec![source]);
    let field = |key: &str, label: &str| Field {
        key: key.into(),
        label: label.into(),
        instructions: String::new(),
        kind: FieldKind::Text,
        reason: false,
    };
    job.setups[0].prompt.schema = Some(AnalysisSchema {
        fields: vec![field("people", "People"), field("date", "Date")],
    });
    // Saved again under its own id: the store keeps a job as first created.
    job.id = kalamu_core::JobId::new();
    f.store.create_job(&job).unwrap();
    f.projects.link_job(p.id, job.id, None).unwrap();
    let fields: Vec<(String, String)> = f
        .projects
        .answer_fields(p.id)
        .unwrap()
        .into_iter()
        .map(|a| (a.key, a.label))
        .collect();
    assert_eq!(
        fields,
        [
            ("date".to_string(), "Date".to_string()),
            ("people".to_string(), "People".to_string())
        ]
    );
}
