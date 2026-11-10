//! The llama.cpp programs the offline engines built on it share: the OCR models (`vision.rs`)
//! and the language models (`llm.rs`). One host for both, so they take turns: each run takes
//! every core the app can spare, or the graphics processor, and two at once would only slow
//! both and could exhaust the graphics processor's memory. The graphics builds are asked once a
//! session which device they find, and a run that fails there where it does not on the
//! processor sends both engines back to the processor until Kalamu restarts (`device.rs`,
//! [`LlamaHost::settle`]).

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, SystemTime};

use kalamu_core::ProviderError;

use crate::device::{self, Backend, Graphics, Outcome, Placement, Verdict};
use crate::llama::{self, Llama, Run, RunError};
use crate::models::{ModelFile, ModelLease, ModelManager};

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// How long a loaded model or language data may sit unused before it is released, to give its
/// memory back.
pub(crate) const IDLE_UNLOAD: Duration = Duration::from_secs(180);

/// Starts, once (`started` remembers it), the task that runs `sweep` on `inner` every thirty
/// seconds to release what was left unused, until `inner` is dropped.
pub(crate) fn sweep_idle<I: Send + Sync + 'static>(
    inner: &Arc<I>,
    started: &Mutex<bool>,
    sweep: impl Fn(&I) + Send + 'static,
) {
    let mut started = lock(started);
    if *started {
        return;
    }
    *started = true;
    let inner = Arc::downgrade(inner);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let Some(inner) = inner.upgrade() else {
                return;
            };
            sweep(&inner);
        }
    });
}

/// The bundled llama.cpp programs and what is known of them this session.
pub struct LlamaHost {
    /// The processor build (Metal on a Mac), `None` when this copy of the app has none.
    llama: Option<Llama>,
    /// The Windows builds that read on the graphics processor (Vulkan, and OpenCL on ARM64),
    /// those shipped.
    graphics: Vec<Graphics>,
    /// The engine's build, asked once.
    version: OnceLock<String>,
    /// One run at a time, whichever engine asks.
    turn: Mutex<()>,
    /// Where a run's files (the page image, a text and its instructions) are written while it
    /// runs: a folder of the app's own, emptied when the host is made, so a crash or a kill
    /// leaves nothing of the researcher's material behind for long. Empty for the system's
    /// temporary folder.
    scratch: PathBuf,
}

/// What a run was, for the message after it failed on the graphics processor.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Work {
    Page,
    Request,
}

impl Work {
    fn again_on_processor(self, reason: &str) -> String {
        match self {
            Self::Page => format!(
                "The graphics processor could not read this page ({reason}); it is read again on the processor"
            ),
            Self::Request => format!(
                "The graphics processor could not run this request ({reason}); it runs again on the processor"
            ),
        }
    }
}

/// A run's fingerprint, from what it reads (the model, the page, the text, the schema): the
/// same request sent again has the same one, so its run on the processor can be told apart
/// from any other. Kept in memory for the session only.
pub(crate) fn fingerprint(parts: &[&[u8]]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for part in parts {
        hasher.write_usize(part.len());
        hasher.write(part);
    }
    hasher.finish()
}

impl LlamaHost {
    /// `llama` is the processor build; `graphics` its builds for the graphics processor, used
    /// when `use_graphics` (Settings) and they find one. The OpenCL build keeps the kernels it
    /// compiles for this device in `kernels`, so later runs skip compiling them. Runs write
    /// their files in `scratch`, a folder of the app's own, emptied here of what runs left when
    /// the app last ended without deleting them: made once, at start-up.
    pub fn new(
        llama: Option<Llama>,
        graphics: Vec<(Backend, Llama)>,
        use_graphics: bool,
        kernels: PathBuf,
        scratch: PathBuf,
    ) -> Self {
        llama::sweep_scratch(&scratch);
        let graphics = graphics
            .into_iter()
            .map(|(backend, program)| {
                let program = match backend {
                    Backend::OpenCl => {
                        program.with_env("GGML_OPENCL_KERNEL_CACHE_DIR", kernels.clone())
                    }
                    Backend::Vulkan => program,
                };
                Graphics::new(backend, program, use_graphics)
            })
            .collect();
        Self {
            llama,
            graphics,
            version: OnceLock::new(),
            turn: Mutex::new(()),
            scratch,
        }
    }

    /// A host with no programs, for a copy of the app built without them and for tests.
    pub fn none() -> Self {
        Self::new(None, Vec::new(), false, PathBuf::new(), PathBuf::new())
    }

    /// Where a run writes its files.
    pub(crate) fn scratch(&self) -> &Path {
        &self.scratch
    }

    /// Whether this copy of the app carries the multimodal program the OCR models run in.
    pub fn available(&self) -> bool {
        self.llama.is_some()
    }

    /// Whether it also carries the text program the language models read texts with.
    pub fn reads_text(&self) -> bool {
        self.llama.as_ref().is_some_and(Llama::reads_text)
    }

    /// The graphics processor a graphics build found, whether or not it is used; `None` when
    /// there is none, no graphics build, or it is still being asked.
    pub fn graphics_processor(&self) -> Option<String> {
        self.graphics
            .iter()
            .find_map(|graphics| graphics.device_if_known().flatten())
            .map(|device| device.name.clone())
    }

    /// Asks the graphics builds for their devices now, so the first run need not wait for
    /// them. Takes up to half a minute each when a driver hangs: run it off the interface's
    /// thread.
    pub fn find_graphics_processor(&self) {
        for graphics in &self.graphics {
            graphics.device();
        }
    }

    /// Settings › Offline: run on the graphics processor when there is one.
    pub fn use_graphics_processor(&self, wanted: bool) {
        for graphics in &self.graphics {
            graphics.set_wanted(wanted);
        }
    }

    pub(crate) fn version(&self) -> &str {
        self.version.get_or_init(|| {
            self.llama
                .as_ref()
                .and_then(Llama::version)
                .unwrap_or_else(|| "unknown".to_owned())
        })
    }

    /// Where a model's next run goes: the first of the graphics builds it runs correctly with
    /// (`backends`) that found a device, else the processor. `wait`: ask the graphics builds
    /// for their devices if they were not asked yet, rather than running on the processor
    /// meanwhile.
    pub(crate) fn placement(&self, backends: &[Backend], wait: bool) -> Placement {
        self.graphics
            .iter()
            .filter(|graphics| backends.contains(&graphics.backend()))
            .find_map(|graphics| {
                let device = if wait {
                    graphics.device()
                } else {
                    graphics.device_if_known().flatten()
                };
                graphics.placement(device)
            })
            .unwrap_or_else(Placement::built_in)
    }

    pub(crate) fn graphics(&self, placement: &Placement) -> Option<&Graphics> {
        let backend = placement.backend()?;
        self.graphics.iter().find(|g| g.backend() == backend)
    }

    /// The program that runs there.
    pub(crate) fn program(&self, placement: &Placement) -> Result<&Llama, ProviderError> {
        match self.graphics(placement) {
            Some(graphics) => Ok(graphics.program()),
            None => self.llama(),
        }
    }

    pub(crate) fn llama(&self) -> Result<&Llama, ProviderError> {
        self.llama.as_ref().ok_or_else(|| {
            ProviderError::Unsupported(
                "This copy of Kalamu has no engine for offline vision and language models".into(),
            )
        })
    }

    /// Held for the length of one run.
    pub(crate) fn turn(&self) -> MutexGuard<'_, ()> {
        lock(&self.turn)
    }

    /// What a run that went to `placement` gives, once what it tells of the graphics builds is
    /// applied ([`device::verdict`]): a failure on the graphics processor comes back as
    /// transient, so the engine sends the request again and it runs on the processor; a run
    /// there of a request the device is suspected for settles whose fault it was. Called while
    /// the turn is held, so no other run comes between.
    pub(crate) fn settle(
        &self,
        placement: &Placement,
        request: u64,
        ran: Result<Run, RunError>,
        work: Work,
    ) -> Result<Run, ProviderError> {
        let outcome = match &ran {
            Ok(_) => Outcome::Answered,
            Err(RunError::Stopped) => Outcome::Stopped,
            Err(RunError::Hung(_)) => Outcome::Hung,
            Err(RunError::Failed(_)) => Outcome::Failed,
            // Not the device's doing, nor the request's.
            Err(RunError::Missing(_)) => return ran.map_err(Into::into),
        };
        let reason = match &ran {
            Err(e) => match ProviderError::from(e.clone()) {
                ProviderError::Other(reason) => reason,
                other => other.to_string(),
            },
            Ok(_) => String::new(),
        };
        match self.graphics(placement) {
            Some(graphics) => {
                let verdict = device::verdict(true, false, outcome);
                graphics.apply(verdict, request, &reason);
                if matches!(verdict, Verdict::Suspect | Verdict::GiveUp) {
                    return Err(ProviderError::Transient(work.again_on_processor(&reason)));
                }
            }
            None => {
                for graphics in &self.graphics {
                    let retry = graphics.suspected() == Some(request);
                    graphics.apply(device::verdict(false, retry, outcome), request, &reason);
                }
            }
        }
        ran.map_err(Into::into)
    }
}

/// Model files whose hash was checked in this session, with the size and time they had then: a
/// gigabyte is read once, not before every run.
#[derive(Default)]
pub(crate) struct Checked(Mutex<BTreeMap<&'static str, (u64, Option<SystemTime>)>>);

impl Checked {
    /// The model's file and the file it pairs with, leased against removal and checked against
    /// their hashes, once per session unless they changed.
    pub(crate) fn leases(
        &self,
        models: &ModelManager,
        model: &'static ModelFile,
    ) -> Result<Vec<ModelLease>, ProviderError> {
        let mut leases = Vec::new();
        for file in std::iter::once(model).chain(model.pair) {
            let lease = models.lease(file)?;
            let meta = std::fs::metadata(lease.path())
                .map_err(|e| ProviderError::Other(format!("Could not read {}: {e}", file.file)))?;
            let seen = (meta.len(), meta.modified().ok());
            if lock(&self.0).get(file.id) != Some(&seen) {
                lease.verify()?;
                lock(&self.0).insert(file.id, seen);
            }
            leases.push(lease);
        }
        Ok(leases)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::llama::Device;

    fn answered() -> Result<Run, RunError> {
        Ok(Run {
            text: "{}".into(),
            generated: Some(2),
            read: Some(10),
            elapsed: Duration::from_secs(1),
        })
    }

    fn failed() -> Result<Run, RunError> {
        Err(RunError::Failed("failed to parse grammar".into()))
    }

    fn host() -> LlamaHost {
        LlamaHost::new(
            Some(Llama::at(PathBuf::from("nowhere"))),
            vec![(Backend::Vulkan, Llama::at(PathBuf::from("nowhere")))],
            true,
            PathBuf::new(),
            PathBuf::new(),
        )
    }

    fn device(name: &str) -> Device {
        Device {
            id: "Vulkan0".into(),
            name: name.into(),
            memory_mib: 12288,
        }
    }

    #[test]
    fn a_request_that_fails_everywhere_keeps_the_graphics_processor() {
        let host = host();
        let device = device("NVIDIA GeForce RTX 3060");
        let gpu = Placement::Vulkan(device.clone());
        let cpu = Placement::built_in();
        let graphics = host.graphics(&gpu).unwrap();
        let on_gpu = || graphics.placement(Some(&device)).is_some();

        // A failure there sends the request back, to run on the processor.
        let first = host.settle(&gpu, 1, failed(), Work::Request);
        assert!(
            matches!(first, Err(ProviderError::Transient(_))),
            "{first:?}"
        );
        assert!(!on_gpu());
        // Another request answered on the processor meanwhile tells nothing.
        assert!(host.settle(&cpu, 2, answered(), Work::Request).is_ok());
        assert!(!on_gpu());
        // The same request fails on the processor too: its own fault, the device comes back.
        let again = host.settle(&cpu, 1, failed(), Work::Request);
        assert!(matches!(again, Err(ProviderError::Other(_))), "{again:?}");
        assert!(on_gpu());

        // A page the processor reads where the device failed: given up for the session.
        let page = host.settle(&gpu, 3, failed(), Work::Page);
        assert!(matches!(page, Err(ProviderError::Transient(_))), "{page:?}");
        assert!(host.settle(&cpu, 3, answered(), Work::Page).is_ok());
        assert!(!on_gpu());
        assert_eq!(graphics.suspected(), None);
    }

    #[test]
    fn a_hang_on_the_graphics_processor_gives_it_up_at_once_and_a_stop_never_does() {
        let host = host();
        let device = device("AMD Radeon RX 7800 XT");
        let gpu = Placement::Vulkan(device.clone());
        let graphics = host.graphics(&gpu).unwrap();
        let stopped = host.settle(&gpu, 1, Err(RunError::Stopped), Work::Page);
        assert_eq!(stopped.unwrap_err(), ProviderError::Interrupted);
        assert!(graphics.placement(Some(&device)).is_some());
        let hung = host.settle(
            &gpu,
            1,
            Err(RunError::Hung(Duration::from_secs(600))),
            Work::Page,
        );
        assert!(matches!(hung, Err(ProviderError::Transient(_))), "{hung:?}");
        assert!(graphics.placement(Some(&device)).is_none());
        assert_eq!(graphics.suspected(), None);
    }

    #[test]
    fn the_same_request_has_the_same_fingerprint() {
        assert_eq!(
            fingerprint(&[b"glm-ocr", b"page"]),
            fingerprint(&[b"glm-ocr", b"page"])
        );
        assert_ne!(
            fingerprint(&[b"glm-ocr", b"page"]),
            fingerprint(&[b"glm-ocrp", b"age"])
        );
    }
}
