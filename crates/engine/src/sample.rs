//! The review queue's random sample (#85): the pages in scope that nothing flags, from which
//! [`kalamu_core::sample::draw`] takes the pages of lowest rank for the seed kept with the
//! project.
//!
//! A page is its document, task and place, so a page read by several set-ups or jobs is one
//! candidate, flagged when any of the results read for it is: the sample never counts an
//! alternative reading as another page. The result that stands for it is the first the queue
//! reads (the preferred one, else the newest job's first set-up).

use std::collections::HashMap;

use kalamu_core::sample::{draw, page_identity};
use kalamu_core::{PieceRef, QueueItem, ReviewQueue, SampleCounts, SampleSpec};

/// One page of the scope, as the draw sees it.
struct Page {
    flagged: bool,
    /// The item standing for it: already listed in the queue (its position), or to be listed if
    /// drawn.
    listed: Option<usize>,
    item: Option<QueueItem>,
}

/// The pages offered for the draw, as the queue reads them.
#[derive(Default)]
pub(crate) struct Pool {
    pages: HashMap<String, Page>,
}

impl Pool {
    /// A piece read for a result in scope: whether anything flags it, and the item that stands
    /// for it, listed at `listed` or not listed.
    pub fn offer(
        &mut self,
        piece: &PieceRef,
        flagged: bool,
        item: QueueItem,
        listed: Option<usize>,
    ) {
        let identity = page_identity(item.document_id, item.task, piece);
        let page = self.pages.entry(identity).or_insert(Page {
            flagged: false,
            listed,
            item: listed.is_none().then_some(item),
        });
        page.flagged |= flagged;
    }

    /// A piece that cannot be drawn: it failed or is not read, or its result's input changed.
    pub fn exclude(&mut self, identity: String) {
        self.pages
            .entry(identity)
            .and_modify(|p| p.flagged = true)
            .or_insert(Page {
                flagged: true,
                listed: None,
                item: None,
            });
    }

    /// Marks the pages drawn for `spec` in the queue, listing those it did not list, and says
    /// how the sample stands.
    pub fn draw_into(self, spec: &SampleSpec, queue: &mut ReviewQueue) -> SampleCounts {
        let candidates: Vec<(String, Page)> =
            self.pages.into_iter().filter(|(_, p)| !p.flagged).collect();
        let total = candidates.len() as u32;
        let drawn = draw(spec.seed, spec.size, candidates);
        let mut reviewed = 0;
        let count = drawn.len() as u32;
        for page in drawn {
            let item = match (page.listed, page.item) {
                (Some(at), _) => queue.items.get_mut(at),
                (None, Some(item)) => {
                    queue.items.push(item);
                    queue.items.last_mut()
                }
                (None, None) => None,
            };
            if let Some(item) = item {
                item.sample = true;
                reviewed += u32::from(item.reviewed);
            }
        }
        SampleCounts {
            seed: spec.seed,
            size: spec.size,
            drawn_at: spec.drawn_at,
            candidates: total,
            drawn: count,
            reviewed,
        }
    }
}
