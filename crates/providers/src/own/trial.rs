//! A model tried before it is offered, with three small requests through the same bodies and
//! readings as real work: an answer at all; a picture bundled with Kalamu that holds a number,
//! which the answer must contain; a form of two fields, which must come back filled. A model is
//! then offered only for what it did.

use kalamu_core::{
    AnalysisSchema, ApiKey, Field, FieldKind, LanguageChoice, PieceRef, PreparedInput,
    ProcessOutput, ProcessRequest, Provider, ProviderError, SetupOptions, StopSignal, Task, Tuning,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{ModelConfig, OwnServer, ServerConfig};
use crate::http::shortened;

/// A small picture: "4729" in large black digits on white, and the word KALAMU.
const PICTURE: &[u8] = include_bytes!("trial.png");
/// The number the picture holds.
const NUMBER: &str = "4729";
/// The longest answer each request of the trial asks for: room for a model that writes a few
/// words before the answer, not for an essay.
const ANSWER_TOKENS: u32 = 1_024;

/// What a model did in its trial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialResult {
    /// Whether it answered at all; why not, in the server's words, shortened.
    pub answers: Result<(), String>,
    /// It read the number in the picture: offered for reading pages.
    pub reads_pages: bool,
    /// It filled the form: offered for analyses, contents, corrections and translations.
    pub fills_forms: bool,
    /// What went wrong in the second or third request, for the researcher.
    pub note: Option<String>,
}

/// Tries `model` on `server`. `vision` is what the server said of the model: where it says the
/// model reads no pictures, the picture is not sent and the answer is no. Each request may take
/// minutes, a model being loaded first; the caller shows progress.
pub async fn try_model(
    client: &Client,
    server: &ServerConfig,
    model: &ModelConfig,
    vision: Option<bool>,
    key: Option<&ApiKey>,
) -> TrialResult {
    let config = ServerConfig {
        models: vec![model.clone()],
        at_once: 1,
        ..server.clone()
    };
    let trial = Trial {
        server: OwnServer::new(client.clone(), config),
        model: &model.id,
        key: key.cloned().unwrap_or_else(|| ApiKey::new("")),
    };
    if let Err(e) = trial.answers().await {
        return TrialResult {
            answers: Err(shortened(&e.to_string())),
            reads_pages: false,
            fills_forms: false,
            note: None,
        };
    }
    let mut notes = Vec::new();
    let reads_pages = if vision == Some(false) {
        notes.push(format!(
            "{} says this model reads no pictures.",
            server.name
        ));
        false
    } else {
        match trial.reads_picture().await {
            Ok(out) if out.text.contains(NUMBER) => true,
            Ok(out) => {
                notes.push(format!(
                    "It did not read the number in the picture; it answered: {}",
                    shortened(out.text.trim())
                ));
                false
            }
            Err(e) => {
                notes.push(format!(
                    "It could not read the picture: {}",
                    shortened(&e.to_string())
                ));
                false
            }
        }
    };
    let fills_forms = match trial.fills_form().await {
        Ok(out) if form_filled(&out) => true,
        Ok(out) => {
            notes.push(format!(
                "It did not fill the form right; it answered: {}",
                shortened(out.text.trim())
            ));
            false
        }
        Err(e) => {
            notes.push(format!(
                "It could not fill the form: {}",
                shortened(&e.to_string())
            ));
            false
        }
    };
    TrialResult {
        answers: Ok(()),
        reads_pages,
        fills_forms,
        note: (!notes.is_empty()).then(|| notes.join(" ")),
    }
}

struct Trial<'a> {
    server: OwnServer,
    model: &'a str,
    key: ApiKey,
}

impl Trial<'_> {
    async fn ask(
        &self,
        task: Task,
        prompt: &str,
        user_prompt: &str,
        input: PreparedInput,
        analysis: Option<&AnalysisSchema>,
    ) -> Result<ProcessOutput, ProviderError> {
        let tuning = Tuning {
            max_output_tokens: Some(ANSWER_TOKENS),
            ..Tuning::default()
        };
        let request = ProcessRequest {
            task,
            model: self.model,
            prompt,
            user_prompt,
            input: &input,
            languages: &LanguageChoice::Auto,
            options: &SetupOptions::default(),
            tuning: &tuning,
            analysis,
            stop: &StopSignal::default(),
        };
        self.server.process(&self.key, request).await
    }

    /// A one-line question; any answer will do.
    async fn answers(&self) -> Result<ProcessOutput, ProviderError> {
        self.ask(
            Task::TextExtraction,
            "Answer in one short sentence.",
            "What is the name in this document?",
            text("trial.txt", "Kalamu"),
            None,
        )
        .await
    }

    async fn reads_picture(&self) -> Result<ProcessOutput, ProviderError> {
        self.ask(
            Task::TextExtraction,
            "You transcribe the text of images exactly, and write nothing else.",
            "Transcribe the text in this image.",
            PreparedInput {
                piece: PieceRef::page(1),
                mime: "image/png".into(),
                bytes: PICTURE.to_vec(),
                file_name: "trial.png".into(),
            },
            None,
        )
        .await
    }

    async fn fills_form(&self) -> Result<ProcessOutput, ProviderError> {
        let fields = form();
        self.ask(
            Task::Analysis,
            "Read the document and fill in the fields from it.",
            "Fill in the fields.",
            text(
                "letter.txt",
                "The letter was written in Bamako in 1957. It reached Dakar a week later.",
            ),
            Some(&fields),
        )
        .await
    }
}

fn text(file_name: &str, words: &str) -> PreparedInput {
    PreparedInput {
        piece: PieceRef::Whole,
        mime: "text/plain".into(),
        bytes: words.as_bytes().to_vec(),
        file_name: file_name.into(),
    }
}

fn form() -> AnalysisSchema {
    let field = |key: &str, label: &str, instructions: &str| Field {
        key: key.into(),
        label: label.into(),
        instructions: instructions.into(),
        kind: FieldKind::Text,
        reason: false,
    };
    AnalysisSchema {
        fields: vec![
            field("city", "City", "The city where the letter was written."),
            field("year", "Year", "The year the letter was written."),
        ],
    }
}

/// Both fields came back, with the right values.
fn form_filled(output: &ProcessOutput) -> bool {
    let value = |key: &str| {
        output
            .fields
            .iter()
            .find(|a| a.key == key)
            .map(|a| a.value.joined().to_lowercase())
            .unwrap_or_default()
    };
    value("city").contains("bamako") && value("year").contains("1957")
}
