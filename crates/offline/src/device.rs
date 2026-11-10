//! Where the OCR models' engine reads a page: the processor, the graphics processor through
//! Metal on a Mac (the processor build already uses it), or, on Windows, the graphics processor
//! with a second build of llama.cpp's program: through Vulkan, or through OpenCL on Snapdragon's
//! Adreno, where Vulkan reads wrongly (docs/decisions/2026-11-06-ocr-graphics-processor.md).
//!
//! Each graphics build is asked once a session which devices it finds. It is used only when it
//! starts (it needs the loader the graphics driver installs) and finds a device it is known to
//! read correctly on, for a model measured correct there, when the researcher has not turned it
//! off in Settings, and until it is shown to fail where the processor does not ([`verdict`]):
//! the session then reads on the processor. The device chosen is passed to the program, so the
//! one recorded in provenance is the one that read the page.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::cpu::DEVICE;
use crate::host::lock;
use crate::llama::{Device, Llama};

/// Below this, a graphics processor cannot hold a model, its image encoder and a page's context
/// (LightOnOCR-3 at Q8_0 needs about two and a half gigabytes).
const MIN_MEMORY_MIB: u64 = 2048;

/// A build of the program for a graphics processor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// `llama-mtmd-cli-vulkan`, Windows x64 and ARM64: the model and its image encoder on the
    /// device.
    Vulkan,
    /// `llama-mtmd-cli-opencl`, Windows ARM64: the image encoder on Snapdragon's Adreno, the
    /// language model on the processor, which writes faster there.
    OpenCl,
}

impl Backend {
    /// The device to read on, of those this build lists.
    fn choose(self, devices: &[Device]) -> Option<&Device> {
        match self {
            Self::Vulkan => choose(devices),
            Self::OpenCl => choose_adreno(devices),
        }
    }
}

/// Where a page is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placement {
    /// The processor build.
    Processor,
    /// The processor build on a Mac, which runs on the graphics processor through Metal.
    Metal,
    /// The Vulkan build, on this device.
    Vulkan(Device),
    /// The OpenCL build: the image encoder on this device, the language model on the processor.
    OpenCl(Device),
}

impl Placement {
    /// Where the processor build runs: Metal on a Mac, the processor elsewhere.
    pub(crate) fn built_in() -> Self {
        if DEVICE == "metal" {
            Self::Metal
        } else {
            Self::Processor
        }
    }

    /// As provenance records it: `cpu`, `metal`, `vulkan` or `opencl`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Processor => "cpu",
            Self::Metal => "metal",
            Self::Vulkan(_) => "vulkan",
            Self::OpenCl(_) => "opencl",
        }
    }

    /// The graphics processor's name, for Vulkan and OpenCL.
    pub fn device_name(&self) -> Option<&str> {
        match self {
            Self::Vulkan(device) | Self::OpenCl(device) => Some(&device.name),
            _ => None,
        }
    }

    /// Whether only the image encoder runs on the graphics processor.
    pub fn encoder_only(&self) -> bool {
        matches!(self, Self::OpenCl(_))
    }

    /// The build that reads there, `None` for the processor build.
    pub(crate) fn backend(&self) -> Option<Backend> {
        match self {
            Self::Vulkan(_) => Some(Backend::Vulkan),
            Self::OpenCl(_) => Some(Backend::OpenCl),
            _ => None,
        }
    }

    /// The arguments that send the model and its image encoder to this device, given after the
    /// fixed ones: `-ngl 0` keeps the language model's layers on the processor, while the image
    /// encoder follows `--device`.
    pub(crate) fn arguments(&self) -> Vec<String> {
        match self {
            Self::Vulkan(device) => vec!["--device".to_owned(), device.id.clone()],
            Self::OpenCl(device) => vec![
                "--device".to_owned(),
                device.id.clone(),
                "-ngl".to_owned(),
                "0".to_owned(),
            ],
            _ => Vec::new(),
        }
    }

    /// Speeds are kept apart for each device: `cpu`, `metal`, `vulkan:<name>`, `opencl:<name>`.
    fn speed_key(&self) -> String {
        match self {
            Self::Vulkan(device) | Self::OpenCl(device) => {
                format!("{}:{}", self.kind(), device.name)
            }
            other => other.kind().to_owned(),
        }
    }
}

/// The graphics processor to read on, of those the Vulkan build lists: a discrete one before an
/// integrated one (llama.cpp's own preference), a translation layer last; software renderers,
/// devices too small for the models and those known to read wrongly never; among equals, the
/// first listed. `None` when none fits.
pub(crate) fn choose(devices: &[Device]) -> Option<&Device> {
    devices
        .iter()
        .filter(|d| usable(d))
        .min_by_key(|d| rank(&d.name))
}

/// Devices measured to read wrongly through Vulkan, by a part of their name. A wrong reading ends
/// normally, so the fallback after a failure would not catch it: these never get a page.
/// Snapdragon's Adreno (X1-85, driver 31.0.152.1, llama.cpp b12110): encoding the sample page
/// took six times as long as on the processor, and both models then wrote nonsense
/// (docs/decisions/2026-11-06-ocr-graphics-processor.md).
const READS_WRONGLY: [&str; 1] = ["adreno"];

fn usable(device: &Device) -> bool {
    let name = device.name.to_ascii_lowercase();
    let software = ["basic render", "llvmpipe", "lavapipe", "swiftshader"]
        .iter()
        .any(|s| name.contains(s));
    let wrong = READS_WRONGLY.iter().any(|s| name.contains(s));
    !software && !wrong && enough_memory(device)
}

fn enough_memory(device: &Device) -> bool {
    device.memory_mib == 0 || device.memory_mib >= MIN_MEMORY_MIB
}

/// Through OpenCL, Snapdragon's Adreno only: the one device it was measured correct on, with
/// Qualcomm's own kernels. Elsewhere the Vulkan build covers the graphics processor.
fn choose_adreno(devices: &[Device]) -> Option<&Device> {
    devices
        .iter()
        .find(|d| d.name.to_ascii_lowercase().contains("adreno") && enough_memory(d))
}

/// 0 for a name that reads as a discrete graphics card, 2 for Microsoft's Direct3D 12 layer
/// (Dozen), which runs Vulkan over another driver, 1 otherwise. The listing does not say whether
/// a device is integrated, so the names tell: every NVIDIA card is discrete, AMD's are the
/// Radeon RX and Pro, Intel's the Arc A and B series (Intel's integrated graphics are also
/// called Arc, without a model letter).
fn rank(name: &str) -> u8 {
    let name = name.to_ascii_lowercase();
    if name.contains("direct3d12") {
        return 2;
    }
    let intel_card = ["arc(tm) ", "arc "].iter().any(|prefix| {
        name.split_once(prefix).is_some_and(|(_, model)| {
            let mut chars = model.chars();
            matches!(chars.next(), Some('a' | 'b'))
                && chars.next().is_some_and(|c| c.is_ascii_digit())
        })
    });
    let discrete = [
        "nvidia",
        "geforce",
        "quadro",
        "radeon rx",
        "radeon pro",
        "radeon vii",
    ]
    .iter()
    .any(|s| name.contains(s));
    if discrete || intel_card { 0 } else { 1 }
}

/// How a run ended, as far as the device is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// It answered.
    Answered,
    /// The job was paused or cancelled: it says nothing either way.
    Stopped,
    /// It wrote nothing for the watchdog's limit and was ended.
    Hung,
    /// It could not start, or ended with an error.
    Failed,
}

/// What a run tells of a graphics build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Nothing new.
    Unchanged,
    /// A request failed there: it runs again on the processor, as does everything else until
    /// that run tells whether the device or the request was the cause.
    Suspect,
    /// The device was the cause: the rest of the session runs on the processor.
    GiveUp,
    /// The request was the cause: the device is used again.
    Keep,
}

/// The rule for giving a graphics processor up. The fallback exists because Vulkan crashed, ran
/// out of memory or failed on some drivers, which the processor build does not; but a request
/// can fail by itself (a schema llama.cpp cannot turn into a grammar, an image its decoder
/// refuses, a command line past Windows' limit), and giving the device up for that sent every
/// later page to the processor for nothing, where the same request then failed again. So:
///
/// - a run on the graphics processor that answers, or is stopped, changes nothing;
/// - one that hangs is the device's fault, as a driver that stopped answering is: given up at
///   once (the processor has never been seen to hang on a page);
/// - one that fails makes the device a suspect: the request runs again on the processor
///   (`retry_of_suspect` then holds for it). If it answers there, the device was the cause and
///   is given up, as before; if it fails or hangs there too, the request was, and the device
///   is used again. A stop there leaves the suspicion for the next try.
pub(crate) fn verdict(on_graphics: bool, retry_of_suspect: bool, outcome: Outcome) -> Verdict {
    match (on_graphics, outcome) {
        (_, Outcome::Stopped) => Verdict::Unchanged,
        (true, Outcome::Answered) => Verdict::Unchanged,
        (true, Outcome::Hung) => Verdict::GiveUp,
        (true, Outcome::Failed) => Verdict::Suspect,
        (false, _) if !retry_of_suspect => Verdict::Unchanged,
        (false, Outcome::Answered) => Verdict::GiveUp,
        (false, Outcome::Hung | Outcome::Failed) => Verdict::Keep,
    }
}

/// A graphics build and what is known of it this session.
pub(crate) struct Graphics {
    backend: Backend,
    program: Llama,
    /// Settings › Offline: use the graphics processor when there is one.
    wanted: AtomicBool,
    /// The device chosen, asked once.
    found: OnceLock<Option<Device>>,
    /// A page failed on it: the rest of the session reads on the processor.
    failed: AtomicBool,
    /// A request that failed on it, by its fingerprint, with why, while it runs again on the
    /// processor to tell whose fault it was ([`verdict`]). Meanwhile everything runs there.
    suspect: Mutex<Option<(u64, String)>>,
}

impl Graphics {
    pub(crate) fn new(backend: Backend, program: Llama, wanted: bool) -> Self {
        Self {
            backend,
            program,
            wanted: AtomicBool::new(wanted),
            found: OnceLock::new(),
            failed: AtomicBool::new(false),
            suspect: Mutex::new(None),
        }
    }

    pub(crate) fn backend(&self) -> Backend {
        self.backend
    }

    pub(crate) fn program(&self) -> &Llama {
        &self.program
    }

    pub(crate) fn set_wanted(&self, wanted: bool) {
        self.wanted.store(wanted, Ordering::Relaxed);
    }

    /// Applies what a run told of this build: `request` is the run's fingerprint, `reason` why
    /// it failed, when it did.
    pub(crate) fn apply(&self, verdict: Verdict, request: u64, reason: &str) {
        match verdict {
            Verdict::Unchanged => {}
            Verdict::Suspect => {
                tracing::warn!(
                    reason,
                    backend = ?self.backend,
                    "a request failed on the graphics processor: it runs again on the processor, which tells whether the device or the request was the cause"
                );
                *lock(&self.suspect) = Some((request, reason.to_owned()));
            }
            Verdict::GiveUp => {
                let first = lock(&self.suspect)
                    .take()
                    .map_or_else(|| reason.to_owned(), |(_, first)| first);
                self.give_up(&first);
            }
            Verdict::Keep => {
                if lock(&self.suspect).take().is_some() {
                    tracing::info!(
                        backend = ?self.backend,
                        "the request failed on the processor too: the graphics processor is used again"
                    );
                }
            }
        }
    }

    /// The fingerprint of the request this build is suspected for, if any.
    pub(crate) fn suspected(&self) -> Option<u64> {
        lock(&self.suspect).as_ref().map(|(request, _)| *request)
    }

    /// After a run showed the graphics processor to be the cause.
    pub(crate) fn give_up(&self, reason: &str) {
        if !self.failed.swap(true, Ordering::Relaxed) {
            tracing::warn!(
                reason,
                backend = ?self.backend,
                "the graphics processor failed where the processor did not: the offline models run on the processor until Kalamu restarts"
            );
        }
    }

    /// The graphics processor found, asked the first time and then remembered; callers wait
    /// while it is asked (at most `LIST_DEVICES_LIMIT`).
    pub(crate) fn device(&self) -> Option<&Device> {
        self.found
            .get_or_init(|| match self.program.devices() {
                Ok(devices) => {
                    let chosen = self.backend.choose(&devices).cloned();
                    tracing::info!(
                        backend = ?self.backend,
                        ?devices,
                        ?chosen,
                        "graphics processors for the OCR models"
                    );
                    chosen
                }
                Err(error) => {
                    tracing::info!(backend = ?self.backend, %error, "no graphics processor for the OCR models");
                    None
                }
            })
            .as_ref()
    }

    /// The device found, without waiting: `None` while it is still being asked.
    pub(crate) fn device_if_known(&self) -> Option<Option<&Device>> {
        self.found.get().map(Option::as_ref)
    }

    /// Where the next page goes: the graphics processor when it is wanted, found, has not
    /// failed and is not suspected; `None` for the processor build.
    pub(crate) fn placement(&self, device: Option<&Device>) -> Option<Placement> {
        let usable = self.wanted.load(Ordering::Relaxed)
            && !self.failed.load(Ordering::Relaxed)
            && self.suspected().is_none();
        device.filter(|_| usable).map(|d| match self.backend {
            Backend::Vulkan => Placement::Vulkan(d.clone()),
            Backend::OpenCl => Placement::OpenCl(d.clone()),
        })
    }
}

/// Seconds a page took on this computer, per device and model, kept in the models' folder, each
/// a running average weighted to the latest pages.
pub(crate) struct Speeds {
    path: PathBuf,
    by_device: BTreeMap<String, BTreeMap<String, f32>>,
}

impl Speeds {
    /// Reads the file, which before the Vulkan build held one map of models for the only
    /// device there was (the processor, or Metal on a Mac).
    pub(crate) fn load(path: PathBuf) -> Self {
        let by_device = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .map(|value| {
                let flat = value
                    .as_object()
                    .is_some_and(|map| map.values().all(serde_json::Value::is_number));
                if flat {
                    let models = serde_json::from_value(value).unwrap_or_default();
                    BTreeMap::from([(Placement::built_in().speed_key(), models)])
                } else {
                    serde_json::from_value(value).unwrap_or_default()
                }
            })
            .unwrap_or_default();
        Self { path, by_device }
    }

    pub(crate) fn record(&mut self, placement: &Placement, model: &str, seconds: f32) {
        let models = self.by_device.entry(placement.speed_key()).or_default();
        let average = models
            .get(model)
            .map_or(seconds, |previous| previous * 0.7 + seconds * 0.3);
        models.insert(model.to_owned(), average);
        if let Ok(text) = serde_json::to_string(&self.by_device) {
            let _ = std::fs::write(&self.path, text);
        }
    }

    pub(crate) fn on(&self, placement: &Placement) -> BTreeMap<String, f32> {
        self.by_device
            .get(&placement.speed_key())
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn device(id: &str, name: &str, memory_mib: u64) -> Device {
        Device {
            id: id.into(),
            name: name.into(),
            memory_mib,
        }
    }

    #[test]
    fn a_graphics_card_comes_before_integrated_graphics() {
        let laptop = [
            device("Vulkan0", "Intel(R) Arc(TM) Graphics", 16384),
            device("Vulkan1", "NVIDIA GeForce RTX 4060 Laptop GPU", 8188),
        ];
        assert_eq!(choose(&laptop).unwrap().id, "Vulkan1");
        let intel = [
            device("Vulkan0", "Intel(R) UHD Graphics 770", 8192),
            device("Vulkan1", "Intel(R) Arc(TM) A770 Graphics", 16256),
        ];
        assert_eq!(choose(&intel).unwrap().id, "Vulkan1");
        let amd = [
            device("Vulkan0", "AMD Radeon(TM) Graphics", 4096),
            device("Vulkan1", "AMD Radeon RX 7800 XT", 16368),
        ];
        assert_eq!(choose(&amd).unwrap().id, "Vulkan1");
        // Lunar Lake's integrated graphics are an Arc with a number, not a letter.
        assert_eq!(rank("Intel(R) Arc(TM) 140V GPU (16GB)"), 1);
        assert_eq!(rank("Intel(R) Arc(TM) B580 Graphics"), 0);
    }

    #[test]
    fn integrated_graphics_are_used_when_alone() {
        let intel = [device("Vulkan0", "Intel(R) Arc(TM) Graphics", 16384)];
        assert_eq!(choose(&intel).unwrap().id, "Vulkan0");
    }

    #[test]
    fn snapdragon_uses_opencl_not_vulkan() {
        // As each build's `--list-devices` prints it on a Snapdragon X Elite.
        let vulkan = [device("Vulkan0", "Qualcomm(R) Adreno(TM) X1-85 GPU", 12086)];
        assert_eq!(Backend::Vulkan.choose(&vulkan), None);
        let opencl = [device(
            "GPUOpenCL",
            "Qualcomm(R) Adreno(TM) X1-85 GPU",
            7990,
        )];
        assert_eq!(Backend::OpenCl.choose(&opencl).unwrap().id, "GPUOpenCL");
        // OpenCL elsewhere is not measured: Vulkan covers it.
        let nvidia = [device("GPUOpenCL", "NVIDIA GeForce RTX 3060", 12288)];
        assert_eq!(Backend::OpenCl.choose(&nvidia), None);
    }

    #[test]
    fn opencl_keeps_the_language_model_on_the_processor() {
        let gpu = Graphics::new(Backend::OpenCl, Llama::at(PathBuf::from("nowhere")), true);
        let adreno = device("GPUOpenCL", "Qualcomm(R) Adreno(TM) X1-85 GPU", 7990);
        let placement = gpu.placement(Some(&adreno)).unwrap();
        assert_eq!(placement.kind(), "opencl");
        assert!(placement.encoder_only());
        assert_eq!(placement.backend(), Some(Backend::OpenCl));
        assert_eq!(
            placement.arguments(),
            ["--device", "GPUOpenCL", "-ngl", "0"]
        );
        assert_eq!(
            placement.speed_key(),
            "opencl:Qualcomm(R) Adreno(TM) X1-85 GPU"
        );
    }

    #[test]
    fn software_and_small_devices_are_left_out() {
        assert_eq!(
            choose(&[
                device("Vulkan0", "Microsoft Basic Render Driver", 8192),
                device("Vulkan1", "llvmpipe (LLVM 17.0.6, 256 bits)", 8192),
                device("Vulkan2", "NVIDIA GeForce GT 710", 1024),
            ]),
            None
        );
        // A translation layer only when nothing else is there.
        let layered = [
            device(
                "Vulkan0",
                "Microsoft Direct3D12 (AMD Radeon(TM) Graphics)",
                0,
            ),
            device("Vulkan1", "AMD Radeon(TM) Graphics", 4096),
        ];
        assert_eq!(choose(&layered).unwrap().id, "Vulkan1");
        assert!(choose(&[]).is_none());
    }

    #[test]
    fn the_graphics_processor_is_used_only_when_wanted_and_working() {
        let gpu = Graphics::new(Backend::Vulkan, Llama::at(PathBuf::from("nowhere")), true);
        let found = device("Vulkan0", "NVIDIA GeForce RTX 3060", 12288);
        let on_gpu = gpu.placement(Some(&found)).unwrap();
        assert_eq!(on_gpu.kind(), "vulkan");
        assert_eq!(on_gpu.device_name(), Some("NVIDIA GeForce RTX 3060"));
        assert_eq!(on_gpu.arguments(), ["--device", "Vulkan0"]);
        assert_eq!(gpu.placement(None), None);
        gpu.set_wanted(false);
        assert_eq!(gpu.placement(Some(&found)), None);
        gpu.set_wanted(true);
        gpu.give_up("out of memory");
        assert_eq!(gpu.placement(Some(&found)), None);
    }

    #[test]
    fn the_device_is_given_up_only_when_the_processor_does_better() {
        use Outcome::*;
        // On the graphics processor.
        assert_eq!(verdict(true, false, Answered), Verdict::Unchanged);
        assert_eq!(verdict(true, false, Stopped), Verdict::Unchanged);
        assert_eq!(verdict(true, false, Failed), Verdict::Suspect);
        assert_eq!(verdict(true, false, Hung), Verdict::GiveUp);
        // The same request again on the processor.
        assert_eq!(verdict(false, true, Answered), Verdict::GiveUp);
        assert_eq!(verdict(false, true, Failed), Verdict::Keep);
        assert_eq!(verdict(false, true, Hung), Verdict::Keep);
        assert_eq!(verdict(false, true, Stopped), Verdict::Unchanged);
        // Another request on the processor says nothing of the device.
        for outcome in [Answered, Failed, Hung, Stopped] {
            assert_eq!(verdict(false, false, outcome), Verdict::Unchanged);
        }
    }

    #[test]
    fn a_suspect_device_waits_for_the_processor_s_answer() {
        let gpu = Graphics::new(Backend::Vulkan, Llama::at(PathBuf::from("nowhere")), true);
        let found = device("Vulkan0", "NVIDIA GeForce RTX 3060", 12288);
        // A request the processor fails too: the device comes back.
        gpu.apply(Verdict::Suspect, 7, "failed to parse grammar");
        assert_eq!(gpu.suspected(), Some(7));
        assert_eq!(gpu.placement(Some(&found)), None);
        gpu.apply(Verdict::Keep, 7, "failed to parse grammar");
        assert_eq!(gpu.suspected(), None);
        assert!(gpu.placement(Some(&found)).is_some());
        // One the processor answers: given up for the session.
        gpu.apply(Verdict::Suspect, 8, "out of device memory");
        gpu.apply(Verdict::GiveUp, 8, "");
        assert_eq!(gpu.suspected(), None);
        assert_eq!(gpu.placement(Some(&found)), None);
    }

    #[test]
    fn a_program_that_does_not_start_finds_no_device() {
        let gpu = Graphics::new(
            Backend::Vulkan,
            Llama::at(PathBuf::from("no-such-program")),
            true,
        );
        assert_eq!(gpu.device_if_known(), None);
        assert_eq!(gpu.device(), None);
        assert_eq!(gpu.device_if_known(), Some(None));
    }

    #[test]
    fn speeds_are_kept_per_device_and_earlier_files_still_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("speeds.json");
        std::fs::write(&path, r#"{"glm-ocr": 40.0}"#).unwrap();
        let mut speeds = Speeds::load(path.clone());
        let built_in = Placement::built_in();
        assert_eq!(speeds.on(&built_in).get("glm-ocr"), Some(&40.0));
        let gpu = Placement::Vulkan(device("Vulkan0", "NVIDIA GeForce RTX 3060", 12288));
        assert!(speeds.on(&gpu).is_empty());
        speeds.record(&gpu, "glm-ocr", 5.0);
        speeds.record(&built_in, "glm-ocr", 50.0);
        let again = Speeds::load(path);
        assert_eq!(again.on(&gpu).get("glm-ocr"), Some(&5.0));
        assert_eq!(again.on(&built_in).get("glm-ocr"), Some(&43.0));
    }
}
