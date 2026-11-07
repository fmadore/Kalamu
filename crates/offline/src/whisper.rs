//! Whisper on this PC, through whisper.cpp: nothing leaves the computer and no key is needed.
//!
//! Model loading, CPU gate and decoding settings follow Live Translation's `whisper` module.
//! What differs is the shape of the work: a whole recording piece at once rather than a live
//! stream, so accuracy is preferred over latency (beam search), and the piece is transcribed
//! in windows of about two minutes, cut at the quietest moment nearby. Between windows the job's
//! stop signal is checked, so pausing a long transcription takes effect within one window
//! without whisper.cpp's abort callback, which whisper-rs 0.16 only offers unsafely.
//!
//! Each window's language, unless the researcher chose one, is detected on its loudest thirty
//! seconds rather than on its first, which may be silence or music. Lines are kept to subtitle
//! length, cut between words at the times whisper.cpp gives each token; the text joins them
//! into sentences. The names and words the researcher listed are Whisper's initial prompt, and
//! a model that can translate writes English when asked. Runs of the same words, Whisper's
//! known failure on silence and noise, are cut back, as are the subtitle credits it learnt from
//! its training data. On a Mac, the model runs on the graphics processor (Metal).

use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, LanguageChoice, LanguageCode, PCM_16K_MONO, ProcessOutput, ProcessRequest, Provider,
    ProviderError, ProviderId, RequestSetting, Segment, StopSignal, Task, Tuning, Usage,
};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

use crate::cpu::{self, DEVICE, threads};
use crate::models::{self, ModelFile, ModelLease, ModelManager};

mod audio;
mod clean;

use audio::{WINDOW, loudest_stretch, to_float, windows};
use clean::{initial_prompt, sentences, without_loops};

pub const PROVIDER_ID: &str = "whisper";

const SAMPLE_RATE: usize = 16_000;
/// whisper.cpp's own command-line default for file transcription.
const BEAM_SIZE: i32 = 5;
/// How much of a window its language is detected on: what Whisper's encoder reads at once.
const DETECT: usize = 30 * SAMPLE_RATE;
/// Longest line, as the subtitle export's cues, so a cue's times are Whisper's own.
const MAX_LINE_CHARS: i32 = 84;
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
        params.use_gpu(DEVICE == "metal");
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
/// detection, again for every window, as mixed recordings need.
fn language(choice: &LanguageChoice) -> Option<String> {
    match choice.codes() {
        [one] if whisper_rs::get_lang_id(one.as_str()).is_some() => Some(one.as_str().to_owned()),
        _ => None,
    }
}

fn whisper_error(e: impl std::fmt::Display) -> ProviderError {
    ProviderError::Other(format!("Whisper failed: {e}"))
}

/// What provenance records of the voice-activity model: whisper.cpp's `vad` switch, and which
/// model it used.
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

struct Transcript {
    segments: Vec<Segment>,
    detected: Option<String>,
}

/// What a transcription asks of whisper.cpp beside the audio.
#[derive(Debug, Default)]
struct Decode<'a> {
    /// The language chosen; detected window by window when `None`.
    language: Option<&'a str>,
    /// The voice-activity model, when downloaded: silences are skipped.
    vad: Option<&'a str>,
    /// The names and words to spell, as the initial prompt.
    prompt: Option<String>,
    /// Write English rather than the language spoken.
    translate: bool,
}

fn transcribe(
    context: &WhisperContext,
    samples: &[f32],
    decode: &Decode<'_>,
    stop: &StopSignal,
) -> Result<Transcript, ProviderError> {
    let mut state = context.create_state().map_err(whisper_error)?;
    let threads = usize::try_from(threads(
        std::thread::available_parallelism().map_or(2, usize::from),
    ))
    .unwrap_or(1);
    let mut segments = Vec::new();
    // How long each language detected was spoken, to name the recording's.
    let mut heard: Vec<(&'static str, usize)> = Vec::new();
    for (start, end) in windows(samples) {
        if stop.is_stopped() {
            return Err(ProviderError::Interrupted);
        }
        // Whisper needs at least a second of audio; padding affects inference, not timestamps.
        let mut window = samples[start..end].to_vec();
        window.resize(window.len().max(SAMPLE_RATE), 0.0);
        let detected = match decode.language {
            Some(_) => None,
            None => detect_language(&mut state, &window, threads)?,
        };
        if let Some(lang) = detected {
            match heard.iter_mut().find(|(l, _)| *l == lang) {
                Some((_, length)) => *length += end - start,
                None => heard.push((lang, end - start)),
            }
        }
        let language = decode.language.or(detected);
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: BEAM_SIZE,
            patience: -1.0,
        });
        params.set_language(language);
        params.set_translate(decode.translate);
        params.set_n_threads(i32::try_from(threads).unwrap_or(1));
        // Each window starts fresh: carrying text over is what lets Whisper fall into
        // repeating itself for minutes. The names to spell are given to every window.
        params.set_no_context(true);
        if let Some(prompt) = &decode.prompt {
            params.set_initial_prompt(prompt);
        }
        // Lines of subtitle length, cut between words at the tokens' own times.
        params.set_token_timestamps(true);
        params.set_split_on_word(true);
        params.set_max_len(MAX_LINE_CHARS);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        if let Some(path) = decode.vad {
            params.set_vad_model_path(Some(path));
            params.enable_vad(true);
        }
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

/// The language spoken in a window, detected on its loudest stretch, where speech most likely
/// is; `None` when Whisper cannot tell.
fn detect_language(
    state: &mut WhisperState,
    window: &[f32],
    threads: usize,
) -> Result<Option<&'static str>, ProviderError> {
    let (from, to) = loudest_stretch(window, DETECT);
    let mut stretch = window[from..to].to_vec();
    stretch.resize(stretch.len().max(SAMPLE_RATE), 0.0);
    state.pcm_to_mel(&stretch, threads).map_err(whisper_error)?;
    let (id, _) = state.lang_detect(0, threads).map_err(whisper_error)?;
    Ok(whisper_rs::get_lang_str(id))
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

    /// The fixed settings, and whether whisper.cpp skips the silences with the voice-activity
    /// model, which it does when that model is downloaded: decided on this PC, so a pipeline's
    /// record leaves it out and each result's provenance says it.
    fn sent_settings(&self, request: &ProcessRequest<'_>, _batched: bool) -> Vec<RequestSetting> {
        let mut settings = self.request_settings(request.task, request.model, request.tuning);
        if request.task == Task::AudioTranscription {
            settings.extend(voice_detection(self.inner.models.vad_path().is_some()));
            settings.push(RequestSetting::new("device", DEVICE));
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
        let inner = self.inner.clone();
        let samples = to_float(&request.input.bytes);
        let language = language(request.languages);
        let prompt = initial_prompt(&request.options.vocabulary_terms());
        let translate = request.options.translate;
        let stop = request.stop.clone();
        let transcript = tokio::task::spawn_blocking(move || {
            let _turn = lock(&inner.turn);
            // A pause that came while waiting for the turn.
            if stop.is_stopped() {
                return Err(ProviderError::Interrupted);
            }
            let context = inner.context(model)?;
            let vad = inner.models.vad_path();
            let decode = Decode {
                language: language.as_deref(),
                vad: vad.as_deref(),
                prompt,
                translate,
            };
            let result = transcribe(&context, &samples, &decode, &stop);
            if let Some(loaded) = lock(&inner.loaded).as_mut() {
                loaded.last_used = Instant::now();
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

    /// A real transcription with a downloaded model; run with
    /// `WHISPER_SMOKE_MODEL=<path to ggml-*.bin> WHISPER_SMOKE_AUDIO=<16 kHz mono s16le file>
    /// cargo test -p kalamu-offline smoke -- --ignored --nocapture`.
    #[test]
    #[ignore = "needs a model file and a recording"]
    fn smoke() {
        let model = std::env::var("WHISPER_SMOKE_MODEL").unwrap();
        let audio = std::fs::read(std::env::var("WHISPER_SMOKE_AUDIO").unwrap()).unwrap();
        let bytes = std::fs::read(model).unwrap();
        let mut params = WhisperContextParameters::default();
        params.use_gpu(DEVICE == "metal");
        let context = WhisperContext::new_from_buffer_with_params(&bytes, params).unwrap();
        let vad = std::env::var("WHISPER_SMOKE_VAD").ok();
        let decode = Decode {
            vad: vad.as_deref(),
            prompt: std::env::var("WHISPER_SMOKE_NAMES").ok(),
            translate: std::env::var("WHISPER_SMOKE_TRANSLATE").is_ok(),
            ..Decode::default()
        };
        let transcript =
            transcribe(&context, &to_float(&audio), &decode, &StopSignal::default()).unwrap();
        println!("{:?} {:#?}", transcript.detected, transcript.segments);
        assert!(!transcript.segments.is_empty());
    }
}
