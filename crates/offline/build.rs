//! With the `metal` feature (turned on by the app for macOS), whisper.cpp's Metal code checks the
//! system's version at run time (`@available`), which compiles to a call to
//! `__isPlatformVersionAtLeast` in clang's own runtime library. Rust links without that library,
//! so the app failed to link; it is added here, from the clang that built whisper.cpp.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_METAL").is_none() {
        return;
    }
    let resources = Command::new("xcrun")
        .args(["clang", "--print-resource-dir"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    match resources {
        Some(dir) => {
            println!(
                "cargo:rustc-link-search=native={}",
                dir.join("lib").join("darwin").display()
            );
            println!("cargo:rustc-link-lib=static:-bundle=clang_rt.osx");
        }
        None => println!(
            "cargo:warning=clang's runtime library was not found: linking Whisper's Metal code may fail"
        ),
    }
}
