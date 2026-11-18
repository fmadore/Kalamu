//! The map: each document's vector projected onto the two directions along which the project's
//! documents differ most (their first two principal components), found by power iteration from
//! a fixed start, so the same vectors always give the same map.
//!
//! No crate does this small and current enough to add: the pure-Rust UMAP and t-SNE crates on
//! crates.io pull in parallel runtimes, sparse matrices and progress bars, or are no longer
//! kept up. A principal-component map shows the broad lines of a collection faithfully, its
//! distances are honest along the two directions it keeps, and it needs nothing; what it loses
//! against UMAP or t-SNE is local structure: small groups of close documents that differ along
//! other directions are drawn on top of each other. The list of nearest documents beside the
//! map is computed on the full vectors, so it says what the map cannot.

use std::collections::HashMap;

use kalamu_core::{CollectionId, DocumentId, EmbeddingSpace, ProjectId};

use super::cache::dot;
use super::scope::Status;
use super::{MapPoint, MapView};
use crate::{Search, SearchError};

/// Fewer documents placed than this and no map is drawn: a sentence says why.
pub const MIN_MAP: usize = 5;

const ITERATIONS: usize = 200;

/// A fixed start for the power iteration: the same numbers every time, spread over every
/// direction.
fn start(dimensions: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..dimensions)
        .map(|_| {
            // A linear congruential generator (Knuth's MMIX constants).
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((state >> 40) as f32 / (1u64 << 24) as f32) - 0.5
        })
        .collect()
}

fn normalise(v: &mut [f32]) -> f32 {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter_mut().for_each(|x| *x /= norm);
    }
    norm
}

/// The direction of greatest variance of the rows of `centred`, orthogonal to `away`, and the
/// variance along it.
fn component(centred: &[Vec<f32>], away: &[Vec<f32>], seed: u64) -> (Vec<f32>, f32) {
    let dimensions = centred.first().map_or(0, Vec::len);
    let mut v = start(dimensions, seed);
    let deflate = |v: &mut Vec<f32>| {
        for a in away {
            let along = dot(v, a);
            v.iter_mut().zip(a).for_each(|(x, y)| *x -= along * y);
        }
    };
    deflate(&mut v);
    normalise(&mut v);
    let mut value = 0.0;
    for _ in 0..ITERATIONS {
        // v ← Xᵀ X v
        let mut next = vec![0f32; dimensions];
        for row in centred {
            let along = dot(row, &v);
            next.iter_mut().zip(row).for_each(|(n, x)| *n += along * x);
        }
        deflate(&mut next);
        value = normalise(&mut next);
        v = next;
    }
    // The sign of a direction is arbitrary: its largest number is made positive, so the map
    // does not flip from one run to the next.
    if let Some(big) = v.iter().copied().max_by(|a, b| {
        a.abs()
            .partial_cmp(&b.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    }) && big < 0.0
    {
        v.iter_mut().for_each(|x| *x = -*x);
    }
    (v, value / centred.len().max(1) as f32)
}

/// Each vector's place on the map, both coordinates from 0 to 1, and how much of the variance
/// among the vectors the two directions show, from 0 to 1.
pub fn project_2d(vectors: &[Vec<f32>]) -> (Vec<(f32, f32)>, f32) {
    let n = vectors.len();
    if n == 0 {
        return (Vec::new(), 0.0);
    }
    let dimensions = vectors[0].len();
    let mut mean = vec![0f32; dimensions];
    for v in vectors {
        mean.iter_mut().zip(v).for_each(|(m, x)| *m += x / n as f32);
    }
    let centred: Vec<Vec<f32>> = vectors
        .iter()
        .map(|v| v.iter().zip(&mean).map(|(x, m)| x - m).collect())
        .collect();
    let total: f32 = centred
        .iter()
        .map(|v| v.iter().map(|x| x * x).sum::<f32>())
        .sum::<f32>()
        / n as f32;
    let (first, a) = component(&centred, &[], 1);
    let (second, b) = component(&centred, std::slice::from_ref(&first), 2);
    let raw: Vec<(f32, f32)> = centred
        .iter()
        .map(|v| (dot(v, &first), dot(v, &second)))
        .collect();
    // Into [0, 1], one scale for both axes so that distances keep their proportions.
    let (min_x, max_x) = bounds(raw.iter().map(|p| p.0));
    let (min_y, max_y) = bounds(raw.iter().map(|p| p.1));
    let span = (max_x - min_x).max(max_y - min_y);
    let place = |value: f32, min: f32, max: f32| {
        if span <= f32::EPSILON {
            0.5
        } else {
            (value - min) / span + (1.0 - (max - min) / span) / 2.0
        }
    };
    let points = raw
        .iter()
        .map(|&(x, y)| (place(x, min_x, max_x), place(y, min_y, max_y)))
        .collect();
    let shown = if total > 0.0 {
        ((a + b) / total).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (points, shown)
}

fn bounds(values: impl Iterator<Item = f32>) -> (f32, f32) {
    values.fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
        (lo.min(v), hi.max(v))
    })
}

impl Search {
    /// The map of the project's documents in `space`, one point per document placed.
    pub fn meaning_map(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        max_chars: usize,
        stop: &dyn Fn() -> bool,
    ) -> Result<MapView, SearchError> {
        let docs = self.document_vectors(project, space, max_chars, stop)?;
        let placed: HashMap<DocumentId, usize> = docs
            .vectors
            .iter()
            .enumerate()
            .map(|(i, (d, _, _))| (*d, i))
            .collect();
        let mut view = MapView {
            space: space.clone(),
            points: Vec::new(),
            placed: placed.len() as u32,
            documents: docs.scope.documents.len() as u32,
            unread: 0,
            not_prepared: 0,
            changed: 0,
            without_preference: docs.scope.without_preference.len() as u32,
            shown: 0.0,
        };
        // Why each document left out is.
        for d in &docs.scope.documents {
            if placed.contains_key(&d.id) {
                continue;
            }
            let reading = docs.scope.chosen.iter().find(|r| r.document_id == d.id);
            let pieces: Vec<_> = docs
                .states
                .pieces
                .iter()
                .filter(|p| {
                    reading.is_some_and(|r| {
                        (p.key.job_id, p.key.setup_id, p.key.source_id)
                            == (r.job_id, r.setup_id, r.source_id)
                    })
                })
                .collect();
            if pieces.is_empty() {
                view.unread += 1;
            } else if pieces.iter().any(|p| p.status == Status::Changed) {
                view.changed += 1;
            } else {
                view.not_prepared += 1;
            }
        }
        if placed.len() < MIN_MAP {
            return Ok(view);
        }
        let mut members: HashMap<DocumentId, Vec<CollectionId>> = HashMap::new();
        for (c, d) in self.projects.members(project)? {
            members.entry(d).or_default().push(c);
        }
        let vectors: Vec<Vec<f32>> = docs.vectors.iter().map(|(_, v, _)| v.clone()).collect();
        let (points, shown) = project_2d(&vectors);
        view.shown = shown;
        for ((document_id, _, chunks), (x, y)) in docs.vectors.iter().zip(points) {
            let document = docs.scope.documents.iter().find(|d| d.id == *document_id);
            view.points.push(MapPoint {
                document_id: *document_id,
                title: document.map(|d| d.title.clone()).unwrap_or_default(),
                x,
                y,
                collections: members.get(document_id).cloned().unwrap_or_default(),
                tags: document.map(|d| d.tags.clone()).unwrap_or_default(),
                chunks: *chunks,
            });
        }
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_same_vectors_give_the_same_map_and_the_main_direction_comes_first() {
        // Points spread far along the first axis, a little along the second.
        let vectors: Vec<Vec<f32>> = (0..12)
            .map(|i| {
                let i = i as f32;
                vec![i, (i * 0.7).sin() * 0.5, 0.01 * i, 0.0]
            })
            .collect();
        let (a, shown) = project_2d(&vectors);
        let (b, _) = project_2d(&vectors);
        assert_eq!(a, b);
        assert!(shown > 0.95, "{shown}");
        // Ordered along x as along the first number.
        for pair in a.windows(2) {
            assert!(pair[1].0 > pair[0].0);
        }
        assert!(
            a.iter()
                .all(|(x, y)| (0.0..=1.0).contains(x) && (0.0..=1.0).contains(y))
        );
        assert_eq!(project_2d(&[]).0, Vec::new());
        assert_eq!(project_2d(&[vec![1.0, 0.0]]).0, vec![(0.5, 0.5)]);
    }
}
