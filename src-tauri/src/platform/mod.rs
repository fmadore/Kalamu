//! What differs between the operating systems Kalamu ships for, in one place: each OS has its
//! own file with the same items, and the rest of the shell uses them without asking which OS
//! it runs on. The crates below the shell receive these facts as values (the player's formats
//! go to the engine through its config), so none of them matches on the OS.
//!
//! Adding an operating system is one more file here with the same items.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(any(windows, target_os = "macos")))]
compile_error!("Kalamu ships for Windows and macOS: add src-tauri/src/platform/<os>.rs first.");
