//! Project search over a real job database and index: passages across documents and
//! recordings, corrections and names replacing the model's text, preferences, links removed,
//! accents and vowel marks, repeated phrases and several set-ups, and an index made again.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use kalamu_core::{
    AnalysisSchema, Collection, CollectionId, Correction, Document, DocumentId, DocumentLink,
    Field, FieldAnswer, FieldKind, FieldValue, InputKind, Job, JobId, JobSettings, JobStatus,
    JobStore, OutputSettings, PieceRef, PieceState, PieceStatus, ProcessOutput, Project,
    ProjectStore, PromptSnapshot, ProviderId, SearchIndex, Segment, Setup, SetupId, SetupOptions,
    SourceFile, SourceId, SpeakerNames, StoredResult, Task, Timestamp,
};
use kalamu_store::{SqliteIndex, SqliteStore};

use super::*;

mod index;
mod kinds;
mod scale;

struct Fixture {
    search: Search,
    store: Arc<SqliteStore>,
    index: Arc<SqliteIndex>,
    project: ProjectId,
}

fn never() -> bool {
    false
}

fn source(name: &str, kind: InputKind) -> SourceFile {
    SourceFile {
        id: SourceId::new(),
        path: format!(r"C:\archive\{name}"),
        file_name: name.into(),
        sha256: name.into(),
        size_bytes: 1,
        modified_ms: None,
        kind,
        page_count: Some(1),
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        title: None,
        derived_from: None,
    }
}

fn setup(label: &str, schema: Option<AnalysisSchema>) -> Setup {
    Setup {
        id: SetupId::new(),
        label: label.into(),
        provider: ProviderId::new("gemini"),
        model: label.into(),
        prompt: PromptSnapshot {
            preset_id: None,
            preset_name: "Preset".into(),
            template: String::new(),
            text: String::new(),
            user_prompt: String::new(),
            schema,
        },
        options: SetupOptions::default(),
        merge: None,
        pages_cap: None,
    }
}

/// What one piece holds: the text, or an analysis's answers, and its timed lines.
#[derive(Clone, Default)]
struct Read {
    piece: Option<PieceRef>,
    text: String,
    segments: Vec<Segment>,
    fields: Vec<FieldAnswer>,
    failed: bool,
}

fn page(text: &str) -> Read {
    Read {
        text: text.into(),
        ..Read::default()
    }
}

fn request(query: &str) -> SearchRequest {
    SearchRequest {
        query: query.into(),
        collection: None,
        tag: None,
        task: None,
        review: ReviewFilter::Any,
        results: ResultChoice::Preferred,
        generated: false,
    }
}

impl Fixture {
    fn new() -> Self {
        Self::on(
            Arc::new(SqliteStore::in_memory().unwrap()),
            Arc::new(SqliteIndex::in_memory().unwrap()),
        )
    }

    fn on(store: Arc<SqliteStore>, index: Arc<SqliteIndex>) -> Self {
        let project = Project {
            id: ProjectId::new(),
            name: "Presse islamique".into(),
            description: String::new(),
            archived: false,
            created_at: Timestamp(1),
            updated_at: Timestamp(1),
        };
        store.put_project(&project).unwrap();
        Self {
            search: Search::new(store.clone(), store.clone(), index.clone()),
            store,
            index,
            project: project.id,
        }
    }

    fn document(&self, title: &str, kind: InputKind) -> (DocumentId, SourceFile) {
        let source = source(title, kind);
        let document = Document {
            id: DocumentId::new(),
            project_id: self.project,
            title: title.into(),
            source: source.clone(),
            created_at: Timestamp::now(),
            tags: Vec::new(),
        };
        self.store.put_document(&document).unwrap();
        (document.id, source)
    }

    /// A job of `task` over one document, one set-up per list of pieces read, tied to it.
    fn job(
        &self,
        task: Task,
        document: &(DocumentId, SourceFile),
        setups: &[(&str, Vec<Read>)],
        schema: Option<AnalysisSchema>,
    ) -> Job {
        let job = Job {
            id: JobId::new(),
            name: format!("{task:?}"),
            task,
            created_at: Timestamp::now(),
            sources: vec![document.1.clone()],
            setups: setups
                .iter()
                .map(|(label, _)| setup(label, schema.clone()))
                .collect(),
            settings: JobSettings::default(),
            output: OutputSettings::default(),
            status: JobStatus::Completed,
        };
        self.store.create_job(&job).unwrap();
        for (setup, (_, reads)) in job.setups.iter().zip(setups) {
            for (index, read) in reads.iter().enumerate() {
                let key = format!("{}-{}-{index}", job.id, setup.id);
                if !read.failed {
                    self.store
                        .put_result(&StoredResult {
                            cache_key: key.clone(),
                            provider: ProviderId::new("gemini"),
                            model: setup.model.clone(),
                            output: ProcessOutput {
                                text: read.text.clone(),
                                segments: read.segments.clone(),
                                fields: read.fields.clone(),
                                ..ProcessOutput::default()
                            },
                            request: Vec::new(),
                            cost_usd: None,
                            duration_ms: 1,
                            created_at: Timestamp::now(),
                        })
                        .unwrap();
                }
                self.store
                    .upsert_piece(&PieceState {
                        job_id: job.id,
                        setup_id: setup.id,
                        source_id: document.1.id,
                        index: index as u32,
                        piece: read
                            .piece
                            .clone()
                            .unwrap_or_else(|| PieceRef::page(index as u32 + 1)),
                        status: if read.failed {
                            PieceStatus::Failed
                        } else {
                            PieceStatus::Done
                        },
                        cache_key: (!read.failed).then_some(key),
                        error: None,
                        attempts: 1,
                        updated_at: Timestamp::now(),
                    })
                    .unwrap();
            }
        }
        self.store
            .put_link(&DocumentLink {
                document_id: document.0,
                job_id: job.id,
                source_id: document.1.id,
                linked_at: Timestamp::now(),
            })
            .unwrap();
        job
    }

    fn extraction(&self, document: &(DocumentId, SourceFile), pages: &[&str]) -> Job {
        self.job(
            Task::TextExtraction,
            document,
            &[("Gemini", pages.iter().map(|p| page(p)).collect())],
            None,
        )
    }

    fn correct(&self, job: &Job, setup: usize, index: u32, text: Option<&str>, reviewed: bool) {
        self.store
            .put_correction(&Correction {
                job_id: job.id,
                setup_id: job.setups[setup].id,
                source_id: job.sources[0].id,
                index,
                text: text.map(str::to_string),
                fields: None,
                reviewed,
                // Each save is later than the one before, as the engine stamps it.
                updated_at: Timestamp(Timestamp::now().0 + i64::from(index) + 1),
            })
            .unwrap();
    }

    fn find(&self, request: &SearchRequest) -> SearchView {
        self.search.search(self.project, request, &never).unwrap()
    }
}

/// Every match listed, as `title | piece | nth | words`.
fn listed(view: &SearchView) -> Vec<String> {
    view.documents
        .iter()
        .flat_map(|d| {
            d.results.iter().flat_map(move |r| {
                r.hits.iter().map(move |h| {
                    format!(
                        "{} | {} | {} | {}",
                        d.title,
                        h.piece.label(),
                        h.nth,
                        h.words
                    )
                })
            })
        })
        .collect()
}

#[test]
fn one_query_finds_passages_across_documents_and_recordings() {
    let f = Fixture::new();
    let issue = f.document("Le Musulman 12", InputKind::Pdf);
    let other = f.document("Le Musulman 13", InputKind::Pdf);
    let interview = f.document("Entretien Bobo", InputKind::Audio);
    f.extraction(
        &issue,
        &[
            "Éditorial",
            "L'Union culturelle musulmane tient son congrès.",
        ],
    );
    f.extraction(&other, &["Rien sur ce sujet."]);
    let start_ms = 20 * 60 * 1000;
    f.job(
        Task::AudioTranscription,
        &interview,
        &[(
            "Whisper",
            vec![
                page("Avant tout."),
                Read {
                    piece: Some(PieceRef::Segment {
                        index: 1,
                        start_ms,
                        end_ms: start_ms + 60_000,
                    }),
                    text: "Speaker 1: Bonjour.\nSpeaker 2: J'étais à l'Union Culturelle \
                       Musulmane en 1975."
                        .into(),
                    segments: vec![
                        Segment {
                            start_ms: 0,
                            end_ms: 2_000,
                            speaker: Some("Speaker 1".into()),
                            text: "Bonjour.".into(),
                        },
                        Segment {
                            start_ms: 2_000,
                            end_ms: 9_000,
                            speaker: Some("Speaker 2".into()),
                            text: "J'étais à l'Union Culturelle Musulmane en 1975.".into(),
                        },
                    ],
                    ..Read::default()
                },
            ],
        )],
        None,
    );

    let view = f.find(&request("union culturelle"));
    assert_eq!(
        listed(&view),
        vec![
            "Entretien Bobo | 00:20:00 | 0 | Union Culturelle",
            "Le Musulman 12 | 2 | 0 | Union culturelle",
        ]
    );
    let recording = &view.documents[0].results[0];
    assert_eq!(recording.source_id, interview.1.id);
    assert_eq!(recording.hits[0].index, 1);
    assert_eq!(recording.hits[0].at_ms, Some(start_ms + 2_000));
    assert_eq!(view.documents[1].results[0].hits[0].before, "L'");
    assert_eq!(view.matches, 2);
    assert_eq!(view.coverage.documents, 3);
    assert_eq!(view.coverage.searched, 3);
    assert_eq!(view.coverage.pieces, 5);
}

#[test]
fn corrections_and_speaker_names_replace_the_models_text() {
    let f = Fixture::new();
    let letter = f.document("Lettre", InputKind::Pdf);
    let job = f.extraction(&letter, &["Le Cheikh Tidjane Sy est venu."]);
    assert_eq!(f.find(&request("tidjane")).matches, 1);

    // The researcher corrects the reading: the model's spelling is no longer found.
    f.correct(&job, 0, 0, Some("Le Cheikh Tidiane Sy est venu."), false);
    assert_eq!(f.find(&request("tidjane")).matches, 0);
    let view = f.find(&request("tidiane"));
    let hit = &view.documents[0].results[0].hits[0];
    assert!(hit.corrected && !hit.reviewed);

    // Marked reviewed: the review filter sees it.
    f.correct(&job, 0, 0, Some("Le Cheikh Tidiane Sy est venu."), true);
    let mut reviewed = request("tidiane");
    reviewed.review = ReviewFilter::NotReviewed;
    assert_eq!(f.find(&reviewed).matches, 0);
    reviewed.review = ReviewFilter::Reviewed;
    assert_eq!(f.find(&reviewed).matches, 1);

    // Speakers named in a recording are found by their names.
    let interview = f.document("Entretien", InputKind::Audio);
    let recording = f.job(
        Task::AudioTranscription,
        &interview,
        &[("Whisper", vec![page("Speaker 2: Je suis né à Kankan.")])],
        None,
    );
    assert_eq!(f.find(&request("Imam Ouédraogo")).matches, 0);
    f.store
        .put_speaker_names(
            recording.id,
            &SpeakerNames {
                setup_id: recording.setups[0].id,
                source_id: interview.1.id,
                names: [("Speaker 2".to_string(), "Imam Ouédraogo".to_string())].into(),
            },
        )
        .unwrap();
    assert_eq!(f.find(&request("imam ouedraogo")).matches, 1);
}

#[test]
fn accents_vowel_marks_and_repeated_phrases_are_each_found() {
    let f = Fixture::new();
    let doc = f.document("Al-Nour", InputKind::Pdf);
    f.extraction(
        &doc,
        &[
            "قَالَ الشَّيْخُ: الشيخ في المسجد، والشيخ يتكلم.",
            "Père Éric, puis le PERE ERIC, puis père éric.",
        ],
    );
    let view = f.find(&request("الشيخ"));
    assert_eq!(
        listed(&view),
        vec![
            "Al-Nour | 1 | 0 | الشَّيْخُ",
            "Al-Nour | 1 | 1 | الشيخ",
            "Al-Nour | 1 | 2 | الشيخ",
        ]
    );
    let view = f.find(&request("pere eric"));
    assert_eq!(view.matches, 3);
    let words: Vec<_> = view.documents[0].results[0]
        .hits
        .iter()
        .map(|h| h.words.as_str())
        .collect();
    assert_eq!(words, vec!["Père Éric", "PERE ERIC", "père éric"]);
    // Too short a query is refused rather than finding everything.
    assert!(matches!(
        f.search.search(f.project, &request("é "), &never),
        Err(SearchError::TooShort)
    ));
}

#[test]
fn several_set_ups_stay_results_of_one_document_and_preferences_choose() {
    let f = Fixture::new();
    let doc = f.document("Le Musulman 12", InputKind::Pdf);
    let job = f.job(
        Task::TextExtraction,
        &doc,
        &[
            ("Gemini", vec![page("Mosquée de Bobo-Dioulasso")]),
            ("Mistral", vec![page("Mosquee de Bobo Dioulasso")]),
        ],
        None,
    );
    // No preference: both readings are searched, one document counted.
    let view = f.find(&request("mosquee de bobo"));
    assert_eq!(view.documents.len(), 1);
    assert_eq!(view.documents[0].results.len(), 2);
    assert_eq!(view.coverage.without_preference, 1);
    assert_eq!(view.coverage.searched, 1);

    // Preferred: only that reading, unless every result is asked for.
    f.store
        .set_preferred(
            doc.0,
            Task::TextExtraction,
            Some((job.id, job.setups[1].id)),
        )
        .unwrap();
    let view = f.find(&request("mosquee de bobo"));
    assert_eq!(view.documents[0].results.len(), 1);
    let result = &view.documents[0].results[0];
    assert!(result.preferred);
    assert_eq!(result.setup_label, "Mistral");
    assert_eq!(view.coverage.without_preference, 0);
    let mut all = request("mosquee de bobo");
    all.results = ResultChoice::All;
    assert_eq!(f.find(&all).documents[0].results.len(), 2);
}
