//! An embedding model on this computer, for the search by meaning and the map of a project's
//! documents: Google's EmbeddingGemma 2, its text part only, with nothing sent anywhere and no
//! key.
//!
//! It runs in llama.cpp's embedding program (`llama-embedding`), shipped beside the processor
//! build of the OCR and language models' programs (Metal on a Mac) and sharing their host, so it
//! takes turns with them (`host.rs`). One run embeds every text of a call: they are written to
//! one file, separated by a marker, so the model is loaded once and no text goes on the command
//! line, which Windows limits to about 32,000 characters. Each text carries the task's prefix
//! the model was trained with, a question and a passage differently. The program prints the
//! vectors as JSON; they are counted, measured, cut to the length asked for (the model's first
//! dimensions carry the most, so 512, 256 or 128 of its 768 still serve) and made of length 1.
//!
//! The model is one file from ggml-org's conversion on Hugging Face at a pinned revision, at
//! 8-bit quantisation, downloaded and checked against its SHA-256 before use.

mod command;

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, EmbedLimits, EmbedRequest, Embedded, Embedder, ProviderError, ProviderId,
};

use crate::cpu::{self, threads};
use crate::host::{Checked, LlamaHost};
use crate::llama::{RunError, Scratch};
use crate::models::{ModelFile, ModelManager, ModelSet};

use command::{arguments, finish, prompt, texts_file, vectors};

/// The provider's id in the catalogue.
pub const EMBEDDING_ID: &str = "local-embedding";

/// ggml-org's GGUF conversion of Google's model (Apache 2.0), made with llama.cpp on 1 November
/// 2026. Its text model only: the image and audio encoders (`mmproj-…`) are not downloaded.
const EMBEDDINGGEMMA_2: &str = "https://huggingface.co/ggml-org/embeddinggemma-2-GGUF/resolve/6ea3e3a309d12a4779cfda98b8c19800d2d5da71";

/// The models. `id` is the model id in the catalogue. Q8_0 rather than the full BF16 (558 MB):
/// 8 bits keep an embedding's ranking all but unchanged at a little over half the size, and a
/// processor without bfloat16 reads it faster.
pub static EMBEDDING_MODELS: [ModelFile; 1] = [ModelFile {
    id: "embeddinggemma-2",
    file: "embeddinggemma-2-Q8_0.gguf",
    bytes: 309_855_456,
    sha256: "2188ac1deca4b77dffefd603c2776a9d76d9d74ec01841392982ebb840b09135",
    source: EMBEDDINGGEMMA_2,
    languages: &[],
    pair: None,
}];

pub static EMBEDDING: ModelSet = ModelSet {
    files: &EMBEDDING_MODELS,
    companion: None,
};

/// The length of EmbeddingGemma 2's vectors, and the shorter ones it was trained to be cut to.
pub const DIMENSIONS: [u32; 4] = [768, 512, 256, 128];

/// The tokens a call's run holds at once, and the most one text may have. The model reads
/// 8,192; half keeps the memory a run takes small on a laptop.
const CONTEXT: u32 = 4096;

/// The most characters in one text: two tokens a character, which no script passes (Gemma's
/// vocabulary holds most of the world's scripts whole; a character it lacks is read as its
/// bytes, which a rare emoji or ideograph may make three or four tokens, never a whole text of
/// them), leaving room for the task's prefix. About 300 words of French.
pub const MAX_CHARS: usize = 2000;

/// The most texts in one call: one run, the model loaded once for all of them.
pub const MAX_TEXTS: usize = 32;

pub struct LocalEmbedder {
    inner: Arc<Inner>,
}

struct Inner {
    models: ModelManager,
    host: Arc<LlamaHost>,
    checked: Checked,
}

impl LocalEmbedder {
    /// The embedding model in `models` (made with [`EMBEDDING`]), run in `host`'s programs,
    /// which the OCR and language models share.
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

    /// Whether this copy of the app carries the program the model runs in.
    pub fn available(&self) -> bool {
        self.inner.host.embeds()
    }
}

impl Inner {
    /// The model's file, from this manager's list (which tests replace).
    fn find(&self, model: &str) -> Result<&'static ModelFile, ProviderError> {
        self.models.find(model).ok_or_else(|| {
            ProviderError::Unsupported(format!("Unknown offline embedding model {model}"))
        })
    }
}

fn ensure_cpu() -> Result<(), ProviderError> {
    cpu::ensure(
        "the offline search by meaning",
        "choose a cloud model for it",
    )
}

/// The length of vector asked for, the model's own when none is.
fn dimensions(asked: Option<u32>) -> Result<usize, ProviderError> {
    let dimensions = asked.unwrap_or(DIMENSIONS[0]);
    if DIMENSIONS.contains(&dimensions) {
        Ok(dimensions as usize)
    } else {
        Err(ProviderError::InvalidRequest(format!(
            "EmbeddingGemma 2 gives vectors of 768, 512, 256 or 128 numbers, not {dimensions}"
        )))
    }
}

/// A text longer than one call may hold, in words a researcher can act on.
fn too_long() -> ProviderError {
    ProviderError::InvalidRequest(format!(
        "A passage is too long for the offline search by meaning, which reads at most {MAX_CHARS} \
         characters at once"
    ))
}

#[async_trait]
impl Embedder for LocalEmbedder {
    fn id(&self) -> ProviderId {
        ProviderId::new(EMBEDDING_ID)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn check_ready(&self, model: &str) -> Result<(), ProviderError> {
        if !self.inner.host.embeds() {
            return Err(ProviderError::Unsupported(
                "This copy of Kalamu has no engine for the offline search by meaning".into(),
            ));
        }
        ensure_cpu()?;
        let file = self.inner.find(model)?;
        if self.inner.models.is_ready(file) {
            Ok(())
        } else {
            Err(ProviderError::NotInstalled(model.to_owned()))
        }
    }

    fn limits(&self, _model: &str) -> EmbedLimits {
        EmbedLimits {
            texts: MAX_TEXTS,
            chars: MAX_CHARS,
        }
    }

    async fn embed(
        &self,
        _key: Option<&ApiKey>,
        request: EmbedRequest<'_>,
    ) -> Result<Embedded, ProviderError> {
        let dimensions = dimensions(request.dimensions)?;
        let count = request.texts.len();
        if count == 0 {
            return Ok(Embedded {
                vectors: Vec::new(),
                input_tokens: 0,
            });
        }
        if count > MAX_TEXTS {
            return Err(ProviderError::InvalidRequest(format!(
                "The offline search by meaning embeds at most {MAX_TEXTS} passages at once, \
                 not {count}"
            )));
        }
        if request.texts.iter().any(|t| t.chars().count() > MAX_CHARS) {
            return Err(too_long());
        }
        ensure_cpu()?;
        let model = self.inner.find(request.model)?;
        let prompts: Vec<String> = request
            .texts
            .iter()
            .map(|text| prompt(request.role, text))
            .collect();
        let inner = self.inner.clone();
        let stop = request.stop.clone();
        let printed = tokio::task::spawn_blocking(move || {
            let _turn = inner.host.turn();
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            let llama = inner.host.llama()?;
            let _leases = inner.checked.leases(&inner.models, model)?;
            let path = |p: &std::path::Path| p.to_string_lossy().into_owned();
            let scratch = Scratch::new(inner.host.scratch())?;
            let texts = path(&scratch.write("texts.txt", texts_file(&prompts).as_bytes())?);
            let logical = std::thread::available_parallelism().map_or(2, usize::from);
            let model_path = path(&inner.models.dir().join(model.file));
            let args = arguments(&model_path, &texts, threads(logical));
            let started = Instant::now();
            let run = llama.run_embedding(&args, &stop).map_err(|e| match e {
                RunError::Failed(reason) if reason.contains("exceeds batch size") => too_long(),
                other => other.into(),
            })?;
            tracing::debug!(
                texts = prompts.len(),
                tokens = run.read,
                seconds = started.elapsed().as_secs_f32(),
                "offline embedding run"
            );
            Ok(run.text)
        })
        .await
        .map_err(|e| {
            ProviderError::Other(format!(
                "The offline embedding model stopped unexpectedly: {e}"
            ))
        })??;
        let vectors = finish(
            vectors(&printed)?,
            count,
            DIMENSIONS[0] as usize,
            dimensions,
        )?;
        Ok(Embedded {
            vectors,
            input_tokens: 0,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use kalamu_core::{EmbedRole, StopSignal};

    use super::*;
    use crate::llama::Llama;

    /// As for the OCR and language models: vectors are kept under the model's id, so a new file
    /// that embeds differently needs a new id (here and in the catalogue), or the index would
    /// compare vectors of two models.
    #[test]
    fn a_new_model_file_is_a_decision_about_the_model_id() {
        let files: Vec<(&str, &str)> = EMBEDDING_MODELS
            .iter()
            .map(|f| (f.id, &f.sha256[..8]))
            .collect();
        assert_eq!(files, [("embeddinggemma-2", "2188ac1d")]);
    }

    /// Set in the environment of this test program started again as the embedding program: the
    /// test below then plays it, as `role` says.
    const ROLE: &str = "KALAMU_FAKE_EMBEDDING";
    /// Where the fake program writes the texts it read, for the test to compare.
    const SEEN: &str = "KALAMU_FAKE_EMBEDDING_SEEN";

    /// Not a test of its own: the fake embedding program, run only when [`ROLE`] is set. It
    /// reads the file of texts as llama.cpp's does (`-f`, split at `--embd-separator`) and prints
    /// vectors of 768 numbers, not of length 1, as the role says.
    #[test]
    fn fake_engine() {
        let Ok(role) = std::env::var(ROLE) else {
            return;
        };
        let args: Vec<String> = std::env::args().collect();
        let after = |flag: &str| {
            let i = args.iter().position(|a| a == flag).expect(flag);
            args[i + 1].clone()
        };
        let file = std::fs::read_to_string(after("-f")).unwrap();
        let texts: Vec<&str> = file.split(after("--embd-separator").as_str()).collect();
        if let Ok(seen) = std::env::var(SEEN) {
            std::fs::write(seen, serde_json::to_string(&texts).unwrap()).unwrap();
        }
        let vector = |i: usize, len: usize| {
            let mut v = vec![0.0f32; len];
            v[0] = 3.0 * (i + 1) as f32;
            v[1] = 4.0 * (i + 1) as f32;
            v
        };
        let (count, len) = match role.as_str() {
            "fewer" => (texts.len() - 1, 768),
            "short" => (texts.len(), 767),
            "slow" => {
                for _ in 0..1200 {
                    eprintln!("still working");
                    std::thread::sleep(Duration::from_millis(50));
                }
                return;
            }
            "long" => {
                eprintln!(
                    "main: number of tokens in input line (5210) exceeds batch size (4096), \
                     increase batch size and re-run"
                );
                std::process::exit(1);
            }
            _ => (texts.len(), 768),
        };
        let vectors: Vec<Vec<f32>> = (0..count).map(|i| vector(i, len)).collect();
        println!("{}", serde_json::to_string(&vectors).unwrap());
    }

    /// The test executable as the embedding program, playing `role`.
    fn fake(role: &str, seen: &Path) -> Llama {
        Llama::embedding_at(
            std::env::current_exe().unwrap(),
            [
                "--exact",
                "--nocapture",
                "--test-threads",
                "1",
                "--",
                "embed::tests::fake_engine",
            ]
            .iter()
            .map(|a| (*a).to_owned())
            .collect(),
        )
        .with_env(ROLE, role.into())
        .with_env(SEEN, seen.to_owned())
    }

    /// A model file small enough for a test, pinned as the real one is.
    static TINY: [ModelFile; 1] = [ModelFile {
        id: "embeddinggemma-2",
        file: "tiny.gguf",
        bytes: 3,
        // The SHA-256 of "abc".
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        source: "",
        languages: &[],
        pair: None,
    }];
    static TINY_SET: ModelSet = ModelSet {
        files: &TINY,
        companion: None,
    };

    struct Setup {
        dir: tempfile::TempDir,
        embedder: LocalEmbedder,
    }

    impl Setup {
        fn seen(&self) -> PathBuf {
            self.dir.path().join("seen.json")
        }

        fn texts_read(&self) -> Vec<String> {
            serde_json::from_str(&std::fs::read_to_string(self.seen()).unwrap()).unwrap()
        }

        /// What the scratch folder holds: nothing once a run is over.
        fn scratch_left(&self) -> usize {
            std::fs::read_dir(self.dir.path().join("scratch")).map_or(0, |d| d.count())
        }
    }

    fn setup(role: &str, downloaded: bool) -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let models = dir.path().join("models");
        std::fs::create_dir_all(&models).unwrap();
        if downloaded {
            std::fs::write(models.join("tiny.gguf"), b"abc").unwrap();
        }
        let llama = fake(role, &dir.path().join("seen.json"));
        let host = Arc::new(LlamaHost::new(
            Some(llama),
            Vec::new(),
            false,
            PathBuf::new(),
            dir.path().join("scratch"),
        ));
        let embedder = LocalEmbedder::new(ModelManager::new(models, &TINY_SET), host);
        Setup { dir, embedder }
    }

    fn request<'a>(
        texts: &'a [String],
        role: EmbedRole,
        dimensions: Option<u32>,
        stop: &'a StopSignal,
    ) -> EmbedRequest<'a> {
        EmbedRequest {
            model: "embeddinggemma-2",
            role,
            texts,
            dimensions,
            stop,
        }
    }

    #[tokio::test]
    async fn texts_come_back_as_unit_vectors_with_their_prefix_and_intact() {
        let setup = setup("echo", true);
        let texts = vec![
            "Le Gouverneur a dit : « \"Non\" ».\nDeuxième ligne\\n".to_owned(),
            "مدرسة دينية في تمبكتو 📜".to_owned(),
        ];
        let stop = StopSignal::default();
        let embedded = setup
            .embedder
            .embed(None, request(&texts, EmbedRole::Passage, None, &stop))
            .await
            .unwrap();
        assert_eq!(embedded.input_tokens, 0);
        assert_eq!(embedded.vectors.len(), 2);
        for vector in &embedded.vectors {
            assert_eq!(vector.len(), 768);
            assert!((vector[0] - 0.6).abs() < 1e-6 && (vector[1] - 0.8).abs() < 1e-6);
            let norm: f32 = vector.iter().map(|x| x * x).sum();
            assert!((norm - 1.0).abs() < 1e-5, "{norm}");
        }
        assert_eq!(
            setup.texts_read(),
            texts
                .iter()
                .map(|t| format!("title: none | text: {t}"))
                .collect::<Vec<_>>()
        );
        assert_eq!(setup.scratch_left(), 0);

        // A query, cut to 256 numbers.
        let query = vec!["les écoles coraniques".to_owned()];
        let embedded = setup
            .embedder
            .embed(None, request(&query, EmbedRole::Query, Some(256), &stop))
            .await
            .unwrap();
        assert_eq!(embedded.vectors[0].len(), 256);
        assert_eq!(
            setup.texts_read(),
            ["task: search result | query: les écoles coraniques"]
        );
    }

    #[tokio::test]
    async fn a_wrong_count_or_length_is_refused() {
        let texts = vec!["un".to_owned(), "deux".to_owned()];
        let stop = StopSignal::default();
        for role in ["fewer", "short"] {
            let setup = setup(role, true);
            let refused = setup
                .embedder
                .embed(None, request(&texts, EmbedRole::Passage, None, &stop))
                .await;
            assert!(
                matches!(refused, Err(ProviderError::Other(_))),
                "{refused:?}"
            );
        }
    }

    #[tokio::test]
    async fn a_stop_ends_the_run_within_a_second() {
        let setup = setup("slow", true);
        let texts = vec!["un".to_owned()];
        let stop = StopSignal::default();
        let raise = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            raise.stop();
        });
        let started = Instant::now();
        let ended = setup
            .embedder
            .embed(None, request(&texts, EmbedRole::Passage, None, &stop))
            .await;
        assert_eq!(ended, Err(ProviderError::Interrupted));
        assert!(
            started.elapsed() < Duration::from_millis(1_500),
            "{:?}",
            started.elapsed()
        );
        assert_eq!(setup.scratch_left(), 0);
        // Stopped before its turn came: nothing runs.
        let ended = setup
            .embedder
            .embed(None, request(&texts, EmbedRole::Passage, None, &stop))
            .await;
        assert_eq!(ended, Err(ProviderError::Interrupted));
    }

    #[tokio::test]
    async fn a_text_the_program_finds_too_long_is_the_requests_fault() {
        let setup = setup("long", true);
        let texts = vec!["un".to_owned()];
        let stop = StopSignal::default();
        let refused = setup
            .embedder
            .embed(None, request(&texts, EmbedRole::Passage, None, &stop))
            .await;
        assert!(
            matches!(&refused, Err(ProviderError::InvalidRequest(m)) if m.contains("too long")),
            "{refused:?}"
        );
    }

    #[tokio::test]
    async fn what_passes_the_limits_is_refused_before_running() {
        let setup = setup("echo", true);
        let stop = StopSignal::default();
        let long = vec!["é".repeat(MAX_CHARS + 1)];
        assert!(matches!(
            setup
                .embedder
                .embed(None, request(&long, EmbedRole::Passage, None, &stop))
                .await,
            Err(ProviderError::InvalidRequest(_))
        ));
        let many = vec!["un".to_owned(); MAX_TEXTS + 1];
        assert!(matches!(
            setup
                .embedder
                .embed(None, request(&many, EmbedRole::Passage, None, &stop))
                .await,
            Err(ProviderError::InvalidRequest(_))
        ));
        let one = vec!["un".to_owned()];
        assert!(matches!(
            setup
                .embedder
                .embed(None, request(&one, EmbedRole::Passage, Some(300), &stop))
                .await,
            Err(ProviderError::InvalidRequest(_))
        ));
        let none = setup
            .embedder
            .embed(None, request(&[], EmbedRole::Query, None, &stop))
            .await
            .unwrap();
        assert!(none.vectors.is_empty());
        assert!(!setup.seen().exists(), "nothing was run");
        assert_eq!(
            setup.embedder.limits("embeddinggemma-2"),
            EmbedLimits {
                texts: MAX_TEXTS,
                chars: MAX_CHARS
            }
        );
    }

    #[tokio::test]
    async fn a_missing_model_or_engine_says_so() {
        let missing = setup("echo", false);
        assert!(!missing.embedder.needs_key());
        assert_eq!(
            missing.embedder.check_ready("embeddinggemma-2"),
            Err(ProviderError::NotInstalled("embeddinggemma-2".into()))
        );
        let texts = vec!["un".to_owned()];
        let stop = StopSignal::default();
        assert_eq!(
            missing
                .embedder
                .embed(None, request(&texts, EmbedRole::Passage, None, &stop))
                .await,
            Err(ProviderError::NotInstalled("embeddinggemma-2".into()))
        );
        let ready = setup("echo", true);
        assert_eq!(ready.embedder.check_ready("embeddinggemma-2"), Ok(()));
        assert!(ready.embedder.available());
        assert!(matches!(
            ready.embedder.check_ready("embeddinggemma-1"),
            Err(ProviderError::Unsupported(_))
        ));

        // No programs at all, or the processor build without its embedding program.
        let dir = tempfile::tempdir().unwrap();
        let models = ModelManager::new(dir.path().to_owned(), &EMBEDDING);
        for host in [
            LlamaHost::none(),
            LlamaHost::new(
                Some(Llama::at(dir.path().join("llama-mtmd-cli"))),
                Vec::new(),
                false,
                PathBuf::new(),
                PathBuf::new(),
            ),
        ] {
            let embedder = LocalEmbedder::new(models.clone(), Arc::new(host));
            assert!(!embedder.available());
            assert!(matches!(
                embedder.check_ready("embeddinggemma-2"),
                Err(ProviderError::Unsupported(m)) if m.contains("no engine")
            ));
        }
    }

    /// A real embedding through the real engine, when both are at hand: `EMBED_SMOKE_DIR` names
    /// a folder holding the pinned model file, and `KALAMU_LLAMA_DIR` the folder with
    /// `llama-embedding`. Run with `cargo test -p kalamu-offline embed::tests::smoke --
    /// --ignored --nocapture`.
    #[tokio::test]
    #[ignore = "needs the engine and the model"]
    async fn smoke() {
        let dir = std::env::var("EMBED_SMOKE_DIR").unwrap();
        let llama = Llama::locate(&[]).expect("KALAMU_LLAMA_DIR names the engine's folder");
        assert!(llama.embeds(), "llama-embedding is beside llama-mtmd-cli");
        let host = Arc::new(LlamaHost::new(
            Some(llama),
            Vec::new(),
            false,
            std::env::temp_dir(),
            std::env::temp_dir().join("kalamu-offline-scratch"),
        ));
        let embedder = LocalEmbedder::new(ModelManager::new(dir.into(), &EMBEDDING), host);
        embedder.check_ready("embeddinggemma-2").unwrap();
        let stop = StopSignal::default();
        let passages = vec![
            "Dakar, le 3 mai 1957. Les cheminots de la ligne Dakar-Niger sont en grève pour leurs \
             salaires."
                .to_owned(),
            "La récolte d'arachide de la région de Kaolack a été abondante cette année.".to_owned(),
        ];
        let query = vec!["grève des travailleurs du chemin de fer".to_owned()];
        let started = Instant::now();
        let p = embedder
            .embed(None, request(&passages, EmbedRole::Passage, None, &stop))
            .await
            .unwrap();
        let q = embedder
            .embed(None, request(&query, EmbedRole::Query, None, &stop))
            .await
            .unwrap();
        let dot = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
        let (strike, harvest) = (
            dot(&q.vectors[0], &p.vectors[0]),
            dot(&q.vectors[0], &p.vectors[1]),
        );
        println!(
            "{:.1} s: strike {strike:.3}, harvest {harvest:.3}",
            started.elapsed().as_secs_f32()
        );
        assert!(strike > harvest);
    }
}
