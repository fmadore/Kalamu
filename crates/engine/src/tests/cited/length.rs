//! A transcript too long for the model to read at once: the estimate says so before Start, and
//! the job is refused when created, before anything is sent or recorded; one that fits, and an
//! analysis that sends no numbered text, go on as before. The offline models' limit in the
//! catalogue is the offline engine's own.

use super::*;

/// The pretend model reads 100 tokens, 400 characters by the catalogue's measure; a text of one
/// paragraph is sent as `[1] ` and its words.
const TOKENS: u32 = 100;

fn paragraph(chars: usize) -> String {
    "a".repeat(chars - "[1] ".len())
}

/// An analysis by the pretend model with the task's default prompt, which cites nothing.
fn plain_analysis(f: &Interviews, sources: Vec<SourceFile>) -> Result<Job, EngineError> {
    f.engine.create_job(NewJob {
        name: "Analyse".into(),
        task: Task::Analysis,
        sources,
        setups: vec![setup("interviewer", "interviewer-1", None, None)],
        settings: Default::default(),
        output: OutputSettings {
            formats: vec![ExportFormat::Txt],
            destination: f.folder(),
            ..Default::default()
        },
    })
}

fn estimate(f: &Interviews, sources: Vec<SourceFile>, preset: Option<&str>) -> Estimate {
    f.engine.estimate(&EstimateRequest {
        task: Task::Analysis,
        sources,
        setups: vec![setup("interviewer", "interviewer-1", preset, None)],
        sample_pages: None,
    })
}

#[tokio::test]
async fn a_transcript_too_long_for_the_model_is_refused_before_anything_is_sent_or_kept() {
    let f = interviews_reading(TOKENS);
    let long = f.text_file("long.txt", &paragraph(401));
    let refused = f.analysis(vec![long.clone()], Some("Le port"));
    let Err(error) = refused else {
        panic!("a text too long was accepted");
    };
    assert_eq!(error.code(), "job.text_too_long");
    assert!(
        matches!(
            &error,
            EngineError::TextTooLong { file, model, words, most }
                if file == "long.txt" && model == "Interviewer One" && *words == 67 && *most == 66
        ),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "long.txt is too long for Interviewer One: about 67 words, where it reads about 66 at once"
    );
    assert_eq!(f.model.calls(), 0);
    assert!(f.engine.store.list_jobs().unwrap_or_default().is_empty());

    // The estimate says the same before Start.
    let row = &estimate(&f, vec![long], Some("analysis.interview")).rows[0];
    assert_eq!(
        row.too_long,
        Some(TextTooLong {
            file: "long.txt".into(),
            words: 67,
            most_words: 66
        })
    );
}

#[tokio::test]
async fn a_transcript_that_fits_is_analysed_and_the_estimate_says_nothing() {
    let f = interviews_reading(TOKENS);
    let fits = f.text_file("fits.txt", &paragraph(400));
    assert_eq!(
        estimate(&f, vec![fits.clone()], Some("analysis.interview")).rows[0].too_long,
        None
    );
    f.model.script(&[serde_json::json!({
        "summary": "s", "themes": [], "guide_themes_not_found": [], "keywords": [], "quotations": [],
    })]);
    let job = f
        .analysis(vec![fits], None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.model.calls(), 1);
    assert_eq!(f.model.sent()[0].0.chars().count(), 400);
}

#[tokio::test]
async fn each_file_is_measured_on_its_own() {
    let f = interviews_reading(TOKENS);
    let short = f.text_file("court.txt", &paragraph(300));
    let long = f.text_file("long.txt", &paragraph(1_000));
    let refused = f.analysis(vec![short.clone(), long], None);
    assert!(
        matches!(&refused, Err(EngineError::TextTooLong { file, .. }) if file == "long.txt"),
        "{refused:?}"
    );
    // Two short ones are each read on their own, so both fit.
    let other = f.text_file("autre.txt", &paragraph(300));
    assert!(f.analysis(vec![short, other], None).is_ok());
}

#[tokio::test]
async fn an_analysis_that_cites_nothing_is_not_measured() {
    let f = interviews_reading(TOKENS);
    let long = f.text_file("long.txt", &paragraph(1_000));
    assert_eq!(
        estimate(&f, vec![long.clone()], None).rows[0].too_long,
        None
    );
    assert!(plain_analysis(&f, vec![long]).is_ok());
}

#[tokio::test]
async fn a_model_the_catalogue_gives_no_limit_is_left_to_its_provider() {
    let f = interviews();
    let long = f.text_file("long.txt", &paragraph(100_000));
    assert_eq!(
        estimate(&f, vec![long.clone()], Some("analysis.interview")).rows[0].too_long,
        None
    );
    assert!(f.analysis(vec![long], None).is_ok());
}

/// The catalogue's limit for the models run on this computer is what the offline engine sends
/// them, so a text the engine would refuse after Start is refused before it, with the same
/// number of words.
#[test]
fn the_offline_models_limit_is_the_offline_engines() {
    let catalog = kalamu_catalog::Catalog::bundled();
    let provider = ProviderId::new(kalamu_offline::LLM_ID);
    for file in &kalamu_offline::LLM_MODELS {
        let model = catalog
            .model(&provider, file.id)
            .unwrap_or_else(|| panic!("{} is in the catalogue", file.id));
        let tokens = model.limits.max_input_tokens.map(u64::from);
        assert_eq!(
            tokens.map(|t| t * kalamu_catalog::CHARS_PER_TOKEN),
            Some(kalamu_offline::LLM_MAX_CHARS as u64),
            "{}",
            file.id
        );
        assert_eq!(
            model.too_long(&"a".repeat(kalamu_offline::LLM_MAX_CHARS)),
            None
        );
        let long = model
            .too_long(&"a".repeat(kalamu_offline::LLM_MAX_CHARS + 1))
            .unwrap_or_else(|| panic!("one character more is too long"));
        assert_eq!(
            u64::from(long.most_words),
            (kalamu_offline::LLM_MAX_CHARS / kalamu_offline::LLM_CHARS_PER_WORD) as u64
        );
    }
    assert_eq!(
        kalamu_catalog::CHARS_PER_WORD,
        kalamu_offline::LLM_CHARS_PER_WORD as u64
    );
}
