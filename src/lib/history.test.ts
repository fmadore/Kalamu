import { describe, expect, it } from 'vitest';
import { HISTORY, kept, leaving, samePlace } from './history.js';
import type { View } from './state.svelte.js';

const review: View = { name: 'review', jobId: 'j1', sourceId: 's1' };
const search: View = { name: 'projects', projectId: 'p1', pane: 'search' };

describe('going back', () => {
	it('keeps the screen left, most recent last', () => {
		const past = leaving([], search, review);
		expect(past).toEqual([search]);
		expect(leaving(past, review, { name: 'jobs', jobId: 'j1' })).toEqual([search, review]);
	});

	it('adds nothing for a change inside the same place', () => {
		const queue: View = { name: 'projects', projectId: 'p1', pane: 'queue' };
		expect(samePlace(search, queue)).toBe(true);
		expect(leaving([], search, queue)).toEqual([]);
		// Another page of the same review is the same place; another source is not.
		expect(samePlace(review, { ...review, index: 3 })).toBe(true);
		expect(samePlace(review, { ...review, sourceId: 's2' })).toBe(false);
	});

	it('tells a project, its documents and another project apart', () => {
		const doc: View = { name: 'projects', projectId: 'p1', documentId: 'd1' };
		expect(samePlace(search, doc)).toBe(false);
		expect(samePlace(search, { name: 'projects', projectId: 'p2' })).toBe(false);
		expect(samePlace({ name: 'projects' }, search)).toBe(false);
	});

	it('comes back to New job as its form was left', () => {
		const opened: View = { name: 'new', paths: ['/a.pdf'], task: 'analysis' };
		expect(kept(opened)).toEqual({ name: 'new' });
		expect(kept(search)).toBe(search);
	});

	it('keeps only the most recent screens', () => {
		let past: View[] = [];
		for (let i = 0; i < HISTORY + 5; i++) {
			past = leaving(past, { name: 'jobs', jobId: `j${i}` }, { name: 'jobs', jobId: `j${i + 1}` });
		}
		expect(past).toHaveLength(HISTORY);
		expect(past[0]).toEqual({ name: 'jobs', jobId: 'j5' });
		expect(past[HISTORY - 1]).toEqual({ name: 'jobs', jobId: `j${HISTORY + 4}` });
	});
});
