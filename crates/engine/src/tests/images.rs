//! The ready-made *Image description and keywords* (#168) over the kinds of images archives
//! hold: JPEG, PNG and TIFF files and a PDF of photographs, in one analysis job, each answered
//! with the fields the preset asks for.

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbImage};
use kalamu_core::{InputKind, PieceRef};

use super::*;

/// The answers a model wrote for each source, recorded as its structured output gives them:
/// one JSON object with the preset's fields, read back as a provider reads an answer.
const ANSWERS: [(&str, &str); 4] = [
    (
        "market",
        r#"{"description": "Women selling grain from baskets under a thatched shelter, a man with a bicycle at the left.", "kind_of_image": "Photograph", "keywords": ["market", "grain", "baskets", "bicycle"], "text_in_the_image": "", "people": [], "places": [], "organisations": []}"#,
    ),
    (
        "poster",
        r#"{"description": "A poster announcing a football match, two players drawn above the date.", "kind_of_image": "Poster or advertisement", "keywords": ["football", "match", "poster"], "text_in_the_image": "GRAND MATCH\nStade de Bamako, 12 mars", "people": [], "places": ["Bamako"], "organisations": []}"#,
    ),
    (
        "plan",
        r#"{"description": "A hand-drawn plan of a compound with a courtyard, a well and four rooms.", "kind_of_image": "Map or plan", "keywords": ["plan", "compound", "courtyard", "well"], "text_in_the_image": "Concession Traoré", "people": ["Traoré"], "places": [], "organisations": []}"#,
    ),
    (
        "album",
        r#"{"description": "A group of pupils posed in three rows before a school building.", "kind_of_image": "Photograph", "keywords": ["pupils", "school", "group portrait"], "text_in_the_image": "École régionale de Ségou, 1952", "people": [], "places": ["Ségou"], "organisations": ["École régionale de Ségou"]}"#,
    ),
];

/// The keys of the preset's fields, in order: what its recorded answers carry.
const KEYS: [&str; 7] = [
    "description",
    "kind_of_image",
    "keywords",
    "text_in_the_image",
    "people",
    "places",
    "organisations",
];

/// A model that reads images and PDFs, and answers each source with what was recorded for it.
#[derive(Default)]
struct Describer {
    /// The file name and type of each input sent, in order of arrival.
    sent: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl Provider for Describer {
    fn id(&self) -> ProviderId {
        ProviderId::new("describer")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf", "image/png", "image/jpeg"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        let input = request.input;
        assert_eq!(
            input.piece,
            PieceRef::Whole,
            "an analysis reads a source whole"
        );
        let schema = request
            .analysis
            .ok_or_else(|| ProviderError::InvalidRequest("no fields".into()))?;
        let keys: Vec<&str> = schema.fields.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, KEYS);
        assert!(
            request.prompt.contains("Describe only what can be seen"),
            "the preset's instructions are sent"
        );
        self.sent
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push((input.file_name.clone(), input.mime.clone()));
        let stem = input.file_name.split('.').next().unwrap_or_default();
        let answer = ANSWERS
            .iter()
            .find(|(name, _)| *name == stem)
            .map(|(_, answer)| *answer)
            .ok_or_else(|| ProviderError::Other(format!("nothing recorded for {stem}")))?;
        kalamu_core::structured::finish(
            ProcessOutput {
                text: answer.into(),
                ..ProcessOutput::default()
            },
            schema,
        )
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

/// A small image in `format`, a little different for each file.
fn image(format: ImageFormat, shade: u8) -> Vec<u8> {
    let mut out = Vec::new();
    let img = RgbImage::from_pixel(64, 48, image::Rgb([shade, 120, 200 - shade]));
    DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), format)
        .unwrap_or_else(|e| panic!("encode: {e}"));
    out
}

fn inspect(dir: &std::path::Path, name: &str, bytes: &[u8]) -> SourceFile {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{e}"));
    kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("inspect {name}: {e}"))
}

#[tokio::test]
async fn the_image_description_answers_jpeg_png_tiff_and_a_pdf_of_photographs() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let describer = Arc::new(Describer::default());
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![describer.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let sources = vec![
        inspect(dir.path(), "market.jpg", &image(ImageFormat::Jpeg, 10)),
        inspect(dir.path(), "poster.png", &image(ImageFormat::Png, 40)),
        inspect(dir.path(), "plan.tif", &image(ImageFormat::Tiff, 70)),
        // Two photographs as scanned pages, with no text layer.
        inspect(
            dir.path(),
            "album.pdf",
            &kalamu_media::sample_pdf(&[(595.0, 842.0), (842.0, 595.0)], false),
        ),
    ];
    let kinds: Vec<InputKind> = sources.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        [
            InputKind::Image,
            InputKind::Image,
            InputKind::Image,
            InputKind::Pdf
        ]
    );
    let job = engine
        .create_job(NewJob {
            name: "Photographs".into(),
            task: Task::Analysis,
            sources: sources.clone(),
            setups: vec![SetupRequest {
                provider: ProviderId::new("describer"),
                model: "describer-1".into(),
                preset_id: Some("analysis.image_description".into()),
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: kalamu_core::JobSettings {
                languages: kalamu_core::LanguageChoice::Explicit(vec![
                    kalamu_core::LanguageCode::new("en"),
                ]),
                ..Default::default()
            },
            output: OutputSettings {
                formats: vec![ExportFormat::Csv],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("create job: {e}"));

    let mut events = engine.subscribe();
    engine
        .start(job.id)
        .unwrap_or_else(|e| panic!("start: {e}"));
    let status = loop {
        match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { job_id, status })) if job_id == job.id => {
                break status;
            }
            Ok(Ok(_)) => {}
            other => panic!("no JobFinished event: {other:?}"),
        }
    };
    assert_eq!(status, JobStatus::Completed);

    // One call a source, each as its kind is read: a JPEG as it is, a TIFF made a PNG, the PDF
    // whole.
    let mut sent = describer
        .sent
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    sent.sort();
    let sent: Vec<(&str, &str)> = sent
        .iter()
        .map(|(name, mime)| (name.as_str(), mime.as_str()))
        .collect();
    assert_eq!(
        sent,
        [
            ("album.pdf", "application/pdf"),
            ("market.jpg", "image/jpeg"),
            ("plan.png", "image/png"),
            ("poster.png", "image/png"),
        ]
    );

    // Each source has its own structured answers, the fields as the preset names them.
    let field = |source: &SourceFile, key: &str| -> FieldValue {
        let pieces = engine
            .source_pieces(job.id, source.id)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(pieces.len(), 1, "{}", source.file_name);
        assert_eq!(pieces[0].status, PieceStatus::Done, "{}", source.file_name);
        let keys: Vec<&str> = pieces[0].ai_fields.iter().map(|a| a.key.as_str()).collect();
        assert_eq!(keys, KEYS, "{}", source.file_name);
        pieces[0]
            .ai_fields
            .iter()
            .find(|a| a.key == key)
            .map(|a| a.value.clone())
            .unwrap_or_else(|| panic!("no {key} for {}", source.file_name))
    };
    let text = |v: FieldValue| match v {
        FieldValue::Text(t) => t,
        other => panic!("not a text: {other:?}"),
    };
    let list = |v: FieldValue| match v {
        FieldValue::List(items) => items,
        other => panic!("not a list: {other:?}"),
    };
    let (market, poster, plan, album) = (&sources[0], &sources[1], &sources[2], &sources[3]);
    assert!(text(field(market, "description")).starts_with("Women selling grain"));
    assert_eq!(
        list(field(market, "keywords")),
        ["market", "grain", "baskets", "bicycle"]
    );
    assert!(text(field(market, "text_in_the_image")).is_empty());
    assert_eq!(
        text(field(poster, "kind_of_image")),
        "Poster or advertisement"
    );
    assert_eq!(list(field(poster, "places")), ["Bamako"]);
    assert_eq!(text(field(plan, "kind_of_image")), "Map or plan");
    assert_eq!(list(field(plan, "people")), ["Traoré"]);
    assert_eq!(
        text(field(album, "text_in_the_image")),
        "École régionale de Ségou, 1952"
    );
    assert_eq!(
        list(field(album, "organisations")),
        ["École régionale de Ségou"]
    );
    assert_eq!(text(field(album, "kind_of_image")), "Photograph");
}
