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
	decisionTags,
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
		field: {
			key: 'relevance',
			label: 'Relevance',
			instructions:
				"Whether the document speaks of the researcher's question, following the instructions above.",
			kind: { type: 'choice', values: ['Relevant', 'Partly relevant', 'Not relevant', 'Unsure'] },
			reason: false
		},
		answers: [
			{ document: 'a', value: 'Not relevant' },
			{ document: 'b', value: 'Relevant' },
			{ document: 'c', value: 'Relevant' },
			{ document: 'd', value: 'Unsure' }
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

describe('a document too long for the model', () => {
	it('is named with the model, where the cost would be', () => {
		const said = tooLongOf(
			{
				rows: [
					{
						provider: 'gemini',
						model: 'none-such',
						too_long: { file: 'long.txt', words: 120_000, most_words: 80_000 }
					}
				]
			} as never,
			{ providers: [], models: [] } as never,
			m
		);
		expect(said).toBe(
			'long.txt is too long for none-such: about 120,000 words, where it reads about 80,000 at once. Choose another model under More choices.'
		);
		expect(tooLongOf(null, { providers: [], models: [] } as never, m)).toBeNull();
	});
});
