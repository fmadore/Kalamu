// Builds Whisper's Vulkan helper (helpers/whisper-vulkan) for one Windows target and puts it where
// a release build looks for it: src-tauri/binaries/whisper-vulkan-<target>.exe, the sidecar
// tauri.ffmpeg.windows-<arch>.conf.json declares
// (docs/decisions/2026-11-08-whisper-graphics-processor.md).
//
//   node scripts/build-whisper-vulkan.mjs x86_64-pc-windows-msvc|aarch64-pc-windows-msvc
//
// Needs LunarG's Vulkan SDK (VULKAN_SDK set, its Bin on PATH for the shader compiler) and the same
// Clang setup as the app's own Whisper build.

import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { join, parse } from 'node:path';

const target = process.argv[2];
const targets = ['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'];
if (!targets.includes(target)) {
	console.error(`Usage: node scripts/build-whisper-vulkan.mjs ${targets.join('|')}`);
	process.exit(1);
}
if (!process.env.VULKAN_SDK) {
	console.error('VULKAN_SDK is not set: install LunarG’s Vulkan SDK first.');
	process.exit(1);
}

// A short target folder: ggml configures its Vulkan shader generator as a nested CMake project
// deep inside Cargo's build folder, and under target/<triple>/release its paths pass Windows'
// 260-character limit (rc.exe then cannot write its files; seen in Live Translation's CI). The
// runner's temporary folder on CI, otherwise a folder at the root of the drive;
// WHISPER_VULKAN_TARGET_DIR chooses another.
const targetDir =
	process.env.WHISPER_VULKAN_TARGET_DIR ??
	(process.env.RUNNER_TEMP
		? join(process.env.RUNNER_TEMP, 'wv')
		: join(parse(process.cwd()).root, 'wv'));

execFileSync(
	'cargo',
	['build', '--release', '--locked', '-p', 'whisper-vulkan', '--target', target],
	{ stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: targetDir } }
);
mkdirSync('src-tauri/binaries', { recursive: true });
copyFileSync(
	join(targetDir, target, 'release', 'whisper-vulkan.exe'),
	`src-tauri/binaries/whisper-vulkan-${target}.exe`
);
console.log(`src-tauri/binaries/whisper-vulkan-${target}.exe`);
