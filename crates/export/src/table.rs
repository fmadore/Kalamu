//! Tables for spreadsheets ([`Table`]): a project's descriptions, an export profile's columns, the
//! dictionary's forms. One writer for all, so each file opens in a spreadsheet with its accents.

use kalamu_core::{ExportError, Table};

/// The table as CSV: the byte-order mark spreadsheets need to read accents, the headings, then
/// each row.
pub fn table_csv(table: &Table) -> Result<String, ExportError> {
    let err = |e: &dyn std::fmt::Display| ExportError::Write {
        format: "CSV",
        message: e.to_string(),
    };
    let mut out = csv::Writer::from_writer(kalamu_core::export::CSV_BOM.to_vec());
    out.write_record(&table.headings).map_err(|e| err(&e))?;
    for row in &table.rows {
        out.write_record(row).map_err(|e| err(&e))?;
    }
    let bytes = out.into_inner().map_err(|e| err(&e))?;
    String::from_utf8(bytes).map_err(|e| err(&e))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn writes_one_bom_then_quoted_rows() {
        let table = Table {
            headings: vec!["title".into(), "notes".into()],
            rows: vec![vec![
                "Lettre à l'imam".into(),
                "deux\nlignes, et virgule".into(),
            ]],
        };
        let csv = table_csv(&table).unwrap();
        assert!(csv.starts_with('\u{feff}'));
        assert_eq!(csv.matches('\u{feff}').count(), 1);
        assert!(csv.contains("\"deux\nlignes, et virgule\""));
        assert!(csv.contains("Lettre à l'imam"));
    }
}
