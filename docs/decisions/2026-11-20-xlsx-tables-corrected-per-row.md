# 2026-11-20 · Tables as Excel workbooks, and `corrected` per row in records

## Context

The review of 20 November ([#209](https://github.com/fmadore/Kalamu/issues/209)) settled what
the roadmap left open after the Windows test pass of 17 November:

- Researchers open a `.csv` by double-clicking it. Excel set to French or German expects
  semicolons, since the comma is their decimal mark, and shows the whole table in column A.
  *Data › From Text/CSV* is not something Kalamu's readers do; they conclude the table is
  broken.
- A records export marked `corrected` on every row of a page as soon as one of its rows was
  corrected, because the flag was the page's.

## Decision

**A workbook rather than a CSV whose separator follows the system's language.** A semicolon
CSV would open in French and German Excel and break in English Excel, R and Python, and which
one a researcher gets would depend on a setting they do not know they have; the same file sent
to a colleague abroad would break again. A workbook has no separator: it opens by double-click
in Excel, LibreOffice and Numbers in any language. So every table Kalamu writes as CSV is also
offered as `.xlsx`, through one writer, `export::table_xlsx`, beside `table_csv`:

- A job's results: `ExportFormat::Xlsx`, offered just before CSV for every task, and an exporter
  that writes the same `Table` as the CSV exporter (the quotations of an interview analysis as a
  second workbook beside it). The engine needed only its quotations listed beside either table
  (`ExportFormat::is_table`); New job, the glossary and the file names read the format from Rust.
- The tables other screens write beside a JSON file (the project's descriptions, an export
  profile's columns, the review manifest, the dictionary's forms, a table of contents' items)
  are written as `.xlsx` too, through one helper in the shell (`commands::write_table`), named
  as the CSV is.
- The writer is `rust_xlsxwriter` 0.103.0 (MIT OR Apache-2.0), default features off; its `zip`
  deflates with zopfli and zlib-rs, so no C library is built. `calamine` (MIT) reads the
  workbooks back in the tests only.
- The CSV is unchanged: commas, UTF-8 with a byte-order mark, the guarding apostrophe. New job
  labels the two "Spreadsheet for Excel (.xlsx)" and "Table for other tools: R, Python…
  (.csv)". A table of contents and records tick the workbook by default instead of the CSV.

**Every cell is a string.** No number, date or formula is guessed: a telephone number
(`0612345678`), a date as the source writes it (`03/05/1957`), a page number or a formula quoted
on a page stay exactly as written and are never computed, so nothing needs the CSV's guarding
apostrophe (`CsvCells` does not change the workbook) and Excel is told not to flag numbers held
as text. One sheet, `Table`; the heading row bold and frozen; columns as wide as their longest
line within 8 to 60 characters; long text wrapped and aligned to the top. A cell over Excel's
limit of 32,767 characters (UTF-16 units) is cut at a character and ends with a note that says
so. The creation date is fixed (and the zip's dates already are), so the same table gives the
same bytes and golden tests read the cells back stably.

**`corrected` per row.** The engine compares a corrected records page's rows with the model's
(`core::records::corrected_rows`) and hands the exporters one flag per row
(`ExportPart::rows_corrected`). The rows are lined up by their cells, in order: the longest run
of rows that are the same in both are the model's, and every other row was typed into or added.
Two rows are the same when each column holds the same value, a missing cell counting as empty
and reasons not counting. Comparing by place would mark every row below a removed one; the
review never reorders rows, so lining them up in order loses nothing. A cell typed back as the
model wrote it leaves its row the model's; a removed row leaves nothing to mark. `reviewed`
stays the page's, since the review marks pages. The CSV, the workbook and the JSON rows say the
same.

## Consequences

- Two table files where there was one. The default selections stay small: a table of contents
  and records tick only the workbook; any other job ticks what the researcher chose before.
- A table written beside a JSON export is now three files (`.json`, `.xlsx`, `.csv`), and the
  messages after export say so.
- A records page corrected before this change exports its rows with the per-row rule as soon as
  its files are written again: the model's answers are kept, so nothing is lost.
- The Windows test pass and the Mac day gain a hand check: the workbook opens by double-click in
  Excel set to French on Windows and in Numbers or Excel on a Mac.
- `.xlsx` stays refused as an input ([14 November](2026-11-14-checkpoint-leftovers.md)): it is an
  output format only, not in `InputKind::EXTENSIONS`.
