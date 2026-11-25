//! The larger sets, started by hand: `release`, every task end to end before a release (the
//! runner's plans since 3 November), and `roadmap`, the roadmap's *Live runs still to do*.
//! These name their models, since each item names the model it is about; a model the catalogue
//! no longer lists is left out of its job and said so.

use kalamu_core::{
    ExportFormat, JobSettings, LanguageChoice, LanguageCode, LineBreaks, MergeModel, Pace,
    ProviderId, SetupOptions, Task,
};

use crate::steps::{Plan, Step, setup};

fn plan(
    name: &str,
    task: Task,
    files: &[&str],
    setups: Vec<kalamu_core::SetupRequest>,
    settings: JobSettings,
    formats: Vec<ExportFormat>,
) -> Step {
    Step::job(Plan {
        name: name.into(),
        task,
        files: files.iter().map(|f| f.to_string()).collect(),
        setups,
        settings,
        formats,
    })
}

/// Every task end to end before a release. `<samples>` holds `newspaper.jpg`, `letter.pdf`,
/// `interview.wav`, `article.txt` (with `article2.txt` and `article3.txt` for the batch,
/// `image.jpg` for the description, `page-1.jpg`, `page-2.jpg` and `page-10.jpg` for the
/// document of images); a job whose files are missing is left out.
pub fn release(offline_ocr: bool) -> Vec<Step> {
    let plain = SetupOptions::default;
    let interview = || SetupOptions {
        diarize: true,
        timestamps: true,
        speakers: Some(2),
        ..SetupOptions::default()
    };
    let ocr = |preset: Option<&'static str>| {
        vec![
            setup("gemini", "gemini-3.8-flash", preset, plain()),
            setup("openai", "gpt-6-luna", preset, plain()),
            setup("anthropic", "claude-haiku-5-5", preset, plain()),
            setup("mistral", "mistral-ocr-4-1", None, plain()),
        ]
    };
    let analysis = |preset: &'static str| {
        vec![
            setup("gemini", "gemini-3.8-flash", Some(preset), plain()),
            setup("openai", "gpt-6-luna", Some(preset), plain()),
            setup("anthropic", "claude-haiku-5-5", Some(preset), plain()),
            setup("mistral", "mistral-small-2603", Some(preset), plain()),
        ]
    };
    let mut steps = Vec::new();
    // Offline: the first pages of a multi-page PDF and an image, French and English.
    if offline_ocr {
        steps.push(plan(
            "offline-ocr",
            Task::TextExtraction,
            &["newspaper.jpg", "letter.pdf"],
            vec![setup("tesseract", "tesseract-5", None, plain())],
            JobSettings {
                languages: LanguageChoice::Explicit(vec![
                    LanguageCode::new("fr"),
                    LanguageCode::new("en"),
                ]),
                sample_pages: Some(3),
                ..JobSettings::default()
            },
            vec![
                ExportFormat::Txt,
                ExportFormat::Json,
                ExportFormat::SearchablePdf,
                ExportFormat::Alto,
            ],
        ));
    }
    steps.extend([
        plan(
            "ocr-as-on-page",
            Task::TextExtraction,
            &["newspaper.jpg", "letter.pdf"],
            ocr(None),
            JobSettings {
                line_breaks: Some(LineBreaks::AsOnPage),
                ..JobSettings::default()
            },
            vec![
                ExportFormat::Txt,
                ExportFormat::Json,
                ExportFormat::Docx,
                ExportFormat::SearchablePdf,
            ],
        ),
        plan(
            "ocr-running-text",
            Task::TextExtraction,
            &["newspaper.jpg"],
            vec![
                setup(
                    "gemini",
                    "gemini-3.8-flash",
                    Some("text_extraction.newspaper"),
                    plain(),
                ),
                setup(
                    "anthropic",
                    "claude-haiku-5-5",
                    Some("text_extraction.newspaper"),
                    plain(),
                ),
            ],
            JobSettings {
                line_breaks: Some(LineBreaks::RunningText),
                ..JobSettings::default()
            },
            vec![ExportFormat::Txt],
        ),
        plan(
            "audio",
            Task::AudioTranscription,
            &["interview.wav"],
            vec![
                setup("gemini", "gemini-3.8-flash", None, interview()),
                setup("openai", "gpt-4o-transcribe-diarize", None, interview()),
                setup("mistral", "voxtral-mini-transcribe-3-0", None, interview()),
            ],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Srt, ExportFormat::Json],
        ),
        plan(
            "analysis-entities",
            Task::Analysis,
            &["article.txt"],
            analysis("analysis.entities"),
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        ),
        plan(
            "analysis-tone",
            Task::Analysis,
            &["article.txt"],
            analysis("analysis.tone"),
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        ),
        plan(
            "analysis-summary",
            Task::Analysis,
            &["article.txt"],
            analysis("analysis.summary_research"),
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Json],
        ),
        // Three texts with the same long instructions, for what the providers' prompt caches
        // save; best run with LIVE_PACE=economy.
        plan(
            "analysis-batch",
            Task::Analysis,
            &["article.txt", "article2.txt", "article3.txt"],
            analysis("analysis.entities"),
            JobSettings::default(),
            vec![ExportFormat::Json],
        ),
        plan(
            "audio-transcribe",
            Task::AudioTranscription,
            &["interview.wav"],
            vec![setup("gemini", "gemini-3.5-transcribe", None, interview())],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Srt, ExportFormat::Json],
        ),
        plan(
            "ocr-terra",
            Task::TextExtraction,
            &["newspaper.jpg", "letter.pdf"],
            vec![setup("openai", "gpt-6.1-terra", None, plain())],
            JobSettings::default(),
            vec![ExportFormat::Txt],
        ),
        plan(
            "analysis-more-models",
            Task::Analysis,
            &["article.txt"],
            vec![
                setup(
                    "openai",
                    "gpt-6.1-terra",
                    Some("analysis.entities"),
                    plain(),
                ),
                setup(
                    "mistral",
                    "mistral-medium-3-5",
                    Some("analysis.entities"),
                    plain(),
                ),
                setup(
                    "mistral",
                    "mistral-large-4-0",
                    Some("analysis.entities"),
                    plain(),
                ),
                setup(
                    "mistral",
                    "zai-glm-5-3-flash",
                    Some("analysis.entities"),
                    plain(),
                ),
                setup(
                    "mistral",
                    "deepseek-v4-1-flash-1017",
                    Some("analysis.entities"),
                    plain(),
                ),
            ],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        ),
        // A text put into English, whatever its language.
        plan(
            "translation",
            Task::Translation,
            &["article.txt"],
            analysis("translation.research"),
            JobSettings {
                languages: LanguageChoice::Explicit(vec![LanguageCode::new("en")]),
                ..JobSettings::default()
            },
            vec![ExportFormat::Txt, ExportFormat::Json],
        ),
        // Three images read as the pages of one document, given out of order.
        plan(
            "document-box-3",
            Task::TextExtraction,
            &["page-10.jpg", "page-1.jpg", "page-2.jpg"],
            vec![
                setup("gemini", "gemini-3.8-flash", None, plain()),
                setup("anthropic", "claude-haiku-5-5", None, plain()),
            ],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Json, ExportFormat::Alto],
        ),
        // What two images show, answered in French.
        plan(
            "image-description",
            Task::Analysis,
            &["newspaper.jpg", "image.jpg"],
            analysis("analysis.image_description"),
            JobSettings {
                languages: LanguageChoice::Explicit(vec![LanguageCode::new("fr")]),
                ..JobSettings::default()
            },
            vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        ),
    ]);
    steps
}

/// The roadmap's *Live runs still to do* that a run can make, on the repository's `samples/`.
/// What each answer is worth (a hand read well, an article found, a refusal) is for a person
/// to judge from the files written to `<out>`; the run makes the calls, keeps the outputs and
/// sets each cost against its estimate. `whisper`: Whisper's model files are there, as the app
/// downloads them (FFmpeg is needed to convert the recording).
pub fn roadmap(whisper: bool, offline_ocr: bool) -> Vec<Step> {
    let plain = SetupOptions::default;
    let at = |level: &str| SetupOptions {
        reasoning: Some(level.into()),
        ..SetupOptions::default()
    };
    let labelled = |provider: &str, model: &str, preset: Option<&str>, options, label: &str| {
        let mut s = setup(provider, model, preset, options);
        s.label = Some(label.into());
        s
    };
    let handwriting_and_print = ["dodington-1755.jpg", "daily-nebraskan-1911.pdf"];
    let mut steps = Vec::new();
    if whisper {
        // Offline Whisper on the two-voice interview: no key, no cost.
        steps.push(plan(
            "roadmap-offline-whisper",
            Task::AudioTranscription,
            &["interview-two-speakers.wav"],
            vec![setup("whisper", "whisper-base", None, plain())],
            JobSettings {
                languages: LanguageChoice::Explicit(vec![LanguageCode::new("fr")]),
                ..JobSettings::default()
            },
            vec![ExportFormat::Txt, ExportFormat::Srt],
        ));
    }
    if offline_ocr {
        // A whole Tesseract job; with the network off, it is the check the roadmap asks for.
        steps.push(plan(
            "roadmap-offline-tesseract",
            Task::TextExtraction,
            &["bulletin-page.jpg", "daily-nebraskan-1911-issue.pdf"],
            vec![setup("tesseract", "tesseract-5", None, plain())],
            JobSettings {
                languages: LanguageChoice::Explicit(vec![
                    LanguageCode::new("fr"),
                    LanguageCode::new("en"),
                ]),
                ..JobSettings::default()
            },
            vec![
                ExportFormat::Txt,
                ExportFormat::SearchablePdf,
                ExportFormat::Alto,
            ],
        ));
    }
    steps.extend([
        // GLM 5.3 through a whole job: three analyses of one text in one job.
        plan(
            "roadmap-glm-whole-job",
            Task::Analysis,
            &["bulletin.txt"],
            [
                "analysis.entities",
                "analysis.tone",
                "analysis.summary_research",
            ]
            .into_iter()
            .map(|preset| labelled("mistral", "zai-glm-5-3", Some(preset), plain(), preset))
            .collect(),
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Csv, ExportFormat::Json],
        ),
        // GPT-6.1 Terra on handwriting and older print, before it is weighed as OpenAI's
        // default.
        plan(
            "roadmap-terra-handwriting-print",
            Task::TextExtraction,
            &handwriting_and_print,
            vec![setup("openai", "gpt-6.1-terra", None, plain())],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Json],
        ),
        // GPT-6.2 Sol and Astra on real pages.
        plan(
            "roadmap-sol-astra-pages",
            Task::TextExtraction,
            &[
                "bulletin-page.jpg",
                "dodington-1755.jpg",
                "daily-nebraskan-1911.pdf",
            ],
            vec![
                setup("openai", "gpt-6.2-sol", None, plain()),
                setup("openai", "gpt-6.2-astra", None, plain()),
            ],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Json],
        ),
        // Gemini 4 Neon against Hydrogen on handwriting, older print and refusals: Gemini has
        // refused the Dodington page without a reason (samples/README.txt).
        plan(
            "roadmap-neon-hydrogen",
            Task::TextExtraction,
            &handwriting_and_print,
            vec![
                setup("gemini", "gemini-4-neon", None, plain()),
                setup("gemini", "gemini-4-hydrogen", None, plain()),
            ],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Json],
        ),
        // A table of contents mixing providers: Gemini reads the pages, Mistral combines them.
        plan(
            "roadmap-contents-mixed",
            Task::Contents,
            &["daily-nebraskan-1911-issue.pdf"],
            vec![{
                let mut s = setup(
                    "gemini",
                    "gemini-4-hydrogen",
                    Some("contents.periodical"),
                    plain(),
                );
                s.merge = Some(MergeModel {
                    provider: ProviderId::new("mistral"),
                    model: "mistral-large-4-0".into(),
                });
                s
            }],
            JobSettings::default(),
            vec![ExportFormat::Markdown, ExportFormat::Json],
        ),
        // A correction by two providers side by side, of the 1911 page's machine-read text.
        plan(
            "roadmap-correction-mixed",
            Task::TextCorrection,
            &["daily-nebraskan-1911.pdf"],
            vec![
                setup(
                    "gemini",
                    "gemini-4-hydrogen",
                    Some("text_correction.general"),
                    plain(),
                ),
                setup(
                    "mistral",
                    "mistral-large-4-0",
                    Some("text_correction.general"),
                    plain(),
                ),
            ],
            JobSettings::default(),
            vec![ExportFormat::Txt, ExportFormat::Json],
        ),
        // A table of contents in economy mode with Claude Sonnet 5.5, at `low`, where the first
        // run found articles on two pages only, and at a higher effort.
        plan(
            "roadmap-contents-economy",
            Task::Contents,
            &["daily-nebraskan-1911-issue.pdf"],
            vec![
                labelled(
                    "anthropic",
                    "claude-sonnet-5-5",
                    Some("contents.periodical"),
                    at("low"),
                    "sonnet-low",
                ),
                labelled(
                    "anthropic",
                    "claude-sonnet-5-5",
                    Some("contents.periodical"),
                    at("medium"),
                    "sonnet-medium",
                ),
            ],
            JobSettings {
                pace: Pace::Economy,
                ..JobSettings::default()
            },
            vec![ExportFormat::Markdown, ExportFormat::Json],
        ),
    ]);
    steps
}
