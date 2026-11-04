//! Tesseract on this computer: text extraction from printed pages, with nothing sent anywhere
//! and no key. Each word comes back with its place on the page, which gives ALTO and the
//! searchable PDF.
//!
//! Tesseract 5 is built from source and linked in (Apache 2.0), through `kreuzberg-tesseract`'s
//! safe API; its language data is downloaded from the app (`tessdata.rs`). Pages are read with
//! the LSTM engine and automatic page segmentation, Tesseract's own defaults. Tesseract does not
//! detect the language: the job names it, and every language named is loaded together.
//!
//! Tesseract reads a page on one core (it is built without OpenMP), so several pages are read
//! at once, each by an engine of its own, up to [`max_engines`]; an engine is kept for the next
//! page and released after a few minutes unused.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, LanguageChoice, ProcessOutput, ProcessRequest, Provider, ProviderError, ProviderId,
    RequestSetting, Task, TextLine, TextRegion, TextWord, Tuning, Usage,
};
use kreuzberg_tesseract::{TessPageSegMode, TesseractAPI};

use crate::models::{ModelFile, ModelLease, ModelManager};
use crate::tessdata::{self, DATA_REVISION};

pub const TESSERACT_ID: &str = "tesseract";
/// The one model in the catalogue: Tesseract 5 with `tessdata_best`. A new engine or data
/// revision that reads differently gets a new id, so results cached under this one stay theirs.
pub const MODEL: &str = "tesseract-5";

/// Loaded language data is released after this long unused, to give its memory back.
const IDLE_UNLOAD: Duration = Duration::from_secs(180);

pub struct Tesseract {
    inner: Arc<Inner>,
}

struct Inner {
    data: ModelManager,
    /// Engines not reading a page, each with its languages loaded.
    idle: Mutex<Vec<Engine>>,
    /// One permit per page read at once.
    slots: Arc<Semaphore>,
    janitor: Mutex<bool>,
}

/// Tesseract with the language data of one job loaded.
struct Engine {
    /// The languages loaded, as Tesseract names them (`fra+eng`).
    names: String,
    api: TesseractAPI,
    _leases: Vec<ModelLease>,
    last_used: Instant,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// How many pages are read at once: one per core, leaving one for the interface, and at most
/// four, beyond which memory grows faster than the job does.
fn max_engines(cores: usize) -> usize {
    cores.saturating_sub(1).clamp(1, 4)
}

impl Tesseract {
    pub fn new(data: ModelManager) -> Self {
        let cores = std::thread::available_parallelism().map_or(2, usize::from);
        Self {
            inner: Arc::new(Inner {
                data,
                idle: Mutex::new(Vec::new()),
                slots: Arc::new(Semaphore::new(max_engines(cores))),
                janitor: Mutex::new(false),
            }),
        }
    }

    pub fn data(&self) -> &ModelManager {
        &self.inner.data
    }

    /// The version of the Tesseract built into the app.
    pub fn version() -> String {
        TesseractAPI::version()
    }

    /// Starts, once, the task that releases engines left unused.
    fn start_janitor(&self) {
        let mut started = lock(&self.inner.janitor);
        if *started {
            return;
        }
        *started = true;
        let inner: Weak<Inner> = Arc::downgrade(&self.inner);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let Some(inner) = inner.upgrade() else {
                    return;
                };
                let mut idle = lock(&inner.idle);
                let before = idle.len();
                idle.retain(|e| e.last_used.elapsed() <= IDLE_UNLOAD);
                if idle.len() < before {
                    tracing::info!("released {} idle Tesseract engines", before - idle.len());
                }
            }
        });
    }
}

/// The language data the job's languages need, or why it cannot run.
fn data_for(languages: &LanguageChoice) -> Result<Vec<&'static ModelFile>, ProviderError> {
    let codes = languages.codes();
    if codes.is_empty() {
        return Err(ProviderError::InvalidRequest(
            "Tesseract does not detect the language: choose the document's languages".into(),
        ));
    }
    let mut files: Vec<&'static ModelFile> = Vec::new();
    for code in codes {
        let file = tessdata::for_language(code.as_str()).ok_or_else(|| {
            ProviderError::Unsupported(format!("Tesseract has no model for {}", code.as_str()))
        })?;
        if !files.iter().any(|f| f.id == file.id) {
            files.push(file);
        }
    }
    Ok(files)
}

fn names(files: &[&'static ModelFile]) -> String {
    files
        .iter()
        .map(|f| tessdata::tesseract_name(f))
        .collect::<Vec<_>>()
        .join("+")
}

impl Inner {
    /// An engine with `files` loaded: an idle one, or a new one, its files checked against
    /// their hashes before Tesseract opens them. Idle engines with other languages are released.
    fn take(&self, files: &[&'static ModelFile]) -> Result<Engine, ProviderError> {
        let wanted = names(files);
        {
            let mut idle = lock(&self.idle);
            idle.retain(|e| e.names == wanted);
            if let Some(engine) = idle.pop() {
                return Ok(engine);
            }
        }
        let mut leases = Vec::new();
        for file in files {
            let lease = self.data.lease(file)?;
            lease.read_verified()?;
            leases.push(lease);
        }
        let api = TesseractAPI::new().map_err(engine_error)?;
        // Tesseract opens the files itself, by a path in the system's encoding; on Windows the
        // app declares UTF-8 as that encoding, so a user folder with accents works.
        let dir = self.data.dir().to_string_lossy();
        api.init(dir.as_ref(), &wanted).map_err(|e| {
            ProviderError::Other(format!("Tesseract could not load its language data: {e}"))
        })?;
        api.set_page_seg_mode(TessPageSegMode::PSM_AUTO)
            .map_err(engine_error)?;
        Ok(Engine {
            names: wanted,
            api,
            _leases: leases,
            last_used: Instant::now(),
        })
    }

    /// Keeps an engine for the next page.
    fn give_back(&self, mut engine: Engine) {
        engine.last_used = Instant::now();
        lock(&self.idle).push(engine);
    }
}

fn engine_error(e: impl std::fmt::Display) -> ProviderError {
    ProviderError::Other(format!("Tesseract failed: {e}"))
}

/// A page decoded to grey levels, as Tesseract reads it.
struct Page {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

fn decode(bytes: &[u8]) -> Result<Page, ProviderError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| ProviderError::InvalidRequest(format!("The page image is unreadable: {e}")))?
        .into_luma8();
    let (width, height) = image.dimensions();
    Ok(Page {
        pixels: image.into_raw(),
        width,
        height,
    })
}

fn recognise(api: &TesseractAPI, page: &Page) -> Result<String, ProviderError> {
    let (w, h) = (
        i32::try_from(page.width).map_err(engine_error)?,
        i32::try_from(page.height).map_err(engine_error)?,
    );
    api.set_image(&page.pixels, w, h, 1, w)
        .map_err(engine_error)?;
    api.recognize().map_err(engine_error)?;
    api.get_tsv_text(0).map_err(engine_error)
}

/// One row of Tesseract's TSV: its level (2 block, 3 paragraph, 4 line, 5 word), where it
/// belongs, its box in pixels, its confidence and its text.
struct Row<'a> {
    level: u8,
    block: u32,
    paragraph: u32,
    line: u32,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    confidence: f32,
    text: &'a str,
}

fn rows(tsv: &str) -> impl Iterator<Item = Row<'_>> {
    tsv.lines().filter_map(|line| {
        let cols: Vec<&str> = line.splitn(12, '\t').collect();
        if cols.len() < 11 {
            return None;
        }
        let n = |i: usize| cols[i].trim().parse::<f32>().ok();
        Some(Row {
            level: cols[0].trim().parse().ok()?,
            block: cols[2].trim().parse().ok()?,
            paragraph: cols[3].trim().parse().ok()?,
            line: cols[4].trim().parse().ok()?,
            left: n(6)?,
            top: n(7)?,
            width: n(8)?,
            height: n(9)?,
            confidence: n(10)?,
            text: cols.get(11).map_or("", |t| t.trim()),
        })
    })
}

/// A box in fractions of the page: left, top, width, height.
type Frac = (f32, f32, f32, f32);

/// Tesseract's TSV as paragraphs of lines of words, in fractions of the page, and the page's
/// text: words joined by spaces, lines by line breaks, paragraphs by blank lines.
fn layout(tsv: &str, width: u32, height: u32) -> (String, Vec<TextRegion>) {
    let (pw, ph) = (width.max(1) as f32, height.max(1) as f32);
    let frac = |r: &Row<'_>| (r.left / pw, r.top / ph, r.width / pw, r.height / ph);
    // Paragraph boxes, line boxes and words, keyed by where they sit in Tesseract's order.
    let mut paragraphs: BTreeMap<(u32, u32), Frac> = BTreeMap::new();
    let mut lines: BTreeMap<(u32, u32, u32), (Frac, Vec<TextWord>)> = BTreeMap::new();
    for row in rows(tsv) {
        match row.level {
            3 => {
                paragraphs.insert((row.block, row.paragraph), frac(&row));
            }
            4 => {
                lines
                    .entry((row.block, row.paragraph, row.line))
                    .or_insert_with(|| (frac(&row), Vec::new()))
                    .0 = frac(&row);
            }
            5 if !row.text.is_empty() && row.confidence >= 0.0 => {
                let (x, y, w, h) = frac(&row);
                let word = TextWord {
                    x,
                    y,
                    width: w,
                    height: h,
                    text: row.text.to_owned(),
                    confidence: Some((row.confidence / 100.0).clamp(0.0, 1.0)),
                };
                lines
                    .entry((row.block, row.paragraph, row.line))
                    .or_insert_with(|| ((x, y, w, h), Vec::new()))
                    .1
                    .push(word);
            }
            _ => {}
        }
    }
    let mut regions: Vec<TextRegion> = Vec::new();
    let mut current: Option<(u32, u32)> = None;
    for ((block, paragraph, _), ((x, y, width, height), words)) in lines {
        if words.is_empty() {
            continue;
        }
        let line = TextLine {
            x,
            y,
            width,
            height,
            words,
        };
        if current != Some((block, paragraph)) {
            current = Some((block, paragraph));
            let (x, y, w, h) = paragraphs.get(&(block, paragraph)).copied().unwrap_or((
                line.x,
                line.y,
                line.width,
                line.height,
            ));
            regions.push(TextRegion::block(x, y, w, h, ""));
        }
        if let Some(region) = regions.last_mut() {
            region.lines.push(line);
        }
    }
    for region in &mut regions {
        region.text = region
            .lines
            .iter()
            .map(TextLine::text)
            .collect::<Vec<_>>()
            .join("\n");
    }
    let text = regions
        .iter()
        .map(|r| r.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    (text, regions)
}

#[async_trait]
impl Provider for Tesseract {
    fn id(&self) -> ProviderId {
        ProviderId::new(TESSERACT_ID)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn check_ready(&self, model: &str, languages: &LanguageChoice) -> Result<(), ProviderError> {
        if model != MODEL {
            return Err(ProviderError::Unsupported(format!(
                "Unknown Tesseract model {model}"
            )));
        }
        for file in data_for(languages)? {
            if !self.inner.data.is_installed(file) {
                return Err(ProviderError::NotInstalled(file.id.to_owned()));
            }
        }
        Ok(())
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["image/png", "image/jpeg"],
            _ => &[],
        }
    }

    fn request_settings(&self, task: Task, _model: &str, _tuning: &Tuning) -> Vec<RequestSetting> {
        if task != Task::TextExtraction {
            return Vec::new();
        }
        vec![
            RequestSetting::new("engine", format!("Tesseract {}", Self::version())),
            RequestSetting::new("language_data", DATA_REVISION),
            RequestSetting::new("ocr_engine_mode", "lstm"),
            RequestSetting::new("page_segmentation", "auto"),
        ]
    }

    /// The fixed settings, and the language data loaded for this job.
    fn sent_settings(&self, request: &ProcessRequest<'_>, _batched: bool) -> Vec<RequestSetting> {
        let mut settings = self.request_settings(request.task, request.model, request.tuning);
        if let Ok(files) = data_for(request.languages) {
            settings.push(RequestSetting::new("languages", names(&files)));
        }
        settings
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        if request.task != Task::TextExtraction || !request.input.mime.starts_with("image/") {
            return Err(ProviderError::Unsupported(format!(
                "Tesseract reads page images, not {}",
                request.input.mime
            )));
        }
        let files = data_for(request.languages)?;
        self.start_janitor();
        let inner = self.inner.clone();
        let bytes = request.input.bytes.clone();
        let stop = request.stop.clone();
        let slot = inner
            .slots
            .clone()
            .acquire_owned()
            .await
            .map_err(engine_error)?;
        let (tsv, width, height) = tokio::task::spawn_blocking(move || {
            let _slot = slot;
            // A pause that came while waiting for a free engine.
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            let page = decode(&bytes)?;
            let engine = inner.take(&files)?;
            let tsv = recognise(&engine.api, &page);
            // An engine that failed on a page is dropped rather than kept.
            if tsv.is_ok() {
                inner.give_back(engine);
            }
            Ok((tsv?, page.width, page.height))
        })
        .await
        .map_err(|e| ProviderError::Other(format!("Tesseract stopped unexpectedly: {e}")))??;
        let (text, regions) = layout(&tsv, width, height);
        Ok(ProcessOutput {
            text,
            segments: Vec::new(),
            detected_language: None,
            usage: Usage {
                pages: 1,
                ..Usage::default()
            },
            model_version: Some(format!("tesseract-{} {DATA_REVISION}", Self::version())),
            fields: Vec::new(),
            regions,
            truncated: false,
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec![MODEL.to_owned()])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use kalamu_core::LanguageCode;

    /// Results are cached under [`MODEL`], which holds neither the engine's version nor the
    /// language data's revision: a pinned `kreuzberg-tesseract` bumped by Dependabot, or a new
    /// `tessdata_best` commit, would otherwise reuse pages read by the old one. When this fails,
    /// decide whether the new one reads differently; if it may, give the model a new id (here
    /// and in the catalogue) so earlier results stay theirs, then update the values below.
    #[test]
    fn a_new_engine_or_language_data_is_a_decision_about_the_model_id() {
        assert_eq!(
            (MODEL, DATA_REVISION, Tesseract::version().as_str()),
            ("tesseract-5", "tessdata_best@e12c65a", "5.5.2")
        );
    }

    /// Two paragraphs, the first of two lines, as Tesseract writes them; a non-word row with
    /// confidence -1 and an empty word are left out.
    const TSV: &str = "1\t1\t0\t0\t0\t0\t0\t0\t1000\t2000\t-1\t\n\
2\t1\t1\t0\t0\t0\t100\t100\t800\t300\t-1\t\n\
3\t1\t1\t1\t0\t0\t100\t100\t800\t120\t-1\t\n\
4\t1\t1\t1\t1\t0\t100\t100\t600\t50\t-1\t\n\
5\t1\t1\t1\t1\t1\t100\t100\t250\t50\t96.5\tDakar,\n\
5\t1\t1\t1\t1\t2\t380\t100\t60\t50\t91\tle\n\
4\t1\t1\t1\t2\t0\t100\t170\t500\t50\t-1\t\n\
5\t1\t1\t1\t2\t1\t100\t170\t200\t50\t88\t3\n\
5\t1\t1\t1\t2\t2\t320\t170\t180\t50\t80\t\n\
3\t1\t1\t2\t0\t0\t100\t300\t800\t60\t-1\t\n\
4\t1\t1\t2\t1\t0\t100\t300\t800\t60\t-1\t\n\
5\t1\t1\t2\t1\t1\t100\t300\t400\t60\t95\tRéception\n";

    #[test]
    fn the_tsv_becomes_paragraphs_lines_and_words_in_page_fractions() {
        let (text, regions) = layout(TSV, 1000, 2000);
        assert_eq!(text, "Dakar, le\n3\n\nRéception");
        assert_eq!(regions.len(), 2);
        let first = &regions[0];
        assert_eq!((first.x, first.y, first.width), (0.1, 0.05, 0.8));
        assert_eq!(first.lines.len(), 2);
        let word = &first.lines[0].words[0];
        assert_eq!(word.text, "Dakar,");
        assert_eq!(
            (word.x, word.y, word.width, word.height),
            (0.1, 0.05, 0.25, 0.025)
        );
        assert_eq!(word.confidence, Some(0.965));
        assert_eq!(regions[1].text, "Réception");
    }

    #[test]
    fn pages_are_read_side_by_side_leaving_a_core_free() {
        assert_eq!(max_engines(1), 1);
        assert_eq!(max_engines(2), 1);
        assert_eq!(max_engines(4), 3);
        assert_eq!(max_engines(16), 4);
    }

    #[test]
    fn the_catalogue_matches_what_tesseract_can_do() {
        let catalog = kalamu_catalog::Catalog::bundled();
        let model = catalog
            .model(&ProviderId::new(TESSERACT_ID), MODEL)
            .unwrap();
        let kalamu_catalog::LanguageSupport::List(listed) = &model.languages else {
            panic!("Tesseract lists its languages");
        };
        let mut listed: Vec<&str> = listed.iter().map(LanguageCode::as_str).collect();
        let mut readable: Vec<&str> = tessdata::languages().collect();
        listed.sort_unstable();
        readable.sort_unstable();
        assert_eq!(listed, readable);
        assert!(!model.detects_language && model.layout && !model.accepts_prompt);
        let tesseract = Tesseract::new(ModelManager::new(".".into(), &tessdata::TESSDATA));
        for input in &model.inputs {
            assert!(
                tesseract
                    .accepted_inputs(Task::TextExtraction)
                    .contains(&input.as_str())
            );
        }
    }

    #[test]
    fn the_language_must_be_named_and_its_data_installed() {
        let dir = tempfile::tempdir().unwrap();
        let tesseract = Tesseract::new(ModelManager::new(
            dir.path().to_owned(),
            &tessdata::TESSDATA,
        ));
        assert!(matches!(
            tesseract.check_ready(MODEL, &LanguageChoice::Auto),
            Err(ProviderError::InvalidRequest(_))
        ));
        let french = LanguageChoice::Explicit(vec![LanguageCode::new("fr")]);
        assert_eq!(
            tesseract.check_ready(MODEL, &french),
            Err(ProviderError::NotInstalled("tesseract-fra".into()))
        );
        let mixed = LanguageChoice::Explicit(vec![
            LanguageCode::new("fr"),
            LanguageCode::new("mos"),
            LanguageCode::new("ha"),
        ]);
        assert_eq!(names(&data_for(&mixed).unwrap()), "fra+Latin");
    }

    /// A real page through the real engine, when the language data is at hand:
    /// `TESSERACT_SMOKE_DATA` names a folder holding `fra.traineddata` and `eng.traineddata`
    /// from the pinned revision.
    #[tokio::test]
    async fn smoke_reads_a_printed_page() {
        let Ok(dir) = std::env::var("TESSERACT_SMOKE_DATA") else {
            return;
        };
        let tesseract = Tesseract::new(ModelManager::new(dir.into(), &tessdata::TESSDATA));
        let languages =
            LanguageChoice::Explicit(vec![LanguageCode::new("fr"), LanguageCode::new("en")]);
        tesseract.check_ready(MODEL, &languages).unwrap();
        let page = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/bulletin-page.jpg"
        ))
        .unwrap();
        let input = kalamu_core::PreparedInput {
            piece: kalamu_core::PieceRef::page(1),
            mime: "image/jpeg".into(),
            bytes: page,
            file_name: "bulletin-p1.jpg".into(),
        };
        let options = kalamu_core::SetupOptions::default();
        let stop = kalamu_core::StopSignal::default();
        let output = tesseract
            .process(
                &ApiKey::new(""),
                ProcessRequest {
                    task: Task::TextExtraction,
                    model: MODEL,
                    prompt: "",
                    user_prompt: "",
                    input: &input,
                    languages: &languages,
                    options: &options,
                    tuning: &Tuning::default(),
                    analysis: None,
                    stop: &stop,
                },
            )
            .await
            .unwrap();
        assert!(output.text.contains("BULLETIN"), "{}", output.text);
        assert!(output.regions.iter().all(|r| !r.lines.is_empty()));
    }
}
