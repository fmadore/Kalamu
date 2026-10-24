// Puts the bundled FFmpeg where `tauri build --config src-tauri/tauri.ffmpeg.conf.json` expects
// it: src-tauri/binaries/ffmpeg-<target>.exe and ffprobe-<target>.exe, plus their licences.
// Downloads only the archive pinned in scripts/ffmpeg.json and checks its SHA-256 first.
//
//   node scripts/fetch-ffmpeg.mjs [target-triple]    (default: this machine's architecture)

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
	copyFileSync,
	mkdirSync,
	mkdtempSync,
	readdirSync,
	readFileSync,
	writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const pin = JSON.parse(readFileSync('scripts/ffmpeg.json', 'utf8'));
const target =
	process.argv[2] ??
	(process.arch === 'arm64' ? 'aarch64-pc-windows-msvc' : 'x86_64-pc-windows-msvc');
const expected = pin.archives[target]?.sha256;
if (!expected) {
	console.error(
		`No pinned FFmpeg archive for ${target} yet: run the FFmpeg workflow and fill scripts/ffmpeg.json.`
	);
	process.exit(1);
}

const name = `ffmpeg-${pin.version}-${target}.zip`;
const url = `https://github.com/fmadore/Scriptorium/releases/download/${pin.release}/${name}`;
const response = await fetch(url);
if (!response.ok) throw new Error(`${url}: ${response.status}`);
const bytes = Buffer.from(await response.arrayBuffer());
const actual = createHash('sha256').update(bytes).digest('hex');
if (actual !== expected) throw new Error(`${name}: SHA-256 ${actual}, expected ${expected}`);

const work = mkdtempSync(join(tmpdir(), 'scriptorium-ffmpeg-'));
writeFileSync(join(work, name), bytes);
// bsdtar, shipped with Windows 10 and later, reads zip archives.
execFileSync('tar', ['-xf', join(work, name), '-C', work]);

mkdirSync('src-tauri/binaries/licenses', { recursive: true });
copyFileSync(join(work, 'ffmpeg.exe'), `src-tauri/binaries/ffmpeg-${target}.exe`);
copyFileSync(join(work, 'ffprobe.exe'), `src-tauri/binaries/ffprobe-${target}.exe`);
for (const file of readdirSync(join(work, 'licenses'))) {
	copyFileSync(join(work, 'licenses', file), join('src-tauri/binaries/licenses', file));
}
copyFileSync(join(work, 'BUILD.txt'), 'src-tauri/binaries/licenses/FFmpeg-BUILD.txt');
console.log(`FFmpeg ${pin.version} for ${target} installed in src-tauri/binaries.`);
