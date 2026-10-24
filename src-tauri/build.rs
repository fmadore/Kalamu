// The command list is `include!`d rather than duplicated, as in Live Translation:
// `AppManifest::commands` generates an `allow-<command>` permission for each name, which makes
// Tauri enforce the capability files, and a test checks the list against the handler.
include!("src/command_names.rs");

fn main() {
    if let Err(e) = tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    ) {
        // Without the manifest the ACL would silently go unenforced; fail the build instead.
        panic!("failed to generate the app ACL manifest: {e}");
    }
}
