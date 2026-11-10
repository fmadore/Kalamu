// The sample jobs the development mock answers with: a newspaper read by two set-ups, a
// recording, a YouTube video and an analysis.

import type { Correction } from '../bindings/Correction.js';
import type { Job } from '../bindings/Job.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { PieceView } from '../bindings/PieceView.js';

/** What the sample settings export with, which the sample jobs carry. */
export const EXPORT_OPTIONS = { piece_markers: true, speaker_labels: true, timestamps: true };

export const source = {
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

export const setupA = {
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
export const setupB = {
	...setupA,
	id: '00000000-0000-0000-0000-00000000000b',
	label: 'claude-opus-5-5',
	provider: 'anthropic',
	model: 'claude-opus-5-5'
};

export const job: Job = {
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
		options: EXPORT_OPTIONS
	},
	status: 'completed'
};

export const summaries: JobSummary[] = [
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
		cost_usd: 0.0162,
		models: ['gemini-3.8-flash'],
		workflow: 'Bulletins: text, then people and places'
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
		cost_usd: 0.02,
		models: ['voxtral-mini-transcribe-2'],
		workflow: null
	}
];

/** Older finished jobs, enough for the sidebar to list only the most recent and link to the
 *  Jobs page; each with the names of its files, for the page's search. */
const ARCHIVE: [string, JobSummary['task'], string][] = [
	['Fonds Cissé, carton 3', 'text_extraction', 'cisse-carton3'],
	['Prêches du vendredi, Bobo-Dioulasso', 'audio_transcription', 'preche'],
	['Journal officiel AOF, 1946', 'text_extraction', 'jo-aof-1946'],
	['Entretien avec El Hadj Moussa', 'audio_transcription', 'moussa'],
	['Correspondance du cercle de Kankan', 'text_extraction', 'kankan'],
	['Revue Islam et Société, nº 12', 'contents', 'islam-societe-12'],
	['Conférences Ramadan 2023', 'audio_transcription', 'ramadan-2023'],
	['Rapports politiques, Niamey, 1952', 'text_extraction', 'niamey-1952'],
	['Personnes et lieux, bulletins', 'analysis', 'bulletins'],
	['Manuscrits ajami, lot 2', 'text_extraction', 'ajami-lot2'],
	['Radio Al Houda, émissions', 'audio_transcription', 'al-houda'],
	['Le Réveil, 1950–1951', 'contents', 'reveil'],
	['Notes de terrain, Korhogo', 'text_extraction', 'korhogo'],
	['Fatwas, recueil Diallo', 'text_correction', 'diallo-fatwas'],
	['Archives de la mission, Lomé', 'text_extraction', 'lome-mission'],
	['Entretiens Cotonou, série B', 'audio_transcription', 'cotonou-b'],
	['Ton des articles, 1957', 'analysis', 'ton-1957'],
	['Tracts électoraux, 1958', 'text_extraction', 'tracts-1958'],
	['Sermons du Maouloud', 'audio_transcription', 'maouloud'],
	['Registres scolaires, médersa', 'text_extraction', 'medersa'],
	['Témoignages de pèlerins', 'audio_transcription', 'pelerins'],
	['Bulletin Afrique-Information', 'text_extraction', 'afrique-info'],
	['Circulaires du gouverneur', 'text_extraction', 'circulaires'],
	['Lettres de Cerno Bokar', 'text_extraction', 'cerno-bokar']
];

export const archiveJobs: (JobSummary & { files: string[] })[] = ARCHIVE.map(
	([name, task, stem], i) => {
		const sources = 1 + ((i * 7) % 5);
		const pieces = sources * (2 + (i % 4));
		const failed = i % 9 === 4 ? 2 : 0;
		const ext = task === 'audio_transcription' ? (i % 2 ? 'mp4' : 'm4a') : 'pdf';
		return {
			id: `00000000-0000-0000-0000-0000000a${String(i).padStart(4, '0')}`,
			name,
			task,
			status: failed > 0 ? 'completed_with_errors' : i === 15 ? 'cancelled' : 'completed',
			created_at: Date.now() - (2 + i * 3) * 86_400_000,
			updated_at: Date.now() - (2 + i * 3) * 86_400_000 + 3_600_000,
			source_count: sources,
			setup_count: 1,
			pieces_total: pieces,
			pieces_finished: pieces - failed,
			pieces_failed: failed,
			pieces_truncated: 0,
			cost_usd: task === 'text_correction' ? null : 0.004 * pieces,
			models: [task === 'audio_transcription' ? 'voxtral-mini-transcribe-2' : 'gemini-3.8-flash'],
			workflow: null,
			files: Array.from({ length: sources }, (_, n) => `${stem}-${n + 1}.${ext}`)
		};
	}
);

export const texts = [
	"Réception au Gouvernement général\nDakar, le 3 mai 1957\n\nLe Gouverneur général a reçu ce matin les délégués de la Mission catholique et de l'Union culturelle musulmane. L'entretien, qui s'est prolongé plus d'une heure, a porté sur l'enseignement dans les écoles de brousse et sur l'organisation du prochain pèlerinage.\nLes délégués ont remis au Gouverneur un mémoire signé par les notables de plusieurs cercles, demandant l'ouverture de classes du soir pour les adultes.\n\nUne réunion à Bamako\n\nLa réunion tenue à la mosquée de Bamako a rassemblé plus de deux cents fidèles autour de l'imam. Après la prière, l'assemblée a désigné une commission chargée de recueillir les cotisations pour la réfection du minaret.\nLe commandant de cercle assistait à la séance. Il a rappelé que les associations doivent déposer leurs statuts auprès de l'administration avant la fin du mois.",
	"La réunion tenue à la mosquée de Bamako a rassemblé plus de deux cents fidèles autour de l'imam.",
	'[BLANK PAGE]'
];
const variant = (t: string) =>
	t.replace('3 mai', '5 mai').replace('deux cents', 'trois cents').replace('minaret', 'marché');

export const pieces: PieceView[] = [setupA, setupB].flatMap((setup, s) =>
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

export const recording = {
	...source,
	id: '00000000-0000-0000-0000-000000000002',
	path: 'C:\\Archives\\Entretiens\\imam-2024-03.m4a',
	file_name: 'imam-2024-03.m4a',
	kind: 'audio' as const,
	page_count: null,
	duration_ms: 2_520_000
};

export const audioSetup = {
	...setupA,
	id: '00000000-0000-0000-0000-00000000000c',
	label: 'voxtral-small-transcribe-3-0',
	provider: 'mistral',
	model: 'voxtral-small-transcribe-3-0',
	options: { temperature: null, diarize: true, timestamps: true, speakers: null }
};

export const audioJob: Job = {
	...job,
	id: '00000000-0000-0000-0000-0000000000j2',
	name: 'Entretiens Ouagadougou',
	task: 'audio_transcription',
	sources: [recording],
	setups: [audioSetup],
	status: 'interrupted'
};

export const lines = [
	['Speaker 1', 'Bonjour, merci de nous recevoir. Pouvez-vous vous présenter ?'],
	['Speaker 2', "Je suis l'imam de la mosquée du quartier depuis 1998."],
	['Speaker 1', 'Comment la communauté a-t-elle changé depuis ?'],
	['Speaker 2', 'Beaucoup de jeunes sont venus de Bobo-Dioulasso et de Koudougou.'],
	['Speaker 2', "Nous avons ouvert une médersa en 2004, avec l'aide des familles."]
];
/** The names given in the preview's review, applied as the engine applies them. */
let speakerNames: Record<string, string> = {};
/** The names the researcher gave the recording's speakers. */
export function nameSpeakers(names: Record<string, string>): void {
	speakerNames = { ...names };
}
export const named = (label: string) => speakerNames[label]?.trim() || label;

export const audioPieces = (): PieceView[] =>
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
export const video = {
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

export const videoJob: Job = {
	...audioJob,
	id: '00000000-0000-0000-0000-0000000000j4',
	name: 'Conférence, mosquée de Ouagadougou',
	sources: [video],
	setups: [videoSetup],
	status: 'completed'
};

export const videoPieces = (): PieceView[] => [
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
export const article = {
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

export const analysisJob: Job = {
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

export const analysisPieces: PieceView[] = analysisSetups.map((s, i) => ({
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

/** What `rename_listed` gives back: the corrections, as they stood, of the analysis set-ups
 * whose answer to the field holds the name. The preview keeps no correction, as with
 * `save_correction`. */
export function listedBefore(a: Record<string, unknown>): Correction[] {
	const same = (v: unknown) =>
		typeof v === 'string' && v.trim().toLowerCase() === String(a.from).trim().toLowerCase();
	return analysisPieces
		.filter((v) => v.source_id === a.sourceId && v.index === a.index && v.status !== 'failed')
		.filter((v) => {
			const value = (v.corrected_fields ?? v.ai_fields).find((f) => f.key === a.key)?.value;
			return Array.isArray(value) ? value.some(same) : same(value);
		})
		.map((v) => ({
			job_id: analysisJob.id,
			setup_id: v.setup_id,
			source_id: v.source_id,
			index: v.index,
			text: v.corrected_text,
			fields: v.corrected_fields ?? undefined,
			reviewed: v.reviewed,
			updated_at: 0
		}));
}
