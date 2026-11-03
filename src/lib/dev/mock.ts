// A pretend back end for looking at the interface in a plain browser during development
// (`npm run dev`, then open http://localhost:5173/?mock). It answers commands with sample data
// through Tauri's own IPC mock. Never loaded in the desktop app or in a production build.

import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { Pipeline } from '../bindings/Pipeline.js';
import type { Catalog } from '../bindings/Catalog.js';
import type { Preset } from '../bindings/Preset.js';
import type { Settings } from '../bindings/Settings.js';
import type { TaskRules } from '../bindings/TaskRules.js';
import { layout } from '../review.svelte.js';
import type { View } from '../state.svelte.js';
import catalogJson from '../../../crates/catalog/catalog.json';
import SAMPLE_SCAN from './sample-scan.jpg?inline';
import taskRulesJson from './taskRules.json';
import { DOCUMENT_ID, PROJECT_ID, projectCommand, seedProjects } from './mockProjects.js';
import { NO_RESULT } from './noResult.js';
import { queueCommand } from './mockQueue.js';
import {
	analysisJob,
	EXPORT_OPTIONS,
	analysisPieces,
	article,
	audioJob,
	audioPieces,
	audioSetup,
	job,
	nameSpeakers,
	pieces,
	recording,
	setupA,
	setupB,
	source,
	summaries,
	texts,
	video,
	videoJob,
	videoPieces
} from './mockJobs.js';

import { RUN_ID, seedWorkflows, WORKFLOW_ID, workflowCommand } from './mockWorkflows.js';
import { unitCommand } from './mockUnits.js';
import {
	contentsJob,
	contentsPieces,
	contentsPresets,
	contentsSource,
	correctionJob,
	correctionPieces,
	gallica
} from './mockContents.js';

const catalog = catalogJson as unknown as Catalog;
/** The engine's rules for the bundled catalogue, written by `cargo test`. */
const taskRules = taskRulesJson as TaskRules[];

const settings: Settings = {
	concurrency: 2,
	text_formats: ['txt', 'docx', 'json'],
	audio_formats: ['txt', 'docx', 'srt', 'json'],
	destination: { kind: 'beside_sources' },
	export_options: EXPORT_OPTIONS,
	theme: new URLSearchParams(location.search).get('theme') ?? 'system',
	language: new URLSearchParams(location.search).get('lang') ?? 'system',
	refresh_catalog: false,
	favourite_languages: ['fr', 'ar'],
	last_folder: null
};

/** The prompt a dictionary sends for proposed merges, as Prompts lists it. */
const dictionaryPreset: Preset = {
	id: 'dictionary.merges',
	task: 'analysis',
	name: 'Dictionary: proposed merges',
	description:
		"Sent when a project's dictionary asks a model to propose merges. Its answer's fields are fixed: the dictionary reads them.",
	instructions:
		'You help a researcher clean the dictionary of a research project… Never group rows the researcher kept apart.',
	user_prompt: 'Propose groups among the rows above, following the instructions given.',
	builtin: true,
	modified: false
};

const presets: Preset[] = [
	{
		id: 'text_extraction.printed',
		task: 'text_extraction',
		name: 'Printed or typed document',
		description:
			'Books, reports, typed letters and forms in any language, left to right or right to left; spelling kept as printed.',
		instructions:
			'# Printed or typed document\n\nYou transcribe printed and typewritten documents…',
		user_prompt: 'Transcribe this page completely, following the instructions given.',
		builtin: true,
		modified: false
	},
	{
		id: 'text_extraction.newspaper',
		task: 'text_extraction',
		name: 'Newspaper or magazine',
		description:
			'Article by article across the columns, with the words the columns split rejoined.',
		instructions: '# Newspaper or magazine page…',
		user_prompt: 'Transcribe this page completely, following the instructions given.',
		builtin: true,
		modified: true
	},
	{
		id: 'audio_transcription.interview',
		task: 'audio_transcription',
		name: 'Interview or recording, transcribed',
		description:
			'Word for word in the languages spoken, with speaker labels and a timestamp at each turn.',
		instructions: '# Interview or recording, transcribed…',
		user_prompt: 'Transcribe this recording.',
		builtin: true,
		modified: false
	},
	{
		id: 'analysis.entities',
		task: 'analysis',
		name: 'People, places and organisations',
		description: 'The people, organisations, places and subjects each document names.',
		instructions: '# People, places and organisations\n\nYou index documents for a researcher…',
		user_prompt: 'Read the document above and fill in each field.',
		builtin: true,
		modified: false,
		schema: {
			fields: [
				{
					key: 'people',
					label: 'People',
					instructions: 'People named in the document.',
					kind: { type: 'list', max: 30 },
					reason: false
				},
				{
					key: 'places',
					label: 'Places',
					instructions: 'Places named in the document.',
					kind: { type: 'list', max: 20 },
					reason: false
				},
				{
					key: 'subjects',
					label: 'Subjects',
					instructions: "The document's specific subjects.",
					kind: { type: 'list', max: 6 },
					reason: false
				}
			]
		}
	},
	{
		id: 'analysis.tone',
		task: 'analysis',
		name: 'Tone of the document',
		description: 'Whether each document presents its subject positively or negatively.',
		instructions: '# Tone of the document\n\nYou read documents for a researcher…',
		user_prompt: 'Read the document above and fill in each field.',
		builtin: true,
		modified: false,
		schema: {
			fields: [
				{
					key: 'main_subject',
					label: 'Main subject',
					instructions: 'What the document is mainly about, in a few words.',
					kind: { type: 'text' },
					reason: false
				},
				{
					key: 'tone',
					label: 'Tone',
					instructions: 'How the document presents its main subject.',
					kind: { type: 'choice', values: ['Positive', 'Neutral', 'Negative', 'Mixed'] },
					reason: true
				}
			]
		}
	}
];

/** Saved pipelines, as New job lists them. */
let pipelines: Pipeline[] = [
	{
		id: 'pipeline-0',
		name: 'Printed bulletins',
		task: 'text_extraction',
		setups: [
			{
				provider: 'gemini',
				model: 'gemini-3.8-flash',
				preset_id: 'text_extraction.printed',
				label: null,
				options: { temperature: null, diarize: false, timestamps: false, speakers: null }
			}
		],
		settings: {
			languages: { mode: 'explicit', languages: ['fr'] },
			sample_pages: null,
			line_breaks: 'running_text',
			pace: 'fast'
		},
		formats: ['txt', 'docx'],
		destination: { kind: 'beside_sources' },
		updated_at: 0
	},
	{
		id: 'pipeline-1',
		name: 'Press, people and places',
		task: 'analysis',
		setups: [
			{
				provider: 'gemini',
				model: 'gemini-3.8-flash',
				preset_id: 'analysis.entities',
				label: null,
				options: { temperature: null, diarize: false, timestamps: false, speakers: null }
			}
		],
		settings: {
			languages: { mode: 'explicit', languages: ['fr'] },
			sample_pages: null,
			line_breaks: null,
			pace: 'fast'
		},
		formats: ['csv', 'json'],
		destination: { kind: 'beside_sources' },
		updated_at: 0
	}
];

/** Nine seconds of a quiet tone as a WAV file, standing in for a recording. */
function toneWav(): ArrayBuffer {
	const rate = 8_000;
	const samples = rate * 9;
	const buffer = new ArrayBuffer(44 + samples * 2);
	const view = new DataView(buffer);
	const text = (at: number, value: string) =>
		[...value].forEach((c, i) => view.setUint8(at + i, c.charCodeAt(0)));
	text(0, 'RIFF');
	view.setUint32(4, 36 + samples * 2, true);
	text(8, 'WAVEfmt ');
	view.setUint32(16, 16, true);
	view.setUint16(20, 1, true);
	view.setUint16(22, 1, true);
	view.setUint32(24, rate, true);
	view.setUint32(28, rate * 2, true);
	view.setUint16(32, 2, true);
	view.setUint16(34, 16, true);
	text(36, 'data');
	view.setUint32(40, samples * 2, true);
	for (let i = 0; i < samples; i++) {
		view.setInt16(44 + i * 2, Math.sin((2 * Math.PI * 220 * i) / rate) * 2_000, true);
	}
	return buffer;
}

/** A word diff like the engine's, so the review's apparatus shows real variants. */
function wordDiff(left: string, right: string): { kind: string; text: string }[] {
	const a = left.split(/(\s+)/).filter(Boolean);
	const b = right.split(/(\s+)/).filter(Boolean);
	const lcs = Array.from({ length: a.length + 1 }, () => new Array<number>(b.length + 1).fill(0));
	for (let i = a.length - 1; i >= 0; i--)
		for (let j = b.length - 1; j >= 0; j--)
			lcs[i][j] = a[i] === b[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
	const out: { kind: string; text: string }[] = [];
	const push = (kind: string, text: string) => {
		const last = out[out.length - 1];
		if (last?.kind === kind) last.text += text;
		else out.push({ kind, text });
	};
	let i = 0;
	let j = 0;
	while (i < a.length || j < b.length) {
		if (i < a.length && j < b.length && a[i] === b[j]) push('same', (i++, j++, a[i - 1]));
		else if (j < b.length && (i >= a.length || lcs[i][j + 1] >= lcs[i + 1][j]))
			push('added', b[j++]);
		else push('removed', a[i++]);
	}
	return out;
}

/** The screen to open on, from `?mock&view=…`, for screenshots of each screen. */
/** Error rates against reviewed pages: the comparison and the interviews have some. */
function mockScores(jobId: string) {
	if (jobId === job.id) {
		return [
			{
				setup_id: setupA.id,
				label: setupA.label,
				cer: 0.012,
				wer: 0.04,
				scored_pages: 1,
				cost_usd: 0.0036
			},
			{
				setup_id: setupB.id,
				label: setupB.label,
				cer: 0.031,
				wer: 0.09,
				scored_pages: 1,
				cost_usd: 0.012
			}
		];
	}
	if (jobId === audioJob.id) {
		return [
			{
				setup_id: audioSetup.id,
				label: audioSetup.label,
				cer: 0.023,
				wer: 0.081,
				scored_pages: 2,
				cost_usd: 0.02
			}
		];
	}
	return [];
}

export function mockView(): View | null {
	switch (new URLSearchParams(location.search).get('view') ?? '') {
		case 'jobs':
			return { name: 'jobs', jobId: job.id };
		case 'review':
			return { name: 'review', jobId: job.id, sourceId: source.id };
		case 'review-audio':
			return { name: 'review', jobId: audioJob.id, sourceId: recording.id };
		case 'review-analysis':
			return { name: 'review', jobId: analysisJob.id, sourceId: article.id };
		case 'review-youtube':
			return { name: 'review', jobId: videoJob.id, sourceId: video.id };
		case 'review-contents':
			return { name: 'review', jobId: contentsJob.id, sourceId: contentsSource.id };
		case 'review-correction':
			return { name: 'review', jobId: correctionJob.id, sourceId: gallica.id };
		case 'prompts':
		case 'prompts-analysis':
			return { name: 'prompts' };
		case 'settings':
			return { name: 'settings' };
		case 'glossary':
			return { name: 'glossary' };
		case 'new':
			return { name: 'new' };
		case 'projects':
			return { name: 'projects' };
		case 'project':
			return { name: 'projects', projectId: PROJECT_ID };
		case 'document':
			return { name: 'projects', projectId: PROJECT_ID, documentId: DOCUMENT_ID };
		case 'queue':
			return { name: 'projects', projectId: PROJECT_ID, pane: 'queue' };
		case 'workflows':
			return { name: 'workflows' };
		case 'workflow':
			return { name: 'workflows', workflowId: WORKFLOW_ID };
		case 'run':
			return { name: 'workflows', runId: RUN_ID };
		default:
			return null;
	}
}

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
	let toneUrl: string | undefined;
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
			}
		]
	});
	seedWorkflows(pipelines[0], pipelines[1], summaries[0], source);
	mockIPC((cmd, args) => {
		const a = (args ?? {}) as Record<string, unknown>;
		const project = projectCommand(cmd, a);
		if (project !== undefined) return project;
		const workflow = workflowCommand(cmd, a);
		if (workflow !== undefined) return workflow;
		const queue = queueCommand(cmd, a);
		if (queue !== undefined) return queue;
		const unit = unitCommand(cmd, a);
		if (unit !== undefined) return unit;
		switch (cmd) {
			case 'app_info':
				return {
					version: '0.1.0',
					data_dir: 'C:\\Users\\you\\AppData\\Local\\Kalamu Data',
					logs_dir: '',
					interrupted: [],
					system_languages: [...navigator.languages]
				};
			case 'get_settings':
				return settings;
			case 'get_catalog':
				return catalog;
			case 'task_rules':
				return taskRules;
			case 'offline_status':
				return {
					models: catalog.models
						.filter((mod) => mod.provider === 'whisper')
						.map((mod, i) => ({
							id: mod.id,
							bytes: [574_041_195, 1_081_140_203, 190_085_487, 59_707_625][i] ?? 0,
							installed: i === 0,
							downloading: i === 2,
							downloaded_bytes: i === 2 ? 71_000_000 : 0,
							in_use: false
						})),
					missing_cpu: []
				};
			case 'key_status':
				return catalog.providers.map((p) => ({ provider: p.id, saved: p.id !== 'openai' }));
			case 'list_presets': {
				// `view=prompts-analysis` opens Prompts on an analysis, for screenshots.
				const first = location.search.includes('prompts-analysis') ? 'analysis' : '';
				const ordered = [...presets, ...contentsPresets, dictionaryPreset].sort(
					(x, y) => Number(y.task === first) - Number(x.task === first)
				);
				// The dictionary's prompt is listed in Prompts, never offered for documents.
				return a.task
					? ordered.filter((p) => p.task === a.task && p !== dictionaryPreset)
					: ordered;
			}
			case 'list_pipelines':
				return pipelines;
			case 'save_pipeline': {
				const saved = { ...(a.pipeline as Pipeline), id: `pipeline-${pipelines.length + 1}` };
				pipelines = [...pipelines.filter((p) => p.name !== saved.name), saved];
				return saved;
			}
			case 'delete_pipeline':
				pipelines = pipelines.filter((p) => p.id !== a.id);
				return null;
			case 'list_jobs':
				return [
					...summaries,
					{
						...summaries[0],
						id: analysisJob.id,
						name: analysisJob.name,
						task: 'analysis',
						pieces_total: 2,
						pieces_finished: 2
					},
					...[contentsJob, correctionJob].map((j, i) => ({
						...summaries[0],
						id: j.id,
						name: j.name,
						task: j.task,
						pieces_total: i === 0 ? 4 : 2,
						pieces_finished: i === 0 ? 4 : 2,
						setup_count: 1
					}))
				];
			case 'get_job':
				if (a.id === videoJob.id) return videoJob;
				if (a.id === contentsJob.id) return contentsJob;
				if (a.id === correctionJob.id) return correctionJob;
				return a.id === audioJob.id ? audioJob : a.id === analysisJob.id ? analysisJob : job;
			case 'rename_job': {
				const name = String(a.name).trim();
				if (!name) return null;
				for (const j of [job, audioJob, videoJob, analysisJob]) if (j.id === a.id) j.name = name;
				for (const s of summaries) if (s.id === a.id) s.name = name;
				return null;
			}
			case 'job_outputs': {
				const stem = source.path.replace(/\.[^.\\]+$/, '');
				return [`${stem}.txt`, `${stem}.docx`, `${stem}.provenance.json`];
			}
			case 'open_output':
				return null;
			// The corrected text of the sample extraction, frozen for an analysis.
			case 'snapshot_job':
				return [
					{
						...article,
						id: '00000000-0000-0000-0000-0000000000f9',
						path: 'C:\\Users\\you\\AppData\\Local\\Kalamu Data\\snapshots\\ab.txt',
						file_name: 'bulletin-1957-05.txt',
						derived_from: {
							snapshot_id: 'snapshot-1',
							job_id: a.id,
							setup_id: a.setup,
							source_id: source.id,
							original_path: source.path
						}
					}
				];
			case 'derived_state':
				return [];
			case 'job_pieces':
				return pieces.map((p) => ({
					...p,
					job_id: job.id,
					cache_key: 'k',
					attempts: 1,
					updated_at: 0
				}));
			case 'source_pieces':
				if (a.jobId === analysisJob.id) return analysisPieces;
				if (a.jobId === contentsJob.id) return contentsPieces;
				if (a.jobId === correctionJob.id) return correctionPieces;
				if (a.jobId === videoJob.id) return videoPieces();
				return a.jobId === audioJob.id ? audioPieces() : pieces;
			case 'source_text':
				return texts[0];
			case 'save_speaker_names':
				nameSpeakers((a.names as { names: Record<string, string> }).names);
				return [];
			case 'piece_audio':
				// The tone stands in for a whole recording, the piece for a span of it.
				toneUrl ??= URL.createObjectURL(new Blob([toneWav()], { type: 'audio/wav' }));
				return { path: toneUrl, start_ms: 2_000, end_ms: 8_000, original: true };
			case 'piece_preview':
				// A synthetic scan as an image: shown everywhere, headless browsers included.
				return { mime: 'image/jpeg', data: SAMPLE_SCAN.split(',')[1] ?? '' };
			case 'compare_source':
				return { scores: mockScores(String(a.jobId)), pages: [] };
			case 'job_scores': {
				const scores = mockScores(String(a.jobId));
				const scored = a.jobId === job.id ? source : a.jobId === audioJob.id ? recording : null;
				return {
					total: scores,
					sources: scored ? [{ source_id: scored.id, scores }] : []
				};
			}
			case 'diff_texts':
				return wordDiff(String(a.left ?? ''), String(a.right ?? ''));
			case 'estimate_cost': {
				const rows = (a.request as { setups: { provider: string; model: string }[] }).setups.map(
					(s) => ({
						provider: s.provider,
						model: s.model,
						pages: 120,
						merge: false,
						estimate: {
							usd: s.provider === 'mistral' ? null : 0.42,
							economy_usd: s.provider === 'mistral' ? null : 0.21
						}
					})
				);
				const total = (economy: boolean) =>
					rows.reduce<number | null>((sum, r) => {
						const usd = economy ? (r.estimate.economy_usd ?? r.estimate.usd) : r.estimate.usd;
						return sum === null || usd === null ? null : sum + usd;
					}, 0);
				return { rows, total_usd: total(false), total_economy_usd: total(true) };
			}
			case 'expand_paths':
				return a.paths;
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
			case 'plugin:dialog|open':
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
			case 'job_text_outputs':
			case 'save_correction':
			case 'export_source':
				return [];
			default:
				// Tauri's own plugins (events, windows) answer nothing a page relies on.
				if (NO_RESULT.has(cmd) || cmd.startsWith('plugin:')) return null;
				throw { id: 'app.notDesktop', detail: cmd };
		}
	});
	// `piece_audio` names a blob URL here, not a file, so it is its own address.
	(
		window as unknown as { __TAURI_INTERNALS__: { convertFileSrc: (path: string) => string } }
	).__TAURI_INTERNALS__.convertFileSrc = (path) => path;
}
