//! Descriptions written through an export profile: its columns, headings and order, lists,
//! missing values and line breaks as it says, from what is stored only.

use kalamu_core::{
    DescriptionField as F, DocumentId, ExportProfile, ProfileColumn, ProfileStarter, ProfileValue,
    SourceDescription, Timestamp,
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
    let csv = f.projects.export_profiled_csv(p, None, &omeka()).unwrap();
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
        .export_profiled_csv(p, Some(letters), &one_line)
        .unwrap();
    assert!(csv.contains("\"« Lettre », 1957, f. 2\""), "{csv}");
    assert!(!csv.contains(",b,"), "a collection keeps to its documents");
}

#[test]
fn a_profiles_json_keeps_its_order_lists_and_missing_values() {
    let f = fixture();
    let (p, _, _) = described_project(&f);
    let json = f.projects.export_profiled_json(p, None, &omeka()).unwrap();
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
        f.projects.export_profiled_csv(p, None, &every).unwrap(),
        f.projects.export_descriptions_csv(p, None).unwrap()
    );
}
