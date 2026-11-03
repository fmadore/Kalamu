//! The dictionary over a real job database: names gathered from several jobs through the
//! researcher's corrections, merges and their undoing, pairs kept apart, and what survives a
//! restart.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use kalamu_core::{
    AnalysisSchema, Collection, CollectionId, Correction, Document, DocumentId, DocumentLink,
    Field, FieldAnswer, FieldKind, FieldValue, InputKind, Job, JobId, JobSettings, JobStatus,
    JobStore, OutputSettings, PieceRef, PieceState, PieceStatus, ProcessOutput, Project,
    ProjectStore, PromptSnapshot, ProviderId, Setup, SetupId, SetupOptions, SourceFile, SourceId,
    StoredResult, Task, Timestamp,
};
use kalamu_store::SqliteStore;

use super::*;

mod export;
mod proposals;

struct Fixture {
    dictionary: Dictionary,
    store: Arc<SqliteStore>,
    project: ProjectId,
}

fn list(key: &str, label: &str) -> Field {
    Field {
        key: key.into(),
        label: label.into(),
        instructions: String::new(),
        kind: FieldKind::List { max: None },
        reason: false,
    }
}

fn source(name: &str) -> SourceFile {
    SourceFile {
        id: SourceId::new(),
        path: format!(r"C:\archive\{name}.pdf"),
        file_name: format!("{name}.pdf"),
        sha256: name.into(),
        size_bytes: 1,
        modified_ms: None,
        kind: InputKind::Pdf,
        page_count: Some(1),
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        title: None,
        derived_from: None,
    }
}

fn fixture_on(store: Arc<SqliteStore>) -> Fixture {
    let project = Project {
        id: ProjectId::new(),
        name: "Missions".into(),
        description: String::new(),
        archived: false,
        source_defaults: Default::default(),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    };
    store.put_project(&project).unwrap();
    Fixture {
        dictionary: Dictionary::new(store.clone(), store.clone(), store.clone()),
        project: project.id,
        store,
    }
}

fn fixture() -> Fixture {
    fixture_on(Arc::new(SqliteStore::in_memory().unwrap()))
}

/// What one set-up answered for one document: people, then places.
type Answer<'a> = (&'a [&'a str], &'a [&'a str]);

impl Fixture {
    fn document(&self, name: &str) -> (DocumentId, SourceFile) {
        let source = source(name);
        let document = Document {
            id: DocumentId::new(),
            project_id: self.project,
            title: name.into(),
            source: source.clone(),
            created_at: Timestamp::now(),
            tags: Vec::new(),
            source_description: Default::default(),
        };
        self.store.put_document(&document).unwrap();
        (document.id, source)
    }

    /// An analysis of `task` over the documents, one set-up per answer list, each document's
    /// one piece answered as given, and tied to the documents.
    fn analysis(&self, documents: &[(DocumentId, SourceFile)], answers: &[&[Answer<'_>]]) -> Job {
        let schema = AnalysisSchema {
            fields: vec![
                list("people", "People"),
                list("places", "Places"),
                Field {
                    key: "summary".into(),
                    label: "Summary".into(),
                    instructions: String::new(),
                    kind: FieldKind::Text,
                    reason: false,
                },
            ],
        };
        let setups: Vec<Setup> = answers
            .iter()
            .enumerate()
            .map(|(i, _)| Setup {
                id: SetupId::new(),
                label: format!("model-{i}"),
                provider: ProviderId::new("gemini"),
                model: format!("model-{i}"),
                prompt: PromptSnapshot {
                    preset_id: None,
                    preset_name: "People and places".into(),
                    template: String::new(),
                    text: String::new(),
                    user_prompt: String::new(),
                    schema: Some(schema.clone()),
                },
                options: SetupOptions::default(),
                merge: None,
                pages_cap: None,
            })
            .collect();
        let job = Job {
            id: JobId::new(),
            name: "People and places".into(),
            task: Task::Analysis,
            created_at: Timestamp::now(),
            sources: documents.iter().map(|(_, s)| s.clone()).collect(),
            setups,
            settings: JobSettings::default(),
            output: OutputSettings::default(),
            status: JobStatus::Completed,
        };
        self.store.create_job(&job).unwrap();
        for (setup, per_document) in job.setups.iter().zip(answers) {
            for ((document, source), (people, places)) in documents.iter().zip(*per_document) {
                let key = format!("{}-{}-{}", job.id, setup.id, source.id);
                let strings = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
                self.store
                    .put_result(&StoredResult {
                        cache_key: key.clone(),
                        provider: ProviderId::new("gemini"),
                        model: setup.model.clone(),
                        output: ProcessOutput {
                            fields: vec![
                                answer("people", FieldValue::List(strings(people))),
                                answer("places", FieldValue::List(strings(places))),
                                answer("summary", FieldValue::Text("A letter.".into())),
                            ],
                            ..ProcessOutput::default()
                        },
                        request: Vec::new(),
                        cost_usd: Some(0.01),
                        duration_ms: 1,
                        created_at: Timestamp::now(),
                    })
                    .unwrap();
                self.store
                    .upsert_piece(&PieceState {
                        job_id: job.id,
                        setup_id: setup.id,
                        source_id: source.id,
                        index: 0,
                        piece: PieceRef::page(1),
                        status: PieceStatus::Done,
                        cache_key: Some(key),
                        error: None,
                        attempts: 1,
                        updated_at: Timestamp::now(),
                    })
                    .unwrap();
                self.store
                    .put_link(&DocumentLink {
                        document_id: *document,
                        job_id: job.id,
                        source_id: source.id,
                        linked_at: Timestamp::now(),
                    })
                    .unwrap();
            }
        }
        job
    }

    fn view(&self) -> DictionaryView {
        self.dictionary.view(self.project, None).unwrap()
    }
}

fn answer(key: &str, value: FieldValue) -> FieldAnswer {
    FieldAnswer {
        key: key.into(),
        value,
        reason: None,
    }
}

fn row<'a>(view: &'a DictionaryView, namespace: &str, label: &str) -> &'a DictionaryRow {
    view.rows
        .iter()
        .find(|r| r.namespace == namespace && r.label == label)
        .unwrap_or_else(|| panic!("no row {namespace}/{label}"))
}

fn form(form: &str) -> Item {
    Item::Form { form: form.into() }
}

#[test]
fn names_are_gathered_across_jobs_counting_documents_and_agreement() {
    let f = fixture();
    let letter = f.document("letter");
    let report = f.document("report");
    // Two models read the letter; only one lists the imam.
    f.analysis(
        std::slice::from_ref(&letter),
        &[
            &[(&["Moussa Traoré", "  El Hadj   Oumar "], &["Kankan"])],
            &[(&["El Hadj Oumar"], &["Kankan"])],
        ],
    );
    f.analysis(
        std::slice::from_ref(&report),
        &[&[(&["El Hadj Oumar", "Kankan"], &["Bamako"])]],
    );

    let view = f.view();
    assert_eq!((view.documents, view.documents_read), (2, 2));
    let keys: Vec<(&str, &str)> = view
        .namespaces
        .iter()
        .map(|n| (n.key.as_str(), n.label.as_str()))
        .collect();
    assert_eq!(keys, vec![("people", "People"), ("places", "Places")]);

    let oumar = row(&view, "people", "El Hadj Oumar");
    assert_eq!((oumar.documents, oumar.listed, oumar.read), (2, 3, 3));
    let moussa = row(&view, "people", "Moussa Traoré");
    assert_eq!((moussa.documents, moussa.listed, moussa.read), (1, 1, 2));
    // The same spelling in two fields is two rows: a person is not a town.
    assert_eq!(row(&view, "people", "Kankan").documents, 1);
    assert_eq!(row(&view, "places", "Kankan").documents, 1);
    assert!(view.rows.iter().all(|r| r.namespace != "summary"));

    let evidence = f
        .dictionary
        .evidence(f.project, "people", &["Moussa Traoré".into()])
        .unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].document, "letter");
    assert_eq!(evidence[0].setup_label, "model-0");
}

#[test]
fn the_researchers_corrected_answers_replace_the_models() {
    let f = fixture();
    let letter = f.document("letter");
    let job = f.analysis(
        std::slice::from_ref(&letter),
        &[&[(&["Moussa Traore"], &[])]],
    );
    f.store
        .put_correction(&Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: letter.1.id,
            index: 0,
            text: None,
            fields: Some(vec![answer(
                "people",
                FieldValue::List(vec!["Moussa Traoré".into()]),
            )]),
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap();
    let view = f.view();
    assert_eq!(row(&view, "people", "Moussa Traoré").documents, 1);
    assert!(view.rows.iter().all(|r| r.label != "Moussa Traore"));
}

#[test]
fn variants_are_merged_relabelled_split_and_kept_apart() {
    let f = fixture();
    let docs = [f.document("a"), f.document("b"), f.document("c")];
    f.analysis(
        &docs,
        &[&[
            (&["Moussa Traoré"], &["Côte d'Ivoire"]),
            (&["Traore Moussa"], &["Ivory Coast"]),
            (&["Moussa Traoré"], &["Côte d’Ivoire"]),
        ]],
    );
    let view = f.view();
    // Accents and word order set aside, the two spellings are offered as possibly one; the
    // translated name is not, and the curly apostrophe is.
    let moussa = row(&view, "people", "Moussa Traoré");
    assert_eq!(moussa.similar, vec!["form:Traore Moussa".to_string()]);
    assert!(row(&view, "places", "Ivory Coast").similar.is_empty());
    assert_eq!(row(&view, "places", "Côte d'Ivoire").similar.len(), 1);

    // Multilingual variants are merged by the researcher, under a label of their own.
    let country = f
        .dictionary
        .merge(
            f.project,
            "places",
            &[
                form("Côte d'Ivoire"),
                form("Ivory Coast"),
                form("Côte d’Ivoire"),
            ],
            " Côte d'Ivoire  (colony) ",
        )
        .unwrap();
    let view = f.view();
    let merged = row(&view, "places", "Côte d'Ivoire (colony)");
    assert_eq!(merged.entry, Some(country.id));
    assert_eq!((merged.documents, merged.forms.len()), (3, 3));
    assert!(merged.reviewed);
    // An export writes the answers under that label, in its field only.
    let labels = f.dictionary.labels(f.project).unwrap();
    let written = ["Ivory Coast".to_string(), "Côte d’Ivoire".to_string()];
    assert_eq!(
        labels.applied("places", &written),
        ["Côte d'Ivoire (colony)"]
    );
    assert_eq!(labels.applied("people", &written), written);

    // Two people who share a look are kept apart, then cannot be merged.
    f.dictionary
        .keep_apart(
            f.project,
            "people",
            &form("Moussa Traoré"),
            &form("Traore Moussa"),
        )
        .unwrap();
    let view = f.view();
    let moussa = row(&view, "people", "Moussa Traoré");
    assert!(moussa.similar.is_empty());
    assert_eq!(moussa.apart.len(), 1);
    let (a, b) = (
        moussa.entry.unwrap(),
        row(&view, "people", "Traore Moussa").entry.unwrap(),
    );
    assert!(matches!(
        f.dictionary.merge(
            f.project,
            "people",
            &[Item::Entry { id: a }, Item::Entry { id: b }],
            "Moussa Traoré"
        ),
        Err(DictionaryError::KeptApart)
    ));
    assert!(matches!(
        f.dictionary.merge(
            f.project,
            "people",
            &[Item::Entry { id: country.id }, form("x")],
            "x"
        ),
        Err(DictionaryError::Namespaces)
    ));
    f.dictionary.undo_apart(f.project, a, b).unwrap();
    f.dictionary
        .merge(
            f.project,
            "people",
            &[Item::Entry { id: b }, Item::Entry { id: a }],
            "Moussa Traoré",
        )
        .unwrap();
    let view = f.view();
    let moussa = row(&view, "people", "Moussa Traoré");
    assert_eq!(moussa.entry, Some(a), "the older entry keeps its id");
    assert_eq!(moussa.documents, 3);

    // Splitting a form out undoes part of a merge; the rest stays reviewed.
    f.dictionary
        .split(f.project, country.id, &["Ivory Coast".into()])
        .unwrap();
    f.dictionary
        .relabel(f.project, country.id, "Côte d'Ivoire")
        .unwrap();
    let view = f.view();
    assert_eq!(row(&view, "places", "Côte d'Ivoire").forms.len(), 2);
    let loose = row(&view, "places", "Ivory Coast");
    assert_eq!((loose.entry, loose.reviewed), (None, false));
    // Splitting every form out removes the entry.
    f.dictionary
        .split(
            f.project,
            country.id,
            &["Côte d'Ivoire".into(), "Côte d’Ivoire".into()],
        )
        .unwrap();
    assert!(f.view().rows.iter().all(|r| r.entry != Some(country.id)));
}

#[test]
fn decisions_survive_a_restart_and_entries_without_evidence_are_marked() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jobs.sqlite");
    let (project, job, entry) = {
        let f = fixture_on(Arc::new(SqliteStore::open(&path).unwrap()));
        let letter = f.document("letter");
        let job = f.analysis(
            std::slice::from_ref(&letter),
            &[&[(&["Père Éric", "Pere Eric"], &[])]],
        );
        let entry = f
            .dictionary
            .merge(
                f.project,
                "people",
                &[form("Père Éric"), form("Pere Eric")],
                "Père Éric",
            )
            .unwrap();
        f.dictionary
            .set_reviewed(f.project, "people", &[Item::Entry { id: entry.id }], false)
            .unwrap();
        (f.project, job, entry.id)
    };
    let store = Arc::new(SqliteStore::open(&path).unwrap());
    let dictionary = Dictionary::new(store.clone(), store.clone(), store.clone());
    let view = dictionary.view(project, None).unwrap();
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].entry, Some(entry));
    assert!(!view.rows[0].reviewed);
    assert_eq!(view.namespaces[0].to_review, 1);

    // The job deleted, the entry stays, its evidence gone.
    store.delete_job(job.id).unwrap();
    let view = dictionary.view(project, None).unwrap();
    assert!(view.rows[0].unseen);
    assert_eq!(view.rows[0].documents, 0);
}

#[test]
fn a_collection_shows_what_its_documents_name() {
    let f = fixture();
    let letter = f.document("letter");
    let report = f.document("report");
    f.analysis(
        &[letter.clone(), report.clone()],
        &[&[(&["Imam Diallo"], &[]), (&["Governor Clozel"], &[])]],
    );
    let letters = Collection {
        id: CollectionId::new(),
        project_id: f.project,
        name: "Letters".into(),
        created_at: Timestamp::now(),
    };
    f.store.put_collection(&letters).unwrap();
    f.store.add_member(letters.id, letter.0).unwrap();
    let view = f.dictionary.view(f.project, Some(letters.id)).unwrap();
    let labels: Vec<&str> = view.rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, vec!["Imam Diallo"]);
    assert_eq!(view.documents, 1);
}
