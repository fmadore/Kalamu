//! A document made of image files, one page each (or several, for a multi-page TIFF), in an
//! order the researcher chose (#52): photographs of an archival file, a letter's sheets, an
//! issue scanned page by page.
//!
//! Each page keeps its identity, its original file (path and content hash), apart from its
//! place in the document: moving a page changes its position, never what it is. A page is read
//! under the key of its own file, never under its position, so a page already read is not read
//! again because it moved (see [`crate::SourceFile::cache_identity`]).
//!
//! The order starts as the files' natural order, the numbers in their names read as numbers
//! (`page 2` before `page 10`), and numbers that skip or repeat are pointed out, never fixed.

use std::cmp::Ordering;
use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::fingerprint::sha256_hex;
use crate::{InputKind, SourceFile, SourceId};

/// One file of an ordered document, as it was when added: never copied, moved or changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PageFile {
    /// The page's identity, from its file's path and content: the same wherever the page is
    /// moved in the document.
    pub id: String,
    /// Absolute path on this PC.
    pub path: String,
    pub file_name: String,
    /// SHA-256 of the file's bytes, lowercase hex: what the page is read under.
    pub sha256: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// When the file was last modified, in milliseconds since 1970, as it was when hashed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub modified_ms: Option<i64>,
    /// The pages the file holds: one, or more for a multi-page TIFF, read in their order.
    #[serde(default = "one")]
    pub frames: u32,
    /// The researcher's name for the page (`f. 3v`), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
}

fn one() -> u32 {
    1
}

/// A page's identity: its file's path and content together.
pub fn page_id(path: &str, sha256: &str) -> String {
    sha256_hex(format!("{path}\n{sha256}").as_bytes())[..16].to_string()
}

impl PageFile {
    /// The page an inspected image file makes; `None` for a file that is not an image.
    pub fn of(source: &SourceFile) -> Option<Self> {
        if source.kind != InputKind::Image || source.is_page_set() {
            return None;
        }
        Some(Self {
            id: page_id(&source.path, &source.sha256),
            path: source.path.clone(),
            file_name: source.file_name.clone(),
            sha256: source.sha256.clone(),
            size_bytes: source.size_bytes,
            modified_ms: source.modified_ms,
            frames: source.page_count.unwrap_or(1).max(1),
            label: None,
        })
    }
}

/// The hash a document of pages is known by: its pages' contents in their order. Moving a page
/// makes another document (whose table of contents is combined again), never another page.
pub fn manifest_sha256(pages: &[PageFile]) -> String {
    let mut material = String::from("kalamu-pages-1\n");
    for page in pages {
        material.push_str(&page.sha256);
        material.push('\n');
    }
    sha256_hex(material.as_bytes())
}

/// A name both Windows and macOS accept for a file: characters either refuses replaced, no
/// trailing dot or space, none of Windows' reserved device names, at most 120 characters.
/// `document` when nothing is left.
pub fn file_name_of(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
                '-'
            } else {
                c
            }
        })
        .take(120)
        .collect();
    let trimmed = cleaned.trim().trim_end_matches(['.', ' ']).trim();
    if trimmed.is_empty() {
        return "document".to_string();
    }
    let stem = trimmed
        .split('.')
        .next()
        .unwrap_or(trimmed)
        .to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        format!("{trimmed}-")
    } else {
        trimmed.to_string()
    }
}

impl SourceFile {
    /// A document of `pages`, in that order, named `name`. Its place, beside which its results
    /// are written, is the first page's folder; `path` names the document there, a file that is
    /// not on disk: the document is its pages.
    pub fn image_document(name: &str, pages: Vec<PageFile>) -> Self {
        let mut source = Self {
            id: SourceId::new(),
            path: String::new(),
            file_name: file_name_of(name),
            sha256: String::new(),
            size_bytes: 0,
            modified_ms: None,
            kind: InputKind::Image,
            page_count: None,
            duration_ms: None,
            split_spreads: false,
            even_out_volume: false,
            title: None,
            derived_from: None,
            pages,
        };
        source.settle_pages();
        source
    }

    /// Whether the source is a document of several files, read page by page in its order.
    /// The one test of it: everything else asks the source what it reads.
    pub fn is_page_set(&self) -> bool {
        !self.pages.is_empty()
    }

    /// Works out again what follows from the pages and their order: the document's hash, size,
    /// page count and place, and each page's identity. Called when a job is made, after the
    /// researcher reordered the pages, so the interface never computes any of it.
    pub fn settle_pages(&mut self) {
        if !self.is_page_set() {
            return;
        }
        for page in &mut self.pages {
            page.id = page_id(&page.path, &page.sha256);
            page.frames = page.frames.max(1);
        }
        self.file_name = file_name_of(&self.file_name);
        self.sha256 = manifest_sha256(&self.pages);
        self.size_bytes = self.pages.iter().map(|p| p.size_bytes).sum();
        self.page_count = Some(self.pages.iter().map(|p| p.frames).sum());
        self.modified_ms = None;
        self.kind = InputKind::Image;
        let folder = Path::new(&self.pages[0].path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        self.path = folder.join(&self.file_name).display().to_string();
    }

    /// The file a page of the document is in, and which of its pages it is (from 1); for
    /// another source, `None`.
    pub fn page_file(&self, number: u32) -> Option<(&PageFile, u32)> {
        let mut first = 1;
        for page in &self.pages {
            let frames = page.frames.max(1);
            if number >= first && number < first + frames {
                return Some((page, number - first + 1));
            }
            first += frames;
        }
        None
    }

    /// What a piece is read under in the cache: the content it comes from and its place in
    /// that content. A page of a document of pages is its own file's page, wherever it stands
    /// in the document, so reordering never reads it again, and a file read alone in another
    /// job is the same page. Everything else is the source's own content and the piece.
    pub fn cache_identity(&self, piece: &crate::PieceRef) -> (&str, crate::PieceRef) {
        if let crate::PieceRef::Page { number, half } = piece
            && let Some((page, frame)) = self.page_file(*number)
        {
            return (
                page.sha256.as_str(),
                crate::PieceRef::Page {
                    number: frame,
                    half: *half,
                },
            );
        }
        (self.sha256.as_str(), piece.clone())
    }

    /// The files on this PC the source reads: its pages' for a document of pages, its own
    /// otherwise, none for a YouTube video.
    pub fn files(&self) -> Vec<&str> {
        if self.is_page_set() {
            self.pages.iter().map(|p| p.path.as_str()).collect()
        } else if self.kind == InputKind::YouTube {
            Vec::new()
        } else {
            vec![self.path.as_str()]
        }
    }

    /// The file a piece is read from: a page's own file in a document of pages, the source's
    /// file otherwise; `None` for a YouTube video, or for a document of pages read whole.
    pub fn file_of(&self, piece: &crate::PieceRef) -> Option<&str> {
        if let crate::PieceRef::Page { number, .. } = piece
            && let Some((page, _)) = self.page_file(*number)
        {
            return Some(page.path.as_str());
        }
        if self.is_page_set() {
            return None;
        }
        self.files().first().copied()
    }

    /// Whether `file_name` is a name rather than a file's (a video's title, a document of
    /// pages' name), so its dots are not an extension.
    pub fn names_itself(&self) -> bool {
        self.kind == InputKind::YouTube || self.is_page_set()
    }
}

/// Two file names in natural order: runs of digits (0 to 9) compared as numbers, so `page 2`
/// comes before `page 10`; letters compared without regard to capitals; spaces, `_`, `-` and
/// `.` taken as one kind of separator, so `page_2` and `page-10` sort together. Digits of other
/// scripts are compared as letters. Names equal in all this are ordered by fewer leading zeros
/// first (`1` before `01`), then as written, so the order is total and the same every time.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (ca, cb) = (chunks(a), chunks(b));
    for (x, y) in ca.iter().zip(cb.iter()) {
        let order = match (x, y) {
            (Chunk::Number(x), Chunk::Number(y)) => {
                let (x, y) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
                x.len().cmp(&y.len()).then_with(|| x.cmp(y))
            }
            (Chunk::Number(_), Chunk::Text(_)) => Ordering::Less,
            (Chunk::Text(_), Chunk::Number(_)) => Ordering::Greater,
            (Chunk::Text(x), Chunk::Text(y)) => x.cmp(y),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    ca.len()
        .cmp(&cb.len())
        .then_with(|| zeros(&ca).cmp(&zeros(&cb)))
        .then_with(|| a.cmp(b))
}

/// Sorts items by their names in natural order.
pub fn natural_sort<T>(items: &mut [T], name: impl Fn(&T) -> &str) {
    items.sort_by(|a, b| natural_cmp(name(a), name(b)));
}

#[derive(Debug)]
enum Chunk {
    Number(String),
    /// Folded: lower case, separators as one space.
    Text(String),
}

fn chunks(name: &str) -> Vec<Chunk> {
    let mut out: Vec<Chunk> = Vec::new();
    for c in name.chars() {
        if c.is_ascii_digit() {
            match out.last_mut() {
                Some(Chunk::Number(n)) => n.push(c),
                _ => out.push(Chunk::Number(c.to_string())),
            }
        } else {
            let folded: String = if matches!(c, ' ' | '_' | '-' | '.') {
                " ".to_string()
            } else {
                c.to_lowercase().collect()
            };
            match out.last_mut() {
                Some(Chunk::Text(t)) => {
                    // Runs of separators are one.
                    if !(folded == " " && t.ends_with(' ')) {
                        t.push_str(&folded);
                    }
                }
                _ => out.push(Chunk::Text(folded)),
            }
        }
    }
    out
}

fn zeros(chunks: &[Chunk]) -> usize {
    chunks
        .iter()
        .map(|c| match c {
            Chunk::Number(n) => n.len() - n.trim_start_matches('0').len(),
            Chunk::Text(_) => 0,
        })
        .sum()
}

/// What a page's file name says of its number, for New job to point out numbers that skip or
/// repeat. Order-free: it is about the names, not where the pages stand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PageNote {
    /// The page's identity ([`PageFile::id`]).
    pub page_id: String,
    /// The last run of digits (0 to 9) in the name without its extension; `None` when it has
    /// none.
    #[ts(type = "number | null")]
    pub number: Option<u64>,
    /// Another page's name carries the same number.
    pub duplicate: bool,
    /// The numbers missing just below this one, from the next lower number a page carries:
    /// `from..=to`.
    #[ts(type = "number | null")]
    pub missing_from: Option<u64>,
    #[ts(type = "number | null")]
    pub missing_to: Option<u64>,
}

/// The number a file's name carries: the last run of digits before its extension.
pub fn number_in_name(file_name: &str) -> Option<u64> {
    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut last: Option<String> = None;
    let mut run = String::new();
    for c in stem.chars() {
        if c.is_ascii_digit() {
            run.push(c);
        } else if !run.is_empty() {
            last = Some(std::mem::take(&mut run));
        }
    }
    if !run.is_empty() {
        last = Some(run);
    }
    last?.parse().ok()
}

/// Each page's number and whether it repeats or follows a gap. Gaps are found only when at
/// least two pages carry numbers; names without one are left alone.
pub fn page_notes(pages: &[PageFile]) -> Vec<PageNote> {
    let numbers: Vec<Option<u64>> = pages.iter().map(|p| number_in_name(&p.file_name)).collect();
    let mut distinct: Vec<u64> = numbers.iter().flatten().copied().collect();
    distinct.sort_unstable();
    distinct.dedup();
    pages
        .iter()
        .zip(&numbers)
        .map(|(page, number)| {
            let duplicate =
                number.is_some_and(|n| numbers.iter().filter(|m| **m == Some(n)).count() > 1);
            let below = number.and_then(|n| {
                let at = distinct.binary_search(&n).ok()?;
                let previous = *distinct.get(at.checked_sub(1)?)?;
                (n > previous + 1).then_some((previous + 1, n - 1))
            });
            PageNote {
                page_id: page.id.clone(),
                number: *number,
                duplicate,
                missing_from: below.map(|b| b.0),
                missing_to: below.map(|b| b.1),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
