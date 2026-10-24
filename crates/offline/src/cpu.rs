//! Whether this processor can run the Whisper engine compiled into the app.
//!
//! Ported from Live Translation. ggml's CPU library is built for a fixed instruction-set level
//! (`.cargo/config.toml`), not dispatched at run time, so on an older processor its first
//! matrix operation would be an illegal instruction and take the app down. This check runs
//! before any native code is touched.

use std::sync::OnceLock;

/// x64: clang-cl compiles ggml-cpu with `/arch:AVX2`, which it treats as Haswell, so the whole
/// Haswell set is required, not only what ggml uses on purpose.
#[cfg(target_arch = "x86_64")]
const REQUIRED: &[&str] = &[
    "sse4.2", "popcnt", "avx", "avx2", "fma", "f16c", "bmi1", "bmi2", "lzcnt", "movbe",
];

/// ARM64: built for `armv8.2-a+dotprod+fp16`. Only dot-product is checked: Windows reports
/// FP16 absent on older builds even where it exists, and every Windows-on-ARM SoC with
/// dot-product (Snapdragon 850 and later) also has FP16.
#[cfg(target_arch = "aarch64")]
const REQUIRED: &[&str] = &["dotprod"];

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const REQUIRED: &[&str] = &[];

/// `None` for a name this table does not know, which counts as missing: a typo in `REQUIRED`
/// fails closed.
#[cfg(target_arch = "x86_64")]
fn detected(feature: &str) -> Option<bool> {
    Some(match feature {
        "sse4.2" => std::arch::is_x86_feature_detected!("sse4.2"),
        "popcnt" => std::arch::is_x86_feature_detected!("popcnt"),
        "avx" => std::arch::is_x86_feature_detected!("avx"),
        "avx2" => std::arch::is_x86_feature_detected!("avx2"),
        "fma" => std::arch::is_x86_feature_detected!("fma"),
        "f16c" => std::arch::is_x86_feature_detected!("f16c"),
        "bmi1" => std::arch::is_x86_feature_detected!("bmi1"),
        "bmi2" => std::arch::is_x86_feature_detected!("bmi2"),
        "lzcnt" => std::arch::is_x86_feature_detected!("lzcnt"),
        "movbe" => std::arch::is_x86_feature_detected!("movbe"),
        _ => return None,
    })
}

#[cfg(target_arch = "aarch64")]
fn detected(feature: &str) -> Option<bool> {
    match feature {
        "dotprod" => Some(std::arch::is_aarch64_feature_detected!("dotprod")),
        _ => None,
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn detected(_feature: &str) -> Option<bool> {
    None
}

/// The instruction-set extensions in `required` that `has` does not confirm, upper-cased.
fn missing(required: &[&str], has: impl Fn(&str) -> Option<bool>) -> Vec<String> {
    required
        .iter()
        .filter(|feature| has(feature) != Some(true))
        .map(|feature| feature.to_uppercase())
        .collect()
}

/// What this processor lacks, worked out once. Empty when Whisper can run.
pub fn missing_features() -> &'static [String] {
    static MISSING: OnceLock<Vec<String>> = OnceLock::new();
    MISSING.get_or_init(|| missing(REQUIRED, detected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_exactly_the_missing_extensions() {
        assert_eq!(
            missing(&["avx", "avx2", "sse4.2"], |f| Some(f == "sse4.2")),
            ["AVX", "AVX2"]
        );
        // A name nothing can detect is treated as absent, never as present.
        assert_eq!(missing(&["avx3"], |_| None), ["AVX3"]);
        assert!(missing(&["dotprod"], |_| Some(true)).is_empty());
    }

    #[test]
    fn every_required_extension_has_a_detector() {
        for feature in REQUIRED {
            assert!(detected(feature).is_some(), "no detector for {feature}");
        }
    }

    #[test]
    fn this_machine_can_run_whisper() {
        assert!(missing_features().is_empty(), "{:?}", missing_features());
    }
}
