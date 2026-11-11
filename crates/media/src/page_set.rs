//! A document of image files read as its pages (#52): each file opened only when one of its
//! pages is read, and checked then against the hash it was added with, so a page whose file
//! was moved, deleted or replaced fails on its own with a reason naming the file, while the
//! other pages are read. Originals are only read.

use std::path::Path;
use std::sync::{Arc, Mutex};

use kalamu_core::{PageFile, PageHalf, WarningKind};
use sha2::{Digest, Sha256};

use crate::{ImageSource, MediaError, checks, modified_ms};

/// The files of a document of pages, in its order.
pub struct PageSet {
    pages: Vec<PageFile>,
    /// The file last opened, kept for the next page, which is most often in the same file (a
    /// multi-page TIFF, or the two halves of a spread).
    last: Mutex<Option<(usize, Arc<ImageSource>)>>,
}

impl PageSet {
    pub fn new(pages: Vec<PageFile>) -> Self {
        Self {
            pages,
            last: Mutex::new(None),
        }
    }

    /// Every page of every file, in order.
    pub fn page_count(&self) -> u32 {
        self.pages.iter().map(|p| p.frames.max(1)).sum()
    }

    /// The file holding page `number` of the document, its place in the list, and which of the
    /// file's pages it is.
    fn locate(&self, number: u32) -> Option<(usize, u32)> {
        let mut first = 1;
        for (i, page) in self.pages.iter().enumerate() {
            let frames = page.frames.max(1);
            if number >= first && number < first + frames {
                return Some((i, number - first + 1));
            }
            first += frames;
        }
        None
    }

    /// The file at `index`, read and checked against what it held when it was added.
    fn open(&self, index: usize) -> Result<Arc<ImageSource>, MediaError> {
        let mut last = self.last.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((i, img)) = last.as_ref()
            && *i == index
        {
            return Ok(img.clone());
        }
        let page = self
            .pages
            .get(index)
            .ok_or_else(|| MediaError::NoSuchPage(index.to_string()))?;
        let img = Arc::new(read_page(page)?);
        *last = Some((index, img.clone()));
        Ok(img)
    }

    pub fn is_spread(&self, number: u32) -> bool {
        self.locate(number)
            .and_then(|(i, frame)| Some((self.open(i).ok()?, frame)))
            .is_some_and(|(img, frame)| img.is_spread(frame))
    }

    /// Page `number` of the document in an accepted format.
    pub fn prepare(
        &self,
        number: u32,
        half: Option<PageHalf>,
        accepted: &[&str],
    ) -> Result<(String, Vec<u8>), MediaError> {
        let (index, frame) = self
            .locate(number)
            .ok_or_else(|| MediaError::NoSuchPage(number.to_string()))?;
        self.open(index)?.prepare(frame, half, accepted)
    }

    /// Each file's checks, its pages numbered as the document numbers them. A file that cannot
    /// be read is left to fail when its page is.
    pub fn warnings(&self) -> Vec<WarningKind> {
        let mut spreads = Vec::new();
        let mut low: Vec<u32> = Vec::new();
        let mut lowest: Option<u32> = None;
        let mut unsupported: Option<String> = None;
        let mut first = 1;
        for (i, page) in self.pages.iter().enumerate() {
            if let Ok(img) = self.open(i) {
                for kind in checks::image_warnings(&img) {
                    match kind {
                        WarningKind::DoublePageSpread { pages } => {
                            spreads.extend(pages.into_iter().map(|n| n + first - 1));
                        }
                        WarningKind::LowResolution { pages, dpi } => {
                            low.extend(pages.into_iter().map(|n| n + first - 1));
                            lowest = Some(lowest.map_or(dpi, |l| l.min(dpi)));
                        }
                        WarningKind::UnsupportedImage { detail } => unsupported = Some(detail),
                        _ => {}
                    }
                }
            }
            first += page.frames.max(1);
        }
        let mut out = Vec::new();
        if let Some(detail) = unsupported {
            out.push(WarningKind::UnsupportedImage { detail });
        }
        if !spreads.is_empty() {
            out.push(WarningKind::DoublePageSpread { pages: spreads });
        }
        if let Some(dpi) = lowest {
            out.push(WarningKind::LowResolution { pages: low, dpi });
        }
        out
    }
}

/// A page's file read whole, refused when it is not what was added: its size or, when the time
/// it was last changed moved, its hash differs.
fn read_page(page: &PageFile) -> Result<ImageSource, MediaError> {
    let path = Path::new(&page.path);
    let missing = |e: std::io::Error| MediaError::Read {
        path: page.path.clone(),
        message: e.to_string(),
    };
    let metadata = std::fs::metadata(path).map_err(missing)?;
    if metadata.len() != page.size_bytes {
        return Err(MediaError::PageChanged(page.file_name.clone()));
    }
    let bytes = std::fs::read(path).map_err(missing)?;
    let same_time = page.modified_ms.is_some() && modified_ms(&metadata) == page.modified_ms;
    if !same_time && hex::encode(Sha256::digest(&bytes)) != page.sha256 {
        return Err(MediaError::PageChanged(page.file_name.clone()));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    ImageSource::from_bytes(bytes, &ext)
}

#[cfg(test)]
mod tests;
