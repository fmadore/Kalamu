//! The researcher's own themes or interview guide (`SetupOptions::themes`): sent with the
//! question, one per line; each theme of the answer saying whether it comes from the guide or
//! emerged from the interview; a guide theme the interview does not take up listed apart, with
//! no quotation; kept with a prompt of the researcher's own as the question is; recorded in
//! provenance; and a job with none keyed as before.

use super::*;

fn guided(f: &Interviews, source: &SourceFile, themes: Option<&str>, preset: &str) -> Job {
    let mut request = setup("interviewer", "interviewer-1", Some(preset), None);
    request.options.themes = themes.map(String::from);
    f.engine
        .create_job(NewJob {
            name: "Analyse".into(),
            task: Task::Analysis,
            sources: vec![source.clone()],
            setups: vec![request],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Json, ExportFormat::Markdown],
                destination: f.folder(),
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"))
}

/// An answer with one theme from the guide, one that emerged, and a guide theme not found.
fn guided_answer() -> serde_json::Value {
    serde_json::json!({
        "summary": "Awa raconte ses débuts au port.",
        "themes": [
            {"name": "Les débuts", "description": "Elle commence en 1998.",
             "origin": "The researcher's guide",
             "quotes": [{"turn": 2, "words": "J’ai commencé en 1998."}]},
            {"name": "Une devise", "description": "Une phrase reprise.",
             "origin": "the interview",
             "quotes": [{"turn": 4, "words": "est devenu notre devise."}]},
        ],
        "guide_themes_not_found": ["La famille"],
        "keywords": ["port"],
        "quotations": [],
    })
}

#[tokio::test]
async fn the_guide_goes_with_the_question_and_each_theme_says_where_it_comes_from() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[guided_answer()]);
    let job = guided(
        &f,
        &source,
        Some("Les débuts\n- La famille\n"),
        "analysis.interview",
    );
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let (_, request) = f.model.sent().remove(0);
    assert!(
        request.ends_with(&format!(
            "{}\n- Les débuts\n- La famille\n\n{}",
            kalamu_core::setup::THEMES_INTRO,
            kalamu_core::analysis::cited::REQUEST
        )),
        "{request}"
    );

    let fields = f.fields(&job, 0);
    let themes = cited(&fields, "themes");
    let origins: Vec<String> = themes
        .entries
        .iter()
        .map(|e| kalamu_core::analysis::answer_text(&e.answers, "origin"))
        .collect();
    // The choice spelled as the prompt spells it, whatever the model's case.
    assert_eq!(origins, ["The researcher's guide", "The interview"]);
    let missing = fields
        .iter()
        .find(|a| a.key == "guide_themes_not_found")
        .map(|a| a.value.joined());
    assert_eq!(missing.as_deref(), Some("La famille"));

    // Provenance records the guide sent; the memo carries where each theme comes from.
    let provenance = f.provenance(&job);
    assert_eq!(
        provenance["options"]["themes"],
        "Les débuts\n- La famille\n"
    );
    let memo = f
        .engine
        .outputs(job.id)
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.ends_with(".md"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    assert!(
        memo.contains("**Origin:** The researcher's guide"),
        "{memo}"
    );
    assert!(
        memo.contains("## Guide themes not found\n\nLa famille"),
        "{memo}"
    );
}

#[tokio::test]
async fn a_guide_kept_with_a_prompt_is_sent_as_the_same_lines_typed_and_none_keys_as_before() {
    let f = interviews();
    let source = f.frozen().await;
    f.model.script(&[guided_answer()]);
    let saved = f.engine.with_prompts(|p| {
        let mut copy = p
            .get("analysis.interview")
            .unwrap_or_else(|e| panic!("{e}"));
        copy.id = String::new();
        copy.name = "Port".into();
        copy.question = Some("Le port".into());
        copy.themes = Some("  Les débuts\nLa famille  ".into());
        p.save(copy).unwrap_or_else(|e| panic!("{e}"))
    });
    assert_eq!(saved.themes.as_deref(), Some("Les débuts\nLa famille"));
    let kept = guided(&f, &source, None, &saved.id);
    assert_eq!(
        kept.setups[0].options.themes.as_deref(),
        Some("Les débuts\nLa famille")
    );
    // Lines typed for the job replace the kept ones.
    let typed = guided(&f, &source, Some("Le marché"), &saved.id);
    assert_eq!(typed.setups[0].options.themes.as_deref(), Some("Le marché"));
    // A job with no guide sends the request it always sent, under the same options.
    let none = guided(&f, &source, Some("  \n "), "analysis.interview");
    assert!(
        !none.setups[0]
            .prompt
            .user_prompt
            .contains(kalamu_core::setup::THEMES_INTRO)
    );
    let plain = guided(&f, &source, None, "analysis.interview");
    assert_eq!(
        none.setups[0].prompt.user_prompt,
        plain.setups[0].prompt.user_prompt
    );
}
