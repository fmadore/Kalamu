// Writes THIRD-PARTY-NOTICES.txt into src-tauri/binaries/licenses/, which every installer and the
// Store package carry beside FFmpeg's licences: each Rust crate and npm package built into the
// app, with its licence and the licence files it ships. Run after `npm ci` and before the build:
//
//   node scripts/third-party-notices.mjs [target-triple]
//
// Rust: the crates `cargo metadata` resolves for the app on that target, other than Kalamu's own.
// Front end: the packages `dependencies` in package.json pulls in (package-lock.json), which the
// build bundles; development tools are not shipped and are left out.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

const target =
	process.argv[2] ??
	`${process.arch === 'arm64' ? 'aarch64' : 'x86_64'}-${process.platform === 'darwin' ? 'apple-darwin' : 'pc-windows-msvc'}`;
const LICENCE_FILE = /^(licen[cs]e|copying|notice|unlicense)([-._].*)?$/i;
/** C and C++ libraries a crate compiles in from a folder of its own, with their own licence. */
const VENDORED = { 'whisper-rs-sys': ['whisper.cpp'] };
/** Libraries a crate's build script downloads and compiles in, whose licences are therefore
 * kept in this repository. */
const DOWNLOADED = {
	'kreuzberg-tesseract': [
		'crates/offline/licenses/Tesseract-LICENSE.txt',
		'crates/offline/licenses/Leptonica-license.txt'
	]
};

/** The licence files at the top of a package's folder, as text. */
function licenceTexts(dir) {
	if (!existsSync(dir)) return [];
	return readdirSync(dir)
		.filter((f) => LICENCE_FILE.test(f))
		.sort()
		.map((f) => ({ file: f, text: readFileSync(join(dir, f), 'utf8').trim() }));
}

function rustCrates() {
	const metadata = JSON.parse(
		execFileSync(
			'cargo',
			[
				'metadata',
				'--format-version',
				'1',
				'--locked',
				'--filter-platform',
				target,
				'--manifest-path',
				'src-tauri/Cargo.toml'
			],
			{ maxBuffer: 256 * 1024 * 1024, encoding: 'utf8' }
		)
	);
	// Every crate reachable from the app through normal dependencies: build scripts and tests
	// are not shipped.
	const nodes = new Map(metadata.resolve.nodes.map((n) => [n.id, n]));
	const packages = new Map(metadata.packages.map((p) => [p.id, p]));
	const app = metadata.packages.find((p) => p.name === 'kalamu');
	const seen = new Set();
	const queue = [app.id];
	while (queue.length > 0) {
		const id = queue.pop();
		if (seen.has(id)) continue;
		seen.add(id);
		for (const dep of nodes.get(id)?.deps ?? []) {
			if (dep.dep_kinds.some((k) => k.kind === null)) queue.push(dep.pkg);
		}
	}
	return [...seen]
		.map((id) => packages.get(id))
		.filter((p) => p.source !== null)
		.map((p) => ({
			name: p.name,
			version: p.version,
			licence: p.license ?? (p.license_file ? `see ${p.license_file}` : 'unknown'),
			home: p.repository ?? p.homepage ?? '',
			texts: [
				...licenceTexts(dirname(p.manifest_path)),
				...(VENDORED[p.name] ?? []).flatMap((sub) =>
					licenceTexts(join(dirname(p.manifest_path), sub)).map((t) => ({
						...t,
						file: `${sub}/${t.file}`
					}))
				),
				...(DOWNLOADED[p.name] ?? []).map((file) => ({
					file: file.split('/').pop(),
					text: readFileSync(file, 'utf8').trim()
				}))
			]
		}));
}

function npmPackages() {
	const lock = JSON.parse(readFileSync('package-lock.json', 'utf8'));
	return (
		Object.entries(lock.packages)
			// Optional packages are platform builds for Node (pdf.js's canvas), never in the bundle.
			.filter(
				([path, entry]) =>
					path.startsWith('node_modules/') && !entry.dev && !entry.optional && existsSync(path)
			)
			.map(([path, entry]) => {
				const manifest = JSON.parse(readFileSync(join(path, 'package.json'), 'utf8'));
				const repository =
					typeof manifest.repository === 'string' ? manifest.repository : manifest.repository?.url;
				return {
					name: manifest.name,
					version: manifest.version,
					licence: manifest.license ?? entry.license ?? 'unknown',
					home: repository ?? manifest.homepage ?? '',
					texts: licenceTexts(path)
				};
			})
	);
}

/** Licence texts already written, so a text shared by many packages appears once. */
const written = new Map();

function section(title, list) {
	const unique = [...new Map(list.map((p) => [`${p.name}@${p.version}`, p])).values()].sort(
		(a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version)
	);
	const lines = [`${title} (${unique.length})`, '='.repeat(72), ''];
	for (const p of unique) {
		lines.push(`${p.name} ${p.version}`, `Licence: ${p.licence}`);
		if (p.home) lines.push(`Source: ${p.home.replace(/^git\+/, '')}`);
		for (const t of p.texts) {
			const first = written.get(t.text);
			if (first) {
				lines.push('', `--- ${t.file}: the same text as ${first} ---`);
			} else {
				written.set(t.text, `${p.name} ${p.version}, ${t.file}`);
				lines.push('', `--- ${t.file} ---`, t.text);
			}
		}
		lines.push('', '-'.repeat(72), '');
	}
	return lines.join('\n');
}

const rust = rustCrates();
const npm = npmPackages();
const out = 'src-tauri/binaries/licenses/THIRD-PARTY-NOTICES.txt';
mkdirSync(dirname(out), { recursive: true });
writeFileSync(
	out,
	[
		'Kalamu: third-party notices',
		'',
		'Kalamu is MIT licensed. It is built with the open-source software below, each under its own',
		'licence. FFmpeg, Opus and llama.cpp, which Kalamu runs as separate programs, have their',
		'licences in the files beside this one, as do the Vulkan headers (Khronos, MIT) built into',
		'llama.cpp’s graphics-processor program on Windows and the OpenCL headers (Khronos, Apache',
		'2.0) built into its Snapdragon program. Whisper models and the OCR models (LightOnOCR-3,',
		'Apache 2.0; GLM-OCR, MIT) and the language models (Gemma 4, Apache 2.0) are downloaded at',
		'the user’s request and are not part of the app.',
		'',
		section('Rust crates', rust),
		section('Front-end packages', npm)
	].join('\n')
);
console.log(`${out}: ${rust.length} Rust crates, ${npm.length} front-end packages.`);
