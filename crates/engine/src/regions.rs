//! Where a page's text sits, for the review's source pane (#84): read from the store, never from
//! a provider. The coordinates and how a region finds its place in the text are core's
//! ([`kalamu_core::regions`]). A block placed whole (Mistral OCR) needs the page's proportions
//! for its estimated lines, which the stored fractions do not keep: they are read from the
//! source file, only for such a result, and kept for the source's other pages.

use std::collections::HashMap;
use std::path::Path;

use kalamu_core::{
    Correction, InputKind, JobId, PageHalf, PageRegions, PieceRef, SetupId, SourceFile, SourceId,
};
use kalamu_media::{ImageSource, PdfSource};

use crate::control::lock;
use crate::{Engine, EngineError};

/// The displayed sizes of one source's pages, by page number, as last read: the review moves
/// through one source's pages at a time, so its file is opened once for them.
#[derive(Default)]
pub(crate) struct PageSizes {
    source: Option<SourceId>,
    sizes: HashMap<u32, (f32, f32)>,
}

impl Engine {
    /// The regions the engine stored for piece `index` of a source under a set-up, each placed in
    /// the text the review shows (the correction, else the model's text) where that is certain.
    /// None for a piece not read, or read by a model that reports no layout.
    pub fn page_regions(
        &self,
        job_id: JobId,
        source_id: SourceId,
        setup_id: SetupId,
        index: u32,
    ) -> Result<PageRegions, EngineError> {
        let pieces = self.store.source_pieces(job_id, setup_id, source_id)?;
        let Some(piece) = pieces.into_iter().find(|p| p.index == index) else {
            return Ok(PageRegions::none(false));
        };
        let corrections = self.store.source_corrections(job_id, setup_id, source_id)?;
        let corrected = Correction::of(&corrections, &piece).is_some_and(|c| c.text.is_some());
        let Some(result) = self.store.finished_result(&piece)? else {
            return Ok(PageRegions::none(corrected));
        };
        if result.output.regions.is_empty() {
            return Ok(PageRegions::none(corrected));
        }
        let speakers = self.store.speakers(job_id)?;
        // The text as the review shows it: `PieceView`'s corrected text, else its model text.
        let shown = self
            .store
            .text(&piece, &corrections, &speakers)?
            .unwrap_or_default();
        let model = kalamu_core::speakers::rename_text(
            &result.output.text,
            speakers.of(setup_id, source_id),
        );
        // Only a block placed whole needs the page's proportions; Tesseract's lines do not.
        let aspect = if result.output.regions.iter().any(|r| r.lines.is_empty()) {
            let job = self.store.job(job_id)?;
            job.sources
                .iter()
                .find(|s| s.id == source_id)
                .and_then(|source| self.page_aspect(source, &piece.piece))
        } else {
            None
        };
        Ok(PageRegions::of(
            &result.output.regions,
            &model,
            &shown,
            corrected,
            aspect,
        ))
    }

    /// The height over the width of `piece` as the review shows it: a PDF page with its own
    /// rotation, an image turned upright, a split spread's half alone. None when the file
    /// cannot be read (moved or deleted: the review cannot show the scan either) or the piece
    /// is not a page.
    fn page_aspect(&self, source: &SourceFile, piece: &PieceRef) -> Option<f32> {
        let PieceRef::Page { number, half } = *piece else {
            return None;
        };
        let (width, height) = self.page_size(source, number)?;
        let width = match half {
            Some(PageHalf::Left | PageHalf::Right) => width / 2.0,
            None => width,
        };
        (width > 0.0 && height > 0.0).then(|| height / width)
    }

    /// Page `number`'s displayed width and height, in any unit, read from the source file the
    /// first time one of its pages is asked for. A failed read is not kept, so a file put back
    /// is read again.
    fn page_size(&self, source: &SourceFile, number: u32) -> Option<(f32, f32)> {
        {
            let cache = lock(&self.page_sizes);
            if cache.source == Some(source.id)
                && let Some(size) = cache.sizes.get(&number)
            {
                return Some(*size);
            }
        }
        // Read without holding the lock: a large PDF takes a moment to parse.
        let read = read_page_sizes(source, number);
        let mut cache = lock(&self.page_sizes);
        if cache.source != Some(source.id) {
            *cache = PageSizes {
                source: Some(source.id),
                sizes: HashMap::new(),
            };
        }
        cache.sizes.extend(read);
        cache.sizes.get(&number).copied()
    }
}

/// The displayed sizes of a source's pages: every page of a PDF or an image file, read at once;
/// of a document of image files, page `number`'s file alone.
fn read_page_sizes(source: &SourceFile, number: u32) -> Vec<(u32, (f32, f32))> {
    if source.is_page_set() {
        let Some((page, frame)) = source.page_file(number) else {
            return Vec::new();
        };
        return ImageSource::open(Path::new(&page.path))
            .ok()
            .and_then(|image| image.display_size(frame))
            .map(|(w, h)| vec![(number, (w as f32, h as f32))])
            .unwrap_or_default();
    }
    let path = Path::new(&source.path);
    match source.kind {
        InputKind::Pdf => PdfSource::open(path)
            .map(|pdf| {
                (1..=pdf.page_count())
                    .filter_map(|n| pdf.display_size(n).map(|size| (n, size)))
                    .collect()
            })
            .unwrap_or_default(),
        InputKind::Image => ImageSource::open(path)
            .map(|image| {
                (1..=image.page_count())
                    .filter_map(|n| {
                        image
                            .display_size(n)
                            .map(|(w, h)| (n, (w as f32, h as f32)))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}
