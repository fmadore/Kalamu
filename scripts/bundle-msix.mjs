// Builds the Microsoft Store package for one architecture with @choochmeque/tauri-windows-bundle,
// FFmpeg, llama.cpp, Whisper's Vulkan helper and their licences inside (see docs/packaging-msix.md).
//
//   node scripts/bundle-msix.mjs x64|arm64      (npm run bundle:msix:x64 / bundle:msix:arm64)
//
// The bundler reads tauri.conf.json and tauri.windows.conf.json only, and `tauri build` merges the
// latter on Windows by itself, so the architecture's bundling configuration
// (tauri.ffmpeg.windows-<arch>.conf.json: FFmpeg and the llama.cpp programs) is laid in as
// tauri.windows.conf.json for the build and removed afterwards: an ordinary build or `tauri dev`
// must not need FFmpeg fetched. Fetch both first: `npm run fetch:ffmpeg -- <target>` and
// `npm run fetch:llama -- <target>`.

import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, rmSync } from 'node:fs';
import { resolve } from 'node:path';

const arch = process.argv[2];
if (arch !== 'x64' && arch !== 'arm64') {
	console.error('Usage: node scripts/bundle-msix.mjs x64|arm64');
	process.exit(1);
}

const target = arch === 'x64' ? 'x86_64-pc-windows-msvc' : 'aarch64-pc-windows-msvc';
for (const [tool, fetch] of [
	['ffmpeg', 'fetch:ffmpeg'],
	['ffprobe', 'fetch:ffmpeg'],
	['llama-mtmd-cli', 'fetch:llama'],
	['llama-mtmd-cli-vulkan', 'fetch:llama'],
	['llama-completion', 'fetch:llama'],
	['llama-completion-vulkan', 'fetch:llama'],
	...(arch === 'arm64' ? [['llama-mtmd-cli-opencl', 'fetch:llama']] : []),
	['whisper-vulkan', 'build:whisper-vulkan']
]) {
	if (!existsSync(`src-tauri/binaries/${tool}-${target}.exe`)) {
		console.error(`${tool} is missing: run \`npm run ${fetch} -- ${target}\` first.`);
		process.exit(1);
	}
}

const windows = 'src-tauri/tauri.windows.conf.json';
copyFileSync(`src-tauri/tauri.ffmpeg.windows-${arch}.conf.json`, windows);
try {
	execFileSync(
		process.execPath,
		[
			'node_modules/@choochmeque/tauri-windows-bundle/dist/cli.js',
			'build',
			'--arch',
			arch,
			'--runner',
			'npm'
		],
		// Kalamu is a Cargo workspace, which builds into ./target rather than src-tauri/target, where
		// the bundler would look; it writes the package to target/msix.
		{ stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: resolve('target') } }
	);
} finally {
	rmSync(windows, { force: true });
}
