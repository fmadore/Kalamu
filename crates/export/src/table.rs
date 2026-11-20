//! Tables for spreadsheets ([`Table`]): a job's results set out as rows, a project's descriptions,
//! an export profile's columns, the dictionary's forms, the review manifest. One writer per file
//! type for all of them: CSV ([`table_csv`]) for R, Python and other tools, and an Excel workbook
//! ([`table_xlsx`]), which opens by double-click in a spreadsheet set to any language.

use std::borrow::Cow;

use kalamu_core::{CsvCells, ExportError, Table};
use rust_xlsxwriter::{DocProperties, ExcelDateTime, Format, FormatAlign, IgnoreError, Workbook};

/// The workbook's one sheet.
const SHEET: &str = "Table";

/// The most a cell of a workbook holds, in UTF-16 units, as Excel counts them.
const CELL_MAX: usize = 32_767;

/// What ends a cell cut to fit a workbook.
const CUT: &str = " […] Cut here: an Excel cell holds 32,767 characters at most. The CSV and JSON formats keep the whole text.";

/// Columns as narrow and as wide as the workbook makes them, in characters.
const WIDTH: (usize, usize) = (8, 60);

/// The table as an Excel workbook: one sheet, the headings in bold on a row that stays in view,
/// each column as wide as its cells within [`WIDTH`], long text wrapped.
///
/// Every cell is text, exactly as written: a telephone number, a date or a formula a model
/// read off a page stays as the researcher sees it, never computed nor turned into a number. So
/// nothing needs guarding, and [`Table::cells`] does not change the workbook.
///
/// The same table always gives the same bytes: the workbook's creation date is fixed, as the
/// dates of its zip entries already are.
pub fn table_xlsx(table: &Table) -> Result<Vec<u8>, ExportError> {
    let err = |e: &dyn std::fmt::Display| ExportError::Write {
        format: "Excel",
        message: e.to_string(),
    };
    let mut workbook = Workbook::new();
    let created = ExcelDateTime::from_ymd(2000, 1, 1).map_err(|e| err(&e))?;
    workbook.set_properties(
        &DocProperties::new()
            .set_creation_datetime(&created)
            .set_author("Kalamu"),
    );
    let heading = Format::new().set_bold().set_align(FormatAlign::Top);
    let cell = Format::new().set_text_wrap().set_align(FormatAlign::Top);
    let sheet = workbook.add_worksheet();
    sheet.set_name(SHEET).map_err(|e| err(&e))?;
    let columns = table
        .rows
        .iter()
        .map(Vec::len)
        .chain([table.headings.len()])
        .max()
        .unwrap_or_default();
    for column in 0..columns {
        let width = std::iter::once(&table.headings)
            .chain(&table.rows)
            .filter_map(|row| row.get(column))
            .map(|text| shown_width(text))
            .max()
            .unwrap_or_default();
        let width = (width + 2).clamp(WIDTH.0, WIDTH.1);
        let at = u16::try_from(column).map_err(|e| err(&e))?;
        sheet
            .set_column_width(at, width as f64)
            .map_err(|e| err(&e))?;
    }
    for (column, text) in table.headings.iter().enumerate() {
        let at = u16::try_from(column).map_err(|e| err(&e))?;
        sheet
            .write_string_with_format(0, at, fitted(text), &heading)
            .map_err(|e| err(&e))?;
    }
    for (row, cells) in table.rows.iter().enumerate() {
        let row = u32::try_from(row + 1).map_err(|e| err(&e))?;
        for (column, text) in cells.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let at = u16::try_from(column).map_err(|e| err(&e))?;
            sheet
                .write_string_with_format(row, at, fitted(text), &cell)
                .map_err(|e| err(&e))?;
        }
    }
    sheet.set_freeze_panes(1, 0).map_err(|e| err(&e))?;
    // A page number or a year held as text is meant to be: Excel marks no cell for it.
    if let Some(last) = columns.checked_sub(1) {
        let last_row = u32::try_from(table.rows.len()).map_err(|e| err(&e))?;
        let last_column = u16::try_from(last).map_err(|e| err(&e))?;
        sheet
            .ignore_error_range(0, 0, last_row, last_column, IgnoreError::NumberStoredAsText)
            .map_err(|e| err(&e))?;
    }
    workbook.save_to_buffer().map_err(|e| err(&e))
}

/// How wide a cell's text is shown: its longest line, in characters.
fn shown_width(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_default()
}

/// A cell's text as a workbook can hold it: whole, or cut at a character before Excel's limit
/// and said to be.
fn fitted(text: &str) -> Cow<'_, str> {
    if text.encode_utf16().count() <= CELL_MAX {
        return Cow::Borrowed(text);
    }
    let room = CELL_MAX - CUT.encode_utf16().count();
    let mut used = 0;
    let mut end = 0;
    for (at, ch) in text.char_indices() {
        if used + ch.len_utf16() > room {
            break;
        }
        used += ch.len_utf16();
        end = at + ch.len_utf8();
    }
    Cow::Owned(format!("{}{CUT}", &text[..end]))
}

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
pub(crate) mod tests {
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

    /// A workbook's cells as text, failing on one that is not text (a number, a date, a
    /// formula).
    pub(crate) fn workbook_cells(bytes: &[u8]) -> Vec<Vec<String>> {
        use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
        let mut book: Xlsx<_> = open_workbook_from_rs(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(book.sheet_names(), [SHEET]);
        let range = book.worksheet_range(SHEET).unwrap();
        range
            .rows()
            .map(|row| {
                row.iter()
                    .map(|cell| match cell {
                        Data::String(s) => s.clone(),
                        Data::Empty => String::new(),
                        other => panic!("not text: {other:?}"),
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn a_workbook_holds_every_cell_as_text_exactly_as_written_whatever_the_csv_guards() {
        for cells in [CsvCells::Guarded, CsvCells::AsWritten] {
            let bytes = table_xlsx(&table(cells)).unwrap();
            assert!(bytes.starts_with(b"PK"), "a workbook is a zip archive");
            let rows = workbook_cells(&bytes);
            assert_eq!(rows[0], ["cell"]);
            let read: Vec<&str> = rows[1..].iter().map(|r| r[0].as_str()).collect();
            let written: Vec<&str> = CELLS.iter().map(|(c, _)| *c).collect();
            assert_eq!(read, written);
        }
    }

    #[test]
    fn a_workbook_keeps_accents_lines_numbers_and_dates_as_written() {
        let table = Table {
            headings: vec!["title".into(), "telephone".into(), "date".into(), "".into()],
            rows: vec![
                vec![
                    "Lettre à l'imam\nsur deux lignes".into(),
                    "0612345678".into(),
                    "03/05/1957".into(),
                    "1e5".into(),
                ],
                // A shorter row, and an empty cell.
                vec!["عبد الله".into(), String::new()],
            ],
            cells: CsvCells::Guarded,
        };
        let rows = workbook_cells(&table_xlsx(&table).unwrap());
        assert_eq!(rows[0], ["title", "telephone", "date", ""]);
        assert_eq!(
            rows[1],
            [
                "Lettre à l'imam\nsur deux lignes",
                "0612345678",
                "03/05/1957",
                "1e5"
            ]
        );
        assert_eq!(rows[2], ["عبد الله", "", "", ""]);
    }

    #[test]
    fn the_same_table_gives_the_same_workbook() {
        let table = table(CsvCells::Guarded);
        assert_eq!(table_xlsx(&table).unwrap(), table_xlsx(&table).unwrap());
    }

    #[test]
    fn a_cell_longer_than_excel_holds_is_cut_and_says_so() {
        let long = "é".repeat(40_000);
        let table = Table {
            headings: vec!["text".into()],
            rows: vec![vec![long.clone()]],
            cells: CsvCells::Guarded,
        };
        let rows = workbook_cells(&table_xlsx(&table).unwrap());
        let cell = &rows[1][0];
        assert!(cell.ends_with(CUT));
        assert_eq!(cell.encode_utf16().count(), CELL_MAX);
        assert!(long.starts_with(cell.strip_suffix(CUT).unwrap()));
        // A character outside the basic plane counts twice and is never split.
        let wide = fitted(&"𝔄".repeat(20_000)).into_owned();
        assert!(wide.encode_utf16().count() <= CELL_MAX);
        assert!(wide.strip_suffix(CUT).unwrap().chars().all(|c| c == '𝔄'));
    }

    #[test]
    fn columns_fit_their_text_within_bounds() {
        assert_eq!(shown_width("deux\nlignes plus longues"), 19);
        assert_eq!(shown_width(""), 0);
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
