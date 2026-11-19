// An interview analysed for the browser preview (`?mock&view=review-interview`): an invented
// conversation in French between Claire, who asks, and Mariam, a tailor, transcribed with
// times, frozen and read for its themes, keywords and quotations, each quotation placed in the
// frozen text in characters as the engine places it. `interview-gone` makes the recording
// unreachable, so quotations are shown in the text instead; `interview-two` adds a second
// set-up. Development builds only.

import type { CitedEdit } from '../bindings/CitedEdit.js';
import type { CitedEvidence } from '../bindings/CitedEvidence.js';
import type { CitedValue } from '../bindings/CitedValue.js';
import type { FieldAnswer } from '../bindings/FieldAnswer.js';
import type { Job } from '../bindings/Job.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { NewJob } from '../bindings/NewJob.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { Turn } from '../bindings/Turn.js';
import { clock } from '../pieces.js';
import { audioSetup, EXPORT_OPTIONS, job, source } from './mockJobs.js';

export const INTERVIEW_JOB = '00000000-0000-0000-0000-0000000000i1';
const TRANSCRIPT_JOB = '00000000-0000-0000-0000-0000000000i2';

const said: [string, number, string][] = [
	[
		'Claire',
		4_000,
		'Bonjour Mariam, merci de me recevoir dans votre atelier. Pouvez-vous me dire comment vous avez commencé ?'
	],
	[
		'Mariam',
		15_000,
		'J’ai appris la couture chez ma tante, à seize ans. Pendant trois ans, je balayais l’atelier et je regardais ses mains.'
	],
	['Claire', 62_000, 'Et l’atelier, vous l’avez ouvert quand ?'],
	[
		'Mariam',
		69_000,
		'En 2015, avec une seule machine à pédale. Les premiers mois, mes clientes étaient mes voisines, et elles payaient souvent à la fin du mois.'
	],
	['Claire', 151_000, 'Comment avez-vous trouvé l’argent pour la deuxième machine ?'],
	[
		'Mariam',
		158_000,
		'Grâce à la tontine du quartier. Nous sommes douze femmes, chacune met deux mille francs chaque semaine, et chaque semaine l’une de nous prend tout. الحمد لله, mon tour est venu au bon moment.'
	],
	['Claire', 252_000, 'Est-ce que la tontine sert à autre chose qu’à l’argent ?'],
	[
		'Mariam',
		260_000,
		'Oui ! C’est là qu’on parle des commandes, des prix du tissu, des enfants. Quand une femme est malade, les autres tiennent son étal (rires 😄).'
	],
	['Claire', 365_000, 'Vous formez des apprenties à votre tour ?'],
	[
		'Mariam',
		371_000,
		'J’en ai trois en ce moment. Je leur dis ce que ma tante me disait : une couture droite, c’est une cliente qui revient.'
	],
	['Claire', 520_000, 'Qu’est-ce qui a le plus changé depuis vos débuts ?'],
	[
		'Mariam',
		527_000,
		'Les téléphones. Maintenant les clientes m’envoient la photo d’un modèle, et je leur réponds avec le prix avant même qu’elles viennent.'
	],
	['Claire', 690_000, 'Merci beaucoup, Mariam.']
];
const END_MS = 700_000;

/** Characters, as the engine counts them (an emoji is one). */
const chars = (s: string) => Array.from(s).length;

/** The frozen transcript: one line per turn, its time and speaker before the words. */
const lines = said.map(([who, ms, words]) => `[${clock(ms)}] ${who}: ${words}`);
export const interviewText = lines.join('\n');

/** Each turn's words in the frozen text, in characters, with its times. */
export const interviewTurns: Turn[] = (() => {
	let at = 0;
	return said.map(([who, ms, words], i) => {
		const start = at + chars(lines[i]) - chars(words);
		at += chars(lines[i]) + 1;
		return {
			start,
			end: start + chars(words),
			speaker: who,
			start_ms: ms,
			end_ms: said[i + 1]?.[1] ?? END_MS
		};
	});
})();

/** A quotation of turn `n` (from 1), placed as the engine places it. */
function quote(n: number, words: string): CitedEvidence {
	const turn = interviewTurns[n - 1];
	const [, , text] = said[n - 1];
	const at = text.indexOf(words);
	if (at < 0) throw new Error(`not in turn ${n}: ${words}`);
	const start = turn.start + chars(text.slice(0, at));
	return {
		anchor: { part: 0, start, end: start + chars(words) },
		quote: words,
		turn: n,
		speaker: turn.speaker,
		start_ms: turn.start_ms,
		end_ms: turn.end_ms
	};
}

const text = (key: string, value: string): FieldAnswer => ({ key, value });

const themes: CitedValue = {
	entries: [
		{
			answers: [
				text('name', 'Apprendre en regardant'),
				text(
					'description',
					'Mariam a appris le métier chez sa tante, d’abord en observant, et transmet la même exigence à ses apprenties.'
				)
			],
			evidence: [
				quote(2, 'Pendant trois ans, je balayais l’atelier et je regardais ses mains.'),
				quote(10, 'une couture droite, c’est une cliente qui revient.')
			]
		},
		{
			answers: [
				text('name', 'La tontine comme appui'),
				text(
					'description',
					'La tontine du quartier a payé la deuxième machine ; c’est aussi un lieu où les femmes s’entraident.'
				)
			],
			evidence: [
				quote(6, 'chaque semaine l’une de nous prend tout'),
				quote(6, 'mon tour est venu au bon moment'),
				quote(8, 'Quand une femme est malade, les autres tiennent son étal')
			]
		},
		{
			answers: [
				text('name', 'Le téléphone et les clientes'),
				text(
					'description',
					'Les commandes passent désormais par des photos envoyées sur le téléphone.'
				)
			],
			evidence: [quote(12, 'les clientes m’envoient la photo d’un modèle')]
		}
	],
	left_out: 1,
	dropped: 0
};

const quotations: CitedValue = {
	entries: [
		{
			answers: [text('why_it_matters', 'Sa façon de voir le métier, en une phrase.')],
			evidence: [quote(10, 'une couture droite, c’est une cliente qui revient.')]
		},
		{
			answers: [text('why_it_matters', 'Le crédit accordé aux premières clientes.')],
			evidence: [quote(4, 'elles payaient souvent à la fin du mois')]
		}
	],
	left_out: 0,
	dropped: 0
};

const field = (key: string, label: string, kind: unknown) => ({
	key,
	label,
	instructions: '',
	kind,
	reason: false
});

/** The schema of the preset `analysis.interview`, as the engine gives it keys. */
export const interviewSchema = {
	fields: [
		field('summary', 'Summary', { type: 'text' }),
		field('themes', 'Themes', {
			type: 'cited',
			fields: [
				field('name', 'Name', { type: 'text' }),
				field('description', 'Description', { type: 'text' })
			],
			max: 8
		}),
		field('keywords', 'Keywords', { type: 'list', max: 12 }),
		field('quotations', 'Quotations', {
			type: 'cited',
			fields: [field('why_it_matters', 'Why it matters', { type: 'text' })],
			max: 10
		})
	]
} as NonNullable<Job['setups'][number]['prompt']['schema']>;

const recording = {
	...source,
	id: '00000000-0000-0000-0000-0000000000i4',
	path: 'C:\\Entretiens\\atelier-mariam-2024.m4a',
	file_name: 'atelier-mariam-2024.m4a',
	kind: 'audio' as const,
	page_count: null,
	duration_ms: END_MS
};
const frozen = {
	...source,
	id: '00000000-0000-0000-0000-0000000000i3',
	path: 'C:\\Users\\you\\AppData\\Local\\Kalamu Data\\snapshots\\cd.txt',
	file_name: 'atelier-mariam-2024.txt',
	kind: 'text' as const,
	page_count: null,
	duration_ms: null,
	derived_from: {
		snapshot_id: 'snapshot-interview',
		job_id: TRANSCRIPT_JOB,
		setup_id: audioSetup.id,
		source_id: recording.id,
		original_path: recording.path
	}
};

const transcriptJob: Job = {
	...job,
	id: TRANSCRIPT_JOB,
	name: 'Entretien, atelier de couture',
	task: 'audio_transcription',
	sources: [recording],
	setups: [audioSetup],
	status: 'completed'
};

const interviewSetup = (i: number) => ({
	id: `00000000-0000-0000-0000-0000000000s${i}`,
	label: i === 0 ? 'gemini-3.8-flash' : 'claude-sonnet-5-5',
	provider: i === 0 ? 'gemini' : 'anthropic',
	model: i === 0 ? 'gemini-3.8-flash' : 'claude-sonnet-5-5',
	prompt: {
		preset_id: 'analysis.interview',
		preset_name: 'Interview: themes and quotations',
		template: '',
		text: '',
		user_prompt: '',
		schema: interviewSchema
	},
	options: {
		temperature: null,
		diarize: false,
		timestamps: false,
		speakers: null,
		question: 'Comment les femmes artisanes financent et apprennent leur métier.'
	}
});

const two = () => new URLSearchParams(location.search).has('interview-two');

export const interviewJob: Job = {
	...job,
	id: INTERVIEW_JOB,
	name: 'Analyse d’entretien : atelier-mariam-2024.txt',
	task: 'analysis',
	sources: [frozen],
	setups: [interviewSetup(0), interviewSetup(1)],
	output: {
		formats: ['markdown', 'json'],
		destination: { kind: 'beside_sources' },
		options: EXPORT_OPTIONS
	},
	status: 'completed'
};

const answers = (i: number): FieldAnswer[] => [
	text(
		'summary',
		'Claire interroge Mariam, couturière, sur ses débuts chez sa tante, l’ouverture de son atelier en 2015, la tontine de son quartier et ses apprenties.'
	),
	{
		key: 'themes',
		value: i === 0 ? themes : { ...themes, entries: themes.entries.slice(1), left_out: 0 }
	},
	{ key: 'keywords', value: ['couture', 'tontine', 'apprentissage', 'téléphone'] },
	{ key: 'quotations', value: quotations }
];

export const interviewPieces: PieceView[] = [0, 1].map((i) => ({
	setup_id: interviewSetup(i).id,
	source_id: frozen.id,
	index: 0,
	piece: { kind: 'whole' as const },
	status: 'done' as const,
	error: null,
	ai_text: '',
	corrected_text: null,
	reviewed: false,
	model_version: `${interviewSetup(i).model}-001`,
	cost_usd: 0.006,
	segments: [],
	ai_fields: answers(i),
	corrected_fields: null,
	speakers: [],
	speaker_names: {},
	source_text: null,
	truncated: false
}));

/** The transcript, in two pieces of six minutes, each turn timed from its piece's start. */
const transcriptPieces = (): PieceView[] =>
	[0, 1].map((index) => {
		const from = index * 360_000;
		const turns = said.filter(([, ms]) => ms >= from && ms < from + 360_000);
		return {
			setup_id: audioSetup.id,
			source_id: recording.id,
			index,
			piece: { kind: 'segment' as const, index, start_ms: from, end_ms: from + 360_000 },
			status: 'done' as const,
			error: null,
			ai_text: turns.map(([who, , words]) => `${who}: ${words}`).join('\n'),
			corrected_text: null,
			reviewed: true,
			model_version: audioSetup.model,
			cost_usd: 0.03,
			ai_fields: [],
			corrected_fields: null,
			segments: turns.map(([who, ms, words]) => {
				const k = said.findIndex(([, at]) => at === ms);
				return {
					start_ms: ms - from,
					end_ms: (said[k + 1]?.[1] ?? END_MS) - from,
					speaker: who,
					text: words
				};
			}),
			speakers: ['Claire', 'Mariam'],
			speaker_names: {},
			source_text: null,
			truncated: false
		};
	});

/** The two jobs as the list shows them. */
export function interviewSummaries(base: JobSummary): JobSummary[] {
	return [
		{
			...base,
			id: TRANSCRIPT_JOB,
			name: transcriptJob.name,
			task: 'audio_transcription',
			pieces_total: 2,
			pieces_finished: 2,
			setup_count: 1,
			workflow: null
		},
		{
			...base,
			id: INTERVIEW_JOB,
			name: interviewJob.name,
			task: 'analysis',
			pieces_total: 1,
			pieces_finished: 1,
			setup_count: two() ? 2 : 1,
			workflow: null
		}
	];
}

/** One change to a cited field, as `core::analysis::cited::edit` makes it. */
export function editCited(answers: FieldAnswer[], key: string, change: CitedEdit): FieldAnswer[] {
	const out: FieldAnswer[] = structuredClone(answers);
	const value = out.find((a) => a.key === key)?.value;
	if (typeof value !== 'object' || value === null || Array.isArray(value))
		throw { id: 'cited.not_cited', detail: key };
	const entry = value.entries[change.entry];
	if (!entry) throw { id: 'cited.no_entry', detail: String(change.entry) };
	if (change.kind === 'remove_entry') value.entries.splice(change.entry, 1);
	else if (change.kind === 'remove_quotation') {
		if (!entry.evidence[change.quotation])
			throw { id: 'cited.no_quotation', detail: String(change.quotation) };
		entry.evidence.splice(change.quotation, 1);
	} else {
		const kept = entry.answers.find((a) => a.key === change.key);
		if (kept) kept.value = change.value;
		else entry.answers.push({ key: change.key, value: change.value });
	}
	return out;
}

/** The job New job's dialog created last, for the end-to-end tests to read. */
declare global {
	interface Window {
		__kalamuCreated?: NewJob;
	}
}

export function interviewCommand(cmd: string, a: Record<string, unknown>): unknown {
	const mine = a.jobId === INTERVIEW_JOB || a.id === INTERVIEW_JOB;
	switch (cmd) {
		case 'get_job':
			if (a.id === TRANSCRIPT_JOB) return transcriptJob;
			if (a.id === INTERVIEW_JOB)
				return two() ? interviewJob : { ...interviewJob, setups: interviewJob.setups.slice(0, 1) };
			return undefined;
		case 'source_pieces':
			if (a.jobId === TRANSCRIPT_JOB) return transcriptPieces();
			return mine ? interviewPieces.slice(0, two() ? 2 : 1) : undefined;
		case 'source_text':
			return mine ? interviewText : undefined;
		case 'cited_source':
			return mine
				? {
						snapshot_id: 'snapshot-interview',
						sha256: 'cd'.repeat(32),
						text: interviewText,
						turns: interviewTurns
					}
				: undefined;
		case 'derived_state':
			if (!mine) return undefined;
			return [
				{
					source_id: frozen.id,
					from: frozen.derived_from,
					freshness: new URLSearchParams(location.search).has('interview-gone') ? 'gone' : 'current'
				}
			];
		case 'edit_cited': {
			if (!mine) return undefined;
			const piece = interviewPieces.find((p) => p.setup_id === a.setupId);
			if (!piece) throw { id: 'job.not_found' };
			piece.corrected_fields = editCited(
				piece.corrected_fields ?? piece.ai_fields,
				String(a.key),
				a.change as CitedEdit
			);
			return null;
		}
		case 'create_job': {
			const asked = a.job as NewJob;
			// The interview prompt, or a copy of it keeping a description of the research.
			const preset = asked.setups[0]?.preset_id ?? '';
			if (preset !== 'analysis.interview' && !preset.startsWith('user-')) return undefined;
			window.__kalamuCreated = asked;
			return { ...interviewJob, setups: interviewJob.setups.slice(0, 1) };
		}
		default:
			return undefined;
	}
}
