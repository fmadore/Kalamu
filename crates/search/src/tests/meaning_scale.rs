//! The search by meaning's exact scan at the size of a large project: 100,000 chunks of 768
//! numbers, read from the vector index and compared with one question.
//! `cargo test -p kalamu-search --release -- --ignored --nocapture meaning_scale`.

use std::time::Instant;

use kalamu_core::{ChunkVector, EmbeddingSpace, PassageKey, VectorIndex};

use super::*;
use crate::meaning::dot;

const CHUNKS: usize = 100_000;
const DIMENSIONS: usize = 768;

/// Numbers spread evenly, the same every run, made of length 1.
fn vector(seed: u64) -> Vec<f32> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut v: Vec<f32> = (0..DIMENSIONS)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 40) as f32 / (1u64 << 24) as f32 - 0.5
        })
        .collect();
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    v.iter_mut().for_each(|x| *x /= norm);
    v
}

#[test]
#[ignore = "measures 100,000 chunks of 768 numbers; run on demand"]
fn meaning_scale() {
    let index = SqliteVectors::in_memory().unwrap();
    let space = EmbeddingSpace {
        provider: ProviderId::new("fake"),
        model: "large".into(),
        dimensions: DIMENSIONS as u32,
    };
    let jobs: Vec<JobId> = (0..10).map(|_| JobId::new()).collect();
    let started = Instant::now();
    for (j, job) in jobs.iter().enumerate() {
        let setup = SetupId::new();
        let source = SourceId::new();
        let chunks: Vec<ChunkVector> = (0..CHUNKS / jobs.len())
            .map(|i| ChunkVector {
                key: PassageKey {
                    job_id: *job,
                    setup_id: setup,
                    source_id: source,
                    index: i as u32,
                },
                chunk: 0,
                of: 1,
                stamp: "s".into(),
                content: "c".into(),
                start: 0,
                end: 1200,
                vector: vector((j * CHUNKS + i) as u64),
            })
            .collect();
        index.put(&space, &chunks).unwrap();
    }
    println!("written: {:?}", started.elapsed());

    let started = Instant::now();
    let rows = index.vectors(&space, &jobs).unwrap();
    let mut matrix = Vec::with_capacity(rows.len() * DIMENSIONS);
    for r in &rows {
        matrix.extend_from_slice(&r.vector);
    }
    println!(
        "read into memory: {:?} ({} chunks)",
        started.elapsed(),
        rows.len()
    );
    assert_eq!(rows.len(), CHUNKS);

    let query = vector(u64::MAX);
    let mut best = Vec::new();
    let mut times = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        let mut scores: Vec<(f32, usize)> = matrix
            .as_chunks::<DIMENSIONS>()
            .0
            .iter()
            .enumerate()
            .map(|(i, v)| (dot(&query, v), i))
            .collect();
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        best = scores[..10].to_vec();
        times.push(started.elapsed());
    }
    println!("scan and sort, five times: {times:?}");
    assert!(best[0].0 >= best[9].0);
}
