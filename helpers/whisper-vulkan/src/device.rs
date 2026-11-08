//! Which graphics processor Whisper runs on, of those ggml's Vulkan backend lists. The rules are
//! those of the OCR models' graphics build (`crates/offline/src/device.rs`, measured in
//! `docs/decisions/2026-11-06-ocr-graphics-processor.md`): a graphics card before integrated
//! graphics, Microsoft's Direct3D 12 layer last; never a software renderer, a device too small,
//! or Snapdragon's Adreno, where Vulkan computed wrong answers that end normally, so no
//! fallback would catch them.

/// What the Vulkan backend says of one device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub name: String,
    /// Its memory in bytes; 0 when the driver does not say.
    pub memory: usize,
}

/// Large v3 Turbo, the largest model offered, takes about 1 GiB on the device: 2 GiB leaves room
/// for the work buffers and for whatever else is drawing on the screen.
const MIN_MEMORY: usize = 2 << 30;

/// Devices known to compute wrongly through Vulkan, by a part of their name.
const READS_WRONGLY: [&str; 1] = ["adreno"];

const SOFTWARE: [&str; 4] = ["basic render", "llvmpipe", "lavapipe", "swiftshader"];

/// The index of the device to run on, `None` when none fits. Among equals, the first listed.
/// `allow_software` admits software renderers, for tests on machines without a graphics
/// processor.
pub fn choose(devices: &[Device], allow_software: bool) -> Option<usize> {
    devices
        .iter()
        .enumerate()
        .filter(|(_, d)| usable(d, allow_software))
        .min_by_key(|(_, d)| rank(&d.name))
        .map(|(index, _)| index)
}

fn usable(device: &Device, allow_software: bool) -> bool {
    let name = device.name.to_ascii_lowercase();
    let software = SOFTWARE.iter().any(|s| name.contains(s));
    if software {
        return allow_software;
    }
    let wrong = READS_WRONGLY.iter().any(|s| name.contains(s));
    !wrong && (device.memory == 0 || device.memory >= MIN_MEMORY)
}

/// 0 for a name that reads as a discrete graphics card, 2 for Microsoft's Direct3D 12 layer
/// (Dozen), which runs Vulkan over another driver, 1 otherwise. The names tell: every NVIDIA card
/// is discrete, AMD's are the Radeon RX and Pro, Intel's the Arc A and B series (Intel's
/// integrated graphics are also called Arc, without a model letter).
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const GIB: usize = 1 << 30;

    fn device(name: &str, memory: usize) -> Device {
        Device {
            name: name.into(),
            memory,
        }
    }

    #[test]
    fn a_graphics_card_comes_before_integrated_graphics() {
        let laptop = [
            device("Intel(R) Arc(TM) Graphics", 16 * GIB),
            device("NVIDIA GeForce RTX 4060 Laptop GPU", 8 * GIB),
        ];
        assert_eq!(choose(&laptop, false), Some(1));
        let amd = [
            device("AMD Radeon(TM) Graphics", 4 * GIB),
            device("AMD Radeon RX 7800 XT", 16 * GIB),
        ];
        assert_eq!(choose(&amd, false), Some(1));
        // Lunar Lake's integrated graphics are an Arc with a number, not a letter.
        assert_eq!(rank("Intel(R) Arc(TM) 140V GPU (16GB)"), 1);
        assert_eq!(rank("Intel(R) Arc(TM) B580 Graphics"), 0);
    }

    #[test]
    fn integrated_graphics_are_used_when_alone_and_dozen_comes_last() {
        assert_eq!(
            choose(&[device("Intel(R) Arc(TM) Graphics", 16 * GIB)], false),
            Some(0)
        );
        let layered = [
            device("Microsoft Direct3D12 (AMD Radeon(TM) Graphics)", 4 * GIB),
            device("AMD Radeon(TM) Graphics", 4 * GIB),
        ];
        assert_eq!(choose(&layered, false), Some(1));
    }

    #[test]
    fn software_small_and_wrong_devices_are_left_out() {
        let none = [
            device("Microsoft Basic Render Driver", 8 * GIB),
            device("llvmpipe (LLVM 17.0.6, 256 bits)", 8 * GIB),
            device("NVIDIA GeForce GT 710", GIB),
            device("Qualcomm(R) Adreno(TM) X1-85 GPU", 12 * GIB),
        ];
        assert_eq!(choose(&none, false), None);
        assert_eq!(choose(&[], false), None);
        // A driver that does not say how much memory it has is given the benefit of the doubt.
        assert_eq!(choose(&[device("AMD Radeon RX 6600", 0)], false), Some(0));
        // The tests' software renderer, when asked for.
        assert_eq!(choose(&none[1..2], true), Some(0));
        assert_eq!(choose(&none[3..], true), None);
    }
}
