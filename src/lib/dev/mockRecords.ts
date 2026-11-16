// Sample records (#171) for the pretend back end (`mock.ts`, `?mock&view=review-records`): a city
// directory read page by page, one row per entry, its second page cut off at the model's limit,
// and the ready-made directory in Prompts. `edit_rows` changes a page's corrected rows as the
// engine does (`core::records::edit`). Development builds only.

import type { AnalysisSchema } from '../bindings/AnalysisSchema.js';
import type { FieldAnswer } from '../bindings/FieldAnswer.js';
import type { Job } from '../bindings/Job.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { Preset } from '../bindings/Preset.js';
import type { RowEdit } from '../bindings/RowEdit.js';
import { contentsJob } from './mockContents.js';

const column = (key: string, label: string, instructions: string) => ({
	key,
	label,
	instructions,
	kind: { type: 'text' as const },
	reason: false
});

const directorySchema: AnalysisSchema = {
	fields: [
		{
			key: 'entries',
			label: 'Entries',
			instructions: 'Every entry on this page, one row each, in reading order, column by column.',
			kind: {
				type: 'entries',
				fields: [
					column('name', 'Name', 'The name as printed.'),
					column('occupation', 'Occupation', 'The occupation, trade or title as printed.'),
					column('address', 'Address', 'The street and number, or the place, as printed.')
				]
			},
			reason: false
		}
	]
};

export const recordsPreset: Preset = {
	id: 'analysis.directory',
	task: 'analysis',
	name: 'Directory or list, one row per entry',
	description:
		'Every entry on every page, in reading order: the name, occupation, address and remarks as printed, for a city directory, a membership list or a printed table.',
	instructions: '# Directory or list, one row per entry\n\nYou turn the pages of a serial source…',
	user_prompt:
		'List every entry on this page, in reading order, and fill in its fields, following the instructions given.',
	builtin: true,
	modified: false,
	schema: directorySchema
};

const directory = {
	id: '00000000-0000-0000-0000-0000000000r1',
	path: 'C:\\Archives\\Annuaires\\annuaire-dakar-1922.pdf',
	file_name: 'annuaire-dakar-1922.pdf',
	sha256: 'ef'.repeat(32),
	size_bytes: 12_400_000,
	kind: 'pdf' as const,
	page_count: 3,
	duration_ms: null,
	split_spreads: false,
	even_out_volume: false
};

export const recordsJob: Job = {
	...contentsJob,
	id: '00000000-0000-0000-0000-0000000000j7',
	name: 'Annuaire de Dakar, 1922',
	task: 'analysis',
	sources: [directory],
	setups: [
		{
			...contentsJob.setups[0],
			id: '00000000-0000-0000-0000-0000000000r2',
			label: 'gemini-3.8-flash',
			model: 'gemini-3.8-flash',
			prompt: {
				preset_id: recordsPreset.id,
				preset_name: recordsPreset.name,
				template: '',
				text: '',
				user_prompt: '',
				schema: directorySchema
			},
			merge: undefined
		}
	],
	output: { ...contentsJob.output, formats: ['csv', 'json'] }
};

const row = (name: string, occupation: string, address: string): FieldAnswer[] => [
	{ key: 'name', value: name },
	{ key: 'occupation', value: occupation },
	{ key: 'address', value: address }
];

const pageRows = [
	[
		row('Abadie (J.)', 'boulanger', '12, rue Carnot'),
		row('Achard (Vve)', 'modiste', '3, place du Marché'),
		row('Allard (P.-M.)', 'avocat', '5, rue Vincens'),
		row('Bonnet (L.)', 'cordonnier', '"'),
		row('Bouvier frères', 'négociants', 'do.')
	],
	[],
	[row('Diop (Amadou)', 'traitant', 'Médina'), row('Durand (E.)', 'pharmacien', '8, rue Blanchot')]
];

export const recordsPieces: PieceView[] = pageRows.map((rows, i) => ({
	setup_id: recordsJob.setups[0].id,
	source_id: directory.id,
	index: i,
	piece: { kind: 'page', number: i + 1 },
	status: i === 1 ? 'failed' : 'done',
	error: i === 1 ? 'The answer was cut off at the model’s output limit.' : null,
	ai_text: '',
	corrected_text: null,
	reviewed: false,
	model_version: 'gemini-3.8-flash-001',
	cost_usd: i === 1 ? null : 0.003,
	segments: [],
	ai_fields: i === 1 ? [] : [{ key: 'entries', value: rows }],
	corrected_fields: null,
	speakers: [],
	speaker_names: {},
	source_text: null,
	truncated: false
}));

export const recordsSource = directory;

/** `edit_rows`: one change to a page's rows, on its rows as the researcher left them. */
export function editRows(a: Record<string, unknown>): null {
	const piece = recordsPieces.find(
		(p) => p.setup_id === a.setupId && p.source_id === a.sourceId && p.index === a.index
	);
	if (!piece || piece.status === 'failed') throw { id: 'job.nothing_read' };
	const answers = piece.corrected_fields ?? piece.ai_fields;
	const value = answers.find((f) => f.key === 'entries')?.value;
	const rows = (
		Array.isArray(value) && Array.isArray(value[0]) ? (value as FieldAnswer[][]) : []
	).map((r) => r.map((cell) => ({ ...cell })));
	const change = a.change as RowEdit;
	if (change.kind === 'cell') {
		const cell = rows[change.row]?.find((c) => c.key === change.key);
		if (!cell) throw { id: 'records.no_row' };
		cell.value = change.value.trim();
	} else if (change.kind === 'add') {
		rows.splice(Math.min(change.at, rows.length), 0, row('', '', ''));
	} else {
		if (change.row >= rows.length) throw { id: 'records.no_row' };
		rows.splice(change.row, 1);
	}
	piece.corrected_fields = [{ key: 'entries', value: rows }];
	return null;
}
