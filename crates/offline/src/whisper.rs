//! Whisper on this PC, through whisper.cpp: nothing leaves the computer and no key is needed.
//!
//! Model loading, CPU gate and decoding settings follow Live Translation's `whisper` module.
//! What differs is the shape of the work: a whole recording piece at once rather than a live
//! stream, so accuracy is preferred over latency (beam search), and the piece is transcribed
//! in windows of about two minutes, cut at the quietest moment nearby. Between windows the job's
//! stop signal is checked, so pausing a long transcription takes effect within one window
//! without whisper.cpp's abort callback, which whisper-rs 0.16 only offers unsafely.

use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use scriptorium_core::{
    ApiKey, LanguageChoice, LanguageCode, PCM_16K_MONO, ProcessOutput, ProcessRequest, Provider,
    ProviderError, ProviderId, Segment, StopSignal, Task, Usage,
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::cpu;
use crate::models::{self, ModelFile, ModelLease, ModelManager};

pub const PROVIDER_ID: &str = "whisper";

const SAMPLE_RATE: usize = 16_000;
/// Target window length, and how far back from it the cut may move to find a pause.
const WINDOW: usize = 120 * SAMPLE_RATE;
const SEARCH_BACK: usize = 20 * SAMPLE_RATE;
/// The frame over which loudness is measured when looking for the quietest moment.
const FRAME: usize = SAMPLE_RATE / 10;
/// A loaded model is released after this long unused, to give its memory back.
const IDLE_UNLOAD: Duration = Duration::from_secs(180);

pub struct Whisper {
    inner: Arc<Inner>,
}

struct Inner {
    models: ModelManager,
    loaded: Mutex<Option<Loaded>>,
    /// One transcription at a time: ggml's workers busy-wait, so two passes side by side take
    /// longer than taking turns, and the interface needs a core.
    turn: Mutex<()>,
    janitor: Mutex<bool>,
}

struct Loaded {
    context: Arc<WhisperContext>,
    lease: ModelLease,
    last_used: Instant,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Whisper {
    pub fn new(models: ModelManager) -> Self {
        Self {
            inner: Arc::new(Inner {
                models,
                loaded: Mutex::new(None),
                turn: Mutex::new(()),
                janitor: Mutex::new(false),
            }),
        }
    }

    pub fn models(&self) -> &ModelManager {
        &self.inner.models
    }

    /// Starts, once, the task that releases a model left unused.
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
                // Never while a transcription holds the turn.
                let Ok(_turn) = inner.turn.try_lock() else {
                    continue;
                };
                let mut loaded = lock(&inner.loaded);
                if loaded
                    .as_ref()
                    .is_some_and(|l| l.last_used.elapsed() > IDLE_UNLOAD)
                {
                    tracing::info!("releasing the idle Whisper model");
                    *loaded = None;
                }
            }
        });
    }
}

impl Inner {
    /// The model's context, loading it (and releasing another) when needed.
    fn context(&self, model: &'static ModelFile) -> Result<Arc<WhisperContext>, ProviderError> {
        let mut loaded = lock(&self.loaded);
        if let Some(current) = loaded.as_mut()
            && current.lease.model.id == model.id
        {
            current.last_used = Instant::now();
            return Ok(current.context.clone());
        }
        // Release the previous model before loading the next: two large ones may not fit.
        *loaded = None;
        let lease = self.models.lease(model)?;
        let bytes = lease.read_verified()?;
        let mut params = WhisperContextParameters::default();
        params.use_gpu(false);
        let context = WhisperContext::new_from_buffer_with_params(&bytes, params)
            .map(Arc::new)
            .map_err(|e| ProviderError::Other(format!("Could not load the model: {e}")))?;
        *loaded = Some(Loaded {
            context: context.clone(),
            lease,
            last_used: Instant::now(),
        });
        Ok(context)
    }
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
/// detection, which whisper.cpp does again for every window, as mixed recordings need.
fn language(choice: &LanguageChoice) -> Option<String> {
    match choice.codes() {
        [one] if whisper_rs::get_lang_id(one.as_str()).is_some() => Some(one.as_str().to_owned()),
        _ => None,
    }
}

/// ggml's workers busy-wait, so a thread per hardware thread starves the interface; beyond
/// about eight these models stop getting faster. Half the logical processors on x64 (one per
/// core with SMT), all but one on ARM64, which has no SMT.
fn threads(logical: usize) -> i32 {
    let threads = if cfg!(target_arch = "x86_64") {
        logical / 2
    } else {
        logical.saturating_sub(1)
    };
    i32::try_from(threads).unwrap_or(i32::MAX).clamp(1, 8)
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

struct Transcript {
    segments: Vec<Segment>,
    detected: Option<String>,
}

fn transcribe(
    context: &WhisperContext,
    samples: &[f32],
    language: Option<&str>,
    vad: Option<&str>,
    stop: &StopSignal,
) -> Result<Transcript, ProviderError> {
    let mut state = context.create_state().map_err(whisper_error)?;
    let threads = threads(std::thread::available_parallelism().map_or(2, usize::from));
    let mut segments = Vec::new();
    let mut detected = None;
    for (start, end) in windows(samples) {
        if stop.is_stopped() {
            return Err(ProviderError::Interrupted);
        }
        // whisper.cpp's own command-line defaults for file transcription: beam search of 5.
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_language(language);
        params.set_n_threads(threads);
        // Each window starts fresh: carrying text over is what lets Whisper fall into
        // repeating itself for minutes.
        params.set_no_context(true);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        if let Some(path) = vad {
            params.set_vad_model_path(Some(path));
            params.enable_vad(true);
        }
        // Whisper needs at least a second of audio; padding affects inference, not timestamps.
        let mut window = samples[start..end].to_vec();
        window.resize(window.len().max(SAMPLE_RATE), 0.0);
        state.full(params, &window).map_err(whisper_error)?;
        let offset_ms = (start * 1000 / SAMPLE_RATE) as u64;
        for segment in state.as_iter() {
            // Lossy: a token boundary inside a multi-byte character costs one character.
            let Ok(text) = segment.to_str_lossy() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            let ms = |centiseconds: i64| offset_ms + u64::try_from(centiseconds).unwrap_or(0) * 10;
            segments.push(Segment {
                start_ms: ms(segment.start_timestamp()),
                end_ms: ms(segment.end_timestamp()),
                speaker: None,
                text: text.to_owned(),
            });
        }
        if language.is_none() && detected.is_none() {
            detected = whisper_rs::get_lang_str(state.full_lang_id_from_state()).map(str::to_owned);
        }
    }
    Ok(Transcript { segments, detected })
}

#[async_trait]
impl Provider for Whisper {
    fn id(&self) -> ProviderId {
        ProviderId::new(PROVIDER_ID)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn check_ready(&self, model: &str) -> Result<(), ProviderError> {
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
        let inner = self.inner.clone();
        let samples = to_float(&request.input.bytes);
        let language = language(request.languages);
        let stop = request.stop.clone();
        let transcript = tokio::task::spawn_blocking(move || {
            let _turn = lock(&inner.turn);
            // A pause that came while waiting for the turn.
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            let context = inner.context(model)?;
            let vad = inner.models.vad_path();
            let result = transcribe(
                &context,
                &samples,
                language.as_deref(),
                vad.as_deref(),
                &stop,
            );
            if let Some(loaded) = lock(&inner.loaded).as_mut() {
                loaded.last_used = Instant::now();
            }
            result.map(|t| (t, samples.len()))
        })
        .await
        .map_err(|e| ProviderError::Other(format!("Whisper stopped unexpectedly: {e}")))?;
        let (transcript, sample_count) = transcript?;
        let text = transcript
            .segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(ProcessOutput {
            text,
            segments: transcript.segments,
            detected_language: transcript.detected.map(LanguageCode::new),
            usage: Usage {
                audio_seconds: sample_count as f64 / SAMPLE_RATE as f64,
                ..Usage::default()
            },
            model_version: Some(model.file.trim_end_matches(".bin").to_owned()),
            keywords: Vec::new(),
            regions: Vec::new(),
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
        let catalog = scriptorium_catalog::Catalog::bundled();
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
            let scriptorium_catalog::LanguageSupport::List(codes) = &model.languages else {
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

    #[test]
    fn inference_threads_leave_room_and_stop_at_eight() {
        assert_eq!(threads(1), 1);
        assert_eq!(threads(64), 8);
    }

    /// A real transcription with a downloaded model; run with
    /// `WHISPER_SMOKE_MODEL=<path to ggml-*.bin> WHISPER_SMOKE_AUDIO=<16 kHz mono s16le file>
    /// cargo test -p scriptorium-offline smoke -- --ignored --nocapture`.
    #[test]
    #[ignore = "needs a model file and a recording"]
    fn smoke() {
        let model = std::env::var("WHISPER_SMOKE_MODEL").unwrap();
        let audio = std::fs::read(std::env::var("WHISPER_SMOKE_AUDIO").unwrap()).unwrap();
        let bytes = std::fs::read(model).unwrap();
        let mut params = WhisperContextParameters::default();
        params.use_gpu(false);
        let context = WhisperContext::new_from_buffer_with_params(&bytes, params).unwrap();
        let vad = std::env::var("WHISPER_SMOKE_VAD").ok();
        let transcript = transcribe(
            &context,
            &to_float(&audio),
            None,
            vad.as_deref(),
            &StopSignal::default(),
        )
        .unwrap();
        println!("{:?} {:#?}", transcript.detected, transcript.segments);
        assert!(!transcript.segments.is_empty());
    }
}
