//! The llama.cpp programs the offline engines built on it share: the OCR models (`vision.rs`)
//! and the language models (`llm.rs`). One host for both, so they take turns: each run takes
//! every core the app can spare, or the graphics processor, and two at once would only slow
//! both and could exhaust the graphics processor's memory. The graphics builds are asked once a
//! session which device they find, and a run that fails there sends both engines back to the
//! processor until Kalamu restarts (`device.rs`).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::SystemTime;

use kalamu_core::ProviderError;

use crate::device::{Backend, Graphics, Placement};
use crate::llama::Llama;
use crate::models::{ModelFile, ModelLease, ModelManager};

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
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
}

impl LlamaHost {
    /// `llama` is the processor build; `graphics` its builds for the graphics processor, used
    /// when `use_graphics` (Settings) and they find one. The OpenCL build keeps the kernels it
    /// compiles for this device in `kernels`, so later runs skip compiling them.
    pub fn new(
        llama: Option<Llama>,
        graphics: Vec<(Backend, Llama)>,
        use_graphics: bool,
        kernels: PathBuf,
    ) -> Self {
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
        }
    }

    /// A host with no programs, for a copy of the app built without them and for tests.
    pub fn none() -> Self {
        Self::new(None, Vec::new(), false, PathBuf::new())
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
