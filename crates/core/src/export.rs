use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{PieceRef, Provenance, Segment, Setup, SourceFile, Task, Timestamp};

/// The byte-order mark every CSV file starts with, so Excel opens accented text as UTF-8.
pub const CSV_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ExportFormat {
    Txt,
    Markdown,
    Docx,
    Json,
    Srt,
    Vtt,
    Csv,
    /// An Excel workbook of the same table as the CSV, every cell text: it opens by
    /// double-click in a spreadsheet set to any language, where a CSV's commas may not be read.
    Xlsx,
    /// The original PDF with an invisible text layer, from models that report layout.
    SearchablePdf,
    /// ALTO XML: the text with where each block, line and word sits, from models that report
    /// layout.
    Alto,
    /// PAGE XML (PRImA's 2019-07-15 schema), one file per page: the same regions as ALTO, for
    /// Transkribus, eScriptorium and Aletheia.
    PageXml,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Markdown => "md",
            Self::Docx => "docx",
            Self::Json => "json",
            Self::Srt => "srt",
            Self::Vtt => "vtt",
            Self::Csv => "csv",
            Self::Xlsx => "xlsx",
            // Never `.pdf` alone, which would replace the source.
            Self::SearchablePdf => "searchable.pdf",
            Self::Alto => "alto.xml",
            // Never `.xml` alone, which the ALTO file's name ends with too.
            Self::PageXml => "page.xml",
        }
    }

    /// A table for a spreadsheet, CSV or a workbook: an analysis that cites the text writes its
    /// quotations as a second table beside it (`quotations.csv`, `quotations.xlsx`).
    pub fn is_table(self) -> bool {
        matches!(self, Self::Csv | Self::Xlsx)
    }

    /// Written only from a model that reports where text sits on the page.
    pub fn needs_layout(self) -> bool {
        matches!(self, Self::SearchablePdf | Self::Alto | Self::PageXml)
    }

    /// Written as one file per page rather than one per document: PAGE XML, whose file holds
    /// a single page.
    pub fn per_page(self) -> bool {
        self == Self::PageXml
    }

    /// The end of the name of the file a per-page format writes for page `number`, after the
    /// source's name and a dot: `p0003.page.xml`, numbered on four digits at least so the
    /// files sort in the order of the pages.
    pub fn page_suffix(self, number: u32) -> String {
        format!("p{number:04}.{}", self.extension())
    }

    /// The page a file's name ends with, after the source's name and its dot, when it is one
    /// this per-page format writes ([`ExportFormat::page_suffix`]).
    pub fn page_of_suffix(self, suffix: &str) -> Option<u32> {
        let digits = suffix
            .strip_prefix('p')?
            .strip_suffix(self.extension())?
            .strip_suffix('.')?;
        if !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let number: u32 = digits.parse().ok()?;
        (self.per_page() && self.page_suffix(number) == suffix).then_some(number)
    }

    /// Subtitles: written only from text whose lines have times, a transcript or a translation
    /// of one that kept them ([`crate::aligned`]).
    pub fn needs_times(self) -> bool {
        matches!(self, Self::Srt | Self::Vtt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportOptions {
    /// Mark where each page or segment starts.
    pub piece_markers: bool,
    pub speaker_labels: bool,
    pub timestamps: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            piece_markers: true,
            speaker_labels: true,
            timestamps: true,
        }
    }
}

/// One page or segment, with the user's correction applied when there is one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportPart {
    pub piece: PieceRef,
    pub text: String,
    /// Timed segments, with times relative to the start of the recording.
    pub segments: Vec<Segment>,
    pub corrected: bool,
    pub reviewed: bool,
    /// For records (#171), one flag per row of `fields`, in order: whether the researcher typed
    /// into it or added it ([`crate::records::corrected_rows`]). Empty for any other part and
    /// for a page not corrected, whose rows are all the model's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows_corrected: Vec<bool>,
    /// An analysis's answers, corrected where the researcher corrected them.
    #[serde(default)]
    pub fields: Vec<crate::FieldAnswer>,
    /// Text blocks placed on the page, for a searchable PDF.
    #[serde(default)]
    pub regions: Vec<crate::TextRegion>,
    /// Why this piece could not be read, when it failed after its retries: the document is
    /// still written, the piece marked in its place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed: Option<String>,
    /// For a PDF page drawn as an image for an engine that reads images only (Tesseract), the
    /// resolution it was drawn at, from the result's record; `None` when the page was not
    /// drawn, or was read before the resolution was recorded, which was at 300 dpi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drawn_dpi: Option<f32>,
    /// When the model's result was stored; `None` for a piece not read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_at: Option<Timestamp>,
    /// When the researcher's correction of the text was last saved; `None` without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corrected_at: Option<Timestamp>,
    /// Whose boxes `regions` holds where they are not the model's, and the researcher's own
    /// regions on this page. Empty where they drew nothing, and the files are then written as
    /// before regions could be drawn.
    #[serde(default, skip_serializing_if = "ResearcherGeometry::is_empty")]
    pub researcher: ResearcherGeometry,
}

/// The researcher's part in a page's exported geometry (#84), beside `ExportPart::regions`,
/// which already holds their boxes: a format says from this whose each box is.
///
/// How it is made is the engine's ([`crate::regions::lay_for_export`] and
/// [`crate::regions::contain`]): a box the researcher gave a region of the model's reading of
/// this page replaces the model's; a block or line is widened just enough to hold what lies in
/// it; a region of theirs that adjusts nothing exported is one of their own.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResearcherGeometry {
    /// The boxes in `regions` that are the researcher's, or widened to hold one of theirs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjusted: Vec<AdjustedBox>,
    /// Blocks of `regions` placed whole by the model whose lines Kalamu estimated and wrote into
    /// the block, so a box the researcher gave one of those lines or words could be kept: their
    /// lines are as approximate as those a format estimates itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub estimated: Vec<usize>,
    /// The researcher's own regions on the piece, in the order they were made: drawn, or giving
    /// a box to a region of a reading no longer exported. Their text is not in `text`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drawn: Vec<ResearcherRegion>,
}

impl ResearcherGeometry {
    pub fn is_empty(&self) -> bool {
        self.adjusted.is_empty() && self.estimated.is_empty() && self.drawn.is_empty()
    }
}

/// Where a box is in `ExportPart::regions`: a block, one of its lines, or one of that line's
/// words, each counted from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RegionPath {
    pub block: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word: Option<usize>,
}

/// A box of `ExportPart::regions` the researcher placed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdjustedBox {
    pub at: RegionPath,
    /// The researcher's region it comes from ([`crate::DrawnRegion::id`]), as `provenance.json`
    /// lists it.
    pub id: String,
    pub kind: BoxKind,
    /// When the researcher last saved the box.
    pub updated_at: Timestamp,
}

/// Whose a box of `ExportPart::regions` is, where the researcher had a hand in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoxKind {
    /// The box the researcher gave the model's block, line or word.
    Researcher,
    /// The researcher's box for a block or line, widened just enough to hold the lines or
    /// words the model placed within it, which they did not move.
    ResearcherWidened,
    /// The model's block or line, widened just enough to hold a box the researcher gave one of
    /// its lines or words.
    ModelWidened,
}

/// A region the researcher placed on a page that adjusts nothing exported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearcherRegion {
    /// Its id ([`crate::DrawnRegion::id`]).
    pub id: String,
    /// The half of a split spread its box is in fractions of, as `ExportPart::piece`'s half
    /// for the parts of a page; an exporter puts it on the whole page with
    /// [`crate::TextRegion::onto_whole_page`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub half: Option<crate::PageHalf>,
    /// Its box and what the researcher wrote there, possibly nothing; no lines.
    pub region: crate::TextRegion,
    pub updated_at: Timestamp,
}

/// Everything an exporter needs for one source under one set-up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDocument {
    pub task: Task,
    pub source: SourceFile,
    pub setup: Setup,
    pub parts: Vec<ExportPart>,
    pub provenance: Provenance,
    pub options: ExportOptions,
}

/// A table for a spreadsheet: its headings, then rows of as many cells. A job's results,
/// projects' descriptions, export profiles, the review manifest and the dictionary make one;
/// the `export` crate writes it as CSV, with the byte-order mark every spreadsheet file
/// carries, or as an Excel workbook.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    pub headings: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// What the CSV writer does with a cell a spreadsheet would take for a formula, decided by
    /// whoever makes the table, for what the file is for. A workbook holds every cell as text
    /// and needs no guarding.
    pub cells: CsvCells,
}

/// How a table's cells are written as CSV. A spreadsheet reads a cell that begins with `=`,
/// `+`, `-` or `@` (or a tab or carriage return before one) as a formula: a telephone number,
/// a list item or a formula quoted in a transcription is computed, shown as an error, or, from a
/// hostile document, runs a command.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CsvCells {
    /// Such a cell is written after an apostrophe, so a spreadsheet shows it as text; a cell
    /// that is a number, negative included, stays a number. For a file meant to be opened in a
    /// spreadsheet, whose exact text the JSON export of the same data keeps.
    #[default]
    Guarded,
    /// Every cell exactly as it is, for a file another program imports, where a changed cell
    /// would be imported changed.
    AsWritten,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("{0} export is not available for this task")]
    Unsupported(&'static str),
    #[error("Could not write the {format} file: {message}")]
    Write {
        format: &'static str,
        message: String,
    },
}

/// One file an exporter writes: the end of its name after the source's name and a dot
/// (`alto.xml`, `p0003.page.xml`), and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFile {
    pub suffix: String,
    pub bytes: Vec<u8>,
}

/// Turns stored results into one output format. Adding a format means adding an exporter and
/// nothing else.
pub trait Exporter: Send + Sync {
    fn format(&self) -> ExportFormat;

    fn supports(&self, task: Task) -> bool;

    /// The document as one file.
    fn export(&self, document: &ExportDocument) -> Result<Vec<u8>, ExportError>;

    /// The files the format writes for a document: one, named with the format's extension,
    /// unless the format writes one per page ([`ExportFormat::per_page`]), each named with
    /// [`ExportFormat::page_suffix`].
    fn export_files(&self, document: &ExportDocument) -> Result<Vec<ExportFile>, ExportError> {
        Ok(vec![ExportFile {
            suffix: self.format().extension().to_owned(),
            bytes: self.export(document)?,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::ExportFormat;

    #[test]
    fn a_page_files_name_gives_back_its_page() {
        let page = ExportFormat::PageXml;
        assert_eq!(page.page_suffix(3), "p0003.page.xml");
        assert_eq!(page.page_suffix(12345), "p12345.page.xml");
        assert_eq!(page.page_of_suffix("p0003.page.xml"), Some(3));
        assert_eq!(page.page_of_suffix("p12345.page.xml"), Some(12345));
        // Not one it writes: too few digits, another format's file, or no number.
        assert_eq!(page.page_of_suffix("p3.page.xml"), None);
        assert_eq!(page.page_of_suffix("alto.xml"), None);
        assert_eq!(page.page_of_suffix("p.page.xml"), None);
        assert_eq!(page.page_of_suffix("p00a3.page.xml"), None);
        assert_eq!(ExportFormat::Alto.page_of_suffix("p0003.alto.xml"), None);
    }
}
