//! Small language models on this computer, for analyses, tables of contents and corrections:
//! Google's Gemma 4 E4B and E2B, which read texts and page images and answer in the fields the
//! researcher asked for, with nothing sent anywhere and no key
//! (docs/decisions/2026-11-07-offline-language-models.md).
//!
//! They run in llama.cpp's programs, shared with the OCR models (`host.rs`), one request at a
//! time: a text through the text program (`llama-completion`), which reads the instructions, the
//! text and the answer's schema from files; a page image through the multimodal program
//! (`llama-mtmd-cli`). Either way the answer is held to the analysis's JSON schema as it is
//! written (llama.cpp turns the schema into a grammar), then checked against the fields as every
//! provider's is. Decoding is greedy, so a cached answer is what a new run would write.
//!
//! Each model is two files from Google's own repositories on Hugging Face at a pinned revision,
//! the language model and its image and audio encoder, at the 4-bit quantisation Google trained
//! them for (QAT), downloaded together and checked against their SHA-256 before use. They are far
//! weaker than the cloud models: the catalogue's descriptions say so, and what they can read at
//! once is limited (`MAX_CHARS`).

use std::sync::Arc;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, LanguageChoice, ProcessOutput, ProcessRequest, Provider, ProviderError, ProviderId,
    RequestSetting, Task, Tuning, Usage, structured,
};

use crate::cpu::{self, threads};
use crate::device::{Backend, Placement};
use crate::host::{Checked, LlamaHost};
use crate::llama::{Run, Scratch, page_image};
use crate::models::{ModelFile, ModelManager, ModelSet};

pub const LLM_ID: &str = "local-llm";

/// Google's own GGUF conversions, at the 4-bit quantisation the models were trained for.
const E4B: &str = "https://huggingface.co/google/gemma-4-E4B-it-qat-q4_0-gguf/resolve/4b4a2c1d584be7264f87aac328a1bc739ce81b6c";
const E2B: &str = "https://huggingface.co/google/gemma-4-E2B-it-qat-q4_0-gguf/resolve/675cff42a74c774d6cb76f76d8eacb49b48c9b93";

static E4B_ENCODER: ModelFile = ModelFile {
    id: "gemma-4-e4b-encoder",
    file: "gemma-4-E4B-it-mmproj.gguf",
    bytes: 991_552_256,
    sha256: "7498a37cb619e55f2fcf87eb931f56e99389ed6d432e4c5c66110694c0d65578",
    source: E4B,
    languages: &[],
    pair: None,
};

static E2B_ENCODER: ModelFile = ModelFile {
    id: "gemma-4-e2b-encoder",
    file: "gemma-4-E2B-it-mmproj.gguf",
    bytes: 986_833_664,
    sha256: "021059cce659fe7f9170d5599761d7bbaf644b798dab9503aca30dc43e6beb14",
    source: E2B,
    languages: &[],
    pair: None,
};

/// The models, each with its encoder. `id` is the model id in the catalogue.
pub static LLM_MODELS: [ModelFile; 2] = [
    ModelFile {
        id: "gemma-4-e4b",
        file: "gemma-4-E4B_q4_0-it.gguf",
        bytes: 5_154_941_280,
        sha256: "676c35070db6dbe52f93e9c864ee0fba4eddea94b9c875d9cb10daff453fbaee",
        source: E4B,
        languages: &[],
        pair: Some(&E4B_ENCODER),
    },
    ModelFile {
        id: "gemma-4-e2b",
        file: "gemma-4-E2B_q4_0-it.gguf",
        bytes: 3_349_516_256,
        sha256: "fa401b55b07ee70a54c6dae3903c783a6e65064312529ea57175cb5f8dec6634",
        source: E2B,
        languages: &[],
        pair: Some(&E2B_ENCODER),
    },
];

pub static LLM: ModelSet = ModelSet {
    files: &LLM_MODELS,
    companion: None,
};

/// How one model runs.
#[derive(Debug)]
struct Recipe {
    model: &'static str,
    quantisation: &'static str,
    /// The graphics builds it runs with, on any device they choose: llama.cpp puts as many of
    /// its layers there as the device's memory holds (`--fit`), the rest on the processor. Not
    /// OpenCL: on the Adreno that build is measured for the OCR models' image encoders only, and
    /// Vulkan reads wrongly there (docs/decisions/2026-11-07-offline-language-models.md).
    graphics: &'static [Backend],
}

const RECIPES: [Recipe; 2] = [
    Recipe {
        model: "gemma-4-e4b",
        quantisation: "Q4_0 (QAT)",
        graphics: &[Backend::Vulkan],
    },
    Recipe {
        model: "gemma-4-e2b",
        quantisation: "Q4_0 (QAT)",
        graphics: &[Backend::Vulkan],
    },
];

fn recipe(model: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.model == model)
}

/// The context: the instructions, the text or image, and the answer. The models take 128K
/// tokens, but reading that many on a processor takes far too long, and its memory grows with
/// it; 32K holds a text of some twenty pages.
const CONTEXT: u32 = 32_768;
/// The most an answer may run to, the catalogue's `max_output_tokens` for these models: a long
/// table of contents or a corrected page fits. A smaller limit from the catalogue is kept.
const MAX_TOKENS: u32 = 8192;
/// Below this an analysis's fields with their reasons would not fit.
const MIN_TOKENS: u32 = 512;

/// The output limit for a request: the catalogue's, as the engine passes it, within the room the
/// context leaves.
fn output_limit(tuning: &Tuning) -> u32 {
    tuning
        .max_output_tokens
        .map_or(MAX_TOKENS, |n| n.clamp(MIN_TOKENS, MAX_TOKENS))
}
/// The most characters of text sent: what fits in the context beside the answer and the
/// instructions at about three characters a token, a cautious rate for European languages. A
/// text in another script may hold more tokens than this allows, which llama.cpp then refuses
/// with the same message.
const MAX_CHARS: usize = 66_000;
/// Read in batches this large: fewer passes over a long text.
const BATCH: u32 = 2048;
/// A page image's tokens, the largest of Gemma 4's budgets, which its makers advise for reading
/// text, and the size the image is sent at.
const IMAGE_TOKENS: u32 = 1120;
const IMAGE_MAX_SIDE: u32 = 1536;

pub struct LocalLlm {
    inner: Arc<Inner>,
}

struct Inner {
    models: ModelManager,
    host: Arc<LlamaHost>,
    checked: Checked,
}

impl LocalLlm {
    pub fn new(models: ModelManager, host: Arc<LlamaHost>) -> Self {
        Self {
            inner: Arc::new(Inner {
                models,
                host,
                checked: Checked::default(),
            }),
        }
    }

    pub fn models(&self) -> &ModelManager {
        &self.inner.models
    }

    /// Whether this copy of the app carries both programs the models run in.
    pub fn available(&self) -> bool {
        self.inner.host.reads_text()
    }

    /// Where each model's next request goes, without waiting for the graphics processor to be
    /// found: the processor (or Metal) until it is.
    pub fn placements(&self) -> std::collections::BTreeMap<String, Placement> {
        RECIPES
            .iter()
            .map(|r| (r.model.to_owned(), self.inner.placement(r, false)))
            .collect()
    }
}

impl Inner {
    fn placement(&self, recipe: &Recipe, wait: bool) -> Placement {
        self.host.placement(recipe.graphics, wait)
    }
}

fn ensure_cpu() -> Result<(), ProviderError> {
    cpu::ensure("offline language models", "use a cloud model")
}

fn find(model: &str) -> Result<(&'static ModelFile, &'static Recipe), ProviderError> {
    LLM_MODELS
        .iter()
        .find(|m| m.id == model)
        .zip(recipe(model))
        .ok_or_else(|| {
            ProviderError::Unsupported(format!("Unknown offline language model {model}"))
        })
}

/// What one request reads.
enum Reading {
    /// A text, through the text program.
    Text { system: String, user: String },
    /// A page image, through the multimodal program.
    Page {
        system: String,
        user: String,
        image: Vec<u8>,
        extension: &'static str,
    },
}

impl Reading {
    /// The request as the model reads it: the instructions as its system prompt; the text, then
    /// the request, as the user's message (the document first, as for the cloud models); or the
    /// page image first, as Gemma's makers advise.
    fn of(request: &ProcessRequest<'_>) -> Result<Self, ProviderError> {
        let system = request.prompt.to_owned();
        let input = request.input;
        if input.mime == "text/plain" {
            let text = structured::document_text(input);
            let chars = text.chars().count();
            if chars > MAX_CHARS {
                return Err(too_long(chars));
            }
            let user = format!(
                "{}\n\n{}",
                structured::tagged_document(input),
                request.user_prompt
            );
            Ok(Self::Text { system, user })
        } else if input.mime.starts_with("image/") {
            let (image, extension) = page_image(&input.bytes, &input.mime, IMAGE_MAX_SIDE)?;
            Ok(Self::Page {
                system,
                user: request.user_prompt.to_owned(),
                image,
                extension,
            })
        } else {
            Err(ProviderError::Unsupported(format!(
                "Offline language models read texts and page images, not {}",
                input.mime
            )))
        }
    }
}

/// A text longer than the models read at once, in words a researcher can act on.
fn too_long(chars: usize) -> ProviderError {
    ProviderError::InvalidRequest(format!(
        "This text is too long for the offline language models: about {} words, where they read \
         at most about {} at once. Split it into shorter documents, or choose a cloud model.",
        chars / 6,
        MAX_CHARS / 6
    ))
}

/// The arguments both programs share, but for the output limit (`-n`, added by `process`): the
/// model, greedy decoding, the answer's schema, the context and the threads; and on a graphics processor (Metal, or the Vulkan build's device)
/// as many layers as its memory holds beside the fixed context, the rest on the processor
/// (`-ngl auto` with `--fit on`), so a card with little memory of its own still helps rather
/// than failing or being passed over. The processor-only build ignores both.
fn common_arguments(model: &str, schema: &str, threads: i32) -> Vec<String> {
    [
        "-m",
        model,
        "-jf",
        schema,
        "--jinja",
        // Greedy decoding: the same request answers the same each time, as the cache assumes.
        "--temp",
        "0",
        "-c",
        &CONTEXT.to_string(),
        "-b",
        &BATCH.to_string(),
        "-ub",
        &BATCH.to_string(),
        "-t",
        &threads.to_string(),
        "-ngl",
        "auto",
        "--fit",
        "on",
        "--no-warmup",
    ]
    .iter()
    .map(|a| (*a).to_owned())
    .collect()
}

/// The text program's arguments: the instructions, the text and the request from files, as one
/// turn of a conversation in the model's own chat template, without thinking (Gemma 4 E4B and
/// E2B write no thoughts when it is off, so the schema holds from the first token), and without
/// printing the request back.
fn text_arguments(
    model: &str,
    schema: &str,
    system: &str,
    user: &str,
    threads: i32,
) -> Vec<String> {
    let mut args = common_arguments(model, schema, threads);
    args.extend(
        [
            "-sysf",
            system,
            "-f",
            user,
            "-cnv",
            "-st",
            "--reasoning",
            "off",
            "--no-display-prompt",
        ]
        .iter()
        .map(|a| (*a).to_owned()),
    );
    args
}

/// The multimodal program's arguments: the page image first, then the request.
fn page_arguments(
    model: &str,
    encoder: &str,
    schema: &str,
    page: &str,
    system: &str,
    user: &str,
    threads: i32,
) -> Vec<String> {
    let mut args = common_arguments(model, schema, threads);
    args.extend([
        "--mmproj".to_owned(),
        encoder.to_owned(),
        "--image".to_owned(),
        page.to_owned(),
        "--image-max-tokens".to_owned(),
        IMAGE_TOKENS.to_string(),
        "-sys".to_owned(),
        system.to_owned(),
        "-p".to_owned(),
        format!("<__media__>{user}"),
    ]);
    args
}

/// The answer the program printed: the text program ends it with ` [end of text]`, and anything
/// around the JSON object is dropped.
fn answer(printed: &str) -> &str {
    let printed = printed.trim();
    let printed = printed.strip_suffix("[end of text]").unwrap_or(printed);
    match (printed.find('{'), printed.rfind('}')) {
        (Some(start), Some(end)) if start < end => &printed[start..=end],
        _ => printed.trim(),
    }
}

#[async_trait]
impl Provider for LocalLlm {
    fn id(&self) -> ProviderId {
        ProviderId::new(LLM_ID)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn check_ready(&self, model: &str, _languages: &LanguageChoice) -> Result<(), ProviderError> {
        self.inner.host.llama()?;
        if !self.inner.host.reads_text() {
            return Err(ProviderError::Unsupported(
                "This copy of Kalamu has no engine for offline language models".into(),
            ));
        }
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
            Task::Analysis => &["text/plain", "image/png", "image/jpeg"],
            _ => &[],
        }
    }

    fn request_settings(&self, task: Task, model: &str, tuning: &Tuning) -> Vec<RequestSetting> {
        let (Task::Analysis, Ok((file, recipe))) = (task, find(model)) else {
            return Vec::new();
        };
        let mut settings = vec![
            RequestSetting::new("engine", format!("llama.cpp {}", self.inner.host.version())),
            RequestSetting::new("model_file", file.file),
            RequestSetting::new("model_sha256", file.sha256),
            RequestSetting::new("quantisation", recipe.quantisation),
            RequestSetting::new("temperature", 0),
            RequestSetting::new("thinking", "off"),
            RequestSetting::new("max_output_tokens", output_limit(tuning)),
            RequestSetting::new("context_tokens", CONTEXT),
        ];
        if let Some(encoder) = file.pair {
            settings.push(RequestSetting::new("encoder_file", encoder.file));
            settings.push(RequestSetting::new("encoder_sha256", encoder.sha256));
        }
        settings
    }

    /// The fixed settings, the image's size for a page, and where the model runs on this
    /// computer: `process` runs it there or, if that changed meanwhile, asks for it to be sent
    /// again.
    fn sent_settings(&self, request: &ProcessRequest<'_>, _batched: bool) -> Vec<RequestSetting> {
        let mut settings = self.request_settings(request.task, request.model, request.tuning);
        if let (false, Ok((_, recipe))) = (settings.is_empty(), find(request.model)) {
            if request.input.mime.starts_with("image/") {
                settings.push(RequestSetting::new("image_max_side", IMAGE_MAX_SIDE));
                settings.push(RequestSetting::new("image_max_tokens", IMAGE_TOKENS));
            }
            let placement = self.inner.placement(recipe, true);
            settings.push(RequestSetting::new("device", placement.kind()));
            if let Some(name) = placement.device_name() {
                settings.push(RequestSetting::new("device_name", name));
            }
        }
        settings
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        if request.task != Task::Analysis {
            return Err(ProviderError::Unsupported(
                "Offline language models answer in fields; they do not read pages or recordings"
                    .into(),
            ));
        }
        let fields = request
            .analysis
            .ok_or_else(|| ProviderError::InvalidRequest("an analysis needs fields".into()))?;
        ensure_cpu()?;
        let (model, recipe) = find(request.model)?;
        let reading = Reading::of(&request)?;
        let schema = serde_json::to_string(&structured::schema(fields))
            .map_err(|e| ProviderError::Other(format!("Could not write the fields: {e}")))?;
        // Where `sent_settings` said, just before, that this request would run.
        let planned = self.inner.placement(recipe, true);
        let inner = self.inner.clone();
        let stop = request.stop.clone();
        let limit = output_limit(request.tuning);
        let (run, version): (Run, String) = tokio::task::spawn_blocking(move || {
            let _turn = inner.host.turn();
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            let placement = inner.placement(recipe, true);
            if placement != planned {
                return Err(ProviderError::Transient(
                    "Offline language models moved to another device; the request is sent again there"
                        .into(),
                ));
            }
            let llama = inner.host.program(&placement)?;
            let leases = inner.checked.leases(&inner.models, model)?;
            let path = |p: &std::path::Path| p.to_string_lossy().into_owned();
            let scratch = Scratch::new()?;
            let schema = path(&scratch.write("schema.json", schema.as_bytes())?);
            let logical = std::thread::available_parallelism().map_or(2, usize::from);
            let threads = threads(logical);
            let model_path = path(leases[0].path());
            let mut args = match &reading {
                Reading::Text { system, user } => {
                    let system = path(&scratch.write("instructions.txt", system.as_bytes())?);
                    let user = path(&scratch.write("request.txt", user.as_bytes())?);
                    text_arguments(&model_path, &schema, &system, &user, threads)
                }
                Reading::Page {
                    system,
                    user,
                    image,
                    extension,
                } => {
                    let page = path(&scratch.write(&format!("page.{extension}"), image)?);
                    page_arguments(
                        &model_path,
                        &path(leases[1].path()),
                        &schema,
                        &page,
                        system,
                        user,
                        threads,
                    )
                }
            };
            args.extend(["-n".to_owned(), limit.to_string()]);
            args.extend(placement.arguments());
            let ran = match reading {
                Reading::Text { .. } => llama.run_text(&args, &stop),
                Reading::Page { .. } => llama.run(&args, &stop),
            };
            let run = match (ran, inner.host.graphics(&placement)) {
                (Err(ProviderError::Other(reason)), _) if reason.contains("too long") => {
                    return Err(ProviderError::InvalidRequest(
                        "This text is too long for the offline language models. Split it into \
                         shorter documents, or choose a cloud model."
                            .into(),
                    ));
                }
                // Out of memory, a driver fault: the rest of the session runs on the processor,
                // starting with this request, which the engine sends again.
                (Err(ProviderError::Other(reason)), Some(graphics)) => {
                    graphics.give_up(&reason);
                    return Err(ProviderError::Transient(format!(
                        "The graphics processor could not run this request ({reason}); it runs again on the processor"
                    )));
                }
                (ran, _) => ran?,
            };
            Ok((run, inner.host.version().to_owned()))
        })
        .await
        .map_err(|e| {
            ProviderError::Other(format!("The offline language model stopped unexpectedly: {e}"))
        })??;
        let usage = Usage {
            input_tokens: run.read.map_or(0, u64::from),
            output_tokens: run.generated.map_or(0, u64::from),
            ..Usage::default()
        };
        // Greedy decoding writes the same again: an answer cut short is not worth retrying.
        if run.generated.is_some_and(|n| n >= limit) {
            return Err(ProviderError::Other(format!(
                "The answer ran past the offline model's limit of {limit} tokens; ask for \
                 fewer or shorter fields, or choose a cloud model"
            )));
        }
        let output = ProcessOutput {
            text: answer(&run.text).to_owned(),
            usage,
            model_version: Some(format!(
                "{} llama.cpp {version}",
                model.file.trim_end_matches(".gguf")
            )),
            ..ProcessOutput::default()
        };
        structured::finish(output, fields)
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(LLM_MODELS
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
    use kalamu_core::{AnalysisSchema, Field, FieldKind, FieldValue, PieceRef, PreparedInput};

    /// As for the OCR models: results are cached under the model's id, which holds neither the
    /// files' revision nor llama.cpp's build. When this fails, decide whether the new files
    /// answer differently; if they may, give the model a new id (here and in the catalogue).
    #[test]
    fn a_new_model_file_is_a_decision_about_the_model_id() {
        let files: Vec<(&str, &str)> = LLM_MODELS
            .iter()
            .flat_map(|m| std::iter::once(m).chain(m.pair))
            .map(|f| (f.id, &f.sha256[..8]))
            .collect();
        assert_eq!(
            files,
            [
                ("gemma-4-e4b", "676c3507"),
                ("gemma-4-e4b-encoder", "7498a37c"),
                ("gemma-4-e2b", "fa401b55"),
                ("gemma-4-e2b-encoder", "021059cc"),
            ]
        );
    }

    #[test]
    fn every_model_has_a_recipe_and_an_encoder() {
        for model in &LLM_MODELS {
            assert!(recipe(model.id).is_some(), "{} has no recipe", model.id);
            assert!(model.pair.is_some(), "{} has no encoder", model.id);
        }
        assert_eq!(RECIPES.len(), LLM_MODELS.len());
    }

    #[test]
    fn the_catalogue_matches_what_the_models_can_do() {
        let catalog = kalamu_catalog::Catalog::bundled();
        let llm = LocalLlm::new(
            ModelManager::new(".".into(), &LLM),
            Arc::new(LlamaHost::none()),
        );
        for recipe in &RECIPES {
            let model = catalog
                .model(&ProviderId::new(LLM_ID), recipe.model)
                .unwrap();
            assert_eq!(model.tasks, [Task::Analysis], "{}", model.id);
            // The output limit the engine passes is this provider's; no thinking to choose, since
            // the text program holds the answer to the schema from its first token.
            assert_eq!(
                model.limits.max_output_tokens,
                Some(MAX_TOKENS),
                "{}",
                model.id
            );
            assert!(model.reasoning.is_none(), "{}", model.id);
            assert_eq!(
                output_limit(&model.tuning(Task::Analysis, None)),
                MAX_TOKENS
            );
            for input in &model.inputs {
                assert!(
                    llm.accepted_inputs(Task::Analysis)
                        .contains(&input.as_str()),
                    "{input}"
                );
            }
        }
    }

    #[test]
    fn a_model_needs_both_programs_and_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let without = LocalLlm::new(
            ModelManager::new(dir.path().to_owned(), &LLM),
            Arc::new(LlamaHost::none()),
        );
        assert!(!without.available());
        assert!(matches!(
            without.check_ready("gemma-4-e4b", &LanguageChoice::Auto),
            Err(ProviderError::Unsupported(_))
        ));
        // The multimodal program alone, as copies built before the text program shipped.
        let multimodal = LocalLlm::new(
            ModelManager::new(dir.path().to_owned(), &LLM),
            Arc::new(LlamaHost::new(
                Some(Llama::at(dir.path().join("llama-mtmd-cli"))),
                Vec::new(),
                true,
                dir.path().to_owned(),
            )),
        );
        assert!(!multimodal.available());
        assert!(matches!(
            multimodal.check_ready("gemma-4-e4b", &LanguageChoice::Auto),
            Err(ProviderError::Unsupported(_))
        ));
    }

    fn request<'a>(
        input: &'a PreparedInput,
        fields: &'a AnalysisSchema,
        options: &'a kalamu_core::SetupOptions,
        tuning: &'a Tuning,
        stop: &'a kalamu_core::StopSignal,
    ) -> ProcessRequest<'a> {
        ProcessRequest {
            task: Task::Analysis,
            model: "gemma-4-e4b",
            prompt: "You analyse historical documents.",
            user_prompt: "Fill in the fields.",
            input,
            languages: &LanguageChoice::Auto,
            options,
            tuning,
            analysis: Some(fields),
            stop,
        }
    }

    fn summary() -> AnalysisSchema {
        let mut schema = AnalysisSchema {
            fields: vec![Field {
                key: String::new(),
                label: "Summary".into(),
                instructions: String::new(),
                kind: FieldKind::Text,
                reason: false,
            }],
        };
        schema.assign_keys();
        schema
    }

    #[test]
    fn a_text_is_sent_whole_after_its_name_and_a_long_one_is_refused() {
        let fields = summary();
        let (options, tuning, stop) = Default::default();
        let input = PreparedInput {
            piece: PieceRef::Whole,
            mime: "text/plain".into(),
            bytes: "Le conseil s'est réuni.".as_bytes().to_vec(),
            file_name: "lettre.txt".into(),
        };
        let Reading::Text { system, user } =
            Reading::of(&request(&input, &fields, &options, &tuning, &stop)).unwrap()
        else {
            panic!("a text reads as a text");
        };
        assert_eq!(system, "You analyse historical documents.");
        assert_eq!(
            user,
            "<document name=\"lettre.txt\">\nLe conseil s'est réuni.\n</document>\n\nFill in the fields."
        );
        let long = PreparedInput {
            bytes: "mot ".repeat(MAX_CHARS / 4 + 1).into_bytes(),
            ..input
        };
        let Err(ProviderError::InvalidRequest(message)) =
            Reading::of(&request(&long, &fields, &options, &tuning, &stop))
        else {
            panic!("a long text is refused");
        };
        assert!(message.contains("too long"), "{message}");
    }

    #[test]
    fn the_arguments_read_files_hold_the_schema_and_turn_thinking_off() {
        let args = text_arguments("m.gguf", "s.json", "i.txt", "r.txt", 6);
        let pair = |flag: &str| {
            let i = args.iter().position(|a| a == flag).unwrap();
            args[i + 1].clone()
        };
        assert_eq!(pair("-m"), "m.gguf");
        assert_eq!(pair("-jf"), "s.json");
        assert_eq!(pair("-sysf"), "i.txt");
        assert_eq!(pair("-f"), "r.txt");
        assert_eq!(pair("--reasoning"), "off");
        assert_eq!(pair("--temp"), "0");
        assert_eq!(pair("-c"), CONTEXT.to_string());
        assert_eq!(pair("-t"), "6");
        assert_eq!(pair("-ngl"), "auto");
        assert_eq!(pair("--fit"), "on");
        assert!(args.iter().any(|a| a == "-st") && args.iter().any(|a| a == "--no-display-prompt"));

        let args = page_arguments("m.gguf", "e.gguf", "s.json", "p.png", "Sys", "Fill.", 6);
        let pair = |flag: &str| {
            let i = args.iter().position(|a| a == flag).unwrap();
            args[i + 1].clone()
        };
        assert_eq!(pair("--mmproj"), "e.gguf");
        assert_eq!(pair("--image"), "p.png");
        assert_eq!(pair("-p"), "<__media__>Fill.");
        assert_eq!(pair("-sys"), "Sys");
        assert_eq!(pair("--image-max-tokens"), IMAGE_TOKENS.to_string());
    }

    #[test]
    fn the_output_limit_follows_the_catalogue_within_bounds() {
        let tuning = |n| Tuning {
            reasoning: None,
            max_output_tokens: n,
        };
        assert_eq!(output_limit(&tuning(None)), MAX_TOKENS);
        assert_eq!(output_limit(&tuning(Some(2048))), 2048);
        assert_eq!(output_limit(&tuning(Some(100))), MIN_TOKENS);
        assert_eq!(output_limit(&tuning(Some(128_000))), MAX_TOKENS);
    }

    #[test]
    fn the_answer_is_the_json_object_the_program_printed() {
        assert_eq!(
            answer("{\"summary\":\"Une réunion.\"} [end of text]\n"),
            "{\"summary\":\"Une réunion.\"}"
        );
        assert_eq!(answer("\n{\"a\":1}\n"), "{\"a\":1}");
        let output = structured::finish(
            ProcessOutput {
                text: answer("{\"summary\":\"Une réunion.\"} [end of text]").to_owned(),
                ..ProcessOutput::default()
            },
            &summary(),
        )
        .unwrap();
        assert_eq!(
            output.fields[0].value,
            FieldValue::Text("Une réunion.".into())
        );
    }

    /// A real analysis through the real engine, when both are at hand: `LLM_SMOKE_DIR` names a
    /// folder holding the pinned model files, and `KALAMU_LLAMA_DIR` the folder with both
    /// programs. Run with `cargo test -p kalamu-offline llm::tests::smoke -- --ignored
    /// --nocapture`.
    #[tokio::test]
    #[ignore = "needs the engine and a model"]
    async fn smoke() {
        let dir = std::env::var("LLM_SMOKE_DIR").unwrap();
        let llama = Llama::locate(&[]).expect("KALAMU_LLAMA_DIR names the engine's folder");
        assert!(
            llama.reads_text(),
            "llama-completion is beside llama-mtmd-cli"
        );
        let host = Arc::new(LlamaHost::new(
            Some(llama),
            Vec::new(),
            true,
            std::env::temp_dir(),
        ));
        let llm = LocalLlm::new(ModelManager::new(dir.into(), &LLM), host);
        let fields = summary();
        let (options, tuning, stop) = Default::default();
        let input = PreparedInput {
            piece: PieceRef::Whole,
            mime: "text/plain".into(),
            bytes: "Dakar, le 3 mai 1957. Le Gouverneur général a reçu ce matin les délégués \
                    des syndicats, qui lui ont remis un mémoire sur les salaires."
                .as_bytes()
                .to_vec(),
            file_name: "bulletin.txt".into(),
        };
        for model in LLM_MODELS.iter().filter(|m| llm.models().is_ready(m)) {
            let started = std::time::Instant::now();
            let mut req = request(&input, &fields, &options, &tuning, &stop);
            req.model = model.id;
            let output = llm.process(&ApiKey::new(""), req).await.unwrap();
            println!(
                "== {} ({:.1} s)\n{}",
                model.id,
                started.elapsed().as_secs_f32(),
                output.text
            );
            assert!(!output.fields.is_empty());
        }
    }
}
