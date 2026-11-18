//! A project's vectors in one model's space, kept in memory between two searches as one block of
//! numbers, so that a question is compared with every chunk without reading the index again.
//! Made again whenever the index has been written since, or another project or model is asked.

use std::sync::MutexGuard;

use kalamu_core::{EmbeddingSpace, JobId, PassageKey};

use crate::{Search, SearchError};

/// What a chunk's vector is, without the vector.
pub(crate) struct Row {
    pub key: PassageKey,
    pub stamp: String,
    pub start: u32,
    pub end: u32,
}

pub(crate) struct Cache {
    space: EmbeddingSpace,
    jobs: Vec<JobId>,
    generation: u64,
    pub rows: Vec<Row>,
    /// Each row's vector, one after the other, `dimensions` numbers each.
    pub matrix: Vec<f32>,
    pub dimensions: usize,
}

impl Cache {
    pub fn vector(&self, row: usize) -> &[f32] {
        &self.matrix[row * self.dimensions..(row + 1) * self.dimensions]
    }
}

/// The dot product of two vectors of one length: their cosine, both being of length 1.
pub(crate) fn dot(a: &[f32], b: &[f32]) -> f32 {
    // Four sums side by side, which the compiler turns into vector instructions.
    let mut sums = [0f32; 4];
    let (a4, a_rest) = a.as_chunks::<4>();
    let (b4, b_rest) = b.as_chunks::<4>();
    for (x, y) in a4.iter().zip(b4) {
        sums[0] += x[0] * y[0];
        sums[1] += x[1] * y[1];
        sums[2] += x[2] * y[2];
        sums[3] += x[3] * y[3];
    }
    let rest: f32 = a_rest.iter().zip(b_rest).map(|(x, y)| x * y).sum();
    sums[0] + sums[1] + sums[2] + sums[3] + rest
}

impl Search {
    /// The vectors of `jobs` in `space`, read again from the index only when it changed.
    pub(crate) fn cached(
        &self,
        space: &EmbeddingSpace,
        jobs: &[JobId],
    ) -> Result<MutexGuard<'_, Option<Cache>>, SearchError> {
        let index = self.vector_index()?;
        let generation = index.generation();
        let mut memory = self
            .memory
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let fresh = memory.as_ref().is_some_and(|c| {
            c.generation == generation && c.space == *space && c.jobs.as_slice() == jobs
        });
        if !fresh {
            // Dropped first, so two projects' vectors are never held at once.
            *memory = None;
            let dimensions = space.dimensions as usize;
            let chunks = index.vectors(space, jobs)?;
            let mut rows = Vec::with_capacity(chunks.len());
            let mut matrix = Vec::with_capacity(chunks.len() * dimensions);
            for c in chunks {
                if c.vector.len() != dimensions {
                    continue;
                }
                matrix.extend_from_slice(&c.vector);
                rows.push(Row {
                    key: c.key,
                    stamp: c.stamp,
                    start: c.start,
                    end: c.end,
                });
            }
            *memory = Some(Cache {
                space: space.clone(),
                jobs: jobs.to_vec(),
                generation,
                rows,
                matrix,
                dimensions,
            });
        }
        Ok(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::dot;

    #[test]
    fn dot_adds_every_product_whatever_the_length() {
        let a: Vec<f32> = (0..11).map(|i| i as f32).collect();
        let b = vec![1.0f32; 11];
        assert_eq!(dot(&a, &b), 55.0);
        assert_eq!(dot(&[], &[]), 0.0);
    }
}
