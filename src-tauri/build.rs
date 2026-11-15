// The command list is `include!`d rather than duplicated, as in Live Translation:
// `AppManifest::commands` generates an `allow-<command>` permission for each name, which makes
// Tauri enforce the capability files, and a test checks the list against the handler.
include!("src/command_names.rs");

fn main() {
    if let Err(e) = tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS))
            // Windows only: Tauri's own manifest (version 6 of the common controls, for the
            // system dialogs) with UTF-8 as the process's code page. Tesseract opens its language
            // data by a path in that code page; with UTF-8, a user folder whose name has accents
            // opens like any other. whisper.cpp needs none of this: it turns its UTF-8 paths into
            // wide characters itself. Rust's own file access is unaffected.
            .windows_attributes(
                tauri_build::WindowsAttributes::new()
                    .app_manifest(include_str!("windows-app-manifest.xml")),
            ),
    ) {
        // Without the manifest the ACL would silently go unenforced; fail the build instead.
        panic!("failed to generate the app ACL manifest: {e}");
    }
}
