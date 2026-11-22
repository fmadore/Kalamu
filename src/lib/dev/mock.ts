// A pretend back end for looking at the interface in a plain browser during development
// (`npm run dev`, then open http://localhost:5173/?mock). It answers commands with sample data
// through Tauri's own IPC mock. Never loaded in the desktop app or in a production build.

import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { Catalog } from '../bindings/Catalog.js';
import type { EstimateRequest } from '../bindings/EstimateRequest.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import type { TaskRules } from '../bindings/TaskRules.js';
import { layout } from '../review.svelte.js';
import catalogJson from '../../../crates/catalog/catalog.json';
import taskRulesJson from './taskRules.json';
import { projectCommand, seedProjects } from './mockProjects.js';
import { projectFileCommand } from './mockProjectFile.js';
import { NO_RESULT } from './noResult.js';
import { enlarge, largeRows } from './mockLarge.js';
import { profileCommand } from './mockProfiles.js';
import { searchCommand } from './mockSearch.js';
import { meaningCommand } from './mockMeaning.js';
import { collectionsOf } from './mockProjects.js';
import { queueCommand } from './mockQueue.js';
import { analysisJob, audioJob, job, recording, source, summaries, video } from './mockJobs.js';
import { jobCommand, keptPiece } from './mockJobCommand.js';
import { interviewCommand } from './mockInterview.js';
import { interviewLength, textLength } from './mockInterviewLong.js';
import { screeningLength } from './mockScreening.js';
import { passageCommand } from './mockPassages.js';
import { regionCommand } from './mockDrawn.js';
import { regionReadingCommand } from './mockRegionReadings.js';
import { presetCommand, samplePipelines } from './mockPresets.js';
import { seedWorkflows, workflowCommand } from './mockWorkflows.js';
import { unitCommand } from './mockUnits.js';
import { mockImageDocument, PHOTO_FOLDER, PHOTOS } from './mockPages.js';
import { photoCommand, READING_ROOM, READING_ROOM_PHOTOS } from './mockPhotos.js';
import { sampleCommand } from './mockSample.js';

const catalog = catalogJson as unknown as Catalog;
/** The engine's rules for the bundled catalogue, written by `cargo test`. */
const taskRules = taskRulesJson as TaskRules[];

export { mockDragging, mockView } from './mockViews.js';

export function installMock(): void {
	const params = new URLSearchParams(location.search);
	// `compare`: open the review with the compared text beside it.
	if (params.has('compare') && !layout.compare) layout.toggleCompare();
	// `open=deux`: open the readings of the highlighted words with that text.
	const open = params.get('open');
	if (open) {
		setTimeout(() => {
			const lemma = [...document.querySelectorAll<HTMLElement>('.mark')].find(
				(l) => l.textContent === open
			);
			lemma?.click();
		}, 2500);
	}
	mockWindows('main');
	// `docs=12`: as many documents in the sample project, enough for a map.
	const more = Math.max(0, Number(params.get('docs') ?? 0) - 4);
	seedProjects({
		job,
		audioJob,
		analysisJob,
		extras: [
			{
				...source,
				id: '00000000-0000-0000-0000-0000000000f1',
				path: 'C:\\Archives\\Letters\\lettre-gouverneur-1956.jpg',
				file_name: 'lettre-gouverneur-1956.jpg',
				kind: 'image',
				page_count: 1
			},
			{
				...source,
				id: '00000000-0000-0000-0000-0000000000f2',
				path: 'C:\\Archives\\Letters\\lettre-1912-pere-eric.pdf',
				file_name: 'lettre-1912-pere-eric.pdf',
				page_count: 4
			},
			...Array.from({ length: more }, (_, i) => ({
				...source,
				id: `00000000-0000-0000-0000-0000000001${String(i).padStart(2, '0')}`,
				path: `C:\\Archives\\Journal\\journal-1958-${String(i + 1).padStart(2, '0')}.pdf`,
				file_name: `journal-1958-${String(i + 1).padStart(2, '0')}.pdf`,
				page_count: 2
			}))
		]
	});
	seedWorkflows(...samplePipelines(), summaries[0], source);
	// `idle`: no Whisper download running, so the other models' downloads can start.
	const whisperDownloading = !params.has('idle');
	// The first launch: `keys=none`, no key saved; `offline=none`, nothing installed offline, or
	// `offline=one`, a single Whisper model.
	const savedKeys = new Set(
		params.get('keys') === 'none'
			? []
			: catalog.providers.filter((p) => p.id !== 'openai').map((p) => p.id)
	);
	const offline = params.get('offline');
	const installedOffline = (installed: boolean, first = false): boolean =>
		offline === 'none' ? false : offline === 'one' ? first : installed;
	// The search by meaning's model: a download takes a second and a half here.
	const embedding = { installed: false, downloading: false, received: 0 };
	const downloadEmbedding = (): Promise<null> => {
		Object.assign(embedding, { downloading: true, received: 120_000_000 });
		return new Promise((resolve) =>
			setTimeout(() => {
				Object.assign(embedding, { installed: embedding.downloading, downloading: false });
				embedding.received = 0;
				resolve(null);
			}, 1500)
		);
	};
	const answer = (cmd: string, args?: unknown): unknown => {
		const a = (args ?? {}) as Record<string, unknown>;
		const filed = projectFileCommand(cmd, a);
		if (filed !== undefined) return filed;
		// `?long` and `?sample` only: a sample first, its rest and its comparison.
		const sampled = sampleCommand(cmd, a);
		if (sampled !== undefined) return sampled;
		const profile = profileCommand(cmd, a);
		if (profile !== undefined) return profile;
		const project = projectCommand(cmd, a);
		if (project !== undefined) return project;
		const workflow = workflowCommand(cmd, a);
		if (workflow !== undefined) return workflow;
		const queue = queueCommand(cmd, a);
		if (queue !== undefined) return queue;
		const unit = unitCommand(cmd, a);
		if (unit !== undefined) return unit;
		const found = searchCommand(cmd, a);
		if (found !== undefined) return found;
		const meant = meaningCommand(cmd, a, collectionsOf);
		if (meant !== undefined) return meant;
		const preset = presetCommand(cmd, a);
		if (preset !== undefined) return preset;
		const passage = passageCommand(
			cmd,
			a,
			keptPiece,
			(id) => catalog.providers.find((p) => p.id === id)?.local === true
		);
		if (passage !== undefined) return passage;
		const reread = regionReadingCommand(
			cmd,
			a,
			keptPiece,
			(id) => catalog.providers.find((p) => p.id === id)?.local === true
		);
		if (reread !== undefined) return reread;
		const region = regionCommand(cmd, a);
		if (region !== undefined) return region;
		const interview = interviewCommand(cmd, a);
		if (interview !== undefined) return interview;
		const jobAnswer = jobCommand(cmd, a);
		if (jobAnswer !== undefined) return jobAnswer;
		const photoAnswer = photoCommand(cmd, a);
		if (photoAnswer !== undefined) return photoAnswer;
		switch (cmd) {
			case 'app_info':
				return {
					version: '0.1.0',
					data_dir: 'C:\\Users\\you\\AppData\\Local\\Kalamu Data',
					logs_dir: '',
					interrupted: [],
					system_languages: [...navigator.languages],
					app_store: false,
					// `?set-aside`: the settings file could not be read at this launch.
					settings_set_aside: new URLSearchParams(location.search).has('set-aside')
						? 'C:\\Users\\you\\AppData\\Local\\Kalamu Data\\settings.json.bad'
						: null
				};
			case 'get_catalog':
				return catalog;
			case 'task_rules':
				return taskRules;
			// `?release`: the model list names a release newer than this copy.
			case 'newer_release':
				return new URLSearchParams(location.search).has('release')
					? { version: '1.6.0', url: 'https://github.com/fmadore/Kalamu/releases/tag/v1.6.0' }
					: null;
			case 'clear_unused_results':
				return { results: 1284, bytes: 48_620_000 };
			// `?unused=0`: nothing to clear, said without asking.
			case 'unused_results':
				return new URLSearchParams(location.search).get('unused') === '0'
					? { results: 0, bytes: 0 }
					: { results: 1284, bytes: 48_620_000 };
			case 'offline_status':
				return {
					models: catalog.models
						.filter((mod) => mod.provider === 'whisper')
						.map((mod, i) => ({
							id: mod.id,
							bytes: [574_041_195, 1_081_140_203, 190_085_487, 59_707_625][i] ?? 0,
							installed: installedOffline(i === 0, i === 0),
							downloading: i === 2 && whisperDownloading && offline === null,
							downloaded_bytes: i === 2 ? 71_000_000 : 0,
							in_use: false,
							languages: []
						})),
					// French and English downloaded, the Latin script model not yet.
					language_data: [
						['tesseract-fra', ['fr'], 3_972_885, true],
						['tesseract-eng', ['en'], 15_400_601, true],
						['tesseract-ara', ['ar'], 12_603_724, false],
						['tesseract-latin', ['ha', 'mos', 'wo', 'bm', 'dyu'], 101_402_885, false]
					].map(([id, languages, bytes, installed]) => ({
						id: id as string,
						bytes: bytes as number,
						installed: installedOffline(installed as boolean),
						downloading: false,
						downloaded_bytes: 0,
						in_use: false,
						languages: languages as string[]
					})),
					tesseract_version: '5.5.2',
					// LightOnOCR downloaded and timed on this computer; GLM-OCR's download stopped
					// part-way, what it received kept.
					ocr_models: [
						['lightonocr-3-1b', 1_088_895_872, true, 0],
						['glm-ocr', 1_434_837_056, false, 512_000_000]
					].map(([id, bytes, installed, kept]) => ({
						id: id as string,
						bytes: bytes as number,
						installed: installedOffline(installed as boolean),
						downloading: false,
						downloaded_bytes: kept as number,
						in_use: false,
						languages: []
					})),
					ocr_engine: true,
					ocr_seconds_per_page: { 'lightonocr-3-1b': 38.4 },
					ocr_devices: { 'lightonocr-3-1b': 'cpu', 'glm-ocr': 'cpu' },
					ocr_graphics_processor: null,
					// Gemma 4 E4B downloaded, E2B not.
					llm_models: [
						['gemma-4-e4b', 6_146_493_536, true],
						['gemma-4-e2b', 4_336_349_920, false]
					].map(([id, bytes, installed]) => ({
						id: id as string,
						bytes: bytes as number,
						installed: installedOffline(installed as boolean),
						downloading: false,
						downloaded_bytes: 0,
						in_use: false,
						languages: []
					})),
					llm_engine: true,
					llm_devices: { 'gemma-4-e4b': 'cpu', 'gemma-4-e2b': 'cpu' },
					// The search by meaning's model, not downloaded until asked.
					embedding_models: [
						{
							id: 'embeddinggemma-2',
							bytes: 309_855_456,
							installed: embedding.installed,
							downloading: embedding.downloading,
							downloaded_bytes: embedding.received,
							in_use: false,
							languages: []
						}
					],
					embedding_engine: true,
					missing_cpu: []
				};
			case 'download_model':
				return a.id === 'embeddinggemma-2' ? downloadEmbedding() : null;
			case 'cancel_model_download':
				Object.assign(embedding, { downloading: false, received: 0 });
				return null;
			case 'remove_model':
				if (a.id === 'embeddinggemma-2') embedding.installed = false;
				return null;
			case 'key_status':
				return catalog.providers.map((p) => ({ provider: p.id, saved: savedKeys.has(p.id) }));
			case 'set_api_key':
				savedKeys.add(a.provider as string);
				return null;
			case 'clear_api_key':
				savedKeys.delete(a.provider as string);
				return null;
			case 'estimate_cost': {
				const request = a.request as EstimateRequest;
				window.__kalamuEstimates = [...(window.__kalamuEstimates ?? []), request];
				const rows = request.setups.map((s) => ({
					provider: s.provider,
					model: s.model,
					pages: 120,
					merge: false,
					estimate: {
						usd: s.provider === 'mistral' ? null : 0.42,
						economy_usd: s.provider === 'mistral' ? null : 0.21
					},
					...interviewLength(request, s, catalog),
					...screeningLength(request, s),
					...textLength(request, s)
				}));
				const total = (economy: boolean) =>
					rows.reduce<number | null>((sum, r) => {
						const usd = economy ? (r.estimate.economy_usd ?? r.estimate.usd) : r.estimate.usd;
						return sum === null || usd === null ? null : sum + usd;
					}, 0);
				return { rows, total_usd: total(false), total_economy_usd: total(true) };
			}
			case 'expand_paths': {
				// Files the task does not read are set aside with their kind, as the app does; a
				// path without an extension stands for a folder already walked.
				const kinds: Record<string, string> = {
					pdf: 'pdf',
					png: 'image',
					jpg: 'image',
					jpeg: 'image',
					tif: 'image',
					tiff: 'image',
					mp3: 'audio',
					wav: 'audio',
					m4a: 'audio',
					mp4: 'video',
					mov: 'video',
					txt: 'text',
					md: 'text',
					docx: 'text',
					csv: 'text'
				};
				const reads = (taskRulesJson as TaskRules[]).find((r) => r.task === a.task)?.sources ?? [];
				const ext = (p: string) => /\.([a-z0-9]+)$/i.exec(p)?.[1]?.toLowerCase();
				const kind = (p: string) => kinds[ext(p) ?? ''] ?? null;
				const kept = (p: string) => !ext(p) || reads.some((k) => k === kind(p));
				// The sample folder of photographs is walked into its images.
				const paths = (a.paths as string[]).flatMap((p) =>
					p === PHOTO_FOLDER ? PHOTOS : p === READING_ROOM ? READING_ROOM_PHOTOS : [p]
				);
				return {
					paths: paths.filter(kept),
					refused: paths
						.filter((p) => !kept(p))
						.map((p) => ({
							path: p,
							file_name: p.split(/[\\/]/).pop() ?? p,
							kind: kind(p),
							folder: false
						}))
				};
			}
			// Every provider counts as agreed to, so the screens and their tests run unasked.
			case 'providers_to_consent':
				return [];
			case 'consent_providers':
				return null;
			case 'folders_to_ask':
				return [];
			case 'keep_access':
				return null;
			case 'inspect_files':
				return (a.paths as string[]).map((path, i) => {
					const id = `00000000-0000-0000-0000-0000000001${i}0`;
					const file_name = path.split('\\').pop();
					// Recordings of 42 and 65 minutes, cut in 20-minute pieces as the engine plans.
					if (/\.(m4a|mp3|wav)$/.test(path)) {
						const minutes = i === 0 ? 42 : 65;
						return {
							path,
							source: { ...recording, id, path, file_name, duration_ms: minutes * 60_000 },
							warnings: [],
							error: null,
							pieces: minutes > 25 ? Math.ceil((minutes - 25) / 20) + 1 : 1
						};
					}
					// A photograph: one page, its own content.
					if (/\.(jpg|png)$/.test(path)) {
						const sha256 = Array.from(path, (c) => c.charCodeAt(0).toString(16))
							.join('')
							.slice(-64)
							.padStart(64, '0');
						return {
							path,
							source: { ...source, id, path, file_name, sha256, kind: 'image', page_count: 1 },
							warnings: [],
							error: null,
							pieces: null
						};
					}
					return {
						path,
						source: { ...source, id, path, file_name, page_count: i === 0 ? 3 : 1 },
						warnings:
							i === 0
								? [
										{ source_id: source.id, kind: 'double_page_spread', pages: [2, 3] },
										{ source_id: source.id, kind: 'low_resolution', pages: [1], dpi: 150 }
									]
								: [],
						error: null,
						pieces: null
					};
				});
			case 'image_document':
				return mockImageDocument(String(a.name ?? ''), a.sources as SourceFile[]);
			case 'open_original':
				return null;
			case 'inspect_links':
				// The engine's check, roughly: an eleven-character id after a YouTube address.
				return (a.links as string[]).map((link, i) => {
					const id = /(?:youtu\.be\/|[?&]v=|shorts\/|live\/)([\w-]{11})/.exec(link)?.[1];
					return {
						path: link,
						source: id
							? {
									...video,
									id: `00000000-0000-0000-0000-0000000002${i}0`,
									path: `https://www.youtube.com/watch?v=${id}`,
									title: `Prêche du vendredi ${i + 1}`,
									file_name: `Prêche du vendredi ${i + 1} [${id}]`
								}
							: null,
						warnings: [],
						error: id ? null : { id: 'source.not_youtube', detail: link },
						pieces: null
					};
				});
			// A question before deleting (the dialog plugin's `ask`, a message with Yes and No):
			// the browser's own, so the answer is the one clicked.
			case 'plugin:dialog|message': {
				if (!a.buttons) return null;
				// With its buttons named, the answer is the name of the one clicked.
				const named = (a.buttons as { OkCancelCustom?: [string, string] }).OkCancelCustom;
				const yes = window.confirm(String(a.message ?? ''));
				if (named) return yes ? named[0] : named[1];
				return yes ? 'Yes' : 'No';
			}
			case 'plugin:dialog|open':
				// A folder of photographs when the picker asks for a folder.
				if (JSON.stringify(a).includes('"directory":true'))
					return params.has('photos') ? READING_ROOM : PHOTO_FOLDER;
				// Recordings when the picker asks for them, documents otherwise.
				if (JSON.stringify(a).includes('m4a'))
					return [
						'C:\\Archives\\Entretiens\\imam-2024-03.m4a',
						'C:\\Archives\\Entretiens\\marche-2024-04.m4a'
					];
				return [
					'C:\\Archives\\Bulletin\\bulletin-1957-06.pdf',
					'C:\\Archives\\Letters\\lettre-gouverneur.jpg'
				];
			default:
				// Tauri's own plugins (events, windows) answer nothing a page relies on.
				if (NO_RESULT.has(cmd) || cmd.startsWith('plugin:')) return null;
				throw { id: 'app.notDesktop', detail: cmd };
		}
	};
	// `rows=3000`: thousands of rows in the long tables and the sample job.
	const rows = largeRows();
	mockIPC((cmd, args) => {
		const a = (args ?? {}) as Record<string, unknown>;
		const result = answer(cmd, args);
		return result instanceof Promise
			? result.then((r) => enlarge(cmd, a, r, rows, job))
			: enlarge(cmd, a, result, rows, job);
	});
	// `piece_audio` names a blob URL here, not a file, so it is its own address.
	(
		window as unknown as { __TAURI_INTERNALS__: { convertFileSrc: (path: string) => string } }
	).__TAURI_INTERNALS__.convertFileSrc = (path) => path;
}
