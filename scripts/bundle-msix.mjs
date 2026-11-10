// Builds the Microsoft Store package for one architecture with @choochmeque/tauri-windows-bundle,
// FFmpeg, llama.cpp, Whisper's Vulkan helper and their licences inside (see docs/packaging-msix.md).
//
//   node scripts/bundle-msix.mjs x64|arm64 [--prebuilt] [--debug]
//   (npm run bundle:msix:x64 / bundle:msix:arm64)
//
// The bundler reads tauri.conf.json and tauri.windows.conf.json only, and `tauri build` merges the
// latter on Windows by itself, so the architecture's bundling configuration
// (tauri.ffmpeg.windows-<arch>.conf.json: FFmpeg and the llama.cpp programs) is laid in as
// tauri.windows.conf.json for the build and removed afterwards: an ordinary build or `tauri dev`
// must not need FFmpeg fetched. Fetch both first: `npm run fetch:ffmpeg -- <target>` and
// `npm run fetch:llama -- <target>`.
//
// The bundler compiles the app itself (`tauri build --no-bundle`). With --prebuilt it packages the
// app already built instead, so a release compiles it once for both the installer and this
// package (the windows-packages action):
//
//   npx tauri build --target <target> --no-bundle --config src-tauri/tauri.ffmpeg.windows-<arch>.conf.json
//   node scripts/bundle-msix.mjs <arch> --prebuilt
//
// --debug packages a debug build (target/<target>/debug), as the packaging dry run does.

import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, rmSync } from 'node:fs';
import { join, resolve } from 'node:path';

const [arch, ...flags] = process.argv.slice(2);
const unknown = flags.filter((f) => f !== '--prebuilt' && f !== '--debug');
if ((arch !== 'x64' && arch !== 'arm64') || unknown.length > 0) {
	console.error('Usage: node scripts/bundle-msix.mjs x64|arm64 [--prebuilt] [--debug]');
	process.exit(1);
}
const prebuilt = flags.includes('--prebuilt');
const debug = flags.includes('--debug');

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

const exe = join('target', target, debug ? 'debug' : 'release', 'kalamu.exe');
if (prebuilt && !existsSync(exe)) {
	console.error(
		`${exe} is missing: build it first with \`npx tauri build --target ${target} --no-bundle${debug ? ' --debug' : ''} --config src-tauri/tauri.ffmpeg.windows-${arch}.conf.json\`.`
	);
	process.exit(1);
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
			// The bundler runs `<runner> tauri build --target <target> --no-bundle` through the shell
			// before packaging. For an app already built the runner is `echo`, which only prints that
			// line: the bundler has no switch to skip its build.
			'--runner',
			prebuilt ? 'echo Packaging the app already built, not running:' : 'npm',
			...(debug ? ['--debug'] : [])
		],
		// Kalamu is a Cargo workspace, which builds into ./target rather than src-tauri/target, where
		// the bundler would look; it writes the package to target/msix.
		{ stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: resolve('target') } }
	);
} finally {
	rmSync(windows, { force: true });
}
