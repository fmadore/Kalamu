import { afterEach, describe, expect, it } from 'vitest';
import type { ChoiceAnswers } from './bindings/ChoiceAnswers.js';
import type { CitedEvidence } from './bindings/CitedEvidence.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import { saidBy } from './cited.js';
import { en } from './i18n/en.js';
import { fr } from './i18n/fr.js';
import { setLocale, type Messages } from './i18n/index.js';
import {
	answerKey,
	answerOptions,
	answeredDocuments,
	chosenAnswer,
	decisionOf,
	decisionTags,
	rowAnswers,
	screeningRequest,
	tableName,
	tooLongOf
} from './screening.js';

const m = en as unknown as Messages;
const mFr = fr as unknown as Messages;

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
		unread: [],
		field: {
			key: 'relevance',
			label: 'Relevance',
			instructions:
				"Whether the document speaks of the researcher's question, following the instructions above.",
			kind: { type: 'choice', values: ['Relevant', 'Partly relevant', 'Not relevant', 'Unsure'] },
			reason: false
		},
		answers: [
			{ document: 'a', value: 'Not relevant', job_id: 'job-1', reason: 'Only the budget.' },
			{
				document: 'b',
				value: 'Relevant',
				job_id: 'job-1',
				reason: '  An awning asked of the town.  ',
				passages: [
					{
						anchor: { part: 1, start: 0, end: 10 },
						quote: 'Les commerçantes demandent un abri.',
						turn: 4,
						speaker: null,
						start_ms: null,
						end_ms: null,
						page: { kind: 'page', number: 2 }
					}
				]
			},
			{ document: 'c', value: 'Relevant', job_id: 'job-1' },
			{ document: 'd', value: 'Unsure', job_id: 'job-1', reason: '   ' }
		],
		...over
	};
}

describe('the screening’s setup', () => {
	const request = { options: { question: null } } as unknown as SetupRequest;
	it('carries the question typed, trimmed, and none when it is empty', () => {
		expect(screeningRequest(request, '  Market women  ').options.question).toBe('Market women');
		expect(screeningRequest(request, '   ')).toBe(request);
	});
});

describe('the filter by an answer', () => {
	afterEach(() => setLocale('en'));

	it('lists each answer given in the field’s order, counted, under the question', () => {
		const [g] = answerOptions([group()], m);
		expect(g.title).toBe('What market women asked of the council');
		expect(g.options.map((o) => o.label)).toEqual([
			'Relevant (2)',
			'Not relevant (1)',
			'Unsure (1)'
		]);
	});

	it('shows a default’s choices in French, the values stored unchanged', () => {
		setLocale('fr');
		const [g] = answerOptions([group()], mFr);
		expect(g.options.map((o) => o.label)).toEqual([
			'Pertinent (2)',
			'Non pertinent (1)',
			'Incertain (1)'
		]);
		expect(chosenAnswer([group()], g.options[0].key)?.value).toBe('Relevant');
	});

	it('names an analysis asked without a question by its field and its job', () => {
		const [g] = answerOptions([group({ question: null, job_name: 'Tone, May' })], m);
		expect(g.title).toBe('Relevance · Tone, May');
	});

	it('keeps the documents given the answer chosen, and filters nothing otherwise', () => {
		const groups = [group()];
		expect([...(answeredDocuments(groups, answerKey(groups[0], 'Relevant')) ?? [])]).toEqual([
			'b',
			'c'
		]);
		expect(answeredDocuments(groups, '')).toBeNull();
		// An answer no longer given (the job deleted) stops filtering.
		expect(answeredDocuments([], answerKey(groups[0], 'Relevant'))).toBeNull();
	});
});

describe('the researcher’s decision', () => {
	it('is one of two tags, the other taken off', () => {
		expect(decisionTags(true, m)).toEqual({ add: 'Kept', remove: 'Set aside' });
		expect(decisionTags(false, mFr)).toEqual({ add: 'Écarté', remove: 'Gardé' });
	});
});

describe('the table exported', () => {
	it('is named without what Windows or macOS refuse in a file name', () => {
		expect(tableName('Missions: AOF', group({ job_name: 'My question: who/what?' }))).toBe(
			'Missions AOF - My question who what'
		);
	});
});

describe('a passage of a document read from pages', () => {
	it('says its page on its card, where a transcript says who speaks and when', () => {
		const e = {
			anchor: { part: 1, start: 0, end: 5 },
			quote: 'Les commerçantes',
			turn: 3,
			speaker: null,
			start_ms: null,
			end_ms: null,
			page: { kind: 'page', number: 2 }
		} as CitedEvidence;
		expect(saidBy(e, m)).toBe('Page 2');
		expect(saidBy({ ...e, page: undefined, speaker: 'Awa', start_ms: 12_000 }, m)).toBe(
			'Awa · 00:00:12'
		);
	});
});

describe('documents too long for the model even in parts', () => {
	it('are named with the model, to be left out of the job', () => {
		const said = tooLongOf(
			{
				rows: [
					{
						provider: 'gemini',
						model: 'none-such',
						too_long: [
							{ source: 's1', file: 'long.txt', words: 120_000, most_words: 80_000 },
							{ source: 's2', file: 'longer.txt', words: 150_000, most_words: 80_000 }
						]
					}
				]
			} as never,
			{ providers: [], models: [] } as never,
			m
		);
		expect(said).toEqual({
			sources: ['s1', 's2'],
			said: 'Left out, too long for none-such even read in parts: long.txt and longer.txt. Another model, under More choices, may read them.'
		});
		expect(tooLongOf(null, { providers: [], models: [] } as never, m)).toBeNull();
		expect(
			tooLongOf({ rows: [{ provider: 'gemini', model: 'x' }] } as never, {} as never, m)
		).toBeNull();
	});
});

describe('the answer in each row', () => {
	afterEach(() => setLocale('en'));

	it('is shown only while the table is filtered by an answer', () => {
		expect(rowAnswers([group()], '')).toBeNull();
	});

	it('gives each document its answer in the interface’s words, its reason and its passages', () => {
		const key = answerKey(group(), 'Relevant');
		const rows = rowAnswers([group()], key);
		const b = rows?.get('b');
		expect(b?.label).toBe('Relevant');
		expect(b?.reason).toBe('An awning asked of the town.');
		expect(b?.passages.map((p) => saidBy(p, m))).toEqual(['Page 2']);
		expect(rows?.get('c')).toEqual({ label: 'Relevant', reason: null, passages: [] });
		expect(rows?.get('d')?.reason).toBeNull();
		setLocale('fr');
		expect(rowAnswers([group()], key)?.get('a')?.label).toBe('Non pertinent');
	});

	it('reads the researcher’s decision from the document’s tags', () => {
		expect(decisionOf(['Kept', 'letters'], m)).toBe('keep');
		expect(decisionOf(['Set aside'], m)).toBe('aside');
		expect(decisionOf(['letters'], m)).toBeNull();
		expect(decisionOf(['Gardé'], mFr)).toBe('keep');
	});
});
