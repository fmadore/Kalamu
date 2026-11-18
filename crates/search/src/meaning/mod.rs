//! The search by meaning and the map of a project's documents, from the vectors of the text read
//! from them: extractions, transcriptions, corrections and translations, as the review shows
//! them. Analyses' answers are left out.
//!
//! Each passage's text is cut into chunks ([`chunks`]); each chunk's vector, made by an
//! embedding model through the engine, is kept in a [`VectorIndex`] with the stamp of the
//! passage it was cut from. Here, nothing calls a model: a search is given the question's
//! vector, ranks every current chunk by its dot product with it and lists documents by their
//! best chunk, each with a few of its best passages, so a long book is neither diluted nor
//! favoured against a one-page letter. A document's point on the map is the mean of its
//! chunks' vectors, weighted by their length, from one reading of it.

use std::sync::Mutex;

use kalamu_core::{
    CollectionId, DocumentId, EmbeddingSpace, JobId, PassageKey, SetupId, SourceId, Task,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Hit, ResultPolicy, ReviewFilter};

mod anchor;
mod cache;
pub mod chunks;
mod map;
mod plan;
mod rank;
mod scope;
mod tokens;

pub use map::{MIN_MAP, project_2d};
pub use rank::document_vector;
pub use tokens::estimated_tokens;

pub(crate) use cache::Cache;
#[cfg(test)]
pub(crate) use cache::dot;

/// The most documents listed for a question.
pub const MAX_DOCUMENTS: usize = 50;
/// The most passages shown for each document.
pub const MAX_PASSAGES: usize = 3;
/// The most documents named as like another.
pub const MAX_NEIGHBOURS: usize = 10;

/// A search by meaning, with the word search's filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningRequest {
    /// The question or description, in the researcher's words.
    pub query: String,
    pub collection: Option<CollectionId>,
    pub tag: Option<String>,
    /// One of the tasks that read text; none for all of them.
    pub task: Option<Task>,
    pub review: ReviewFilter,
    pub results: ResultPolicy,
}

/// What a search by meaning found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningView {
    pub space: EmbeddingSpace,
    /// The closest documents, closest first, at most [`MAX_DOCUMENTS`].
    pub documents: Vec<MeaningDocument>,
    /// Documents that had some chunk compared, of which the closest are listed.
    pub compared: u32,
    pub coverage: MeaningCoverage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningDocument {
    pub document_id: DocumentId,
    pub title: String,
    /// Its best chunk's similarity to the question, from -1 to 1: not a probability.
    pub similarity: f32,
    /// Its best passages, best first, none overlapping, at most [`MAX_PASSAGES`].
    pub passages: Vec<MeaningPassage>,
}

/// One chunk close to the question, opening the review where it begins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningPassage {
    pub job_id: JobId,
    pub job_name: String,
    pub setup_id: SetupId,
    pub setup_label: String,
    pub source_id: SourceId,
    pub task: Task,
    pub preferred: bool,
    pub similarity: f32,
    /// Where it stands, as a word search's match: `words` holds the chunk's opening words, and
    /// `term` and `nth` what the review searches for to open at its start.
    pub hit: Hit,
}

/// What a search by meaning or the map covered, so that finding nothing is not read as absence.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningCoverage {
    /// Documents in the collection or tag searched, or the whole project.
    pub documents: u32,
    /// Documents with some text compared.
    pub compared: u32,
    /// Documents with text read, none of it prepared for this model yet or still current.
    pub not_prepared: u32,
    /// Documents with nothing read yet of the kinds searched.
    pub unread: u32,
    /// Pieces read in the results searched.
    pub pieces: u32,
    /// Pieces prepared and current.
    pub pieces_prepared: u32,
    /// Pieces never prepared for this model, or only in part.
    pub pieces_missing: u32,
    /// Pieces prepared from a text since corrected: left out until prepared again.
    pub pieces_changed: u32,
    /// Pieces of the results searched not read: still to do, or failed.
    pub pieces_unread: u32,
    /// Documents where a task has several results and none is preferred.
    pub without_preference: u32,
}

/// What preparing a project's text for a model would embed, before anything is sent.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PreparationCounts {
    /// Pieces read in the results prepared: each document's preferred reading, or every
    /// reading where none is preferred.
    pub pieces: u32,
    /// Pieces already prepared and current.
    pub prepared: u32,
    /// Pieces never prepared, or only in part.
    pub missing: u32,
    /// Pieces prepared from a text since corrected.
    pub changed: u32,
    /// Chunks to embed.
    pub chunks: u32,
    /// Their characters.
    #[ts(type = "number")]
    pub chars: u64,
    /// The tokens they would be billed, estimated from their characters.
    #[ts(type = "number")]
    pub tokens: u64,
    /// Chunks left out for having almost no letters (a page number, a stamp).
    pub skipped: u32,
}

/// A chunk to embed, with what its vector is stored under.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingChunk {
    pub key: PassageKey,
    pub chunk: u32,
    pub of: u32,
    pub stamp: String,
    pub content: String,
    pub start: u32,
    pub end: u32,
    pub text: String,
}

/// What preparing a project's text needs: the chunks to embed, and the passages with nothing
/// worth embedding, recorded at once as such.
#[derive(Debug, Clone, Default)]
pub struct Preparation {
    pub counts: PreparationCounts,
    pub pending: Vec<PendingChunk>,
    pub empty: Vec<kalamu_core::ChunkVector>,
}

/// The map of a project's documents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MapView {
    pub space: EmbeddingSpace,
    /// One point per document placed; none when fewer than [`MIN_MAP`] are.
    pub points: Vec<MapPoint>,
    /// Documents placed, even when too few to draw.
    pub placed: u32,
    /// Documents in the project.
    pub documents: u32,
    /// Documents not placed: nothing read yet.
    pub unread: u32,
    /// Documents not placed: their reading not prepared for this model.
    pub not_prepared: u32,
    /// Documents not placed: their reading corrected since it was prepared.
    pub changed: u32,
    /// Documents with several readings and none preferred: the first one is placed.
    pub without_preference: u32,
    /// How much of the differences between the documents' vectors the two directions of the
    /// map show, from 0 to 1.
    pub shown: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MapPoint {
    pub document_id: DocumentId,
    pub title: String,
    /// From 0 to 1.
    pub x: f32,
    pub y: f32,
    pub collections: Vec<CollectionId>,
    pub tags: Vec<String>,
    /// Chunks averaged into its point.
    pub chunks: u32,
}

/// A document like another, by their documents' vectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Neighbour {
    pub document_id: DocumentId,
    pub title: String,
    pub similarity: f32,
}

/// Vectors kept in memory between two searches, made again when the index changes.
#[derive(Default)]
pub(crate) struct Memory(pub(crate) Mutex<Option<Cache>>);
