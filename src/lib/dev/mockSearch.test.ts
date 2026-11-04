import { beforeAll, describe, expect, it } from 'vitest';
import type { SearchRequest } from '../bindings/SearchRequest.js';
import type { SearchView } from '../bindings/SearchView.js';
import { blankRequest } from '../projectSearch.js';
import { analysisJob, audioJob, job, source } from './mockJobs.js';
import { DOCUMENT_ID, seedProjects } from './mockProjects.js';
import { parseQuery, searchCommand } from './mockSearch.js';

const INTERVIEW = '00000000-0000-0000-0000-0000000000d2';
const BULLETINS = '00000000-0000-0000-0000-0000000000c2';

beforeAll(() =>
	seedProjects({
		job,
		audioJob,
		analysisJob,
		extras: [1, 2].map((n) => ({
			...source,
			id: `00000000-0000-0000-0000-0000000000f${n}`,
			file_name: `letter-${n}.jpg`
		}))
	})
);

const search = (over: Partial<SearchRequest>) =>
	searchCommand('project_search', { request: { ...blankRequest(), ...over } }) as SearchView;
const found = (view: SearchView) =>
	view.documents.flatMap((d) => d.results.map((r) => `${d.title} · ${r.setup_label}`));

describe('the project search mock honours the search’s options', () => {
	it('searches the preferred reading, or every reading', () => {
		expect(found(search({ query: 'imam' }))).toEqual([
			`Bulletin, May 1957 · ${job.setups[1].label}`,
			`Interview with the imam · ${audioJob.setups[0].label}`
		]);
		expect(found(search({ query: 'imam', results: 'all' }))).toHaveLength(3);
	});

	it('keeps to the collection, the tag and the task chosen', () => {
		expect(
			search({ query: 'imam', collection: BULLETINS }).documents.map((d) => d.document_id)
		).toEqual([DOCUMENT_ID]);
		const tagged = search({ query: 'imam', tag: 'interviews' });
		expect(tagged.documents.map((d) => d.document_id)).toEqual([INTERVIEW]);
		expect(tagged.coverage.documents).toBe(1);
		const heard = search({ query: 'imam', task: 'audio_transcription' });
		expect(heard.documents.map((d) => d.document_id)).toEqual([INTERVIEW]);
	});

	it('keeps to the pages reviewed or not', () => {
		expect(search({ query: 'imam', review: 'reviewed' }).documents).toEqual([]);
		const reviewed = search({ query: 'imam', review: 'reviewed', results: 'all' });
		expect(reviewed.documents[0].results[0].hits.map((h) => h.index)).toEqual([0]);
	});

	it('finds every word, each match its own, or words near each other', () => {
		const every = search({ query: 'imam Bamako', mode: 'all_words', task: 'text_extraction' });
		const hits = every.documents[0].results[0].hits;
		expect(hits.map((h) => h.term)).toEqual(['Bamako', 'Bamako', 'imam', 'Bamako', 'imam']);
		expect(every.terms).toEqual(['imam', 'Bamako']);

		const near = (n: number) =>
			search({ query: 'Bamako imam', mode: 'near', near: n, task: 'text_extraction' });
		expect(near(3).documents).toEqual([]);
		const close = near(10).documents[0].results[0].hits;
		expect(close.map((h) => h.index)).toEqual([0, 1]);
		expect(close[1].term).toBe('Bamako');
		expect(close[1].marks).toHaveLength(2);
	});

	it('searches what models wrote only when asked', () => {
		expect(search({ query: 'cordial', task: 'analysis' }).documents).toEqual([]);
		const answers = search({ query: 'cordial', task: 'analysis', generated: true });
		const hit = answers.documents[0].results[0];
		expect(hit.generated).toBe(true);
		expect(hit.hits[0]).toMatchObject({ field: { key: 'tone', label: 'Tone' }, reason: true });
	});

	it('leaves out words too short, and refuses a query with none long enough', () => {
		expect(parseQuery('de « l’imam » imam,', 'all_words')).toEqual({
			terms: [
				{ typed: 'l’imam', needle: 'l’imam' },
				{ typed: 'imam', needle: 'imam' }
			],
			ignored: ['de']
		});
		expect(() => search({ query: 'de' })).toThrow(
			expect.objectContaining({ id: 'search.too_short' })
		);
	});
});
