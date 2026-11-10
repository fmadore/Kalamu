//! A model's abilities come from the catalogue, through the tuning the engine sends, and never
//! from its name: a model called anything that has the ability is sent the request that uses
//! it, and one named like a model that has it, without it, is not.

use kalamu_core::{
    AnalysisSchema, ApiKey, Field, FieldKind, LanguageChoice, LanguageCode, PieceRef,
    PreparedInput, ProcessRequest, Provider, SetupOptions, StopSignal, Task, Tuning,
};
use kalamu_providers::{Anthropic, Gemini, Mistral, OpenAi, client};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

fn page() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::page(1),
        mime: "application/pdf".into(),
        bytes: b"%PDF-1.5 fake".to_vec(),
        file_name: "lettre-p1.pdf".into(),
    }
}

fn audio() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::Segment {
            index: 0,
            start_ms: 0,
            end_ms: 60_000,
        },
        mime: "audio/mpeg".into(),
        bytes: vec![0u8; 64],
        file_name: "entretien.mp3".into(),
    }
}

fn text() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::Whole,
        mime: "text/plain".into(),
        bytes: "Réunion à Bamako.".as_bytes().to_vec(),
        file_name: "rapport.txt".into(),
    }
}

static FIELDS: std::sync::LazyLock<AnalysisSchema> = std::sync::LazyLock::new(|| {
    let mut schema = AnalysisSchema {
        fields: vec![Field {
            key: String::new(),
            label: "Summary".into(),
            instructions: "What it says.".into(),
            kind: FieldKind::Text,
            reason: false,
        }],
    };
    schema.assign_keys();
    schema
});

static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);

/// The one request `provider` sends for `model` with `tuning`, as the mock server received it.
async fn sent(
    provider: &dyn Provider,
    server: &MockServer,
    task: Task,
    model: &str,
    input: &PreparedInput,
    tuning: &Tuning,
) -> Request {
    let languages =
        LanguageChoice::Explicit(vec![LanguageCode::new("fr"), LanguageCode::new("en")]);
    let options = SetupOptions::default();
    let request = ProcessRequest {
        task,
        model,
        prompt: "Transcribe diplomatically.",
        user_prompt: "Transcribe this page.",
        input,
        languages: &languages,
        options: &options,
        tuning,
        analysis: (task == Task::Analysis).then(|| &*FIELDS),
        stop: &STOP,
    };
    let before = server.received_requests().await.unwrap_or_default().len();
    // Only the request is checked; the empty answer may fail to parse.
    let _ = provider.process(&ApiKey::new("k"), request).await;
    let received = server.received_requests().await.unwrap_or_default();
    assert_eq!(received.len(), before + 1, "{model}: one request");
    received
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("{model}"))
}

fn json(request: &Request) -> serde_json::Value {
    serde_json::from_slice(&request.body).unwrap_or_default()
}

async fn server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn claude_s_fallback_follows_the_catalogue_not_the_model_s_name() {
    let server = server().await;
    let claude = Anthropic::with_base_url(client(), &server.uri());
    let able = Tuning {
        refusal_fallback: true,
        ..Tuning::default()
    };
    let request = sent(
        &claude,
        &server,
        Task::TextExtraction,
        "claude-next",
        &page(),
        &able,
    )
    .await;
    assert_eq!(json(&request)["fallbacks"], "default");
    assert!(request.headers.get("anthropic-beta").is_some());
    let named = claude.request_settings(Task::TextExtraction, "claude-next", &able);
    assert!(named.iter().any(|s| s.name == "fallbacks"));

    // Opus by name, without the ability: nothing asks for a fallback.
    let plain = Tuning::default();
    let request = sent(
        &claude,
        &server,
        Task::TextExtraction,
        "claude-opus-5-5",
        &page(),
        &plain,
    )
    .await;
    assert!(json(&request).get("fallbacks").is_none());
    assert!(request.headers.get("anthropic-beta").is_none());
    let named = claude.request_settings(Task::TextExtraction, "claude-opus-5-5", &plain);
    assert!(named.iter().all(|s| s.name != "fallbacks"));
}

#[tokio::test]
async fn openai_asks_for_speakers_of_a_model_that_labels_them_whatever_its_name() {
    let server = server().await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let form = |request: &Request| String::from_utf8_lossy(&request.body).into_owned();
    let labels = Tuning {
        speakers: true,
        no_prompt: true,
        ..Tuning::default()
    };
    let body = form(
        &sent(
            &openai,
            &server,
            Task::AudioTranscription,
            "speech-1",
            &audio(),
            &labels,
        )
        .await,
    );
    assert!(body.contains("diarized_json"), "{body}");
    assert!(body.contains("chunking_strategy"), "{body}");
    // The diarizing request takes one language; with two it detects them.
    assert!(!body.contains("languages[]"), "{body}");

    // The diarizing model's name, without the ability: plain text, every language hinted.
    let plain = Tuning {
        no_prompt: true,
        ..Tuning::default()
    };
    let body = form(
        &sent(
            &openai,
            &server,
            Task::AudioTranscription,
            "gpt-4o-transcribe-diarize",
            &audio(),
            &plain,
        )
        .await,
    );
    assert!(!body.contains("diarized_json"), "{body}");
    assert!(!body.contains("chunking_strategy"), "{body}");
    assert!(body.contains("languages[]"), "{body}");
}

#[tokio::test]
async fn gemini_sends_the_speech_request_to_a_model_that_takes_no_prompt() {
    let server = server().await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let speech = Tuning {
        no_prompt: true,
        ..Tuning::default()
    };
    let request = sent(
        &gemini,
        &server,
        Task::AudioTranscription,
        "voice-2",
        &audio(),
        &speech,
    )
    .await;
    assert!(
        request
            .url
            .path()
            .ends_with("/models/voice-2:generateContent")
    );
    let body = json(&request);
    assert!(body.get("systemInstruction").is_none(), "{body}");
    assert!(
        body.pointer("/generationConfig/audioTranscriptionConfig")
            .is_some(),
        "{body}"
    );
    assert!(
        gemini
            .request_settings(Task::AudioTranscription, "voice-2", &speech)
            .is_empty()
    );

    // Named like Google's speech model but taking instructions: the ordinary request.
    let chat = Tuning::default();
    let body = json(
        &sent(
            &gemini,
            &server,
            Task::AudioTranscription,
            "gemini-3.5-transcribe",
            &audio(),
            &chat,
        )
        .await,
    );
    assert!(body.get("systemInstruction").is_some(), "{body}");
    assert!(
        body.pointer("/generationConfig/audioTranscriptionConfig")
            .is_none(),
        "{body}"
    );
}

#[tokio::test]
async fn mistral_sends_the_catalogue_s_level_and_guesses_none_from_a_name() {
    let server = server().await;
    let mistral = Mistral::with_base_url(client(), &server.uri());
    let low = Tuning {
        reasoning: Some("low".into()),
        ..Tuning::default()
    };
    let body = json(&sent(&mistral, &server, Task::Analysis, "hosted-9", &text(), &low).await);
    assert_eq!(body["reasoning_effort"], "low");

    // A model the catalogue lacks, named like GLM: no level is guessed from its name, and
    // none is recorded.
    let lacking = Tuning::default();
    let body = json(
        &sent(
            &mistral,
            &server,
            Task::Analysis,
            "zai-glm-9",
            &text(),
            &lacking,
        )
        .await,
    );
    assert!(body.get("reasoning_effort").is_none(), "{body}");
    assert!(
        mistral
            .request_settings(Task::Analysis, "zai-glm-9", &lacking)
            .iter()
            .all(|s| s.name != "reasoning_effort")
    );
}
