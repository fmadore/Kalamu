// Sample data for a table of contents and a correction, for the pretend back end (`mock.ts`).

import type { Field } from '../bindings/Field.js';
import type { FieldAnswer } from '../bindings/FieldAnswer.js';
import type { Job } from '../bindings/Job.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { Preset } from '../bindings/Preset.js';

const text = (key: string, label: string): Field => ({
	key,
	label,
	instructions: '',
	kind: { type: 'text' },
	reason: false
});
const authors: Field = {
	key: 'authors',
	label: 'Authors',
	instructions: '',
	kind: { type: 'list', max: null },
	reason: false
};
const entryFields = [authors, text('summary', 'Summary')];

export const contentsPresets: Preset[] = [
	{
		id: 'contents.periodical',
		task: 'contents',
		name: 'Newspaper or magazine issue',
		description: 'One entry per article, with its section, authors, pages and a summary.',
		instructions: '# Articles on one page of a newspaper or magazine issue…',
		user_prompt: 'List the articles or items on this page and fill in their fields.',
		builtin: true,
		modified: false,
		schema: { fields: entryFields },
		merge_instructions: '# A detailed table of contents of a newspaper or magazine issue…'
	},
	{
		id: 'contents.archival_file',
		task: 'contents',
		name: 'Archival file, item by item',
		description: 'Each letter, report or clipping with its date, sender, addressee and a summary.',
		instructions: '# Items on one page of an archival file…',
		user_prompt: 'List the articles or items on this page and fill in their fields.',
		builtin: true,
		modified: false,
		schema: { fields: [text('date', 'Date'), text('summary', 'Summary')] },
		merge_instructions: '# An item-by-item inventory of an archival file…'
	},
	{
		id: 'text_correction.general',
		task: 'text_correction',
		name: 'Printed document',
		description:
			'Corrects machine-reading errors only; spelling, wording and style of the time kept.',
		instructions: '# OCR correction…',
		user_prompt: 'Correct the machine-read text above, following the instructions given.',
		builtin: true,
		modified: false
	}
];

const issue = {
	id: '00000000-0000-0000-0000-0000000000c1',
	path: 'C:\\Archives\\An-Nasr\\an-nasr-12.pdf',
	file_name: 'an-nasr-12.pdf',
	sha256: 'cd'.repeat(32),
	size_bytes: 9_400_000,
	kind: 'pdf' as const,
	page_count: 3,
	duration_ms: null,
	split_spreads: false,
	even_out_volume: false
};

const prompt = (preset: string) => ({
	preset_id: preset,
	preset_name: 'Newspaper or magazine issue',
	template: '',
	text: '',
	user_prompt: ''
});

export const contentsJob: Job = {
	id: '00000000-0000-0000-0000-0000000000j5',
	name: 'An-Nasr, issue 12',
	task: 'contents',
	created_at: Date.now() - 1_800_000,
	sources: [issue],
	setups: [
		{
			id: '00000000-0000-0000-0000-0000000000c2',
			label: 'gemini-3.1-pro-preview',
			provider: 'gemini',
			model: 'gemini-3.1-pro-preview',
			prompt: prompt('contents.periodical'),
			options: { temperature: null, diarize: false, timestamps: false, speakers: null },
			merge: {
				provider: 'mistral',
				model: 'mistral-small-2603',
				prompt: {
					...prompt('contents.periodical'),
					schema: {
						fields: [
							{
								key: 'entries',
								label: 'Entries',
								instructions: '',
								kind: {
									type: 'entries',
									fields: [text('title', 'Title'), text('pages', 'Pages'), ...entryFields]
								},
								reason: false
							}
						]
					}
				}
			}
		}
	],
	settings: {
		languages: { mode: 'explicit', languages: ['fr'] },
		sample_pages: null,
		line_breaks: null,
		pace: 'fast'
	},
	output: {
		formats: ['markdown', 'csv'],
		destination: { kind: 'beside_sources' },
		options: { piece_markers: true, speaker_labels: true, timestamps: true }
	},
	status: 'completed'
};

const answer = (key: string, value: string | string[]): FieldAnswer => ({ key, value });
const entry = (title: string, pages: string, by: string[], summary: string, continues = '') => [
	answer('title', title),
	answer('pages', pages),
	answer('continues', continues),
	answer('authors', by),
	answer('summary', summary)
];

const combined = [
	entry(
		"[Éditorial] Le Hajj, pilier de l'islam",
		'1, 3',
		['La Rédaction'],
		"L'éditorial rappelle les conditions du pèlerinage de 1985, le rôle de l'Union des musulmans et les difficultés des pèlerins voltaïques à obtenir leurs visas."
	),
	entry(
		'[Interview] Imam Ismaël Tiendrébéogo',
		'2',
		['Moussa Ouédraogo'],
		"L'imam revient sur la fondation de la medersa de Ouagadougou et sur l'enseignement en arabe et en français."
	),
	entry(
		'Fatwa sur le jeûne des voyageurs',
		'3',
		[],
		'Réponse aux lecteurs sur le jeûne pendant le voyage.'
	)
];

const pageEntries = [
	[entry("[Éditorial] Le Hajj, pilier de l'islam", '', ['La Rédaction'], '…', 'suite page 3')],
	[entry('[Interview] Imam Ismaël Tiendrébéogo', '', ['Moussa Ouédraogo'], '…')],
	[
		entry("Suite : Le Hajj, pilier de l'islam", '', [], '…'),
		entry('Fatwa sur le jeûne des voyageurs', '', [], '…')
	]
];

const view = (index: number, piece: PieceView['piece'], fields: FieldAnswer[]): PieceView => ({
	setup_id: contentsJob.setups[0].id,
	source_id: issue.id,
	index,
	piece,
	status: 'done',
	error: null,
	ai_text: '',
	corrected_text: null,
	reviewed: false,
	model_version: null,
	cost_usd: 0.006,
	segments: [],
	ai_fields: fields,
	corrected_fields: null,
	speakers: [],
	speaker_names: {},
	source_text: null,
	truncated: false
});

export const contentsPieces: PieceView[] = [
	...pageEntries.map((entries, i) =>
		view(i, { kind: 'page', number: i + 1 }, [
			{ key: 'entries', value: entries },
			answer('page_note', i === 1 ? 'Advertisement for a bookshop in Bobo-Dioulasso' : '')
		])
	),
	view(3, { kind: 'whole' }, [{ key: 'entries', value: combined }])
];

export const contentsSource = issue;

/** A Gallica text, its OCR corrected page by page. */
export const gallica = {
	...issue,
	id: '00000000-0000-0000-0000-0000000000c3',
	path: 'C:\\Archives\\Gallica\\afrique-francaise-1931.pdf',
	file_name: 'afrique-francaise-1931.pdf'
};

export const correctionJob: Job = {
	...contentsJob,
	id: '00000000-0000-0000-0000-0000000000j6',
	name: "L'Afrique française, 1931",
	task: 'text_correction',
	sources: [gallica],
	setups: [
		{
			...contentsJob.setups[0],
			id: '00000000-0000-0000-0000-0000000000c4',
			label: 'deepseek-v4-1-flash-1017',
			provider: 'mistral',
			model: 'deepseek-v4-1-flash-1017',
			merge: undefined
		}
	],
	settings: { ...contentsJob.settings, line_breaks: 'running_text' }
};

const ocr = [
	"Le Gouvernenr général a reçu à Dakar les délé- gués de la Mission catholique et de l'Union cul- turelle musulmane. L'entretien a porté sur l'en- seignement et le pèlerinage de la Mecque.",
	"A Bamako, deux cents fidèles se sont réunis à la mosquée pour désigner une commis- sion chargée de la réfection du minaret, sous la présidence de l'imam."
];
const corrected = [
	"Le Gouverneur général a reçu à Dakar les délégués de la Mission catholique et de l'Union culturelle musulmane. L'entretien a porté sur l'enseignement et le pèlerinage de la Mecque.",
	"A Bamako, deux cents fidèles se sont réunis à la mosquée pour désigner une commission chargée de la réfection du minaret, sous la présidence de l'imam."
];

export const correctionPieces: PieceView[] = ocr.map((original, i) => ({
	...view(i, { kind: 'page', number: i + 1 }, []),
	setup_id: correctionJob.setups[0].id,
	source_id: gallica.id,
	ai_text: corrected[i],
	source_text: original,
	truncated: false
}));
