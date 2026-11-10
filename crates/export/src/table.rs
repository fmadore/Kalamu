//! Tables for spreadsheets ([`Table`]): a project's descriptions, an export profile's columns, the
//! dictionary's forms. One writer for all, so each file opens in a spreadsheet with its accents.

use std::borrow::Cow;

use kalamu_core::{CsvCells, ExportError, Table};

/// The table as CSV: the byte-order mark spreadsheets need to read accents, the headings, then
/// each row, each cell guarded or not as the table says ([`CsvCells`]).
pub fn table_csv(table: &Table) -> Result<String, ExportError> {
    let err = |e: &dyn std::fmt::Display| ExportError::Write {
        format: "CSV",
        message: e.to_string(),
    };
    let mut out = csv::Writer::from_writer(kalamu_core::export::CSV_BOM.to_vec());
    let cells = table.cells;
    out.write_record(table.headings.iter().map(|h| written(h, cells)))
        .map_err(|e| err(&e))?;
    for row in &table.rows {
        out.write_record(row.iter().map(|c| written(c, cells)))
            .map_err(|e| err(&e))?;
    }
    let bytes = out.into_inner().map_err(|e| err(&e))?;
    String::from_utf8(bytes).map_err(|e| err(&e))
}

/// One cell's bytes, guarded if the table asks for it.
fn written(text: &str, cells: CsvCells) -> Cow<'_, [u8]> {
    match (cells, guarded(text)) {
        (CsvCells::Guarded, Cow::Owned(shown)) => Cow::Owned(shown.into_bytes()),
        _ => Cow::Borrowed(text.as_bytes()),
    }
}

/// A cell as a spreadsheet will show it: after an apostrophe when it begins with what a
/// spreadsheet takes for the start of a formula, unless it is a plain number.
///
/// An apostrophe rather than a tab: it is seen, so a researcher who copies the cell knows what
/// to take off, and a program can take it off again; a tab is invisible in the cell and in what
/// is copied, and some programs trim leading space and then read the formula all the same.
fn guarded(cell: &str) -> Cow<'_, str> {
    let risky = matches!(
        cell.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r')
    );
    if risky && !is_number(cell) {
        Cow::Owned(format!("'{cell}"))
    } else {
        Cow::Borrowed(cell)
    }
}

/// `-12`, `+3.5`, `-1,5`, `-2e3`: a number a spreadsheet should go on reading as one.
fn is_number(cell: &str) -> bool {
    let digits = cell.strip_prefix(['+', '-']).unwrap_or(cell);
    let (mantissa, exponent) = match digits.find(['e', 'E']) {
        Some(at) => (&digits[..at], Some(&digits[at + 1..])),
        None => (digits, None),
    };
    let mut parts = mantissa.splitn(2, ['.', ',']);
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next();
    let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let mantissa_ok = match fraction {
        None => all_digits(whole),
        Some(f) => (all_digits(whole) || whole.is_empty()) && all_digits(f),
    };
    let exponent_ok = exponent.is_none_or(|e| all_digits(e.strip_prefix(['+', '-']).unwrap_or(e)));
    mantissa_ok && exponent_ok
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
            cells: CsvCells::Guarded,
        };
        let csv = table_csv(&table).unwrap();
        assert!(csv.starts_with('\u{feff}'));
        assert_eq!(csv.matches('\u{feff}').count(), 1);
        assert!(csv.contains("\"deux\nlignes, et virgule\""));
        assert!(csv.contains("Lettre à l'imam"));
    }

    /// The cells a transcription or a description meets, each as a spreadsheet would read it.
    const CELLS: [(&str, &str); 16] = [
        ("=SUM(A1:A3)", "'=SUM(A1:A3)"),
        ("+33 6 12 34 56 78", "'+33 6 12 34 56 78"),
        ("- premier point", "'- premier point"),
        ("@imam_bamako", "'@imam_bamako"),
        ("=cmd|' /C calc'!A0", "'=cmd|' /C calc'!A0"),
        ("\t=1+1", "'\t=1+1"),
        ("\r=1+1", "'\r=1+1"),
        ("-", "'-"),
        ("-1+2", "'-1+2"),
        ("-12", "-12"),
        ("+3.5", "+3.5"),
        ("-1,5", "-1,5"),
        ("-2e3", "-2e3"),
        ("-.5", "-.5"),
        ("Bamako, le 3 mai 1957", "Bamako, le 3 mai 1957"),
        ("'déjà cité'", "'déjà cité'"),
    ];

    #[test]
    fn a_cell_read_as_a_formula_is_guarded_and_a_number_is_not() {
        for (cell, shown) in CELLS {
            assert_eq!(guarded(cell), shown, "{cell:?}");
        }
    }

    fn read_back(csv: &str) -> Vec<Vec<String>> {
        let body = csv.strip_prefix('\u{feff}').unwrap();
        csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(body.as_bytes())
            .records()
            .map(|r| r.unwrap().iter().map(String::from).collect())
            .collect()
    }

    fn table(cells: CsvCells) -> Table {
        Table {
            headings: vec!["cell".into()],
            rows: CELLS.iter().map(|(c, _)| vec![(*c).to_string()]).collect(),
            cells,
        }
    }

    #[test]
    fn a_table_for_another_program_reads_back_exactly_as_it_was() {
        let csv = table_csv(&table(CsvCells::AsWritten)).unwrap();
        let rows = read_back(&csv);
        assert_eq!(rows[0], ["cell"]);
        let cells: Vec<&str> = rows[1..].iter().map(|r| r[0].as_str()).collect();
        let written: Vec<&str> = CELLS.iter().map(|(c, _)| *c).collect();
        assert_eq!(cells, written);
    }

    #[test]
    fn a_guarded_table_reads_back_with_one_apostrophe_before_each_formula() {
        let csv = table_csv(&table(CsvCells::Guarded)).unwrap();
        let rows = read_back(&csv);
        for (row, (cell, shown)) in rows[1..].iter().zip(CELLS) {
            assert_eq!(row[0], shown);
            // Taking the one apostrophe off gives the text back.
            if row[0] != cell {
                assert_eq!(row[0].strip_prefix('\''), Some(cell));
            }
        }
    }
}
