//! A reproducible random sample of the pages the review queue does not flag
//! ([#85](https://github.com/fmadore/Kalamu/issues/85)).
//!
//! Flagged pages are chosen for what is wrong with them, so what the researcher finds there says
//! nothing of the rest of the corpus. A sample drawn at random from the pages nothing flagged
//! does: it shows how the unflagged material reads.
//!
//! The draw is decided by a seed, shown and kept with the project, so the same sample comes back
//! after a restart and can be cited. Each page is ranked by the SHA-256 of the seed and the page's
//! identity (its document, task and place); the sample is the pages of lowest rank. The order
//! therefore depends on neither the order rows are read in nor the system, and a page keeps its
//! rank whatever else is in scope: a page flagged since leaves the sample and the next in rank
//! takes its place, the others staying.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use crate::{DocumentId, PieceRef, Task, Timestamp};

/// The most pages a sample can hold: enough for any check by hand.
pub const MAX_SAMPLE: u32 = 1000;

/// The largest seed: six digits at most, short enough to write down and cite.
pub const MAX_SEED: u32 = 999_999;

/// A sample asked for: its seed, how many pages, and when it was drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SampleSpec {
    pub seed: u32,
    pub size: u32,
    pub drawn_at: Timestamp,
}

impl SampleSpec {
    /// A sample of `size` pages (at least 1, at most [`MAX_SAMPLE`]), with `seed` or, without
    /// one, a new seed: a new draw.
    pub fn new(size: u32, seed: Option<u32>, drawn_at: Timestamp) -> Self {
        Self {
            seed: seed.unwrap_or_else(fresh_seed),
            size: size.clamp(1, MAX_SAMPLE),
            drawn_at,
        }
    }
}

/// A new seed of six digits at most, short enough to write down and cite.
pub fn fresh_seed() -> u32 {
    (uuid::Uuid::new_v4().as_u128() % (u128::from(MAX_SEED) + 1)) as u32
}

/// What a page is, for the draw: its document, the task that read it and its place. Two set-ups
/// or two jobs reading the same page give the same identity, so a page read by several models is
/// drawn once.
pub fn page_identity(document: DocumentId, task: Task, piece: &PieceRef) -> String {
    format!("{document}/{}/{}", task.as_str(), piece.descriptor())
}

/// A page's rank in the draw of `seed`: the SHA-256 of `seed:identity`.
pub fn rank(seed: u32, identity: &str) -> [u8; 32] {
    Sha256::digest(format!("{seed}:{identity}").as_bytes()).into()
}

/// The `size` candidates of lowest rank for `seed`, in rank order. The candidates' order does
/// not matter; an identity offered twice is drawn once.
pub fn draw<T>(seed: u32, size: u32, candidates: Vec<(String, T)>) -> Vec<T> {
    let mut ranked: Vec<([u8; 32], String, T)> = candidates
        .into_iter()
        .map(|(identity, value)| (rank(seed, &identity), identity, value))
        .collect();
    ranked.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    ranked.dedup_by(|a, b| a.1 == b.1);
    ranked
        .into_iter()
        .take(size as usize)
        .map(|(_, _, value)| value)
        .collect()
}

/// How a sample stands in the queue: its seed and size, the pages it was drawn from, and how many
/// of those drawn the researcher has reviewed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SampleCounts {
    pub seed: u32,
    pub size: u32,
    pub drawn_at: Timestamp,
    /// Pages in scope that nothing flags, read and reviewable: what the sample is drawn from.
    pub candidates: u32,
    /// Pages drawn: the size asked, or every candidate when there are fewer.
    pub drawn: u32,
    pub reviewed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages(n: u32) -> Vec<(String, u32)> {
        let doc = DocumentId(uuid::Uuid::nil());
        (1..=n)
            .map(|p| {
                (
                    page_identity(doc, Task::TextExtraction, &PieceRef::page(p)),
                    p,
                )
            })
            .collect()
    }

    #[test]
    fn the_same_seed_draws_the_same_pages_whatever_their_order() {
        let a = draw(1957, 5, pages(40));
        let mut reversed = pages(40);
        reversed.reverse();
        assert_eq!(a, draw(1957, 5, reversed));
        assert_eq!(a.len(), 5);
    }

    #[test]
    fn another_seed_draws_other_pages() {
        assert_ne!(draw(1957, 5, pages(40)), draw(1958, 5, pages(40)));
    }

    #[test]
    fn the_draw_is_fixed_across_systems() {
        // The ranks are SHA-256 digests of text: the same on every system and every build. These
        // pages are what seed 1957 gives, and must stay so for a cited sample to come back.
        let doc = DocumentId(uuid::Uuid::nil());
        let identity = page_identity(doc, Task::TextExtraction, &PieceRef::page(3));
        assert_eq!(
            identity,
            "00000000-0000-0000-0000-000000000000/text_extraction/page:3"
        );
        assert_eq!(draw(1957, 3, pages(40)), [11, 28, 25]);
    }

    #[test]
    fn a_larger_sample_keeps_the_smaller_one_and_a_page_offered_twice_counts_once() {
        let small = draw(7, 4, pages(30));
        let large = draw(7, 8, pages(30));
        assert_eq!(&large[..4], &small[..]);
        let mut twice = pages(3);
        twice.extend(pages(3));
        assert_eq!(draw(7, 10, twice).len(), 3);
    }

    #[test]
    fn a_sample_is_at_least_one_page_and_at_most_the_limit() {
        let now = Timestamp(0);
        assert_eq!(SampleSpec::new(0, Some(1), now).size, 1);
        assert_eq!(SampleSpec::new(5000, Some(1), now).size, MAX_SAMPLE);
        assert!(SampleSpec::new(5, None, now).seed < 1_000_000);
    }
}
