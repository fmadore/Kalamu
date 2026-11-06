// Puts the bundled llama.cpp programs, which run the offline OCR models, where `tauri build`
// expects them with the bundling configuration (src-tauri/tauri.ffmpeg.conf.json, or
// tauri.ffmpeg.windows-<arch>.conf.json on Windows): src-tauri/binaries/llama-mtmd-cli-<target>
// (with `.exe` on Windows), on Windows also llama-mtmd-cli-vulkan-<target>.exe, which uses the
// graphics processor, and on Windows ARM64 llama-mtmd-cli-opencl-<target>.exe, for Snapdragon's
// Adreno, plus their licences. Downloads only the archives pinned in scripts/llama.json and
// checks their SHA-256 first.
//
//   node scripts/fetch-llama.mjs [target-triple]    (default: this machine's system and architecture)

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
	chmodSync,
	copyFileSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { releaseAsset } from './release-asset.mjs';

const pin = JSON.parse(readFileSync('scripts/llama.json', 'utf8'));
const arch = process.arch === 'arm64' ? 'aarch64' : 'x86_64';
const target =
	process.argv[2] ??
	(process.platform === 'darwin' ? `${arch}-apple-darwin` : `${arch}-pc-windows-msvc`);
const windows = target.includes('windows');
const exe = windows ? '.exe' : '';

// The processor build everywhere; the Vulkan build on Windows only (a Mac's build already uses
// the graphics processor through Metal); the OpenCL build on Windows ARM64 only.
const builds = [{ archive: 'llama', program: 'llama-mtmd-cli', pins: pin.archives, build: '' }];
if (windows) {
	builds.push({
		archive: 'llama-vulkan',
		program: 'llama-mtmd-cli-vulkan',
		pins: pin.vulkan ?? {},
		build: '-vulkan'
	});
}
if (target === 'aarch64-pc-windows-msvc') {
	builds.push({
		archive: 'llama-opencl',
		program: 'llama-mtmd-cli-opencl',
		pins: pin.opencl ?? {},
		build: '-opencl'
	});
}

// bsdtar, shipped with Windows 10 and later, reads zip archives. Named by its full path: from Git
// Bash, `tar` is GNU tar, which reads `C:` as a remote host.
const tar =
	process.platform === 'win32'
		? join(process.env.SystemRoot ?? 'C:\\Windows', 'System32', 'tar.exe')
		: 'tar';

mkdirSync('src-tauri/binaries/licenses', { recursive: true });
for (const { archive, program, pins, build } of builds) {
	const expected = pins[target]?.sha256;
	if (!expected) {
		console.error(
			`No pinned ${archive} archive for ${target} yet: run the llama.cpp workflow and fill scripts/llama.json.`
		);
		process.exit(1);
	}
	const name = `${archive}-${pin.version}-${target}.zip`;
	const bytes = await releaseAsset(pin.release, name);
	const actual = createHash('sha256').update(bytes).digest('hex');
	if (actual !== expected) throw new Error(`${name}: SHA-256 ${actual}, expected ${expected}`);

	const work = mkdtempSync(join(tmpdir(), 'kalamu-llama-'));
	writeFileSync(join(work, name), bytes);
	execFileSync(tar, ['-xf', join(work, name), '-C', work]);

	const installed = `src-tauri/binaries/${program}-${target}${exe}`;
	copyFileSync(join(work, `${program}${exe}`), installed);
	chmodSync(installed, 0o755);
	copyFileSync(
		join(work, 'licenses', 'llama.cpp-MIT.txt'),
		'src-tauri/binaries/licenses/llama.cpp-MIT.txt'
	);
	copyFileSync(join(work, 'BUILD.txt'), `src-tauri/binaries/licenses/llama.cpp${build}-BUILD.txt`);
	// The headers compiled into the graphics-processor builds.
	for (const licence of ['Vulkan-Headers-MIT.txt', 'OpenCL-Headers-Apache-2.0.txt']) {
		const path = join(work, 'licenses', licence);
		if (existsSync(path)) copyFileSync(path, `src-tauri/binaries/licenses/${licence}`);
	}
	console.log(
		`${program} (llama.cpp ${pin.version}) for ${target} installed in src-tauri/binaries.`
	);
}
