// Puts the bundled FFmpeg where `tauri build --config src-tauri/tauri.ffmpeg.conf.json` expects
// it: src-tauri/binaries/ffmpeg-<target> and ffprobe-<target> (with `.exe` on Windows), plus
// their licences.
// Downloads only the archive pinned in scripts/ffmpeg.json and checks its SHA-256 first.
//
//   node scripts/fetch-ffmpeg.mjs [target-triple]    (default: this machine's system and architecture)

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
	chmodSync,
	copyFileSync,
	mkdirSync,
	mkdtempSync,
	readdirSync,
	readFileSync,
	writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { releaseAsset } from './release-asset.mjs';

const pin = JSON.parse(readFileSync('scripts/ffmpeg.json', 'utf8'));
const arch = process.arch === 'arm64' ? 'aarch64' : 'x86_64';
const target =
	process.argv[2] ??
	(process.platform === 'darwin' ? `${arch}-apple-darwin` : `${arch}-pc-windows-msvc`);
const exe = target.includes('windows') ? '.exe' : '';
const expected = pin.archives[target]?.sha256;
if (!expected) {
	console.error(
		`No pinned FFmpeg archive for ${target} yet: run the FFmpeg workflow and fill scripts/ffmpeg.json.`
	);
	process.exit(1);
}

const name = `ffmpeg-${pin.version}-${target}.zip`;
const bytes = await releaseAsset(pin.release, name);
const actual = createHash('sha256').update(bytes).digest('hex');
if (actual !== expected) throw new Error(`${name}: SHA-256 ${actual}, expected ${expected}`);

const work = mkdtempSync(join(tmpdir(), 'kalamu-ffmpeg-'));
writeFileSync(join(work, name), bytes);
// bsdtar, shipped with Windows 10 and later, reads zip archives. Named by its full path: from Git
// Bash, `tar` is GNU tar, which reads `C:` as a remote host.
const tar =
	process.platform === 'win32'
		? join(process.env.SystemRoot ?? 'C:\\Windows', 'System32', 'tar.exe')
		: 'tar';
execFileSync(tar, ['-xf', join(work, name), '-C', work]);

mkdirSync('src-tauri/binaries/licenses', { recursive: true });
for (const tool of ['ffmpeg', 'ffprobe']) {
	const installed = `src-tauri/binaries/${tool}-${target}${exe}`;
	copyFileSync(join(work, `${tool}${exe}`), installed);
	chmodSync(installed, 0o755);
}
for (const file of readdirSync(join(work, 'licenses'))) {
	copyFileSync(join(work, 'licenses', file), join('src-tauri/binaries/licenses', file));
}
copyFileSync(join(work, 'BUILD.txt'), 'src-tauri/binaries/licenses/FFmpeg-BUILD.txt');
console.log(`FFmpeg ${pin.version} for ${target} installed in src-tauri/binaries.`);
