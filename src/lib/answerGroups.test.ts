import { describe, expect, it } from 'vitest';
import type { ChoiceAnswers } from './bindings/ChoiceAnswers.js';
import { distinctTitles, supersededAt } from './answerGroups.js';

function group(over: Partial<ChoiceAnswers> = {}): ChoiceAnswers {
	return {
		job_id: 'job-1',
		setup_id: 'setup-1',
		job_name: 'My question: market women',
		preset_id: 'analysis.screening',
		question: 'What market women asked of the council',
		provider: 'gemini',
		model: 'gemini-4-carbon',
		languages: { mode: 'auto' },
		prompt: 'prompt-a',
		first_asked: 1000,
		unread: [],
		field: {
			key: 'relevance',
			label: 'Relevance',
			instructions: '',
			kind: { type: 'choice', values: ['Relevant', 'Unsure'] },
			reason: false
		},
		answers: [],
		...over
	};
}

const since = (asked: number) => `asked since ${asked}`;

describe('a question asked under two versions of its prompt', () => {
	const newer = group({
		job_id: 'job-2',
		setup_id: 'setup-2',
		prompt: 'prompt-b',
		first_asked: 2000
	});
	const older = group();

	it('names each group by when it was first asked', () => {
		const title = older.question!;
		expect(distinctTitles([title, title], [newer, older], since)).toEqual([
			`${title} · asked since 2000`,
			`${title} · asked since 1000`
		]);
	});

	it('leaves a title alone that no other analysis shares', () => {
		const other = group({ job_id: 'job-3', question: 'Another' });
		expect(distinctTitles(['Q', 'Another'], [older, other], since)).toEqual(['Q', 'Another']);
	});

	it('leaves alone the fields of one analysis that share a question', () => {
		const second = group({ field: { ...older.field, key: 'tone' } });
		expect(distinctTitles(['Q', 'Q'], [older, second], since)).toEqual(['Q', 'Q']);
	});

	it('offers asking again for the newest version only', () => {
		const groups = [newer, older];
		expect(supersededAt(groups, 0)).toBe(false);
		expect(supersededAt(groups, 1)).toBe(true);
	});

	it('keeps asking again for the same question read by another model', () => {
		const other = group({ job_id: 'job-2', model: 'gemini-4-pro', prompt: 'prompt-b' });
		expect(supersededAt([other, older], 1)).toBe(false);
	});
});
