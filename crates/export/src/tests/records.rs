//! Records (#171): one row per entry with its file, page and place on the page, from the
//! corrected rows; a page not read marked in its place; cells a spreadsheet would compute
//! written as text in the CSV and exactly in the JSON.

use kalamu_core::{PieceRef, records};

use super::*;

fn column(label: &str, kind: FieldKind) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn schema(columns: Vec<Field>) -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![column("Entries", FieldKind::Entries { fields: columns })],
    };
    schema.assign_keys();
    schema
}

fn row(schema: &AnalysisSchema, cells: &[FieldValue]) -> Vec<FieldAnswer> {
    records::columns(schema)
        .iter()
        .zip(cells)
        .map(|(c, v)| FieldAnswer {
            key: c.key.clone(),
            value: v.clone(),
            reason: None,
        })
        .collect()
}

fn t(s: &str) -> FieldValue {
    FieldValue::Text(s.into())
}

fn page(
    schema: &AnalysisSchema,
    number: u32,
    rows: Vec<Vec<FieldAnswer>>,
    corrected: bool,
) -> ExportPart {
    let fields = vec![FieldAnswer {
        key: "entries".into(),
        value: FieldValue::Entries(rows),
        reason: None,
    }];
    ExportPart {
        piece: PieceRef::page(number),
        text: schema.render(&fields),
        segments: vec![],
        corrected,
        reviewed: corrected,
        rows_corrected: Vec::new(),
        fields,
        regions: vec![],
        failed: None,
        drawn_dpi: None,
        read_at: None,
        corrected_at: None,
        researcher: Default::default(),
    }
}

fn records_document(name: &str, schema: AnalysisSchema, parts: Vec<ExportPart>) -> ExportDocument {
    let mut doc = document(Task::Analysis, source(name, InputKind::Pdf), parts);
    doc.setup.prompt.schema = Some(schema.clone());
    doc.provenance.prompt.schema = Some(schema);
    doc.provenance.corrected = vec![3];
    doc.provenance.reviewed = vec![3];
    doc.provenance.failed = vec![2];
    doc
}

/// A two-column directory page read down the left column, then the right; a page cut off at
/// the model's limit; a page the researcher corrected, a row added and one removed, with cells
/// a spreadsheet would compute.
fn directory() -> ExportDocument {
    let schema = schema(vec![
        column("Name", FieldKind::Text),
        column("Occupation", FieldKind::Text),
        column("Address", FieldKind::Text),
    ]);
    let r = |cells: [&str; 3]| row(&schema, &cells.map(t));
    let first = page(
        &schema,
        1,
        vec![
            // Left column.
            r(["Abadie (J.)", "boulanger", "12, r. Carnot"]),
            r(["Achard (Vve)", "modiste", "3, pl. du Marché"]),
            // Right column.
            r(["Bonnet (L.)", "cordonnier", "\""]),
            r(["Bouvier frères", "négociants", "do."]),
        ],
        false,
    );
    let mut cut = page(&schema, 2, vec![], false);
    cut.fields = vec![];
    cut.text = String::new();
    cut.failed = Some("The answer was cut off at the model's output limit".into());
    let mut corrected = page(
        &schema,
        3,
        vec![
            r(["=Durand", "-", "+33 4 75 00 00"]),
            r(["@Dupont", "tailleur", "8, r. Neuve"]),
        ],
        true,
    );
    // The first row is the one the researcher added; the second is the model's.
    corrected.rows_corrected = vec![true, false];
    records_document(
        "annuaire-1902.pdf",
        schema.clone(),
        vec![first, cut, corrected],
    )
}

/// A register page with ditto marks, a choice and a list of the people each entry names.
fn register() -> ExportDocument {
    let schema = schema(vec![
        column("Date", FieldKind::Text),
        column(
            "Kind of entry",
            FieldKind::Choice {
                values: vec!["Baptism".into(), "Burial".into()],
            },
        ),
        column("Name", FieldKind::Text),
        column("People named", FieldKind::List { max: None }),
    ]);
    let names = |n: &[&str]| FieldValue::List(n.iter().map(|s| (*s).to_string()).collect());
    let rows = vec![
        row(
            &schema,
            &[
                t("3 mai 1857"),
                t("Baptism"),
                t("Marie Coulibaly"),
                names(&["Jean Coulibaly", "Awa Keïta"]),
            ],
        ),
        row(
            &schema,
            &[t("〃"), t("Burial"), t("Pierre Sow"), names(&["le curé"])],
        ),
        row(&schema, &[t("do."), t("Baptism"), t("Anne"), names(&[])]),
    ];
    let mut doc = records_document(
        "registre.pdf",
        schema.clone(),
        vec![page(&schema, 1, rows, false)],
    );
    doc.provenance.failed = vec![];
    doc.provenance.corrected = vec![];
    doc.provenance.reviewed = vec![];
    doc
}

#[test]
fn records_are_written_one_row_per_entry_as_golden_files() {
    check(
        "records-directory.csv",
        &run(ExportFormat::Csv, &directory()),
    );
    check(
        "records-directory.json",
        &run(ExportFormat::Json, &directory()),
    );
    check("records-register.csv", &run(ExportFormat::Csv, &register()));
    check(
        "records-register.json",
        &run(ExportFormat::Json, &register()),
    );
    super::check_workbook(
        "records-directory.xlsx.json",
        &run(ExportFormat::Xlsx, &directory()),
    );
    super::check_workbook(
        "records-register.xlsx.json",
        &run(ExportFormat::Xlsx, &register()),
    );
}

/// A page with one corrected row exports one `corrected` row, in the CSV, the workbook and the
/// JSON; `reviewed` stays the page's.
#[test]
fn a_page_with_one_corrected_row_exports_one_corrected_row() {
    let schema = schema(vec![
        column("Name", FieldKind::Text),
        column("Town", FieldKind::Text),
    ]);
    let r = |cells: [&str; 2]| row(&schema, &cells.map(t));
    let mut corrected = page(
        &schema,
        1,
        vec![
            r(["Diallo", "Ségou"]),
            r(["Traoré", "Kayes"]),
            r(["Sow", "Mopti"]),
        ],
        true,
    );
    corrected.rows_corrected = vec![false, true, false];
    let doc = records_document("liste.pdf", schema.clone(), vec![corrected]);

    let csv = String::from_utf8(run(ExportFormat::Csv, &doc)).unwrap_or_default();
    let lines: Vec<&str> = csv.lines().skip(1).collect();
    assert_eq!(
        lines,
        [
            "liste.pdf,1,1,Diallo,Ségou,false,true,",
            "liste.pdf,1,2,Traoré,Kayes,true,true,",
            "liste.pdf,1,3,Sow,Mopti,false,true,",
        ]
    );

    let cells = crate::table::tests::workbook_cells(&run(ExportFormat::Xlsx, &doc));
    let marks: Vec<&str> = cells[1..].iter().map(|r| r[5].as_str()).collect();
    assert_eq!(marks, ["false", "true", "false"]);

    let json: serde_json::Value =
        serde_json::from_slice(&run(ExportFormat::Json, &doc)).unwrap_or_default();
    let rows = json["rows"].as_array().cloned().unwrap_or_default();
    let marks: Vec<bool> = rows
        .iter()
        .filter_map(|r| r["corrected"].as_bool())
        .collect();
    assert_eq!(marks, [false, true, false]);
    assert!(rows.iter().all(|r| r["reviewed"] == true));
}

#[test]
fn a_records_cell_a_spreadsheet_would_compute_is_text_in_the_csv_exact_in_the_json() {
    let csv = String::from_utf8(run(ExportFormat::Csv, &directory())).unwrap_or_default();
    for guarded in ["'=Durand", "'-", "'+33 4 75 00 00", "'@Dupont"] {
        assert!(csv.contains(guarded), "{guarded}: {csv}");
    }
    let json = String::from_utf8(run(ExportFormat::Json, &directory())).unwrap_or_default();
    assert!(json.contains("\"=Durand\""), "{json}");
    // The page cut off is marked, never a page with no entries.
    assert!(csv.contains("annuaire-1902.pdf,2,,,,,false,false,[Not read: The answer was cut off"));
}

/// A list in a right-to-left script: its cells as written, in the order read.
#[test]
fn a_right_to_left_list_is_written_as_read() {
    let schema = schema(vec![
        column("Name", FieldKind::Text),
        column("Town", FieldKind::Text),
    ]);
    let rows = vec![
        row(&schema, &[t("عبد الله بن محمد"), t("تمبكتو")]),
        row(&schema, &[t("أحمد بابا"), t("جني")]),
    ];
    let doc = records_document(
        "liste.pdf",
        schema.clone(),
        vec![page(&schema, 1, rows, false)],
    );
    let csv = String::from_utf8(run(ExportFormat::Csv, &doc)).unwrap_or_default();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(
        lines[1],
        "liste.pdf,1,1,عبد الله بن محمد,تمبكتو,false,false,"
    );
    assert_eq!(lines[2], "liste.pdf,1,2,أحمد بابا,جني,false,false,");
}
