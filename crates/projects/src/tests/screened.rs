//! Documents read against a research question (#51): which text each is read in, the choices
//! an analysis answered for the table to be filtered by, the researcher's correction winning
//! over the model's answer, and the analysis written as one table with the researcher's tags.

use kalamu_core::{AnalysisSchema, DocumentId, Field, FieldAnswer, FieldKind, FieldValue};

use super::*;

fn field(label: &str, kind: FieldKind) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

pub(super) fn screening() -> AnalysisSchema {
    let mut schema = AnalysisSchema {
        fields: vec![
            field(
                "Relevance",
                FieldKind::Choice {
                    values: ["Relevant", "Partly relevant", "Not relevant", "Unsure"]
                        .map(String::from)
                        .to_vec(),
                },
            ),
            field("Reason", FieldKind::Text),
        ],
    };
    schema.assign_keys();
    schema
}

fn answer(key: &str, value: &str) -> FieldAnswer {
    FieldAnswer {
        key: key.into(),
        value: FieldValue::Text(value.into()),
        reason: None,
    }
}

impl Fixture {
    /// An analysis over `sources` with the screening fields and a question, one piece per
    /// source answered as `relevance` says.
    pub(super) fn screening(&self, sources: Vec<SourceFile>, relevance: &[&str]) -> Job {
        self.screening_with(
            sources,
            relevance,
            screening(),
            "  Women traders at the market  ",
        )
    }

    /// [`Fixture::screening`] with fields and a question of its own.
    pub(super) fn screening_with(
        &self,
        sources: Vec<SourceFile>,
        relevance: &[&str],
        schema: AnalysisSchema,
        question: &str,
    ) -> Job {
        self.screening_as(
            kalamu_core::JobId::new(),
            sources,
            relevance,
            schema,
            question,
        )
    }

    /// [`Fixture::screening_with`] under a job id given, for golden files.
    pub(super) fn screening_as(
        &self,
        id: kalamu_core::JobId,
        sources: Vec<SourceFile>,
        relevance: &[&str],
        schema: AnalysisSchema,
        question: &str,
    ) -> Job {
        let job = Job {
            id,
            name: "Screening".into(),
            task: Task::Analysis,
            created_at: Timestamp::now(),
            sources,
            setups: vec![Setup {
                windows: None,
                id: SetupId::new(),
                label: "gemini-3.8-flash".into(),
                provider: ProviderId::new("gemini"),
                model: "gemini-3.8-flash".into(),
                prompt: PromptSnapshot {
                    preset_id: Some("analysis.screening".into()),
                    preset_name: "Does it speak of my question?".into(),
                    template: String::new(),
                    text: String::new(),
                    user_prompt: String::new(),
                    schema: Some(schema),
                },
                options: SetupOptions {
                    question: Some(question.into()),
                    ..SetupOptions::default()
                },
                merge: None,
                pages_cap: None,
            }],
            settings: JobSettings::default(),
            output: OutputSettings::default(),
            status: JobStatus::Completed,
        };
        self.store.create_job(&job).unwrap();
        for (source, relevance) in job.sources.iter().zip(relevance) {
            let key = format!("{}-{}", job.id, source.id);
            self.store
                .put_result(&StoredResult {
                    cache_key: key.clone(),
                    provider: ProviderId::new("gemini"),
                    model: "gemini-3.8-flash".into(),
                    output: ProcessOutput {
                        fields: vec![
                            answer("relevance", relevance),
                            answer("reason", &format!("{relevance}, because.")),
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
                    setup_id: job.setups[0].id,
                    source_id: source.id,
                    index: 0,
                    piece: PieceRef::Whole,
                    status: PieceStatus::Done,
                    cache_key: Some(key),
                    error: None,
                    attempts: 1,
                    updated_at: Timestamp::now(),
                })
                .unwrap();
        }
        job
    }
}

#[test]
fn each_document_is_read_in_its_one_text_never_one_chosen_for_the_researcher() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let read = f.file("read.pdf", "a");
    let twice = f.file("twice.pdf", "b");
    let mut own = f.file("notes.txt", "c");
    own.kind = InputKind::Text;
    let unread = f.file("unread.pdf", "d");
    let docs = f
        .projects
        .add_documents(
            p.id,
            vec![read.clone(), twice.clone(), own.clone(), unread],
            None,
        )
        .unwrap();
    let extraction = f.job(Task::TextExtraction, vec![read.clone(), twice.clone()]);
    let again = f.job(Task::TextExtraction, vec![twice]);
    f.projects
        .link_jobs(p.id, &[extraction.id, again.id], None)
        .unwrap();

    let texts = f.projects.document_texts(p.id, &docs).unwrap();
    assert_eq!(
        texts[0].text,
        TextOf::Result {
            job_id: extraction.id,
            setup_id: extraction.setups[0].id,
            source_id: read.id,
        }
    );
    assert_eq!(
        texts[1].text,
        TextOf::Undecided,
        "two readings, none preferred"
    );
    assert!(matches!(&texts[2].text, TextOf::Own(s) if s.id == own.id));
    assert_eq!(texts[3].text, TextOf::Nothing);
    assert_eq!(texts[3].title, "unread");

    // Once the researcher prefers one reading, it is the one read.
    f.projects
        .set_preferred(
            p.id,
            docs[1],
            Task::TextExtraction,
            Some((again.id, again.setups[0].id)),
        )
        .unwrap();
    let texts = f.projects.document_texts(p.id, &docs[1..2]).unwrap();
    assert!(matches!(texts[0].text, TextOf::Result { job_id, .. } if job_id == again.id));
    // A document of another project is refused.
    assert!(matches!(
        f.projects.document_texts(p.id, &[DocumentId::new()]),
        Err(ProjectError::NotFound)
    ));
}

#[test]
fn the_choices_answered_filter_the_documents_and_a_correction_wins() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    let b = f.file("b.txt", "b");
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone(), b.clone()], None)
        .unwrap();
    let job = f.screening(vec![a.clone(), b.clone()], &["Relevant", "Unsure"]);
    f.projects.link_job(p.id, job.id, None).unwrap();
    // The researcher corrects the second answer: the table filters on theirs.
    f.store
        .put_correction(&Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: b.id,
            index: 0,
            text: None,
            fields: Some(vec![
                answer("relevance", "Not relevant"),
                answer("reason", "Only the price of rice."),
            ]),
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap();

    let view = f.projects.view(p.id).unwrap();
    assert_eq!(view.choices.len(), 1, "one choice: the reason is not one");
    let group = &view.choices[0];
    assert_eq!(group.field.key, "relevance");
    assert_eq!(
        group.question.as_deref(),
        Some("Women traders at the market")
    );
    assert_eq!(group.preset_id.as_deref(), Some("analysis.screening"));
    let answers: Vec<(DocumentId, &str)> = group
        .answers
        .iter()
        .map(|x| (x.document, x.value.as_str()))
        .collect();
    assert_eq!(
        answers,
        vec![(docs[0], "Relevant"), (docs[1], "Not relevant")]
    );
    // Each row's reason beside its answer: the analysis's text, as corrected.
    let reasons: Vec<Option<&str>> = group.answers.iter().map(|x| x.reason.as_deref()).collect();
    assert_eq!(
        reasons,
        [Some("Relevant, because."), Some("Only the price of rice.")]
    );
    assert!(group.answers.iter().all(|x| x.job_id == job.id));
}

#[test]
fn each_answer_carries_its_passages_with_their_pages() {
    use kalamu_core::analysis::cited::{CitedEntry, CitedValue, Evidence};
    use kalamu_core::snapshot::Anchor;

    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    f.projects
        .add_documents(p.id, vec![a.clone()], None)
        .unwrap();
    // A schema with the passages field, as the screening prompt has.
    let mut schema = screening();
    schema.fields.push(Field {
        key: "passages".into(),
        label: "Passages".into(),
        instructions: String::new(),
        kind: FieldKind::Cited {
            fields: Vec::new(),
            max: Some(3),
        },
        reason: false,
    });
    let job = f.screening_with(vec![a.clone()], &["Relevant"], schema, "Market women");
    f.projects.link_job(p.id, job.id, None).unwrap();
    let passage = Evidence {
        anchor: Anchor {
            part: 1,
            start: 10,
            end: 40,
        },
        quote: "Les commerçantes demandent un abri.".into(),
        turn: 4,
        speaker: None,
        start_ms: None,
        end_ms: None,
        page: Some(PieceRef::page(2)),
        added: false,
    };
    // The researcher's corrected answers hold the passage, as the review keeps them.
    f.store
        .put_correction(&Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: a.id,
            index: 0,
            text: None,
            fields: Some(vec![
                answer("relevance", "Relevant"),
                answer("reason", "An awning asked of the town."),
                FieldAnswer {
                    key: "passages".into(),
                    value: FieldValue::Cited(CitedValue {
                        entries: vec![CitedEntry {
                            answers: Vec::new(),
                            evidence: vec![passage.clone()],
                            claims: Vec::new(),
                        }],
                        left_out: 0,
                        dropped: 0,
                    }),
                    reason: None,
                },
            ]),
            reviewed: false,
            updated_at: Timestamp::now(),
        })
        .unwrap();

    let view = f.projects.view(p.id).unwrap();
    let row = &view.choices[0].answers[0];
    assert_eq!(row.reason.as_deref(), Some("An awning asked of the town."));
    assert_eq!(row.passages, vec![passage]);
}

/// The table matches its golden file: each field under its label, the analysis that gave each
/// row's answer and the prompt it froze, then the researcher's tags (#51, #241).
#[test]
fn an_analysis_is_written_as_one_table_with_the_researchers_tags() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    let b = f.file("b.txt", "b");
    let docs = f
        .projects
        .add_documents(p.id, vec![b.clone(), a.clone()], None)
        .unwrap();
    let job = f.screening_as(
        super::queue::fixed(7),
        vec![a, b],
        &["Relevant", "=Not relevant"],
        screening(),
        "Women traders at the market",
    );
    f.projects.link_job(p.id, job.id, None).unwrap();
    f.projects.tag_documents(p.id, &docs[1..], "Keep").unwrap();

    let table = f
        .projects
        .answers_table(p.id, job.id, job.setups[0].id)
        .unwrap();
    assert_eq!(
        table.headings,
        [
            "document",
            "file",
            "Relevance",
            "Reason",
            "job_id",
            "prompt_fingerprint",
            "tags"
        ]
    );
    let prompt = job.setups[0].prompt.short_fingerprint();
    assert_eq!(prompt.len(), 12);
    assert!(
        table
            .rows
            .iter()
            .all(|r| r[4] == job.id.to_string() && r[5] == prompt)
    );
    super::queue::golden("screening-table.csv", &csv_body(table));
    assert!(matches!(
        f.projects.answers_table(p.id, job.id, SetupId::new()),
        Err(ProjectError::NotFound)
    ));
}

#[test]
fn a_long_document_read_in_parts_is_answered_by_what_brought_them_together() {
    let f = fixture();
    let p = f.projects.create("Market", "").unwrap();
    let a = f.file("a.txt", "a");
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone()], None)
        .unwrap();
    let job = f.screening(vec![a.clone()], &["Relevant"]);
    // The answer above is the step's, moved after a part read first that said otherwise.
    let mut whole = f.store.pieces(job.id).unwrap().remove(0);
    let part_key = format!("{}-part", whole.cache_key.clone().unwrap());
    f.store
        .put_result(&StoredResult {
            cache_key: part_key.clone(),
            provider: ProviderId::new("gemini"),
            model: "gemini-3.8-flash".into(),
            output: ProcessOutput {
                fields: vec![answer("relevance", "Not relevant")],
                ..ProcessOutput::default()
            },
            request: Vec::new(),
            cost_usd: Some(0.01),
            duration_ms: 1,
            created_at: Timestamp::now(),
        })
        .unwrap();
    f.store
        .upsert_piece(&PieceState {
            index: 0,
            piece: PieceRef::Turns {
                number: 1,
                first: 1,
                last: 9,
            },
            cache_key: Some(part_key),
            ..whole.clone()
        })
        .unwrap();
    whole.index = 1;
    f.store.upsert_piece(&whole).unwrap();
    f.projects.link_job(p.id, job.id, None).unwrap();

    let view = f.projects.view(p.id).unwrap();
    let answers: Vec<(DocumentId, &str)> = view.choices[0]
        .answers
        .iter()
        .map(|x| (x.document, x.value.as_str()))
        .collect();
    assert_eq!(answers, vec![(docs[0], "Relevant")]);
    let table = f
        .projects
        .answers_table(p.id, job.id, job.setups[0].id)
        .unwrap();
    assert_eq!(table.rows[0][2], "Relevant");
}
