//! The search by meaning and the map over a real job database and vector index, vectors from
//! the fake embedder (each text's bag of words): preparing, ranking long and short documents,
//! stale chunks after a correction and kept ones after a page marked reviewed, filters and
//! coverage, one reading per document, the map and the documents like another.

use kalamu_core::test_support::fake_vector;
use kalamu_core::{ChunkVector, EmbeddingSpace, VectorIndex};

use super::*;
use crate::meaning::{MIN_MAP, document_vector};

const DIMENSIONS: u32 = 256;
const MAX_CHARS: usize = 2000;

fn space() -> EmbeddingSpace {
    EmbeddingSpace {
        provider: ProviderId::new("fake"),
        model: "words".into(),
        dimensions: DIMENSIONS,
    }
}

fn ask(query: &str) -> MeaningRequest {
    MeaningRequest {
        query: query.into(),
        collection: None,
        tag: None,
        task: None,
        review: ReviewFilter::Any,
        results: ResultPolicy::Preferred,
    }
}

impl Fixture {
    /// Embeds what preparing the project needs, as the engine would, and returns how many
    /// chunks it embedded.
    fn prepare(&self) -> u32 {
        let preparation = self
            .search
            .meaning_preparation(self.project, &space(), MAX_CHARS, &never)
            .unwrap();
        self.vectors.put(&space(), &preparation.empty).unwrap();
        let chunks: Vec<ChunkVector> = preparation
            .pending
            .iter()
            .map(|c| ChunkVector {
                key: c.key,
                chunk: c.chunk,
                of: c.of,
                stamp: c.stamp.clone(),
                content: c.content.clone(),
                start: c.start,
                end: c.end,
                vector: fake_vector(&c.text, DIMENSIONS as usize),
            })
            .collect();
        self.vectors.put(&space(), &chunks).unwrap();
        chunks.len() as u32
    }

    fn by_meaning(&self, request: &MeaningRequest) -> MeaningView {
        let query = fake_vector(&request.query, DIMENSIONS as usize);
        self.search
            .meaning_search(self.project, request, &space(), MAX_CHARS, &query, &never)
            .unwrap()
    }

    fn counts(&self) -> PreparationCounts {
        self.search
            .meaning_preparation(self.project, &space(), MAX_CHARS, &never)
            .unwrap()
            .counts
    }
}

fn titles(view: &MeaningView) -> Vec<&str> {
    view.documents.iter().map(|d| d.title.as_str()).collect()
}

const MOSQUE: &str = "The new mosque of Bobo-Dioulasso was built by the community of traders, \
                      who gathered every Friday for prayer in the mosque.";

fn filler(i: usize) -> String {
    format!(
        "Page {i}. The colonial administration counted the tax receipts of the cercle, the \
         cattle levied, the roads repaired by forced labour and the harvest of cotton sold."
    )
}

#[test]
fn a_long_book_is_neither_diluted_nor_favoured_against_a_short_letter() {
    let f = Fixture::new();
    let letter = f.document("Letter on the mosque", InputKind::Pdf);
    let book = f.document("Annual report", InputKind::Pdf);
    let other = f.document("Cotton prices", InputKind::Pdf);
    f.extraction(&letter, &[MOSQUE]);
    let mut pages: Vec<String> = (1..=40).map(filler).collect();
    pages[23] = format!("{MOSQUE} The traders also opened a school.");
    let pages: Vec<&str> = pages.iter().map(String::as_str).collect();
    f.extraction(&book, &pages);
    f.extraction(
        &other,
        &["Cotton was sold at the market of Koudougou for a low price."],
    );

    assert_eq!(f.counts().chunks, 42);
    assert_eq!(f.prepare(), 42);
    assert_eq!(f.counts().chunks, 0, "nothing embedded twice");

    let view = f.by_meaning(&ask("the mosque of Bobo-Dioulasso and its traders"));
    // Both ranked above the unrelated one by their best chunk: the book's one page counts as
    // much as the whole letter.
    let mut first_two = titles(&view)[..2].to_vec();
    first_two.sort();
    assert_eq!(first_two, ["Annual report", "Letter on the mosque"]);
    let book_hit = view
        .documents
        .iter()
        .find(|d| d.title == "Annual report")
        .unwrap();
    assert_eq!(book_hit.passages[0].hit.index, 23);
    assert!(view.documents.iter().all(|d| d.passages.len() <= 3));
    assert!(view.documents[0].similarity > view.documents[2].similarity);
    assert_eq!(view.compared, 3);
    assert_eq!(view.coverage.documents, 3);
    assert_eq!(view.coverage.pieces, 42);
    assert_eq!(view.coverage.pieces_prepared, 42);
    // The passage opens the review where it begins.
    let letter_hit = view
        .documents
        .iter()
        .find(|d| d.title == "Letter on the mosque")
        .unwrap();
    let hit = &letter_hit.passages[0].hit;
    assert_eq!(hit.term, "The new mosque of Bobo-Dioulasso was");
    assert_eq!(hit.nth, 0);
    assert!(hit.words.starts_with("The new mosque"));
}

#[test]
fn a_correction_leaves_a_page_out_until_prepared_again_and_a_review_mark_does_not() {
    let f = Fixture::new();
    let doc = f.document("Letter", InputKind::Pdf);
    let job = f.extraction(&doc, &[MOSQUE, &filler(2)]);
    f.prepare();

    // Marked reviewed: the text is the same, the chunks stay current, nothing to embed.
    f.correct(&job, 0, 0, None, true);
    let counts = f.counts();
    assert_eq!((counts.prepared, counts.changed, counts.chunks), (2, 0, 0));
    let reviewed = f.by_meaning(&MeaningRequest {
        review: ReviewFilter::Reviewed,
        ..ask("mosque")
    });
    assert_eq!(reviewed.documents[0].passages.len(), 1);
    assert!(reviewed.documents[0].passages[0].hit.reviewed);

    // Corrected into other words: left out, counted, and offered again.
    f.correct(
        &job,
        0,
        1,
        Some("Nothing about taxes here: a list of pupils."),
        false,
    );
    let view = f.by_meaning(&ask("tax receipts of the cercle"));
    assert_eq!(view.coverage.pieces_changed, 1);
    assert!(view.documents[0].passages.iter().all(|p| p.hit.index != 1));
    let counts = f.counts();
    assert_eq!((counts.changed, counts.chunks), (1, 1));
    f.prepare();
    let view = f.by_meaning(&ask("list of pupils"));
    assert_eq!(view.coverage.pieces_changed, 0);
    assert_eq!(view.documents[0].passages[0].hit.index, 1);
    assert!(view.documents[0].passages[0].hit.corrected);
}

#[test]
fn coverage_says_what_was_not_prepared_or_not_read_and_filters_narrow_the_search() {
    let f = Fixture::new();
    let prepared = f.document("Prepared", InputKind::Pdf);
    let later = f.document("Added later", InputKind::Pdf);
    let unread = f.document("Not read", InputKind::Pdf);
    f.extraction(&prepared, &[MOSQUE]);
    f.job(
        Task::TextExtraction,
        &unread,
        &[(
            "Gemini",
            vec![Read {
                failed: true,
                ..Read::default()
            }],
        )],
        None,
    );
    f.prepare();
    f.extraction(&later, &[MOSQUE]);

    let view = f.by_meaning(&ask("mosque"));
    assert_eq!(titles(&view), ["Prepared"]);
    let c = &view.coverage;
    assert_eq!(
        (c.documents, c.compared, c.not_prepared, c.unread),
        (3, 1, 1, 1)
    );
    assert_eq!((c.pieces, c.pieces_missing, c.pieces_unread), (2, 1, 1));

    // A collection holding only the one added later: nothing compared, and said so.
    let collection = Collection {
        id: CollectionId::new(),
        project_id: f.project,
        name: "Later".into(),
        created_at: Timestamp(1),
    };
    f.store.put_collection(&collection).unwrap();
    f.store.add_member(collection.id, later.0).unwrap();
    let view = f.by_meaning(&MeaningRequest {
        collection: Some(collection.id),
        ..ask("mosque")
    });
    assert!(view.documents.is_empty());
    assert_eq!(
        (view.coverage.documents, view.coverage.not_prepared),
        (1, 1)
    );

    // A task with no results: nothing.
    let view = f.by_meaning(&MeaningRequest {
        task: Some(Task::Translation),
        ..ask("mosque")
    });
    assert!(view.documents.is_empty());
}

#[test]
fn a_question_of_another_length_is_refused() {
    let f = Fixture::new();
    let err = f
        .search
        .meaning_search(f.project, &ask("x"), &space(), MAX_CHARS, &[1.0], &never)
        .unwrap_err();
    assert!(matches!(err, SearchError::WrongLength));
}

#[test]
fn a_documents_vector_is_the_length_weighted_mean_of_its_chunks() {
    let v = document_vector([(&[1.0f32, 0.0][..], 3.0), (&[0.0f32, 1.0][..], 1.0)]).unwrap();
    assert!((v[0] - 0.9486833).abs() < 1e-5 && (v[1] - 0.3162278).abs() < 1e-5);
    assert!(document_vector(std::iter::empty()).is_none());
}

fn map_fixture() -> (Fixture, Vec<(DocumentId, SourceFile)>) {
    let f = Fixture::new();
    let texts = [
        ("Mosque A", MOSQUE.to_string()),
        ("Mosque B", format!("{MOSQUE} A second mosque.")),
        ("Tax 1", filler(1)),
        ("Tax 2", filler(2)),
        ("Tax 3", filler(3)),
        (
            "Cotton",
            "Cotton sold at the market of Koudougou.".to_string(),
        ),
    ];
    let mut docs = Vec::new();
    for (title, text) in texts {
        let doc = f.document(title, InputKind::Pdf);
        f.extraction(&doc, &[&text]);
        docs.push(doc);
    }
    (f, docs)
}

#[test]
fn the_map_places_one_point_per_document_the_same_each_time() {
    let (f, docs) = map_fixture();
    // A second reading of one document, not preferred: the first one is placed.
    f.extraction(
        &docs[5],
        &["Another reading entirely about mosques and prayer."],
    );
    let before = f
        .search
        .meaning_map(f.project, &space(), MAX_CHARS, &never)
        .unwrap();
    assert!(before.points.is_empty());
    assert_eq!((before.placed, before.not_prepared), (0, 6));

    f.prepare();
    let map = f
        .search
        .meaning_map(f.project, &space(), MAX_CHARS, &never)
        .unwrap();
    assert_eq!(map.points.len(), 6);
    assert_eq!(map.without_preference, 1);
    assert!(map.shown > 0.0 && map.shown <= 1.0);
    let again = f
        .search
        .meaning_map(f.project, &space(), MAX_CHARS, &never)
        .unwrap();
    assert_eq!(map, again);

    // The documents most like the first: the other about the mosque, then the rest.
    let near = f
        .search
        .meaning_neighbours(f.project, &space(), MAX_CHARS, docs[0].0, &never)
        .unwrap();
    assert_eq!(near.len(), 5);
    assert_eq!(near[0].title, "Mosque B");
    assert!(near.windows(2).all(|w| w[0].similarity >= w[1].similarity));
}

#[test]
fn too_few_documents_give_no_map_and_a_deleted_job_takes_its_vectors_away() {
    let (f, docs) = map_fixture();
    f.prepare();
    let job = f.store.list_jobs().unwrap()[0].id;
    f.store.delete_job(job).unwrap();
    f.search.forget_job(job).unwrap();
    assert!(!f.vectors.indexed_jobs().unwrap().contains(&job));
    for d in &docs[..2] {
        f.store.delete_document(d.0).unwrap();
    }
    let map = f
        .search
        .meaning_map(f.project, &space(), MAX_CHARS, &never)
        .unwrap();
    assert!(map.placed < MIN_MAP as u32);
    assert!(map.points.is_empty());
    assert_eq!(map.documents, 4);

    // The space's size, and its removal.
    let sizes = f.search.meaning_spaces(f.project).unwrap();
    assert_eq!(sizes.len(), 1);
    assert_eq!(sizes[0].space, space());
    f.search.remove_meaning_space(f.project, &space()).unwrap();
    assert!(f.search.meaning_spaces(f.project).unwrap().is_empty());
}

#[test]
fn a_blank_page_is_recorded_as_nothing_to_embed_and_not_asked_again() {
    let f = Fixture::new();
    let doc = f.document("Letter", InputKind::Pdf);
    f.extraction(&doc, &["— 3 —", MOSQUE]);
    let counts = f.counts();
    assert_eq!((counts.chunks, counts.skipped), (1, 1));
    f.prepare();
    let counts = f.counts();
    assert_eq!((counts.chunks, counts.prepared, counts.missing), (0, 2, 0));
}

#[test]
fn a_recordings_passage_opens_at_its_line() {
    let f = Fixture::new();
    let interview = f.document("Interview", InputKind::Audio);
    let start_ms = 60_000;
    let line = "We built the mosque of Bobo-Dioulasso with the traders, every Friday.";
    f.job(
        Task::AudioTranscription,
        &interview,
        &[(
            "Whisper",
            vec![Read {
                piece: Some(PieceRef::Segment {
                    index: 0,
                    start_ms,
                    end_ms: start_ms + 60_000,
                }),
                text: format!("Hello.\n{line}"),
                segments: vec![
                    Segment {
                        start_ms: 0,
                        end_ms: 2_000,
                        speaker: None,
                        text: "Hello.".into(),
                    },
                    Segment {
                        start_ms: 2_000,
                        end_ms: 9_000,
                        speaker: None,
                        text: line.into(),
                    },
                ],
                ..Read::default()
            }],
        )],
        None,
    );
    f.prepare();
    let view = f.by_meaning(&ask("mosque traders"));
    let hit = &view.documents[0].passages[0].hit;
    // The whole piece is one chunk: it opens at its start.
    assert_eq!(hit.term, "Hello.");
    assert_eq!(hit.at_ms, Some(start_ms));
}
