//! Each cloud embedder against a local mock server: the request must carry the texts, the role
//! and the length asked for in the provider's documented shape, and the answer must come back
//! as one vector of length 1 for each text, in order, with the tokens billed. Response bodies
//! follow each API's documented shape; never a live call.

use std::sync::Arc;

use kalamu_core::{
    ApiKey, EmbedRequest, EmbedRole, Embedded, Embedder, ProviderError, ProviderId, StopSignal,
};
use kalamu_providers::{Gemini, Mistral, OpenAi, client, embedders};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);

fn texts() -> Vec<String> {
    vec![
        "Réunion à la mosquée de Bamako, 3 mai 1957.".into(),
        "اجتماع في المسجد".into(),
    ]
}

fn request<'a>(
    model: &'a str,
    role: EmbedRole,
    texts: &'a [String],
    dimensions: Option<u32>,
) -> EmbedRequest<'a> {
    EmbedRequest {
        model,
        role,
        texts,
        dimensions,
        stop: &STOP,
    }
}

fn length(v: &[f32]) -> f64 {
    v.iter()
        .map(|x| f64::from(*x) * f64::from(*x))
        .sum::<f64>()
        .sqrt()
}

/// Two vectors of length 1, one for each text, in the order asked.
fn assert_unit_pair(out: &Result<Embedded, ProviderError>, dimensions: usize) {
    let Ok(out) = out else {
        panic!("{out:?}");
    };
    assert_eq!(out.vectors.len(), 2);
    for v in &out.vectors {
        assert_eq!(v.len(), dimensions);
        assert!((length(v) - 1.0).abs() < 1e-5, "{}", length(v));
    }
    // The second text's vector points elsewhere than the first's.
    assert!(out.vectors[0] != out.vectors[1]);
}

fn openai_answer(vectors: &[(usize, Vec<f32>)], tokens: u64) -> serde_json::Value {
    json!({
        "object": "list",
        "data": vectors.iter().map(|(i, v)| json!({
            "object": "embedding", "index": i, "embedding": v
        })).collect::<Vec<_>>(),
        "model": "text-embedding-3-small",
        "usage": { "prompt_tokens": tokens, "total_tokens": tokens }
    })
}

#[tokio::test]
async fn gemini_embeds_a_passage_and_a_query_each_with_its_own_prefix() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/gemini-embedding-2:batchEmbedContents"))
        .and(header("x-goog-api-key", "test-key"))
        .and(body_partial_json(json!({ "requests": [
            {
                "model": "models/gemini-embedding-2",
                "content": { "parts": [{ "text": "title: none | text: Réunion à la mosquée de Bamako, 3 mai 1957." }] }
            },
            {
                "model": "models/gemini-embedding-2",
                "content": { "parts": [{ "text": "title: none | text: اجتماع في المسجد" }] }
            }
        ]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            // Already of length 1, as Google returns them at the full length.
            "embeddings": [
                { "values": [0.6, 0.8, 0.0] },
                { "values": [0.0, 0.6, 0.8] }
            ],
            "usageMetadata": { "promptTokenCount": 31 }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/gemini-embedding-2:batchEmbedContents"))
        .and(body_partial_json(json!({ "requests": [{
            "content": { "parts": [{ "text": "task: search result | query: imams de Bamako" }] }
        }]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embeddings": [{ "values": [0.0, 0.0, 2.0] }]
        })))
        .mount(&server)
        .await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let key = ApiKey::new("test-key");
    let texts = texts();
    let out = gemini
        .embed(
            Some(&key),
            request("gemini-embedding-2", EmbedRole::Passage, &texts, None),
        )
        .await;
    assert_unit_pair(&out, 3);
    assert_eq!(out.map(|o| o.input_tokens).ok(), Some(31));

    let query = vec!["imams de Bamako".to_string()];
    let out = gemini
        .embed(
            Some(&key),
            request("gemini-embedding-2", EmbedRole::Query, &query, None),
        )
        .await;
    // Brought to length 1; no count in the answer, none billed.
    assert_eq!(
        out,
        Ok(Embedded {
            vectors: vec![vec![0.0, 0.0, 1.0]],
            input_tokens: 0
        })
    );
}

#[tokio::test]
async fn gemini_asks_for_a_shorter_length_and_brings_it_to_length_one() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/gemini-embedding-2:batchEmbedContents"))
        .and(body_partial_json(json!({ "requests": [
            { "outputDimensionality": 4 },
            { "outputDimensionality": 4 }
        ]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embeddings": [
                { "values": [1.0, 1.0, 1.0, 1.0] },
                { "values": [3.0, 0.0, 4.0, 0.0] }
            ]
        })))
        .mount(&server)
        .await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let texts = texts();
    let out = gemini
        .embed(
            Some(&ApiKey::new("k")),
            request("gemini-embedding-2", EmbedRole::Passage, &texts, Some(4)),
        )
        .await;
    assert_unit_pair(&out, 4);
    let first = out.map(|o| o.vectors[0].clone()).unwrap_or_default();
    assert!(first.iter().all(|x| (x - 0.5).abs() < 1e-6), "{first:?}");
}

#[tokio::test]
async fn openai_sends_the_texts_and_the_length_and_puts_the_vectors_back_in_order() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "model": "text-embedding-3-small",
            "input": texts(),
            "dimensions": 3,
            "encoding_format": "float"
        })))
        // Listed out of order, as `index` allows.
        .respond_with(ResponseTemplate::new(200).set_body_json(openai_answer(
            &[(1, vec![0.0, 0.6, 0.8]), (0, vec![0.6, 0.8, 0.0])],
            24,
        )))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let texts = texts();
    let out = openai
        .embed(
            Some(&ApiKey::new("test-key")),
            request("text-embedding-3-small", EmbedRole::Query, &texts, Some(3)),
        )
        .await;
    assert_unit_pair(&out, 3);
    let out = out.unwrap_or(Embedded {
        vectors: Vec::new(),
        input_tokens: 0,
    });
    assert_eq!(out.vectors[0], vec![0.6, 0.8, 0.0]);
    assert_eq!(out.input_tokens, 24);
}

#[tokio::test]
async fn openai_and_mistral_send_no_role_and_mistral_its_own_length_field() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "model": "mistral-embed",
            "input": ["Bamako"],
            "output_dimension": 3
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(openai_answer(&[(0, vec![0.0, 1.0, 0.0])], 3)),
        )
        .mount(&server)
        .await;
    let mistral = Mistral::with_base_url(client(), &server.uri());
    let one = vec!["Bamako".to_string()];
    for role in [EmbedRole::Passage, EmbedRole::Query] {
        let out = mistral
            .embed(
                Some(&ApiKey::new("test-key")),
                request("mistral-embed", role, &one, Some(3)),
            )
            .await;
        assert_eq!(
            out,
            Ok(Embedded {
                vectors: vec![vec![0.0, 1.0, 0.0]],
                input_tokens: 3
            })
        );
    }
    // Neither a query nor a passage changed what was sent.
    let bodies: Vec<serde_json::Value> = server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .filter_map(|r| serde_json::from_slice(&r.body).ok())
        .collect();
    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[0], bodies[1]);
    assert!(bodies[0].get("role").is_none() && bodies[0].get("input_type").is_none());
}

#[tokio::test]
async fn rate_limits_refused_keys_and_texts_too_long_are_told_apart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/limited:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": { "code": 429, "message": "Quota exceeded. Please retry in 7s.", "status": "RESOURCE_EXHAUSTED" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/badkey:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "code": 400, "message": "API key not valid. Please pass a valid API key.", "status": "INVALID_ARGUMENT" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/short:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "code": 400, "message": "The input token count (9001) exceeds the maximum number of tokens allowed (8192).", "status": "INVALID_ARGUMENT" }
        })))
        .mount(&server)
        .await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let key = ApiKey::new("secret-test-key");
    let texts = texts();
    let call = |model: &'static str| {
        let gemini = &gemini;
        let key = &key;
        let texts = &texts;
        async move {
            gemini
                .embed(Some(key), request(model, EmbedRole::Passage, texts, None))
                .await
        }
    };
    assert!(matches!(
        call("limited").await,
        Err(ProviderError::RateLimited { retry_after: Some(d), .. }) if d.as_secs() == 7
    ));
    let refused = call("badkey").await;
    assert!(
        matches!(refused, Err(ProviderError::Auth(_))),
        "{refused:?}"
    );
    let long = call("short").await;
    assert!(
        matches!(&long, Err(ProviderError::InvalidRequest(m)) if m.contains("exceeds")),
        "{long:?}"
    );
    assert!(long.as_ref().is_err_and(|e| !e.is_retryable()));
    // The key is in no error.
    for e in [refused, long].into_iter().filter_map(Result::err) {
        assert!(!e.to_string().contains("secret-test-key"));
    }

    // OpenAI's refusal of a key, by its status.
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "message": "Incorrect API key provided", "type": "invalid_request_error" }
        })))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let refused = openai
        .embed(
            Some(&key),
            request("text-embedding-3-large", EmbedRole::Passage, &texts, None),
        )
        .await;
    assert!(
        matches!(refused, Err(ProviderError::Auth(_))),
        "{refused:?}"
    );
}

#[tokio::test]
async fn a_text_beyond_the_limit_is_refused_before_anything_is_sent() {
    let server = MockServer::start().await;
    let embedders: Vec<Arc<dyn Embedder>> = vec![
        Arc::new(Gemini::with_base_url(client(), &server.uri())),
        Arc::new(OpenAi::with_base_url(client(), &server.uri())),
        Arc::new(Mistral::with_base_url(client(), &server.uri())),
    ];
    for embedder in &embedders {
        let limits = embedder.limits("any");
        let long = vec!["a".repeat(limits.chars + 1)];
        let out = embedder
            .embed(
                Some(&ApiKey::new("k")),
                request("any", EmbedRole::Passage, &long, None),
            )
            .await;
        assert!(
            matches!(&out, Err(ProviderError::InvalidRequest(m)) if m.contains("too long")),
            "{}: {out:?}",
            embedder.id()
        );
        let many = vec!["mot".to_string(); limits.texts + 1];
        let out = embedder
            .embed(
                Some(&ApiKey::new("k")),
                request("any", EmbedRole::Passage, &many, None),
            )
            .await;
        assert!(
            matches!(out, Err(ProviderError::InvalidRequest(_))),
            "{}",
            embedder.id()
        );
        // No text, no call.
        let out = embedder
            .embed(None, request("any", EmbedRole::Query, &[], None))
            .await;
        assert_eq!(out.map(|o| o.vectors.len()), Ok(0));
    }
    assert_eq!(server.received_requests().await.map(|r| r.len()), Some(0));
}

#[tokio::test]
async fn vectors_of_the_wrong_count_or_length_are_refused() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/one-missing:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embeddings": [{ "values": [0.6, 0.8] }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/two-lengths:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embeddings": [{ "values": [0.6, 0.8] }, { "values": [1.0, 0.0, 0.0] }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/not-asked:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embeddings": [{ "values": [0.6, 0.8] }, { "values": [0.0, 1.0] }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/zero:batchEmbedContents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embeddings": [{ "values": [0.0, 0.0] }, { "values": [0.0, 1.0] }]
        })))
        .mount(&server)
        .await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    let texts = texts();
    for (model, dimensions, says) in [
        ("one-missing", None, "1 vectors came back for 2 texts"),
        ("two-lengths", None, "has 3 numbers where 2 were expected"),
        (
            "not-asked",
            Some(768),
            "has 2 numbers where 768 were expected",
        ),
        ("zero", None, "cannot be compared"),
    ] {
        let out = gemini
            .embed(
                Some(&key),
                request(model, EmbedRole::Passage, &texts, dimensions),
            )
            .await;
        assert!(
            matches!(&out, Err(ProviderError::Other(m)) if m.contains(says)),
            "{model}: {out:?}"
        );
    }

    // OpenAI's answer that names one text twice and the other not at all.
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(openai_answer(
            &[(0, vec![1.0, 0.0]), (0, vec![0.0, 1.0])],
            5,
        )))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let out = openai
        .embed(
            Some(&key),
            request("text-embedding-3-small", EmbedRole::Passage, &texts, None),
        )
        .await;
    assert!(matches!(out, Err(ProviderError::Other(_))), "{out:?}");
}

/// What the catalogue says of each cloud embedding model is what its embedder does: the same
/// provider, the same most characters in one text. A model whose provider has no embedder here
/// runs on this computer.
#[test]
fn the_catalogue_and_the_embedders_agree() {
    let catalog = kalamu_catalog::Catalog::bundled();
    let embedders = embedders(client());
    let ids: Vec<ProviderId> = embedders.iter().map(|e| e.id()).collect();
    assert_eq!(
        ids,
        [
            ProviderId::new("gemini"),
            ProviderId::new("openai"),
            ProviderId::new("mistral")
        ]
    );
    let models = catalog.embedding_models();
    assert!(!models.is_empty());
    for model in models {
        match embedders.iter().find(|e| e.id() == model.provider) {
            Some(embedder) => {
                assert!(!model.local, "{}", model.id);
                assert!(embedder.needs_key());
                assert_eq!(
                    embedder.limits(&model.id).chars,
                    model.max_chars as usize,
                    "{}",
                    model.id
                );
            }
            None => assert!(model.local, "{} has no embedder", model.id),
        }
    }
    // Every embedder has a model in the catalogue.
    for id in &ids {
        assert!(models.iter().any(|m| &m.provider == id), "{id}");
    }
}
