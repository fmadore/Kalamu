//! A job whose model cannot read its sources is refused when it is made, with media's own
//! reason, instead of failing piece by piece once it runs.

use super::*;

/// Stands in for a provider whose catalogue lists both models that read text only and models
/// that read PDFs and images: what the job is refused for comes from the model's inputs.
struct Mixed;

#[async_trait]
impl Provider for Mixed {
    fn id(&self) -> ProviderId {
        ProviderId::new("mistral")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["text/plain", "application/pdf", "image/png", "image/jpeg"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        _request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        Err(ProviderError::Other("nothing is sent".into()))
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

fn engine() -> Arc<Engine> {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    Engine::new(
        store,
        vec![Arc::new(Mixed)],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig::default(),
    )
}

fn job(task: Task, model: &str, sources: Vec<SourceFile>) -> NewJob {
    NewJob {
        name: "Test".into(),
        task,
        sources,
        setups: vec![SetupRequest {
            provider: ProviderId::new("mistral"),
            model: model.into(),
            preset_id: None,
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: kalamu_core::JobSettings {
            languages: kalamu_core::LanguageChoice::Explicit(vec![kalamu_core::LanguageCode::new(
                "en",
            )]),
            ..Default::default()
        },
        output: OutputSettings::default(),
    }
}

/// A source of the given kind, with a name to read in the error.
fn source(f: &Fixture, kind: kalamu_core::InputKind, name: &str) -> SourceFile {
    SourceFile {
        kind,
        file_name: name.into(),
        ..f.pdf("any.pdf", 1)
    }
}

#[test]
fn a_text_only_model_is_refused_images_and_whole_pdfs() {
    let f = fixture();
    let engine = engine();
    let photo = source(&f, kalamu_core::InputKind::Image, "photo.jpg");
    let scan = source(&f, kalamu_core::InputKind::Pdf, "scan.pdf");
    let text = source(&f, kalamu_core::InputKind::Text, "notes.txt");
    for refused in [photo.clone(), scan.clone()] {
        let name = refused.file_name.clone();
        let result = engine.create_job(job(Task::Analysis, "zai-glm-5-3", vec![refused]));
        match result {
            Err(e @ EngineError::CannotRead { .. }) => {
                assert_eq!(e.code(), "provider.cannot_read");
                assert!(e.to_string().contains(&name), "{e}");
                assert!(e.to_string().contains("reads text only"), "{e}");
            }
            other => panic!("expected CannotRead, got {other:?}"),
        }
    }
    // It reads a text, and the same images on a model that reads them are accepted.
    assert!(
        engine
            .create_job(job(Task::Analysis, "zai-glm-5-3", vec![text.clone()]))
            .is_ok()
    );
    assert!(
        engine
            .create_job(job(
                Task::Analysis,
                "zai-glm-5-3-flash",
                vec![photo, scan.clone()]
            ))
            .is_ok()
    );
    // A translation of a PDF reads its text layer, which a text-only model reads.
    assert!(
        engine
            .create_job(job(Task::Translation, "zai-glm-5-3", vec![scan, text]))
            .is_ok()
    );
}

/// The image description (#168) on a model that reads text only is refused for JPEG, PNG and
/// TIFF files as they are inspected, naming the file; a model that reads images takes them.
#[test]
fn the_image_description_is_refused_on_a_model_that_does_not_read_images() {
    use std::io::Cursor;

    let f = fixture();
    let engine = engine();
    let images: Vec<SourceFile> = [
        ("photo.jpg", image::ImageFormat::Jpeg),
        ("poster.png", image::ImageFormat::Png),
        ("plan.tif", image::ImageFormat::Tiff),
    ]
    .into_iter()
    .map(|(name, format)| {
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::new(32, 24))
            .write_to(&mut Cursor::new(&mut bytes), format)
            .unwrap_or_else(|e| panic!("{e}"));
        let path = f.dir.path().join(name);
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{e}"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("inspect: {e}"))
    })
    .collect();
    let described = |model: &str, sources: Vec<SourceFile>| {
        let mut new = job(Task::Analysis, model, sources);
        new.setups[0].preset_id = Some("analysis.image_description".into());
        new
    };
    for refused in &images {
        assert_eq!(refused.kind, kalamu_core::InputKind::Image);
        match engine.create_job(described("zai-glm-5-3", vec![refused.clone()])) {
            Err(e @ EngineError::CannotRead { .. }) => {
                assert!(e.to_string().contains(&refused.file_name), "{e}");
            }
            other => panic!("expected CannotRead, got {other:?}"),
        }
    }
    assert!(
        engine
            .create_job(described("zai-glm-5-3-flash", images))
            .is_ok()
    );
}

/// A workflow is checked the same way when it starts: a step analysing the files themselves on a
/// model that reads text only is refused, a step analysing an earlier step's text is not.
#[test]
fn a_workflow_step_that_cannot_read_the_files_is_refused_when_the_run_starts() {
    let f = fixture();
    let engine = engine();
    let photo = source(&f, kalamu_core::InputKind::Image, "photo.jpg");
    let check = |new: &NewJob| {
        let catalog = engine.current_catalog();
        engine.check_new(&catalog, new)
    };
    assert!(matches!(
        check(&job(Task::Analysis, "zai-glm-5-3", vec![photo.clone()])),
        Err(EngineError::CannotRead { .. })
    ));
    let text = crate::workflow::as_text(&photo);
    assert!(check(&job(Task::Analysis, "zai-glm-5-3", vec![text])).is_ok());
}
