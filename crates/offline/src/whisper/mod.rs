//! Whisper on this PC, through whisper.cpp: nothing leaves the computer and no key is needed.
//!
//! Model loading, CPU gate and decoding settings follow Live Translation's `whisper` module.
//! What differs is the shape of the work: a whole recording piece at once rather than a live
//! stream, so accuracy is preferred over latency (beam search), and the piece is transcribed
//! in windows of about two minutes, cut at the quietest moment nearby. The job's stop signal is
//! checked between windows and, through whisper.cpp's abort callback, inside them: whisper.cpp
//! asks it after each pass of the encoder (thirty seconds of audio) and each decoding step, so
//! pausing takes effect within one encoder pass rather than one window.
//!
//! Where the voice-activity model is downloaded, only each window's speech is decoded
//! (`speech.rs`). Each window's language, unless the researcher chose one, is detected on its
//! loudest thirty seconds of speech. Lines are kept to subtitle length, cut between words at the
//! times whisper.cpp gives each token; the text joins them into sentences. The names and words
//! the researcher listed are Whisper's prompt (`text.rs`), and a model that can translate writes
//! English when asked. Lines invented over silence, loops, the prompt read back and subtitle
//! credits are cut.
//!
//! Where it runs: on a Mac, the graphics processor through Metal, in the app; on Windows, the
//! graphics processor through Vulkan, in a helper program (`helper.rs`), when it is shipped,
//! finds a usable device and Settings allows it; otherwise the processor. A piece that fails on
//! the graphics processor is transcribed again on the processor, which the rest of the session
//! keeps to.

// Shared word for word with the Vulkan helper (`helpers/whisper-vulkan`); some of it (the
// helper's half of the messages) is only used there.
#[allow(dead_code, reason = "the helper's half of the shared file")]
mod engine;
mod helper;
mod speech;
mod text;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, LanguageChoice, LanguageCode, PCM_16K_MONO, ProcessOutput, ProcessRequest, Provider,
    ProviderError, ProviderId, RequestSetting, Segment, StopSignal, Task, Tuning, Usage,
};
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters, WhisperState};

use crate::cpu::{self, DEVICE, threads};
use crate::host::{self, IDLE_UNLOAD, lock};
use crate::models::{self, ModelFile, ModelLease, ModelManager};
use engine::{BEAM_SIZE, MAX_LINE_CHARS, RawSegment};
use speech::{Speech, Vad};
use text::{Prompt, initial_prompt, is_silence, sentences, without_loops};

pub const PROVIDER_ID: &str = "whisper";

const SAMPLE_RATE: usize = 16_000;
/// Target window length, and how far back from it the cut may move to find a pause.
const WINDOW: usize = 120 * SAMPLE_RATE;
const SEARCH_BACK: usize = 20 * SAMPLE_RATE;
/// The frame over which loudness is measured when looking for the quietest moment.
const FRAME: usize = SAMPLE_RATE / 10;
/// How much of a window its language is detected on: what Whisper's encoder reads at once.
const DETECT: usize = 30 * SAMPLE_RATE;
/// The helper's file name, beside the app's executable (Tauri's sidecar layout).
const HELPER: &str = "whisper-vulkan";

pub struct Whisper {
    inner: Arc<Inner>,
}

struct Inner {
    models: ModelManager,
    loaded: Mutex<Option<Loaded>>,
    /// The voice-activity model, loaded with the first piece after it was downloaded.
    vad: Mutex<Option<Vad>>,
    /// One transcription at a time: ggml's workers busy-wait, so two passes side by side take
    /// longer than taking turns, and the interface needs a core.
    turn: Mutex<()>,
    janitor: Mutex<bool>,
    graphics: Graphics,
}

/// Where a piece is transcribed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Placement {
    /// In the app: on the processor, or on a Mac's graphics processor (Metal).
    BuiltIn,
    /// The Vulkan helper, on this graphics processor.
    Vulkan(String),
}

impl Placement {
    /// As provenance records it: `cpu`, `metal` or `vulkan`.
    fn kind(&self) -> &'static str {
        match self {
            Self::BuiltIn => DEVICE,
            Self::Vulkan(_) => "vulkan",
        }
    }
}

/// The Vulkan helper and what is known of it this session, as `device.rs` keeps for the OCR
/// models' graphics builds.
struct Graphics {
    program: Option<PathBuf>,
    /// Settings › Offline: use the graphics processor when there is one.
    wanted: AtomicBool,
    /// The device the helper would use, asked once.
    found: OnceLock<Option<String>>,
    /// A piece failed there: the rest of the session runs on the processor.
    failed: AtomicBool,
}

impl Graphics {
    /// The device the helper chooses, asked the first time (at most half a minute).
    fn device(&self) -> Option<&str> {
        let program = self.program.as_ref()?;
        self.found
            .get_or_init(|| helper::find_device(program))
            .as_deref()
    }

    fn placement(&self) -> Placement {
        let usable = self.wanted.load(Ordering::Relaxed) && !self.failed.load(Ordering::Relaxed);
        match self.device() {
            Some(device) if usable => Placement::Vulkan(device.to_owned()),
            _ => Placement::BuiltIn,
        }
    }

    fn give_up(&self, reason: &str) {
        if !self.failed.swap(true, Ordering::Relaxed) {
            tracing::warn!(
                reason,
                "Whisper failed on the graphics processor: it runs on the processor until Kalamu restarts"
            );
        }
    }
}

/// A loaded model, where it was loaded.
enum Runtime {
    InProcess(Arc<WhisperContext>),
    Helper(helper::Worker),
}

struct Loaded {
    runtime: Runtime,
    placement: Placement,
    lease: ModelLease,
    last_used: Instant,
}

impl Whisper {
    /// Whisper in the app only.
    pub fn new(models: ModelManager) -> Self {
        Self::with_helper(models, None, false)
    }

    /// With the Vulkan `helper` program (Windows), used when `use_graphics` (Settings) and it
    /// finds a graphics processor.
    pub fn with_helper(models: ModelManager, helper: Option<PathBuf>, use_graphics: bool) -> Self {
        Self {
            inner: Arc::new(Inner {
                models,
                loaded: Mutex::new(None),
                vad: Mutex::new(None),
                turn: Mutex::new(()),
                janitor: Mutex::new(false),
                graphics: Graphics {
                    program: helper,
                    wanted: AtomicBool::new(use_graphics),
                    found: OnceLock::new(),
                    failed: AtomicBool::new(false),
                },
            }),
        }
    }

    /// The Vulkan helper among `dirs` (the app's own folder), when this copy of the app has it.
    pub fn locate_helper(dirs: &[PathBuf]) -> Option<PathBuf> {
        let name = format!("{HELPER}{}", std::env::consts::EXE_SUFFIX);
        dirs.iter().map(|dir| dir.join(&name)).find(|p| p.is_file())
    }

    pub fn models(&self) -> &ModelManager {
        &self.inner.models
    }

    /// Asks the helper for its graphics processor now, so the first piece need not wait. Takes
    /// up to half a minute when a driver hangs: run it off the interface's thread.
    pub fn find_graphics_processor(&self) {
        self.inner.graphics.device();
    }

    /// Settings › Offline: run on the graphics processor when there is one.
    pub fn use_graphics_processor(&self, wanted: bool) {
        self.inner.graphics.wanted.store(wanted, Ordering::Relaxed);
    }

    /// The graphics processor the helper found, whether or not it is used; `None` when there
    /// is none, no helper, or it is still being asked.
    pub fn graphics_processor(&self) -> Option<String> {
        self.inner.graphics.found.get().cloned().flatten()
    }

    /// Where the next piece goes: `cpu`, `metal` or `vulkan`.
    pub fn device(&self) -> &'static str {
        match self.inner.graphics.found.get() {
            Some(_) => self.inner.graphics.placement().kind(),
            None => DEVICE,
        }
    }

    /// Starts, once, the task that releases a model left unused.
    fn start_janitor(&self) {
        host::sweep_idle(&self.inner, &self.inner.janitor, |inner| {
            // Never while a transcription holds the turn.
            let Ok(_turn) = inner.turn.try_lock() else {
                return;
            };
            let mut loaded = lock(&inner.loaded);
            if loaded
                .as_ref()
                .is_some_and(|l| l.last_used.elapsed() > IDLE_UNLOAD)
            {
                tracing::info!("releasing the idle Whisper model");
                *loaded = None;
            }
        });
    }
}

impl Inner {
    /// `model` loaded at `placement`, loading it (and releasing another) when needed.
    fn load<'a>(
        &self,
        loaded: &'a mut Option<Loaded>,
        model: &'static ModelFile,
        placement: &Placement,
    ) -> Result<&'a mut Loaded, ProviderError> {
        let current = loaded
            .as_ref()
            .is_some_and(|l| l.lease.model.id == model.id && l.placement == *placement);
        if !current {
            // Release the previous model before loading the next: two large ones may not fit.
            *loaded = None;
            let lease = self.models.lease(model)?;
            let runtime = match (placement, &self.graphics.program) {
                (Placement::Vulkan(_), Some(program)) => {
                    match helper::Worker::start(program, &lease) {
                        Ok(worker) => {
                            tracing::info!(device = %worker.device, "Whisper loaded on the graphics processor");
                            Runtime::Helper(worker)
                        }
                        Err(failure) => return Err(self.helper_failed(&failure)),
                    }
                }
                _ => Runtime::InProcess(in_process(&lease)?),
            };
            *loaded = Some(Loaded {
                runtime,
                placement: placement.clone(),
                lease,
                last_used: Instant::now(),
            });
        }
        loaded
            .as_mut()
            .ok_or_else(|| ProviderError::Other("Whisper has no model loaded".into()))
    }

    /// The helper failed: the rest of the session runs on the processor, and the piece is sent
    /// again, there.
    fn helper_failed(&self, failure: &helper::Failure) -> ProviderError {
        match failure {
            helper::Failure::Stopped => ProviderError::Interrupted,
            helper::Failure::Failed(reason) => {
                self.graphics.give_up(reason);
                ProviderError::Transient(format!(
                    "Whisper failed on the graphics processor ({reason}); the piece is transcribed again on the processor"
                ))
            }
        }
    }

    /// The voice-activity model, loading it once it is downloaded.
    fn vad(&self) -> std::sync::MutexGuard<'_, Option<Vad>> {
        let mut vad = lock(&self.vad);
        if vad.is_none()
            && let Some(path) = self.models.vad_path()
        {
            *vad = Vad::load(&path);
        }
        vad
    }
}

/// The leased model in this process, on a Mac's graphics processor through Metal.
fn in_process(lease: &ModelLease) -> Result<Arc<WhisperContext>, ProviderError> {
    let bytes = lease.read_verified()?;
    let metal = DEVICE == "metal";
    let mut params = WhisperContextParameters::default();
    params.use_gpu(metal);
    // Attention in one pass, as whisper.cpp's own tools do on a graphics processor.
    params.flash_attn(metal);
    WhisperContext::new_from_buffer_with_params(&bytes, params)
        .map(Arc::new)
        .map_err(|e| ProviderError::Other(format!("Could not load the model: {e}")))
}

fn ensure_cpu() -> Result<(), ProviderError> {
    let missing = cpu::missing_features();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(ProviderError::Unsupported(format!(
            "This processor lacks {}, which offline transcription needs; use a cloud model",
            missing.join(", ")
        )))
    }
}

/// The language to decode in: the user's single choice when Whisper knows it, otherwise
/// detection, again for every window, as mixed recordings need.
fn language(choice: &LanguageChoice) -> Option<String> {
    match choice.codes() {
        [one] if whisper_rs::get_lang_id(one.as_str()).is_some() => Some(one.as_str().to_owned()),
        _ => None,
    }
}

/// 16-bit little-endian samples to the floats whisper.cpp reads.
fn to_float(pcm: &[u8]) -> Vec<f32> {
    pcm.as_chunks::<2>()
        .0
        .iter()
        .map(|b| f32::from(i16::from_le_bytes(*b)) / 32_768.0)
        .collect()
}

/// Where to cut `samples` into windows of about [`WINDOW`]: at the quietest frame of the last
/// [`SEARCH_BACK`] before each target, so a cut falls in a pause rather than inside a word. The
/// last window may be up to a quarter longer rather than leaving a scrap.
fn windows(samples: &[f32]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    while samples.len() - start > WINDOW + WINDOW / 4 {
        let target = start + WINDOW;
        let from = target - SEARCH_BACK;
        let quietest = samples[from..target]
            .chunks(FRAME)
            .enumerate()
            .map(|(i, frame)| (i, frame.iter().map(|s| s * s).sum::<f32>()))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(SEARCH_BACK, |(i, _)| i * FRAME + FRAME / 2);
        let cut = from + quietest;
        out.push((start, cut));
        start = cut;
    }
    out.push((start, samples.len()));
    out
}

fn whisper_error(e: impl std::fmt::Display) -> ProviderError {
    ProviderError::Other(format!("Whisper failed: {e}"))
}

/// What provenance records of the voice-activity model: whether only the speech was decoded,
/// and with which model.
fn voice_detection(on: bool) -> Vec<RequestSetting> {
    let mut settings = vec![RequestSetting::new("vad", on)];
    if on {
        settings.push(RequestSetting::new(
            "vad_model",
            models::VAD.file.trim_end_matches(".bin"),
        ));
    }
    settings
}

/// whisper.cpp's abort callback, without `unsafe`, which this workspace forbids. whisper-rs
/// 0.16's `set_abort_callback_safe` gives its trampoline a pointer to a `Box<dyn FnMut() ->
/// bool>` but reads it as the closure's own type: correct only when that type is the boxed one,
/// as here, never for a bare closure. It also never frees the box: a few bytes, and one
/// `StopSignal` handle, per window.
fn stop_when(params: &mut FullParams<'_, '_>, stop: &StopSignal) {
    let stop = stop.clone();
    let callback: Box<dyn FnMut() -> bool> = Box::new(move || stop.is_stopped());
    params.set_abort_callback_safe::<_, Box<dyn FnMut() -> bool>>(Some(callback));
}

/// Where the windows of one piece are decoded.
enum Runner<'a> {
    InProcess {
        context: &'a WhisperContext,
        state: WhisperState,
    },
    Helper(&'a mut helper::Worker),
}

impl Runner<'_> {
    fn text_context(&self) -> i32 {
        match self {
            Self::InProcess { context, .. } => context.n_text_ctx(),
            Self::Helper(worker) => worker.text_context,
        }
    }

    fn tokenize(&mut self, text: &str) -> Option<Vec<i32>> {
        match self {
            // Every token stands for at least one byte, so the text's length is room enough:
            // whisper-rs mishandles whisper.cpp's "needs more room" answer.
            Self::InProcess { context, .. } => context.tokenize(text, text.len() + 1).ok(),
            Self::Helper(worker) => worker.tokenize(text),
        }
    }

    fn detect(
        &mut self,
        samples: &[f32],
        threads: usize,
        stop: &StopSignal,
    ) -> Result<Option<i32>, helper::Failure> {
        match self {
            Self::InProcess { state, .. } => engine::detect_language(state, samples, threads)
                .map_err(|e| helper::Failure::Failed(e.to_string())),
            Self::Helper(worker) => {
                worker.detect(samples, i32::try_from(threads).unwrap_or(1), stop)
            }
        }
    }

    fn decode(
        &mut self,
        decode: &engine::Decode,
        samples: &[f32],
        stop: &StopSignal,
    ) -> Result<Vec<RawSegment>, helper::Failure> {
        match self {
            Self::InProcess { state, .. } => {
                let mut params = engine::params(decode);
                stop_when(&mut params, stop);
                match state.full(params, samples) {
                    Ok(()) => Ok(engine::segments(state)),
                    Err(_) if stop.is_stopped() => Err(helper::Failure::Stopped),
                    Err(error) => Err(helper::Failure::Failed(error.to_string())),
                }
            }
            Self::Helper(worker) => worker.decode(decode, samples, stop),
        }
    }
}

struct Transcript {
    segments: Vec<Segment>,
    detected: Option<String>,
}

/// What a transcription asks beside the audio.
#[derive(Debug, Default)]
struct Job {
    /// The language chosen; detected window by window when `None`.
    language: Option<String>,
    /// The names and words to spell.
    terms: Vec<String>,
    /// Write English rather than the language spoken.
    translate: bool,
}

/// Transcribe `samples` with `runner`, decoding only the speech when `vad` is given. The caller
/// turns a failure into the error the engine needs: a helper's sends the piece to the processor.
fn transcribe(
    runner: &mut Runner<'_>,
    samples: &[f32],
    job: &Job,
    mut vad: Option<&mut Vad>,
    stop: &StopSignal,
) -> Result<Transcript, helper::Failure> {
    let threads = usize::try_from(threads(
        std::thread::available_parallelism().map_or(2, usize::from),
    ))
    .unwrap_or(1);
    let budget = usize::try_from(runner.text_context() / 2 - 1).unwrap_or(0);
    let prompt = Prompt::fit(&job.terms, budget, |text| runner.tokenize(text));
    if initial_prompt(&job.terms).is_some_and(|offered| offered != prompt.text) {
        tracing::warn!(
            kept = prompt.len(),
            "Whisper's prompt holds only the first names and words; provenance lists all those offered"
        );
    }
    let mut segments = Vec::new();
    // How long each language detected was spoken, to name the recording's.
    let mut heard: Vec<(&'static str, usize)> = Vec::new();
    for (start, end) in windows(samples) {
        if stop.is_stopped() {
            return Err(helper::Failure::Stopped);
        }
        let window = &samples[start..end];
        let speech = match vad.as_deref_mut().map(|vad| vad.speech(window)) {
            Some(Some(ranges)) => match Speech::join(window, &ranges) {
                Some(speech) => speech,
                // No speech in this window: nothing to decode.
                None => continue,
            },
            _ => Speech::whole(window),
        };
        // Whisper needs at least a second of audio; padding affects inference, not timestamps.
        let mut audio = speech.samples.clone();
        audio.resize(audio.len().max(SAMPLE_RATE), 0.0);
        let detected = match job.language {
            Some(_) => None,
            None => {
                let (from, to) = loudest_stretch(&audio, DETECT);
                let mut stretch = audio[from..to].to_vec();
                stretch.resize(stretch.len().max(SAMPLE_RATE), 0.0);
                runner
                    .detect(&stretch, threads, stop)?
                    .and_then(whisper_rs::get_lang_str)
            }
        };
        // Detection is a pass of its own, which whisper.cpp cannot stop midway.
        if stop.is_stopped() {
            return Err(helper::Failure::Stopped);
        }
        if let Some(lang) = detected {
            match heard.iter_mut().find(|(l, _)| *l == lang) {
                Some((_, length)) => *length += speech.samples.len(),
                None => heard.push((lang, speech.samples.len())),
            }
        }
        let decode = engine::Decode {
            language: job.language.clone().or(detected.map(str::to_owned)),
            translate: job.translate,
            threads: i32::try_from(threads).unwrap_or(1),
            prompt: prompt.tokens.clone(),
        };
        let offset_ms = (start * 1000 / SAMPLE_RATE) as u64;
        for line in runner.decode(&decode, &audio, stop)? {
            let text = line.text.trim();
            if text.is_empty() || is_silence(&line) || prompt.is_echo(text) {
                continue;
            }
            segments.push(Segment {
                start_ms: offset_ms + speech.original_ms(line.start_cs),
                end_ms: offset_ms + speech.original_ms(line.end_cs),
                speaker: None,
                text: text.to_owned(),
            });
        }
    }
    let detected = heard
        .iter()
        .max_by_key(|(_, length)| *length)
        .map(|(lang, _)| (*lang).to_owned());
    Ok(Transcript {
        segments: without_loops(segments),
        detected,
    })
}

/// Where the stretch of `len` samples holding the most sound begins and ends, in whole frames;
/// all of `samples` when shorter.
fn loudest_stretch(samples: &[f32], len: usize) -> (usize, usize) {
    if samples.len() <= len {
        return (0, samples.len());
    }
    let energy: Vec<f32> = samples
        .chunks(FRAME)
        .map(|frame| frame.iter().map(|s| s * s).sum())
        .collect();
    let frames = (len / FRAME).clamp(1, energy.len());
    let mut sum: f32 = energy[..frames].iter().sum();
    let mut best = (sum, 0);
    for i in frames..energy.len() {
        sum += energy[i] - energy[i - frames];
        if sum > best.0 {
            best = (sum, i + 1 - frames);
        }
    }
    let start = (best.1 * FRAME).min(samples.len() - len);
    (start, start + len)
}

#[async_trait]
impl Provider for Whisper {
    fn id(&self) -> ProviderId {
        ProviderId::new(PROVIDER_ID)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn check_ready(&self, model: &str, _languages: &LanguageChoice) -> Result<(), ProviderError> {
        ensure_cpu()?;
        let file = models::find(model)
            .ok_or_else(|| ProviderError::Unsupported(format!("Unknown Whisper model {model}")))?;
        if self.inner.models.is_installed(file) {
            Ok(())
        } else {
            Err(ProviderError::NotInstalled(model.to_owned()))
        }
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::AudioTranscription => &[PCM_16K_MONO],
            _ => &[],
        }
    }

    fn request_settings(&self, task: Task, _model: &str, _tuning: &Tuning) -> Vec<RequestSetting> {
        if task != Task::AudioTranscription {
            return Vec::new();
        }
        vec![
            RequestSetting::new("beam_size", BEAM_SIZE),
            RequestSetting::new("no_context", true),
            RequestSetting::new("window_seconds", WINDOW / SAMPLE_RATE),
            RequestSetting::new("language_detection", "loudest_30_seconds_of_each_window"),
            RequestSetting::new("max_line_chars", MAX_LINE_CHARS),
            RequestSetting::new("split_on_word", true),
            RequestSetting::new("repetition_guard", true),
        ]
    }

    /// The fixed settings, and what is decided on this PC, so a pipeline's record leaves it out
    /// and each result's provenance says it: whether only the speech is decoded (when the
    /// voice-activity model is downloaded), and on which device.
    fn sent_settings(&self, request: &ProcessRequest<'_>, _batched: bool) -> Vec<RequestSetting> {
        let mut settings = self.request_settings(request.task, request.model, request.tuning);
        if request.task == Task::AudioTranscription {
            settings.extend(voice_detection(self.inner.models.vad_path().is_some()));
            let placement = self.inner.graphics.placement();
            settings.push(RequestSetting::new("device", placement.kind()));
            if let Placement::Vulkan(name) = &placement {
                settings.push(RequestSetting::new("device_name", name.as_str()));
            }
            if request.options.translate {
                settings.push(RequestSetting::new("translate", true));
            }
            if let Some(prompt) = initial_prompt(&request.options.vocabulary_terms()) {
                settings.push(RequestSetting::new("initial_prompt", prompt));
            }
        }
        settings
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        if request.input.mime != PCM_16K_MONO {
            return Err(ProviderError::Unsupported(format!(
                "Whisper reads 16 kHz audio, not {}",
                request.input.mime
            )));
        }
        ensure_cpu()?;
        let model = models::find(request.model).ok_or_else(|| {
            ProviderError::Unsupported(format!("Unknown Whisper model {}", request.model))
        })?;
        self.start_janitor();
        // Where `sent_settings` said, just before: the record must name the device that ran.
        let planned = self.inner.graphics.placement();
        let inner = self.inner.clone();
        let samples = to_float(&request.input.bytes);
        let job = Job {
            language: language(request.languages),
            terms: request.options.vocabulary_terms(),
            translate: request.options.translate,
        };
        let stop = request.stop.clone();
        let transcript = tokio::task::spawn_blocking(move || {
            let _turn = lock(&inner.turn);
            // A pause that came while waiting for the turn.
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            let placement = inner.graphics.placement();
            if placement != planned {
                return Err(ProviderError::Transient(
                    "Whisper moved to another device; the piece is transcribed again there".into(),
                ));
            }
            let mut loaded = lock(&inner.loaded);
            let current = inner.load(&mut loaded, model, &placement)?;
            let mut vad = inner.vad();
            let result = match &mut current.runtime {
                Runtime::InProcess(context) => {
                    let mut runner = Runner::InProcess {
                        context: context.as_ref(),
                        state: context.create_state().map_err(whisper_error)?,
                    };
                    transcribe(&mut runner, &samples, &job, vad.as_mut(), &stop).map_err(
                        |failure| match failure {
                            helper::Failure::Stopped => ProviderError::Interrupted,
                            helper::Failure::Failed(reason) => whisper_error(reason),
                        },
                    )
                }
                Runtime::Helper(worker) => {
                    let mut runner = Runner::Helper(worker);
                    transcribe(&mut runner, &samples, &job, vad.as_mut(), &stop)
                        .map_err(|failure| inner.helper_failed(&failure))
                }
            };
            current.last_used = Instant::now();
            // A helper that failed or was stopped mid-window is of no further use.
            if result.is_err() && matches!(current.runtime, Runtime::Helper(_)) {
                *loaded = None;
            }
            result.map(|t| (t, samples.len()))
        })
        .await
        .map_err(|e| ProviderError::Other(format!("Whisper stopped unexpectedly: {e}")))?;
        let (transcript, sample_count) = transcript?;
        let text = sentences(&transcript.segments);
        Ok(ProcessOutput {
            text,
            segments: transcript.segments,
            detected_language: transcript.detected.map(LanguageCode::new),
            usage: Usage {
                audio_seconds: sample_count as f64 / SAMPLE_RATE as f64,
                ..Usage::default()
            },
            model_version: Some(model.file.trim_end_matches(".bin").to_owned()),
            fields: Vec::new(),
            regions: Vec::new(),
            truncated: false,
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(models::MODELS
            .iter()
            .filter(|m| self.inner.models.is_installed(m))
            .map(|m| m.id.to_owned())
            .collect())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn windows_cover_the_audio_and_cut_in_the_quietest_moment() {
        // Five minutes of "speech" with one silent frame in the search range before 2:00.
        let mut samples = vec![0.5f32; 300 * SAMPLE_RATE];
        let quiet = WINDOW - 5 * SAMPLE_RATE;
        for s in &mut samples[quiet..quiet + FRAME] {
            *s = 0.0;
        }
        let cuts = windows(&samples);
        assert_eq!(cuts.first().unwrap().0, 0);
        assert_eq!(cuts.last().unwrap().1, samples.len());
        for pair in cuts.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "windows must touch without gaps");
        }
        assert_eq!(cuts[0].1, quiet + FRAME / 2);
        // Short audio is one window, and a scrap is folded into the last one.
        assert_eq!(windows(&samples[..60 * SAMPLE_RATE]).len(), 1);
        assert_eq!(windows(&samples[..145 * SAMPLE_RATE]).len(), 1);
    }

    #[test]
    fn language_is_detected_where_the_window_is_loudest() {
        // Ten seconds of silence, then forty of sound: the thirty seconds start in the sound.
        let mut samples = vec![0.0f32; 50 * SAMPLE_RATE];
        for s in &mut samples[10 * SAMPLE_RATE..] {
            *s = 0.3;
        }
        let (from, to) = loudest_stretch(&samples, DETECT);
        assert!(from >= 10 * SAMPLE_RATE, "starts at {from}");
        assert_eq!(to - from, DETECT);
        // A window shorter than the stretch is read whole.
        assert_eq!(
            loudest_stretch(&samples[..SAMPLE_RATE], DETECT),
            (0, SAMPLE_RATE)
        );
    }

    #[test]
    fn pcm_bytes_become_floats() {
        let pcm = [0x00, 0x80, 0xff, 0x7f, 0x00, 0x00];
        assert_eq!(to_float(&pcm), [-1.0, 32_767.0 / 32_768.0, 0.0]);
    }

    #[test]
    fn only_a_single_known_language_is_forced() {
        let one = LanguageChoice::Explicit(vec![LanguageCode::new("fr")]);
        assert_eq!(language(&one).as_deref(), Some("fr"));
        let unknown = LanguageChoice::Explicit(vec![LanguageCode::new("mos")]);
        assert_eq!(language(&unknown), None);
        let mixed =
            LanguageChoice::Explicit(vec![LanguageCode::new("fr"), LanguageCode::new("en")]);
        assert_eq!(language(&mixed), None);
        assert_eq!(language(&LanguageChoice::Auto), None);
    }

    /// The catalogue offers the languages Whisper knows, and every model the code can fetch.
    #[test]
    fn the_catalogue_matches_what_whisper_can_do() {
        let catalog = kalamu_catalog::Catalog::bundled();
        let offered: Vec<_> = catalog
            .models
            .iter()
            .filter(|m| m.provider.as_str() == PROVIDER_ID)
            .collect();
        assert_eq!(offered.len(), models::MODELS.len());
        for model in offered {
            assert!(
                models::find(&model.id).is_some(),
                "{} cannot be downloaded",
                model.id
            );
            assert_eq!(model.inputs, [PCM_16K_MONO]);
            assert!(model.vocabulary, "{} takes names to spell", model.id);
            // Turbo was trained without translation; the others translate into English.
            assert_eq!(model.translates, model.id != "whisper-large-v3-turbo");
            let kalamu_catalog::LanguageSupport::List(codes) = &model.languages else {
                panic!("{} must list its languages", model.id);
            };
            for code in codes {
                assert!(
                    whisper_rs::get_lang_id(code.as_str()).is_some(),
                    "Whisper does not know {}",
                    code.as_str()
                );
            }
        }
        assert!(
            catalog
                .provider(&ProviderId::new(PROVIDER_ID))
                .is_some_and(|p| p.local)
        );
    }

    /// Provenance says whether the silences were skipped, which depends on the voice-activity
    /// model being downloaded.
    #[test]
    fn the_settings_sent_say_whether_voice_detection_was_on() {
        let dir = tempfile::tempdir().unwrap();
        let whisper = Whisper::new(ModelManager::new(
            dir.path().to_path_buf(),
            &models::WHISPER,
        ));
        let input = kalamu_core::PreparedInput {
            piece: kalamu_core::PieceRef::Whole,
            mime: PCM_16K_MONO.into(),
            bytes: Vec::new(),
            file_name: "interview.wav".into(),
        };
        let (languages, options, stop) = (
            LanguageChoice::Auto,
            kalamu_core::SetupOptions::default(),
            StopSignal::default(),
        );
        let sent = |whisper: &Whisper| {
            whisper.sent_settings(
                &ProcessRequest {
                    task: Task::AudioTranscription,
                    model: "whisper-large-v3-turbo",
                    prompt: "",
                    user_prompt: "",
                    input: &input,
                    languages: &languages,
                    options: &options,
                    tuning: &Tuning::default(),
                    analysis: None,
                    stop: &stop,
                },
                false,
            )
        };
        let off = sent(&whisper);
        assert!(off.contains(&RequestSetting::new("beam_size", BEAM_SIZE)));
        assert!(off.contains(&RequestSetting::new("device", DEVICE)));
        assert!(
            !off.iter()
                .any(|s| s.name == "translate" || s.name == "initial_prompt")
        );
        assert!(off.contains(&RequestSetting::new("vad", false)));
        assert!(!off.iter().any(|s| s.name == "vad_model"));

        let vad = dir.path().join(models::VAD.file);
        std::fs::write(&vad, vec![0u8; models::VAD.bytes as usize]).unwrap();
        let on = sent(&whisper);
        assert!(on.contains(&RequestSetting::new("vad", true)));
        assert!(on.contains(&RequestSetting::new("vad_model", "ggml-silero-v5.1.2")));
        // A pipeline's record, the same on every PC, leaves it out.
        let fixed = whisper.request_settings(
            Task::AudioTranscription,
            "whisper-large-v3-turbo",
            &Tuning::default(),
        );
        assert!(!fixed.iter().any(|s| s.name.starts_with("vad")));
    }

    /// The device follows the helper's answer, Settings and a failure, in what provenance says.
    #[test]
    fn the_device_recorded_is_where_the_piece_goes() {
        let dir = tempfile::tempdir().unwrap();
        let models = || ModelManager::new(dir.path().to_path_buf(), &models::WHISPER);
        // No helper, or one that cannot start: in the app.
        assert_eq!(
            Whisper::new(models()).inner.graphics.placement(),
            Placement::BuiltIn
        );
        let missing = Whisper::with_helper(models(), Some(dir.path().join("nowhere")), true);
        assert_eq!(missing.inner.graphics.placement(), Placement::BuiltIn);
        assert_eq!(missing.graphics_processor(), None);
        // A helper that found a graphics card.
        let whisper = Whisper::with_helper(models(), Some(dir.path().join("helper")), true);
        let card = "NVIDIA GeForce RTX 4060".to_owned();
        whisper
            .inner
            .graphics
            .found
            .set(Some(card.clone()))
            .unwrap();
        assert_eq!(
            whisper.inner.graphics.placement(),
            Placement::Vulkan(card.clone())
        );
        assert_eq!(whisper.device(), "vulkan");
        assert_eq!(whisper.graphics_processor().as_deref(), Some(card.as_str()));
        whisper.use_graphics_processor(false);
        assert_eq!(whisper.inner.graphics.placement(), Placement::BuiltIn);
        whisper.use_graphics_processor(true);
        whisper.inner.graphics.give_up("out of memory");
        assert_eq!(whisper.inner.graphics.placement(), Placement::BuiltIn);
        assert_eq!(whisper.device(), DEVICE);
    }

    /// The recording `WHISPER_SMOKE_AUDIO` names (16 kHz mono 16-bit, raw or WAV), or the
    /// English sample interview.
    fn smoke_audio() -> Vec<f32> {
        let path = std::env::var_os("WHISPER_SMOKE_AUDIO")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/interview.wav")
            });
        let bytes = std::fs::read(path).unwrap();
        let pcm = bytes
            .windows(4)
            .position(|w| w == b"data")
            .map_or(&bytes[..], |at| &bytes[at + 8..]);
        to_float(pcm)
    }

    fn smoke_job() -> Job {
        Job {
            terms: std::env::var("WHISPER_SMOKE_NAMES")
                .map(|names| names.split(',').map(str::to_owned).collect())
                .unwrap_or_default(),
            translate: std::env::var("WHISPER_SMOKE_TRANSLATE").is_ok(),
            ..Job::default()
        }
    }

    fn check(transcript: &Transcript, samples: &[f32]) {
        println!("{:?} {:#?}", transcript.detected, transcript.segments);
        assert!(!transcript.segments.is_empty());
        let length_ms = (samples.len() * 1000 / SAMPLE_RATE) as u64;
        for pair in transcript.segments.windows(2) {
            assert!(pair[0].start_ms <= pair[1].start_ms, "{pair:?}");
        }
        for line in &transcript.segments {
            assert!(
                line.start_ms <= line.end_ms && line.end_ms <= length_ms + 1_000,
                "{line:?}"
            );
        }
    }

    /// A real transcription with a downloaded model, with speech detection when
    /// `WHISPER_SMOKE_VAD` names the voice-activity model; then the same recording stopped a
    /// moment in, which must end inside its one window (whisper.cpp's abort callback). Run with
    /// `WHISPER_SMOKE_MODEL=<path to ggml-*.bin> cargo test -p kalamu-offline smoke -- --ignored
    /// --nocapture`; CI does, with the pinned Base model.
    #[test]
    #[ignore = "needs a model file"]
    fn smoke() {
        let bytes = std::fs::read(std::env::var("WHISPER_SMOKE_MODEL").unwrap()).unwrap();
        let mut params = WhisperContextParameters::default();
        params
            .use_gpu(DEVICE == "metal")
            .flash_attn(DEVICE == "metal");
        let context = WhisperContext::new_from_buffer_with_params(&bytes, params).unwrap();
        let mut vad = std::env::var("WHISPER_SMOKE_VAD")
            .ok()
            .and_then(|path| Vad::load(&path));
        let samples = smoke_audio();
        let started = Instant::now();
        let mut runner = Runner::InProcess {
            context: &context,
            state: context.create_state().unwrap(),
        };
        let transcript = transcribe(
            &mut runner,
            &samples,
            &smoke_job(),
            vad.as_mut(),
            &StopSignal::default(),
        )
        .map_err(|f| format!("{f:?}"))
        .unwrap();
        println!(
            "{DEVICE}, speech detection {}: {:.2?}",
            vad.is_some(),
            started.elapsed()
        );
        check(&transcript, &samples);

        // Stopped within the window: the abort callback ends whisper.cpp's pass.
        let stop = StopSignal::default();
        let stopper = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            stopper.stop();
        });
        let started = Instant::now();
        let mut runner = Runner::InProcess {
            context: &context,
            state: context.create_state().unwrap(),
        };
        let stopped = transcribe(&mut runner, &samples, &smoke_job(), None, &stop);
        assert!(
            matches!(stopped, Err(helper::Failure::Stopped)),
            "not stopped"
        );
        println!("stopped after {:.2?}", started.elapsed());
    }

    /// The same through the Vulkan helper `WHISPER_VULKAN_HELPER` names (`cargo build -p
    /// whisper-vulkan`), with `WHISPER_SMOKE_MODEL` the pinned Base model. A software renderer
    /// is accepted only with `WHISPER_VULKAN_ALLOW_SOFTWARE=1`, so a machine without a graphics
    /// processor can run it on Mesa's lavapipe.
    #[test]
    #[ignore = "needs the Vulkan helper and a model file"]
    fn vulkan_smoke() {
        let helper = PathBuf::from(std::env::var_os("WHISPER_VULKAN_HELPER").unwrap());
        assert!(
            helper::find_device(&helper).is_some(),
            "the helper found no device"
        );
        let dir = tempfile::tempdir().unwrap();
        let model = models::find("whisper-base").unwrap();
        std::fs::copy(
            std::env::var("WHISPER_SMOKE_MODEL").unwrap(),
            dir.path().join(model.file),
        )
        .unwrap();
        let manager = ModelManager::new(dir.path().to_path_buf(), &models::WHISPER);
        let lease = manager.lease(model).unwrap();
        let mut worker = helper::Worker::start(&helper, &lease)
            .map_err(|f| format!("{f:?}"))
            .unwrap();
        println!("device: {}", worker.device);
        let samples = smoke_audio();
        let transcript = transcribe(
            &mut Runner::Helper(&mut worker),
            &samples,
            &smoke_job(),
            None,
            &StopSignal::default(),
        )
        .map_err(|f| format!("{f:?}"))
        .unwrap();
        check(&transcript, &samples);
    }
}
