//! Small OCR vision models on this computer: text extraction from page images with a
//! vision-language model of about a billion parameters, for the print Tesseract reads badly
//! (degraded type, old newspapers in columns, mixed layouts, tables), with nothing sent anywhere
//! and no key.
//!
//! The models run in llama.cpp's own program, shipped next to the app (`llama.rs`), one page at
//! a time: each page takes the whole processor, or the graphics processor with Metal on a Mac and
//! through Vulkan on Windows when there is one (`device.rs`).
//! Each model is two files from Hugging Face at a pinned revision, the language model and its
//! image encoder, downloaded together and checked against their SHA-256 before use.
//!
//! These models write text, not word positions. LightOnOCR-3, asked for its `grounding` mode,
//! places each block of text on the page, which gives ALTO and a searchable PDF as Mistral OCR's
//! blocks do; GLM-OCR returns the text alone. Language models can also write text that is not on
//! the page; the catalogue's descriptions say so.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, LanguageChoice, ProcessOutput, ProcessRequest, Provider, ProviderError, ProviderId,
    RequestSetting, Task, TextRegion, Tuning, Usage, plain_words,
};

use crate::cpu::{self, threads};
use crate::device::{Backend, Placement, Speeds};
use crate::host::{Checked, LlamaHost, Work, fingerprint, lock};
use crate::llama::{PageFile, page_image};
use crate::models::{ModelFile, ModelManager, ModelSet};

pub const VISION_ID: &str = "local-ocr";

/// LightOnOCR-3 1B (Apache 2.0), converted to GGUF by prithivMLmods; LightOn publishes no GGUF
/// of its own. Q8_0: the language model is small enough that a lower quantisation saves little
/// and costs accuracy.
const LIGHTON: &str = "https://huggingface.co/prithivMLmods/LightOnOCR-3-1B-GGUF/resolve/ff5c331ec2cfee9de1c8054c21ccf88b8c5b0faa";
/// GLM-OCR (MIT), converted by llama.cpp's own maintainers (ggml-org).
const GLM: &str =
    "https://huggingface.co/ggml-org/GLM-OCR-GGUF/resolve/65a42de1148dbed2297e922b5dbc7d9b70c36578";

static LIGHTON_ENCODER: ModelFile = ModelFile {
    id: "lightonocr-3-1b-encoder",
    file: "LightOnOCR-3-1B.mmproj-q8_0.gguf",
    bytes: 449_452_288,
    sha256: "7bbf40e6f3e91626843fdcbba67c7fe774821f3f0c537dcc8c769026a528839a",
    source: LIGHTON,
    languages: &[],
    pair: None,
};

static GLM_ENCODER: ModelFile = ModelFile {
    id: "glm-ocr-encoder",
    file: "mmproj-GLM-OCR-Q8_0.gguf",
    bytes: 484_403_648,
    sha256: "9c4b58e33e316ed142eb5dcb41abec3844d3e6e5dc361ffb782c3fa9d175141f",
    source: GLM,
    languages: &[],
    pair: None,
};

/// The models, each with its image encoder. `id` is the model id in the catalogue.
pub static VISION_MODELS: [ModelFile; 2] = [
    ModelFile {
        id: "lightonocr-3-1b",
        file: "LightOnOCR-3-1B.Q8_0.gguf",
        bytes: 639_443_584,
        sha256: "2cf9771aaf236a1c25f389a344510c9ad4daa863e5fdd82651cf7d7834fede1c",
        source: LIGHTON,
        languages: &[],
        pair: Some(&LIGHTON_ENCODER),
    },
    ModelFile {
        id: "glm-ocr",
        file: "GLM-OCR-Q8_0.gguf",
        bytes: 950_433_408,
        sha256: "45bc244a6446aff850521dc41f18bc8d7105ad5f0c2c8c28af04e7cc4f4d50b1",
        source: GLM,
        languages: &[],
        pair: Some(&GLM_ENCODER),
    },
];

pub static VISION: ModelSet = ModelSet {
    files: &VISION_MODELS,
    companion: None,
};

/// How a model's answer reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// Blocks of Markdown, each after a placeholder giving its kind and its box on the page,
    /// in thousandths: `![text](80,175,920,225)`.
    Grounded,
    /// Markdown for the whole page.
    Plain,
}

/// How one model is asked to read a page: the fixed instruction it was trained with (they take
/// no other), how its answer reads, and the size of image it reads best.
#[derive(Debug)]
struct Recipe {
    model: &'static str,
    prompt: &'static str,
    reading: Reading,
    /// The longest side of the page image sent, in pixels. The engine resizes larger images to
    /// what the model's encoder takes anyway; sending them smaller saves the time.
    max_side: u32,
    quantisation: &'static str,
    /// The graphics builds it reads correctly with, in the order tried. LightOnOCR-3's image
    /// encoder reads wrongly on the Adreno through OpenCL, GLM-OCR's correctly
    /// (docs/decisions/2026-11-06-ocr-graphics-processor.md).
    graphics: &'static [Backend],
}

const RECIPES: [Recipe; 2] = [
    // LightOn's own advice: pages at about 1540 pixels on their longest side.
    Recipe {
        model: "lightonocr-3-1b",
        prompt: "grounding",
        reading: Reading::Grounded,
        max_side: 1540,
        quantisation: "Q8_0",
        graphics: &[Backend::Vulkan],
    },
    Recipe {
        model: "glm-ocr",
        prompt: "Text Recognition:",
        reading: Reading::Plain,
        max_side: 1800,
        quantisation: "Q8_0",
        graphics: &[Backend::Vulkan, Backend::OpenCl],
    },
];

fn recipe(model: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.model == model)
}

/// The most a page's answer may run to. A dense newspaper page in French is about 4,000 tokens;
/// a page that reaches this is marked as cut short.
const MAX_TOKENS: u32 = 6144;
/// Room for the page image's tokens (up to about 3,000 for LightOnOCR's encoder) and the answer.
const CONTEXT: u32 = 10_240;
/// The image's tokens are read in one batch: some encoders need the whole image at once, and
/// llama.cpp would otherwise lower the image's resolution to fit its default batch of 512.
const BATCH: u32 = 4096;

/// Block kinds that hold no text from the page: LightOnOCR describes pictures and turns charts
/// into tables of their values, which are its reading of the page, not words on it.
const NOT_TEXT: [&str; 4] = ["image", "header_image", "footer_image", "chart"];

pub struct Vision {
    inner: Arc<Inner>,
}

struct Inner {
    models: ModelManager,
    /// The llama.cpp programs, shared with the offline language models, which take turns with
    /// these.
    host: Arc<LlamaHost>,
    checked: Checked,
    /// Seconds a page took on this computer, per device and model, kept in the models' folder.
    speeds: Mutex<Speeds>,
}

const SPEEDS_FILE: &str = "speeds.json";

impl Vision {
    pub fn new(models: ModelManager, host: Arc<LlamaHost>) -> Self {
        let speeds = Speeds::load(models.dir().join(SPEEDS_FILE));
        Self {
            inner: Arc::new(Inner {
                models,
                host,
                checked: Checked::default(),
                speeds: Mutex::new(speeds),
            }),
        }
    }

    pub fn models(&self) -> &ModelManager {
        &self.inner.models
    }

    /// Whether this build of the app carries the engine.
    pub fn available(&self) -> bool {
        self.inner.host.available()
    }

    /// Seconds a page has taken on this computer, per model, on the device that model's next
    /// page goes to, from the pages it read last.
    pub fn speeds(&self) -> BTreeMap<String, f32> {
        let speeds = lock(&self.inner.speeds);
        RECIPES
            .iter()
            .filter_map(|recipe| {
                let on = speeds.on(&self.inner.placement(recipe, false));
                on.get(recipe.model).map(|s| (recipe.model.to_owned(), *s))
            })
            .collect()
    }

    /// Where each model's next page goes, without waiting for the graphics processor to be
    /// found: the processor (or Metal) until it is.
    pub fn placements(&self) -> BTreeMap<String, Placement> {
        RECIPES
            .iter()
            .map(|recipe| (recipe.model.to_owned(), self.inner.placement(recipe, false)))
            .collect()
    }
}

impl Inner {
    /// Where this model's next page goes: the first graphics build it reads correctly with that
    /// found a device, else the processor. `wait`: ask the graphics builds for their devices if
    /// they were not asked yet, rather than reading on the processor meanwhile.
    fn placement(&self, recipe: &Recipe, wait: bool) -> Placement {
        self.host.placement(recipe.graphics, wait)
    }

    /// Keeps a running average of the seconds a page takes there, weighted to the latest pages.
    fn record_speed(&self, placement: &Placement, model: &str, seconds: f32) {
        lock(&self.speeds).record(placement, model, seconds);
    }
}

fn ensure_cpu() -> Result<(), ProviderError> {
    cpu::ensure("offline OCR models", "use Tesseract or a cloud model")
}

fn find(model: &str) -> Result<(&'static ModelFile, &'static Recipe), ProviderError> {
    VISION_MODELS
        .iter()
        .find(|m| m.id == model)
        .zip(recipe(model))
        .ok_or_else(|| ProviderError::Unsupported(format!("Unknown offline OCR model {model}")))
}

/// The engine's arguments for one page.
fn arguments(
    model: &std::path::Path,
    encoder: &std::path::Path,
    page: &std::path::Path,
    recipe: &Recipe,
    threads: i32,
) -> Vec<String> {
    let path = |p: &std::path::Path| p.to_string_lossy().into_owned();
    // `<__media__>` places the image, the instruction follows it, and a line break ends it as the
    // models' own chat templates do (`<|image_pad|>grounding\n<|im_end|>`): without it
    // LightOnOCR ignored `grounding` and wrote no block positions.
    let prompt = format!("<__media__>{}\n", recipe.prompt);
    [
        "-m".to_owned(),
        path(model),
        "--mmproj".to_owned(),
        path(encoder),
        "--image".to_owned(),
        path(page),
        "-p".to_owned(),
        prompt,
        "--jinja".to_owned(),
        // Greedy decoding: the same page reads the same each time, as the cache assumes.
        "--temp".to_owned(),
        "0".to_owned(),
        "-n".to_owned(),
        MAX_TOKENS.to_string(),
        "-c".to_owned(),
        CONTEXT.to_string(),
        "-b".to_owned(),
        BATCH.to_string(),
        "-ub".to_owned(),
        BATCH.to_string(),
        "-t".to_owned(),
        threads.to_string(),
        // Every layer on the graphics processor where the engine has one (Metal, or the Vulkan
        // build on the device `device.rs` names); ignored by the processor-only build.
        "-ngl".to_owned(),
        "99".to_owned(),
        // The warm-up pass only helps a program that answers many requests.
        "--no-warmup".to_owned(),
    ]
    .into()
}

/// A placeholder at the start of a line, `![kind](x0,y0,x1,y1)`, as its kind, its box in
/// thousandths of the page, and what follows it on the same line: through llama.cpp the block's
/// first line of text comes right after it (`![title](110,51,880,133) # BULLETIN…`), where the
/// model's own examples break the line.
fn placeholder(line: &str) -> Option<(&str, [f32; 4], &str)> {
    let rest = line.trim_start().strip_prefix("![")?;
    let (kind, rest) = rest.split_once("](")?;
    let (numbers, after) = rest.split_once(')')?;
    if kind.is_empty()
        || !kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_' || c == '+')
    {
        return None;
    }
    let mut values = numbers.split(',').map(|n| n.trim().parse::<f32>().ok());
    let mut corners = [0.0; 4];
    for corner in &mut corners {
        *corner = values.next()??;
    }
    values
        .next()
        .is_none()
        .then_some((kind, corners, after.trim()))
}

/// LightOnOCR's grounded answer as the page's Markdown and its blocks of text on the page.
/// Without a single placeholder the whole answer is the page's text, placed nowhere.
fn grounded(answer: &str) -> (String, Vec<TextRegion>) {
    let mut blocks: Vec<(String, [f32; 4], Vec<&str>)> = Vec::new();
    let mut before: Vec<&str> = Vec::new();
    for line in answer.lines() {
        if let Some((kind, corners, first)) = placeholder(line) {
            let lines = if first.is_empty() {
                Vec::new()
            } else {
                vec![first]
            };
            blocks.push((kind.trim_end_matches('+').to_owned(), corners, lines));
        } else if let Some(block) = blocks.last_mut() {
            block.2.push(line);
        } else {
            before.push(line);
        }
    }
    if blocks.is_empty() {
        return (answer.trim().to_owned(), Vec::new());
    }
    let mut texts: Vec<String> = Vec::new();
    let lead = before.join("\n").trim().to_owned();
    if !lead.is_empty() {
        texts.push(lead);
    }
    let mut regions = Vec::new();
    for (kind, [x0, y0, x1, y1], lines) in blocks {
        if NOT_TEXT.contains(&kind.as_str()) {
            continue;
        }
        let text = lines.join("\n").trim().to_owned();
        if text.is_empty() {
            continue;
        }
        let words = plain_words(&text);
        let (x0, y0) = ((x0 / 1000.0).clamp(0.0, 1.0), (y0 / 1000.0).clamp(0.0, 1.0));
        let (x1, y1) = ((x1 / 1000.0).clamp(0.0, 1.0), (y1 / 1000.0).clamp(0.0, 1.0));
        if !words.is_empty() && x1 > x0 && y1 > y0 {
            regions.push(TextRegion::block(x0, y0, x1 - x0, y1 - y0, words));
        }
        texts.push(text);
    }
    (texts.join("\n\n"), regions)
}

/// An answer that ran to the limit often ends in a line written over and over: kept once.
fn without_runaway(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(last) = lines.iter().rposition(|l| !l.trim().is_empty()) else {
        return text.to_owned();
    };
    let repeated = lines[last].trim();
    let mut first = last;
    let mut times = 1;
    for (i, line) in lines[..last].iter().enumerate().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line != repeated {
            break;
        }
        first = i;
        times += 1;
    }
    if times < 4 {
        return text.to_owned();
    }
    lines[..=first].join("\n")
}

#[async_trait]
impl Provider for Vision {
    fn id(&self) -> ProviderId {
        ProviderId::new(VISION_ID)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn check_ready(&self, model: &str, _languages: &LanguageChoice) -> Result<(), ProviderError> {
        self.inner.host.llama()?;
        ensure_cpu()?;
        let (file, _) = find(model)?;
        if self.inner.models.is_ready(file) {
            Ok(())
        } else {
            Err(ProviderError::NotInstalled(model.to_owned()))
        }
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["image/png", "image/jpeg"],
            _ => &[],
        }
    }

    fn request_settings(&self, task: Task, model: &str, _tuning: &Tuning) -> Vec<RequestSetting> {
        let (Task::TextExtraction, Ok((file, recipe))) = (task, find(model)) else {
            return Vec::new();
        };
        let mut settings = vec![
            RequestSetting::new("engine", format!("llama.cpp {}", self.inner.host.version())),
            RequestSetting::new("model_file", file.file),
            RequestSetting::new("model_sha256", file.sha256),
            RequestSetting::new("quantisation", recipe.quantisation),
            RequestSetting::new("instruction", recipe.prompt),
            RequestSetting::new("temperature", 0),
            RequestSetting::new("max_output_tokens", MAX_TOKENS),
            RequestSetting::new("image_max_side", recipe.max_side),
        ];
        if let Some(encoder) = file.pair {
            settings.push(RequestSetting::new("encoder_file", encoder.file));
            settings.push(RequestSetting::new("encoder_sha256", encoder.sha256));
        }
        settings
    }

    /// The fixed settings, and where the model runs on this computer: `process` reads the page
    /// there or, if that changed meanwhile, asks for it to be sent again.
    fn sent_settings(&self, request: &ProcessRequest<'_>, _batched: bool) -> Vec<RequestSetting> {
        let mut settings = self.request_settings(request.task, request.model, request.tuning);
        if let (false, Ok((_, recipe))) = (settings.is_empty(), find(request.model)) {
            let placement = self.inner.placement(recipe, true);
            settings.push(RequestSetting::new("device", placement.kind()));
            if let Some(name) = placement.device_name() {
                settings.push(RequestSetting::new("device_name", name));
            }
            if placement.encoder_only() {
                settings.push(RequestSetting::new("device_runs", "image encoder"));
            }
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
                "Offline OCR models read page images, not {}",
                request.input.mime
            )));
        }
        ensure_cpu()?;
        let (model, recipe) = find(request.model)?;
        // Where `sent_settings` said, just before, that this page would be read.
        let planned = self.inner.placement(recipe, true);
        let inner = self.inner.clone();
        let bytes = request.input.bytes.clone();
        let mime = request.input.mime.clone();
        let stop = request.stop.clone();
        let (run, version) = tokio::task::spawn_blocking(move || {
            let _turn = inner.host.turn();
            // A pause that came while waiting for the turn.
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            // A page failed on the graphics processor, or Settings changed, while this one
            // waited: sent again, its record names where it is read.
            let placement = inner.placement(recipe, true);
            if placement != planned {
                return Err(ProviderError::Transient(
                    "Offline OCR models moved to another device; the page is read again there"
                        .into(),
                ));
            }
            let llama = inner.host.program(&placement)?;
            let leases = inner.checked.leases(&inner.models, model)?;
            let (image, extension) = page_image(&bytes, &mime, recipe.max_side)?;
            let page = PageFile::write(inner.host.scratch(), &image, extension)?;
            let logical = std::thread::available_parallelism().map_or(2, usize::from);
            let mut args = arguments(
                leases[0].path(),
                leases[1].path(),
                page.path(),
                recipe,
                threads(logical),
            );
            args.extend(placement.arguments());
            // A failure on the graphics processor sends the page back to the engine, which
            // reads it again on the processor; whether the device is then given up depends on
            // how that goes (`device::verdict`).
            let request = fingerprint(&[recipe.model.as_bytes(), &bytes]);
            let run =
                inner
                    .host
                    .settle(&placement, request, llama.run(&args, &stop), Work::Page)?;
            inner.record_speed(&placement, recipe.model, run.elapsed.as_secs_f32());
            Ok((run, inner.host.version().to_owned()))
        })
        .await
        .map_err(|e| {
            ProviderError::Other(format!("The offline OCR model stopped unexpectedly: {e}"))
        })??;
        let truncated = run.generated.is_some_and(|n| n >= MAX_TOKENS);
        let answer = if truncated {
            without_runaway(&run.text)
        } else {
            run.text
        };
        let (text, regions) = match recipe.reading {
            Reading::Grounded => grounded(&answer),
            Reading::Plain => (answer.trim().to_owned(), Vec::new()),
        };
        Ok(ProcessOutput {
            text,
            segments: Vec::new(),
            detected_language: None,
            usage: Usage {
                pages: 1,
                ..Usage::default()
            },
            model_version: Some(format!(
                "{} llama.cpp {version}",
                model.file.trim_end_matches(".gguf")
            )),
            fields: Vec::new(),
            regions,
            truncated,
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(VISION_MODELS
            .iter()
            .filter(|m| self.inner.models.is_ready(m))
            .map(|m| m.id.to_owned())
            .collect())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::llama::Llama;
    use kalamu_core::LanguageCode;

    /// A page from the repository's samples.
    fn sample(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../samples")
            .join(name)
    }

    /// Results are cached under the model's id, which holds neither the files' revision nor
    /// llama.cpp's build: a new conversion of a model, or a new llama.cpp in
    /// `scripts/llama.json`, would otherwise reuse pages read by the old one. When this fails,
    /// decide whether the new one reads differently; if it may, give the model a new id (here
    /// and in the catalogue) so earlier results stay theirs, then update the values below.
    #[test]
    fn a_new_engine_or_model_file_is_a_decision_about_the_model_id() {
        let pin: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/llama.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(pin["version"], "b12110");
        let files: Vec<(&str, &str)> = VISION_MODELS
            .iter()
            .flat_map(|m| std::iter::once(m).chain(m.pair))
            .map(|f| (f.id, &f.sha256[..8]))
            .collect();
        assert_eq!(
            files,
            [
                ("lightonocr-3-1b", "2cf9771a"),
                ("lightonocr-3-1b-encoder", "7bbf40e6"),
                ("glm-ocr", "45bc244a"),
                ("glm-ocr-encoder", "9c4b58e3"),
            ]
        );
    }

    #[test]
    fn every_model_has_a_recipe_and_an_encoder() {
        for model in &VISION_MODELS {
            assert!(recipe(model.id).is_some(), "{} has no recipe", model.id);
            assert!(model.pair.is_some(), "{} has no image encoder", model.id);
        }
        assert_eq!(RECIPES.len(), VISION_MODELS.len());
    }

    #[test]
    fn grounded_blocks_are_placed_and_pictures_left_out() {
        let answer = "![title](80,70,920,140)\n# Le Réveil du Dahomey\n\n\
![text](80,175,920,225)\nLa **réunion** s'est tenue à Porto-Novo.\n\n\
![image](80,285,460,545)\nUn homme devant une mosquée.\n\n\
![text+](80,560,920,600)\nElle reprendra demain.\n";
        let (text, regions) = grounded(answer);
        assert_eq!(
            text,
            "# Le Réveil du Dahomey\n\nLa **réunion** s'est tenue à Porto-Novo.\n\nElle reprendra demain."
        );
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].text, "Le Réveil du Dahomey");
        assert_eq!(regions[1].text, "La réunion s'est tenue à Porto-Novo.");
        let r = &regions[1];
        assert!((r.x - 0.08).abs() < 1e-6 && (r.y - 0.175).abs() < 1e-6);
        assert!((r.width - 0.84).abs() < 1e-6 && (r.height - 0.05).abs() < 1e-6);
        assert!(!text.contains("mosquée"));
    }

    #[test]
    fn an_answer_without_placeholders_is_text_placed_nowhere() {
        let (text, regions) = grounded("  Une page sans blocs.\n");
        assert_eq!(text, "Une page sans blocs.");
        assert!(regions.is_empty());
        // Square brackets in the text are not placeholders.
        assert_eq!(placeholder("![voir](page 3)"), None);
        assert_eq!(placeholder("![text](1,2,3)"), None);
        assert_eq!(
            placeholder("![aside_text](10, 20, 30, 40)"),
            Some(("aside_text", [10.0, 20.0, 30.0, 40.0], ""))
        );
    }

    /// LightOnOCR's answer through llama.cpp, from the workflow's run on the sample page: each
    /// block's text starts on its placeholder's line.
    #[test]
    fn a_block_whose_text_starts_on_its_placeholder_line_is_placed() {
        let answer = "\n![title](110,51,880,133) # BULLETIN DE L'AFRIQUE OCCIDENTALE FRANÇAISE\n\n\
![text](841,155,922,171) DAKAR 3 MAI 1957 PAGE 1\n\n\
![text](82,268,499,392) Le Gouverneur général a reçu ce matin les délégués.\n\
Ils ont remis un mémoire.\n";
        let (text, regions) = grounded(answer);
        assert_eq!(
            text,
            "# BULLETIN DE L'AFRIQUE OCCIDENTALE FRANÇAISE\n\nDAKAR 3 MAI 1957 PAGE 1\n\n\
Le Gouverneur général a reçu ce matin les délégués.\nIls ont remis un mémoire."
        );
        assert_eq!(regions.len(), 3);
        assert_eq!(
            regions[0].text,
            "BULLETIN DE L'AFRIQUE OCCIDENTALE FRANÇAISE"
        );
        assert!((regions[1].x - 0.841).abs() < 1e-6 && (regions[1].y - 0.155).abs() < 1e-6);
        assert!(!text.contains("]("));
    }

    #[test]
    fn a_line_written_over_and_over_at_the_limit_is_kept_once() {
        let looped = "Titre\nLigne\nLigne\nLigne\nLigne\nLigne";
        assert_eq!(without_runaway(looped), "Titre\nLigne");
        let fine = "Titre\nLigne\nLigne";
        assert_eq!(without_runaway(fine), fine);
    }

    #[test]
    fn a_large_page_is_sent_at_the_size_the_model_reads() {
        let page = image::DynamicImage::new_luma8(3000, 2000);
        let mut jpeg = std::io::Cursor::new(Vec::new());
        page.write_to(&mut jpeg, image::ImageFormat::Jpeg).unwrap();
        let (bytes, extension) = page_image(jpeg.get_ref(), "image/jpeg", 1540).unwrap();
        assert_eq!(extension, "png");
        let resized = image::load_from_memory(&bytes).unwrap();
        assert_eq!((resized.width(), resized.height()), (1540, 1027));
        let (same, extension) = page_image(jpeg.get_ref(), "image/jpeg", 4000).unwrap();
        assert_eq!((same.len(), extension), (jpeg.get_ref().len(), "jpg"));
    }

    #[test]
    fn the_arguments_name_the_files_and_the_fixed_settings() {
        let recipe = recipe("lightonocr-3-1b").unwrap();
        let args = arguments(
            std::path::Path::new("m.gguf"),
            std::path::Path::new("e.gguf"),
            std::path::Path::new("p.png"),
            recipe,
            4,
        );
        let pair = |flag: &str| {
            let i = args.iter().position(|a| a == flag).unwrap();
            args[i + 1].clone()
        };
        assert_eq!(pair("-m"), "m.gguf");
        assert_eq!(pair("--mmproj"), "e.gguf");
        assert_eq!(pair("--image"), "p.png");
        assert_eq!(pair("-p"), "<__media__>grounding\n");
        assert_eq!(pair("--temp"), "0");
        assert_eq!(pair("-t"), "4");
    }

    #[test]
    fn the_catalogue_matches_what_the_models_can_do() {
        let catalog = kalamu_catalog::Catalog::bundled();
        for recipe in &RECIPES {
            let model = catalog
                .model(&ProviderId::new(VISION_ID), recipe.model)
                .unwrap();
            assert!(!model.accepts_prompt && !model.takes_language);
            assert_eq!(
                model.layout,
                recipe.reading == Reading::Grounded,
                "{}",
                model.id
            );
            let vision = Vision::new(
                ModelManager::new(".".into(), &VISION),
                Arc::new(LlamaHost::none()),
            );
            for input in &model.inputs {
                assert!(
                    vision
                        .accepted_inputs(Task::TextExtraction)
                        .contains(&input.as_str())
                );
            }
        }
    }

    #[test]
    fn a_model_needs_the_engine_and_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let without = Vision::new(
            ModelManager::new(dir.path().to_owned(), &VISION),
            Arc::new(LlamaHost::none()),
        );
        assert!(!without.available());
        assert!(matches!(
            without.check_ready("glm-ocr", &LanguageChoice::Auto),
            Err(ProviderError::Unsupported(_))
        ));
        let engine = Llama::at(dir.path().join("llama-mtmd-cli"));
        let with = Vision::new(
            ModelManager::new(dir.path().to_owned(), &VISION),
            Arc::new(LlamaHost::new(
                Some(engine),
                Vec::new(),
                true,
                dir.path().to_owned(),
                dir.path().join("scratch"),
            )),
        );
        let french = LanguageChoice::Explicit(vec![LanguageCode::new("fr")]);
        if cpu::missing_features().is_empty() {
            assert_eq!(
                with.check_ready("glm-ocr", &french),
                Err(ProviderError::NotInstalled("glm-ocr".into()))
            );
        }
        assert!(matches!(
            with.check_ready("nothing", &french),
            Err(ProviderError::Unsupported(_))
        ));
    }

    /// A real page through the real engine, when both are at hand: `VISION_SMOKE_DIR` names a
    /// folder holding the pinned model files, and `KALAMU_LLAMA_DIR` the engine. Run with
    /// `cargo test -p kalamu-offline vision::tests::smoke -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore = "needs the engine and a model"]
    async fn smoke() {
        let dir = std::env::var("VISION_SMOKE_DIR").unwrap();
        let llama = Llama::locate(&[]).expect("KALAMU_LLAMA_DIR names the engine's folder");
        // The graphics builds too, when they are in the same folder and find a device.
        let graphics = [
            Llama::locate_vulkan(&[]).map(|p| (Backend::Vulkan, p)),
            Llama::locate_opencl(&[]).map(|p| (Backend::OpenCl, p)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let host = Arc::new(LlamaHost::new(
            Some(llama),
            graphics,
            true,
            std::env::temp_dir().join("kalamu-opencl-kernels"),
            std::env::temp_dir().join("kalamu-offline-scratch"),
        ));
        host.find_graphics_processor();
        let vision = Vision::new(ModelManager::new(dir.into(), &VISION), host);
        println!("== reading on {:?}", vision.placements());
        let page = std::fs::read(sample("bulletin-page.jpg")).unwrap();
        let input = kalamu_core::PreparedInput {
            piece: kalamu_core::PieceRef::page(1),
            mime: "image/jpeg".into(),
            bytes: page,
            file_name: "bulletin-p1.jpg".into(),
        };
        for model in VISION_MODELS.iter().filter(|m| vision.models().is_ready(m)) {
            let started = std::time::Instant::now();
            let output = vision
                .process(
                    &ApiKey::new(""),
                    ProcessRequest {
                        task: Task::TextExtraction,
                        model: model.id,
                        prompt: "",
                        user_prompt: "",
                        input: &input,
                        languages: &LanguageChoice::Auto,
                        options: &kalamu_core::SetupOptions::default(),
                        tuning: &Tuning::default(),
                        analysis: None,
                        stop: &kalamu_core::StopSignal::default(),
                    },
                )
                .await
                .unwrap();
            println!(
                "== {} ({:.1} s, {} blocks)\n{}",
                model.id,
                started.elapsed().as_secs_f32(),
                output.regions.len(),
                output.text
            );
            assert!(
                output.text.to_uppercase().contains("BULLETIN"),
                "{}",
                output.text
            );
        }
    }
}
