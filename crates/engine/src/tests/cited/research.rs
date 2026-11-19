//! The researcher's question kept with a prompt of their own (`Preset::question`): sent exactly
//! as the same words typed for the set-up, so the two share a cache key; a question typed
//! replacing the prompt's; and prompts without one keyed as before.

use super::*;

/// A copy of the interview prompt under `name`, keeping `question`.
fn research_prompt(f: &Interviews, name: &str, question: Option<&str>) -> String {
    f.engine.with_prompts(|p| {
        let mut copy = p
            .get("analysis.interview")
            .unwrap_or_else(|e| panic!("{e}"));
        copy.id = String::new();
        copy.name = name.into();
        copy.question = question.map(String::from);
        p.save(copy).unwrap_or_else(|e| panic!("{e}")).id
    })
}

fn job_with(
    f: &Interviews,
    source: &SourceFile,
    preset: &str,
    question: Option<&str>,
) -> Result<Job, EngineError> {
    f.engine.create_job(NewJob {
        name: "Analyse".into(),
        task: Task::Analysis,
        sources: vec![source.clone()],
        setups: vec![setup(
            "interviewer",
            "interviewer-1",
            Some(preset),
            question,
        )],
        settings: Default::default(),
        output: OutputSettings {
            formats: vec![ExportFormat::Json],
            destination: f.folder(),
            ..Default::default()
        },
    })
}

fn key_of(f: &Interviews, job: &Job) -> Option<String> {
    f.engine
        .store
        .pieces(job.id)
        .unwrap_or_default()
        .first()
        .and_then(|p| p.cache_key.clone())
}

#[tokio::test]
async fn a_question_kept_with_a_prompt_is_sent_as_the_same_words_typed() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[answer()]);
    let saved = research_prompt(&f, "Port", Some("  Le travail au port  "));

    let typed = job_with(
        &f,
        &source,
        "analysis.interview",
        Some("Le travail au port"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(typed.id).await, JobStatus::Completed);
    let kept = job_with(&f, &source, &saved, None).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(kept.id).await, JobStatus::Completed);

    // The set-up holds the question as if it had been typed, the request carries it, and the
    // prompt is recorded as the researcher's own.
    let setup = &kept.setups[0];
    assert_eq!(
        setup.options.question.as_deref(),
        Some("Le travail au port")
    );
    assert_eq!(setup.prompt.user_prompt, typed.setups[0].prompt.user_prompt);
    assert!(
        setup
            .prompt
            .user_prompt
            .ends_with("The researcher's work, in their own words: Le travail au port"),
        "{}",
        setup.prompt.user_prompt
    );
    assert_eq!(setup.prompt.preset_id.as_deref(), Some(saved.as_str()));
    assert_eq!(setup.prompt.preset_name, "Port");
    // Same words, same key: the second analysis is read from the cache.
    assert_eq!(key_of(&f, &kept), key_of(&f, &typed));
    assert_eq!(f.model.calls(), 1, "the same question costs nothing twice");
    assert_eq!(
        f.engine.store.pieces(kept.id).unwrap_or_default()[0].status,
        PieceStatus::Cached
    );
}

#[tokio::test]
async fn a_question_typed_replaces_the_one_the_prompt_keeps() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[answer()]);
    let saved = research_prompt(&f, "Famille", Some("La famille"));

    let job =
        job_with(&f, &source, &saved, Some("Le travail au port")).unwrap_or_else(|e| panic!("{e}"));
    let setup = &job.setups[0];
    assert_eq!(
        setup.options.question.as_deref(),
        Some("Le travail au port")
    );
    assert!(!setup.prompt.user_prompt.contains("La famille"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let (_, request) = f.model.sent().remove(0);
    assert!(request.contains("Le travail au port"), "{request}");
    assert!(!request.contains("La famille"), "{request}");

    // A question left blank is no question: the prompt's is sent.
    let blank = job_with(&f, &source, &saved, Some("   ")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        blank.setups[0].options.question.as_deref(),
        Some("La famille")
    );
}

#[tokio::test]
async fn prompts_without_a_question_are_keyed_as_before() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[answer()]);
    let copy = research_prompt(&f, "Copie", None);

    let shipped =
        job_with(&f, &source, "analysis.interview", None).unwrap_or_else(|e| panic!("{e}"));
    let own = job_with(&f, &source, &copy, None).unwrap_or_else(|e| panic!("{e}"));
    let preset = f
        .engine
        .with_prompts(|p| p.get("analysis.interview"))
        .unwrap_or_else(|e| panic!("{e}"));
    for job in [&shipped, &own] {
        let setup = &job.setups[0];
        // Nothing added: no question in the options, the request the prompt's own.
        assert_eq!(setup.options.question, None);
        assert_eq!(setup.prompt.user_prompt, preset.user_prompt);
        assert_eq!(
            serde_json::to_value(&setup.options).unwrap_or_default()["question"],
            serde_json::Value::Null
        );
    }
    assert_eq!(f.run(shipped.id).await, JobStatus::Completed);
    assert_eq!(f.run(own.id).await, JobStatus::Completed);
    assert_eq!(key_of(&f, &own), key_of(&f, &shipped));
    assert_eq!(f.model.calls(), 1);
}
