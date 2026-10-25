// A pretend back end for looking at the interface in a plain browser during development
// (`npm run dev`, then open http://localhost:5173/?mock). It answers commands with sample data
// through Tauri's own IPC mock. Never loaded in the desktop app or in a production build.

import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { Catalog } from '../bindings/Catalog.js';
import type { Job } from '../bindings/Job.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { Preset } from '../bindings/Preset.js';
import type { Settings } from '../bindings/Settings.js';
import type { View } from '../state.svelte.js';
import catalogJson from '../../../crates/catalog/catalog.json';
import SAMPLE_SCAN from './sample-scan.jpg?inline';

const catalog = catalogJson as unknown as Catalog;

const settings: Settings = {
	concurrency: 2,
	text_formats: ['txt', 'docx', 'json'],
	audio_formats: ['txt', 'docx', 'srt', 'json'],
	destination: { kind: 'beside_sources' },
	export_options: { piece_markers: true, speaker_labels: true, timestamps: true },
	theme: new URLSearchParams(location.search).get('theme') ?? 'system',
	refresh_catalog: false
};

const presets: Preset[] = [
	{
		id: 'text_extraction.printed_diplomatic',
		task: 'text_extraction',
		name: 'Printed document, diplomatic',
		description: 'Keeps spelling, line breaks and end-of-line hyphens exactly as on the page.',
		instructions: '# Diplomatic OCR/HTR\n\nPreserve original spelling…',
		user_prompt: 'Transcribe this page completely, following the instructions given.',
		builtin: true,
		modified: false
	},
	{
		id: 'text_extraction.newspaper',
		task: 'text_extraction',
		name: 'Newspaper or magazine (columns)',
		description: 'Joins column lines into paragraphs.',
		instructions: '# OCR System Prompt for Newspaper Articles…',
		user_prompt: 'Please perform complete OCR transcription of this single page.',
		builtin: true,
		modified: true
	},
	{
		id: 'audio_transcription.interview',
		task: 'audio_transcription',
		name: 'Interview, verbatim with speakers',
		description: 'Speaker labels and a timestamp at each turn.',
		instructions: '# Full audio transcription…',
		user_prompt: 'Transcribe this recording.',
		builtin: true,
		modified: false
	}
];

const source = {
	id: '00000000-0000-0000-0000-000000000001',
	path: 'C:\\Archives\\Bulletin\\bulletin-1957-05.pdf',
	file_name: 'bulletin-1957-05.pdf',
	sha256: 'ab'.repeat(32),
	size_bytes: 4_200_000,
	kind: 'pdf' as const,
	page_count: 3,
	duration_ms: null,
	split_spreads: false,
	even_out_volume: false
};

const setupA = {
	id: '00000000-0000-0000-0000-00000000000a',
	label: 'gemini-3.8-flash',
	provider: 'gemini',
	model: 'gemini-3.8-flash',
	prompt: {
		preset_id: 'text_extraction.newspaper',
		preset_name: 'Newspaper',
		template: '',
		text: '',
		user_prompt: ''
	},
	options: { temperature: null, diarize: false, timestamps: false, speakers: null }
};
const setupB = {
	...setupA,
	id: '00000000-0000-0000-0000-00000000000b',
	label: 'claude-opus-5-5',
	provider: 'anthropic',
	model: 'claude-opus-5-5'
};

const job: Job = {
	id: '00000000-0000-0000-0000-0000000000j1',
	name: 'Bulletin, May 1957',
	task: 'text_extraction',
	created_at: Date.now() - 3_600_000,
	sources: [source],
	setups: [setupA, setupB],
	settings: { languages: { mode: 'explicit', languages: ['fr'] }, sample_pages: null },
	output: {
		formats: ['txt', 'docx'],
		destination: { kind: 'beside_sources' },
		options: settings.export_options
	},
	status: 'completed'
};

const summaries: JobSummary[] = [
	{
		id: job.id,
		name: job.name,
		task: 'text_extraction',
		status: 'completed',
		created_at: job.created_at,
		updated_at: Date.now(),
		source_count: 1,
		setup_count: 2,
		pieces_total: 6,
		pieces_finished: 6,
		pieces_failed: 0
	},
	{
		id: '00000000-0000-0000-0000-0000000000j2',
		name: 'Entretiens Ouagadougou',
		task: 'audio_transcription',
		status: 'interrupted',
		created_at: Date.now() - 86_400_000,
		updated_at: Date.now() - 80_000_000,
		source_count: 4,
		setup_count: 1,
		pieces_total: 4,
		pieces_finished: 2,
		pieces_failed: 0
	}
];

const texts = [
	"Réception au Gouvernement général\nDakar, le 3 mai 1957\n\nLe Gouverneur général a reçu ce matin les délégués de la Mission catholique et de l'Union culturelle musulmane. L'entretien, qui s'est prolongé plus d'une heure, a porté sur l'enseignement dans les écoles de brousse et sur l'organisation du prochain pèlerinage.\nLes délégués ont remis au Gouverneur un mémoire signé par les notables de plusieurs cercles, demandant l'ouverture de classes du soir pour les adultes.\n\nUne réunion à Bamako\n\nLa réunion tenue à la mosquée de Bamako a rassemblé plus de deux cents fidèles autour de l'imam. Après la prière, l'assemblée a désigné une commission chargée de recueillir les cotisations pour la réfection du minaret.\nLe commandant de cercle assistait à la séance. Il a rappelé que les associations doivent déposer leurs statuts auprès de l'administration avant la fin du mois.",
	"La réunion tenue à la mosquée de Bamako a rassemblé plus de deux cents fidèles autour de l'imam.",
	'[BLANK PAGE]'
];
const variant = (t: string) =>
	t.replace('3 mai', '5 mai').replace('deux cents', 'trois cents').replace('minaret', 'marché');

const pieces: PieceView[] = [setupA, setupB].flatMap((setup, s) =>
	texts.map((text, index) => ({
		setup_id: setup.id,
		source_id: source.id,
		index,
		piece: { kind: 'page' as const, number: index + 1 },
		status: 'done' as const,
		error: null,
		ai_text: s === 0 ? text : variant(text),
		corrected_text: null,
		reviewed: index === 0 && s === 0,
		model_version: `${setup.model}-001`,
		cost_usd: 0.0012,
		segments: [],
		keywords: []
	}))
);

const recording = {
	...source,
	id: '00000000-0000-0000-0000-000000000002',
	path: 'C:\\Archives\\Entretiens\\imam-2024-03.m4a',
	file_name: 'imam-2024-03.m4a',
	kind: 'audio' as const,
	page_count: null,
	duration_ms: 2_520_000
};

const audioSetup = {
	...setupA,
	id: '00000000-0000-0000-0000-00000000000c',
	label: 'voxtral-small-transcribe-3-0',
	provider: 'mistral',
	model: 'voxtral-small-transcribe-3-0',
	options: { temperature: null, diarize: true, timestamps: true, speakers: null }
};

const audioJob: Job = {
	...job,
	id: '00000000-0000-0000-0000-0000000000j2',
	name: 'Entretiens Ouagadougou',
	task: 'audio_transcription',
	sources: [recording],
	setups: [audioSetup],
	status: 'interrupted'
};

const lines = [
	['Speaker 1', 'Bonjour, merci de nous recevoir. Pouvez-vous vous présenter ?'],
	['Speaker 2', "Je suis l'imam de la mosquée du quartier depuis 1998."],
	['Speaker 1', 'Comment la communauté a-t-elle changé depuis ?'],
	['Speaker 2', 'Beaucoup de jeunes sont venus de Bobo-Dioulasso et de Koudougou.'],
	['Speaker 2', "Nous avons ouvert une médersa en 2004, avec l'aide des familles."]
];
const audioPieces: PieceView[] = [0, 1].map((index) => ({
	setup_id: audioSetup.id,
	source_id: recording.id,
	index,
	piece: {
		kind: 'segment' as const,
		index,
		start_ms: index * 1_260_000,
		end_ms: (index + 1) * 1_260_000
	},
	status: 'done' as const,
	error: null,
	ai_text: lines.map(([who, text]) => `${who}: ${text}`).join('\n'),
	corrected_text: null,
	reviewed: false,
	model_version: 'voxtral-small-transcribe-3-0',
	cost_usd: 0.084,
	keywords: [],
	segments: lines.map(([speaker, text], k) => ({
		start_ms: k * 1_800,
		end_ms: (k + 1) * 1_800,
		speaker,
		text
	}))
}));

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
export function mockView(): View | null {
	switch (new URLSearchParams(location.search).get('view') ?? '') {
		case 'jobs':
			return { name: 'jobs', jobId: job.id };
		case 'review':
			return { name: 'review', jobId: job.id, sourceId: source.id };
		case 'review-audio':
			return { name: 'review', jobId: audioJob.id, sourceId: recording.id };
		case 'prompts':
			return { name: 'prompts' };
		case 'settings':
			return { name: 'settings' };
		case 'new':
			return { name: 'new' };
		default:
			return null;
	}
}

export function installMock(): void {
	const params = new URLSearchParams(location.search);
	if (params.get('focus') === 'apparatus') {
		setTimeout(() => document.querySelector<HTMLElement>('.apparatus .entry')?.focus(), 2500);
	}
	// `hover=deux`: rest the pointer on the lemma with that text, for the reverse highlight.
	const hover = params.get('hover');
	if (hover) {
		setTimeout(() => {
			const lemma = [...document.querySelectorAll('.lemma')].find((l) => l.textContent === hover);
			lemma?.dispatchEvent(new PointerEvent('pointerover', { bubbles: true }));
		}, 2500);
	}
	mockWindows('main');
	mockIPC((cmd, args) => {
		const a = (args ?? {}) as Record<string, unknown>;
		switch (cmd) {
			case 'app_info':
				return {
					version: '0.1.0',
					data_dir: 'C:\\Users\\you\\AppData\\Local\\io.github.fmadore.scriptorium',
					logs_dir: '',
					interrupted: []
				};
			case 'get_settings':
				return settings;
			case 'get_catalog':
				return catalog;
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
			case 'list_presets':
				return a.task ? presets.filter((p) => p.task === a.task) : presets;
			case 'list_jobs':
				return summaries;
			case 'get_job':
				return a.id === audioJob.id ? audioJob : job;
			case 'job_pieces':
				return pieces.map((p) => ({
					...p,
					job_id: job.id,
					cache_key: 'k',
					attempts: 1,
					updated_at: 0
				}));
			case 'source_pieces':
				return a.jobId === audioJob.id ? audioPieces : pieces;
			case 'piece_audio':
				return toneWav();
			case 'piece_preview':
				// A synthetic scan as an image: shown everywhere, headless browsers included.
				return { mime: 'image/jpeg', data: SAMPLE_SCAN.split(',')[1] ?? '' };
			case 'compare_source':
				return {
					scores: [
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
					],
					pages: []
				};
			case 'diff_texts':
				return wordDiff(String(a.left ?? ''), String(a.right ?? ''));
			case 'estimate_cost':
				return (a.request as { setups: { provider: string; model: string }[] }).setups.map((s) => ({
					provider: s.provider,
					model: s.model,
					pages: 120,
					estimate: { usd: s.provider === 'mistral' ? null : 0.42, note: null }
				}));
			case 'expand_paths':
				return a.paths;
			case 'inspect_files':
				return (a.paths as string[]).map((path, i) => ({
					path,
					source: {
						...source,
						id: `00000000-0000-0000-0000-0000000001${i}0`,
						path,
						file_name: path.split('\\').pop()
					},
					warnings:
						i === 0
							? [
									{ source_id: source.id, kind: 'double_page_spread', pages: [2, 3] },
									{ source_id: source.id, kind: 'low_resolution', pages: [1], dpi: 150 }
								]
							: [],
					error: null
				}));
			case 'plugin:dialog|open':
				return [
					'C:\\Archives\\Bulletin\\bulletin-1957-06.pdf',
					'C:\\Archives\\Letters\\lettre-gouverneur.jpg'
				];
			default:
				return null;
		}
	});
}
