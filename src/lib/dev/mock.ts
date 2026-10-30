// A pretend back end for looking at the interface in a plain browser during development
// (`npm run dev`, then open http://localhost:5173/?mock). It answers commands with sample data
// through Tauri's own IPC mock. Never loaded in the desktop app or in a production build.

import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { Pipeline } from '../bindings/Pipeline.js';
import type { Catalog } from '../bindings/Catalog.js';
import type { Job } from '../bindings/Job.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { Preset } from '../bindings/Preset.js';
import type { Settings } from '../bindings/Settings.js';
import type { TaskRules } from '../bindings/TaskRules.js';
import { layout } from '../review.svelte.js';
import type { View } from '../state.svelte.js';
import catalogJson from '../../../crates/catalog/catalog.json';
import SAMPLE_SCAN from './sample-scan.jpg?inline';
import taskRulesJson from './taskRules.json';
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
	export_options: { piece_markers: true, speaker_labels: true, timestamps: true },
	theme: new URLSearchParams(location.search).get('theme') ?? 'system',
	refresh_catalog: false,
	favourite_languages: ['fr', 'ar'],
	last_folder: null
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
	settings: {
		languages: { mode: 'explicit', languages: ['fr'] },
		sample_pages: null,
		line_breaks: 'running_text',
		pace: 'fast'
	},
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
		pieces_failed: 0,
		pieces_truncated: 1,
		cost_usd: 0.0162
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
		pieces_failed: 0,
		pieces_truncated: 0,
		cost_usd: 0.02
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
		ai_fields: [],
		corrected_fields: null,
		speakers: [],
		speaker_names: {},
		source_text: null,
		truncated: s === 0 && index === 2
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
/** The names given in the preview's review, applied as the engine applies them. */
let speakerNames: Record<string, string> = {};
const named = (label: string) => speakerNames[label]?.trim() || label;

const audioPieces = (): PieceView[] =>
	[0, 1].map((index) => ({
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
		ai_text: lines.map(([who, text]) => `${named(who)}: ${text}`).join('\n'),
		corrected_text: null,
		reviewed: false,
		model_version: 'voxtral-small-transcribe-3-0',
		cost_usd: 0.084,
		ai_fields: [],
		corrected_fields: null,
		segments: lines.map(([speaker, text], k) => ({
			start_ms: k * 1_800,
			end_ms: (k + 1) * 1_800,
			speaker: named(speaker),
			text
		})),
		speakers: ['Speaker 1', 'Speaker 2'],
		speaker_names: { ...speakerNames },
		source_text: null,
		truncated: false
	}));

/** A YouTube video transcribed by Gemini, its times written into the text as Gemini does. */
const video = {
	...recording,
	id: '00000000-0000-0000-0000-000000000005',
	path: 'https://www.youtube.com/watch?v=dQw4w9WgXcQ',
	file_name: 'Conférence sur le jeûne, mosquée de Ouagadougou [dQw4w9WgXcQ]',
	title: 'Conférence sur le jeûne, mosquée de Ouagadougou',
	kind: 'youtube' as const,
	size_bytes: 0,
	duration_ms: null
};

const videoSetup = {
	...audioSetup,
	id: '00000000-0000-0000-0000-00000000000d',
	label: 'gemini-3.8-flash',
	provider: 'gemini',
	model: 'gemini-3.8-flash'
};

const videoJob: Job = {
	...audioJob,
	id: '00000000-0000-0000-0000-0000000000j4',
	name: 'Conférence, mosquée de Ouagadougou',
	sources: [video],
	setups: [videoSetup],
	status: 'completed'
};

const videoPieces = (): PieceView[] => [
	{
		...audioPieces()[0],
		setup_id: videoSetup.id,
		source_id: video.id,
		piece: { kind: 'whole' as const },
		model_version: 'gemini-3.8-flash-001',
		cost_usd: 0.031,
		segments: lines.map(([speaker, text], k) => ({
			start_ms: k * 95_000 + 4_000,
			end_ms: (k + 1) * 95_000 + 4_000,
			speaker: named(speaker),
			text
		}))
	}
];

/** An analysis of one article by two models, for the analysis review. */
const article = {
	...source,
	id: '00000000-0000-0000-0000-000000000003',
	path: 'C:\\Archives\\Bulletin\\bulletin-1957-05.txt',
	file_name: 'bulletin-1957-05.txt',
	kind: 'text' as const,
	page_count: null,
	duration_ms: null
};

const analysisSchema = {
	fields: [
		{
			key: 'summary',
			label: 'Summary',
			instructions: 'What this document in particular says.',
			kind: { type: 'text' as const },
			reason: false
		},
		{
			key: 'people',
			label: 'People',
			instructions: 'People named in the document.',
			kind: { type: 'list' as const, max: 30 },
			reason: false
		},
		{
			key: 'places',
			label: 'Places',
			instructions: 'Places named in the document.',
			kind: { type: 'list' as const, max: 20 },
			reason: false
		},
		{
			key: 'tone',
			label: 'Tone',
			instructions: 'How the document presents its main subject.',
			kind: { type: 'choice' as const, values: ['Positive', 'Neutral', 'Negative', 'Mixed'] },
			reason: true
		}
	]
};

const analysisSetups = [setupA, setupB].map((s, i) => ({
	...s,
	id: `00000000-0000-0000-0000-00000000000${i ? 'e' : 'd'}`,
	prompt: { ...s.prompt, preset_name: 'People, places and tone', schema: analysisSchema }
}));

const analysisJob: Job = {
	...job,
	id: '00000000-0000-0000-0000-0000000000j3',
	name: 'Bulletin, people and places',
	task: 'analysis',
	sources: [article],
	setups: analysisSetups
};

const analysisAnswers = [
	[
		{
			key: 'summary',
			value:
				"Le Gouverneur général reçoit à Dakar les délégués de la Mission catholique et de l'Union culturelle musulmane sur l'enseignement et le pèlerinage ; à Bamako, deux cents fidèles désignent une commission pour la réfection du minaret."
		},
		{ key: 'people', value: ['Gouverneur général', 'imam'] },
		{
			key: 'places',
			value: ['Dakar', 'Bamako']
		},
		{
			key: 'tone',
			value: 'Neutral',
			reason: 'The bulletin reports the meetings without judging them.'
		}
	],
	[
		{
			key: 'summary',
			value:
				'Audience du Gouverneur général avec les délégués catholiques et musulmans à Dakar, et réunion de deux cents fidèles à la mosquée de Bamako.'
		},
		{ key: 'people', value: ['Gouverneur général', 'commandant de cercle'] },
		{
			key: 'places',
			value: ['Dakar', 'Bamako', 'mosquée de Bamako']
		},
		{
			key: 'tone',
			value: 'Positive',
			reason: 'The meetings are described as cordial and constructive.'
		}
	]
];

const analysisPieces: PieceView[] = analysisSetups.map((s, i) => ({
	setup_id: s.id,
	source_id: article.id,
	index: 0,
	piece: { kind: 'whole' as const },
	status: 'done' as const,
	error: null,
	ai_text: '',
	corrected_text: null,
	reviewed: i === 0,
	model_version: `${s.model}-001`,
	cost_usd: 0.004,
	segments: [],
	ai_fields: analysisAnswers[i],
	corrected_fields: null,
	speakers: [],
	speaker_names: {},
	source_text: null,
	truncated: false
}));

/** Saved pipelines, as New job lists them. */
let pipelines: Pipeline[] = [
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
	mockIPC((cmd, args) => {
		const a = (args ?? {}) as Record<string, unknown>;
		switch (cmd) {
			case 'app_info':
				return {
					version: '0.1.0',
					data_dir: 'C:\\Users\\you\\AppData\\Local\\io.github.fmadore.kalamu',
					logs_dir: '',
					interrupted: []
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
				const ordered = [...presets, ...contentsPresets].sort(
					(x, y) => Number(y.task === first) - Number(x.task === first)
				);
				return a.task ? ordered.filter((p) => p.task === a.task) : ordered;
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
				speakerNames = { ...(a.names as { names: Record<string, string> }).names };
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
			case 'estimate_cost':
				return (a.request as { setups: { provider: string; model: string }[] }).setups.map((s) => ({
					provider: s.provider,
					model: s.model,
					pages: 120,
					estimate: {
						usd: s.provider === 'mistral' ? null : 0.42,
						economy_usd: s.provider === 'mistral' ? null : 0.21,
						note: null
					}
				}));
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
			default:
				return null;
		}
	});
	// `piece_audio` names a blob URL here, not a file, so it is its own address.
	(
		window as unknown as { __TAURI_INTERNALS__: { convertFileSrc: (path: string) => string } }
	).__TAURI_INTERNALS__.convertFileSrc = (path) => path;
}
