import { describe, expect, it } from 'vitest';
import type { JobSummary } from './bindings/JobSummary.js';
import { finishedShown, groupOf, shownJobs, sortJobs } from './jobList.js';

function job(id: string, options: Partial<JobSummary> = {}): JobSummary {
	return {
		id,
		name: id,
		task: 'text_extraction',
		status: 'completed',
		created_at: 0,
		updated_at: 0,
		source_count: 1,
		setup_count: 1,
		pieces_total: 1,
		pieces_finished: 1,
		pieces_failed: 0,
		pieces_truncated: 0,
		cost_usd: null,
		models: [],
		workflow: null,
		...options
	};
}

const none = { query: '', task: '', status: '' } as const;

describe('groupOf', () => {
	it('puts a waiting workflow step first, whatever its status', () => {
		expect(groupOf(job('a'), { a: {} })).toBe('waiting');
		expect(groupOf(job('a', { status: 'running' }), {})).toBe('running');
		expect(groupOf(job('a', { status: 'paused' }), {})).toBe('unfinished');
		expect(groupOf(job('a', { status: 'cancelled' }), {})).toBe('finished');
	});
});

describe('finishedShown', () => {
	const jobs = Array.from({ length: 12 }, (_, i) => job(`j${i}`));

	it('keeps the most recent ones', () => {
		expect(finishedShown(jobs, undefined, 8).map((j) => j.id)).toEqual(
			jobs.slice(0, 8).map((j) => j.id)
		);
	});

	it('adds the open job when it is older, never twice', () => {
		expect(finishedShown(jobs, 'j10', 8).map((j) => j.id)).toEqual([
			...jobs.slice(0, 8).map((j) => j.id),
			'j10'
		]);
		expect(finishedShown(jobs, 'j3', 8)).toHaveLength(8);
	});

	it('lists them all when there are few', () => {
		expect(finishedShown(jobs.slice(0, 3), 'j1', 8)).toHaveLength(3);
	});
});

describe('shownJobs', () => {
	const jobs = [
		job('a', { name: 'Lettres de l’évêque', task: 'text_extraction' }),
		job('b', { name: 'Entretiens', task: 'audio_transcription', status: 'paused' }),
		job('c', { name: 'Sermons', task: 'analysis', workflow: 'Revue de presse' })
	];
	const sources = new Map([
		['a', ['lettre_1957.pdf']],
		['b', ['Ouagadougou interview.m4a']]
	]);

	it('finds by name, workflow or file, capitals and accents ignored', () => {
		const ids = (query: string) =>
			shownJobs(jobs, { ...none, query }, sources, {}).map((j) => j.id);
		expect(ids('EVEQUE')).toEqual(['a']);
		expect(ids('ouaga')).toEqual(['b']);
		expect(ids('revue presse')).toEqual(['c']);
		expect(ids('1957 lettres')).toEqual(['a']);
		expect(ids('nothing')).toEqual([]);
		expect(ids('')).toEqual(['a', 'b', 'c']);
	});

	it('filters by task and status, and by waiting for the researcher', () => {
		expect(shownJobs(jobs, { ...none, task: 'analysis' }, sources, {}).map((j) => j.id)).toEqual([
			'c'
		]);
		expect(shownJobs(jobs, { ...none, status: 'paused' }, sources, {}).map((j) => j.id)).toEqual([
			'b'
		]);
		expect(
			shownJobs(jobs, { ...none, status: 'waiting' }, sources, { c: {} }).map((j) => j.id)
		).toEqual(['c']);
	});
});

describe('sortJobs', () => {
	const jobs = [
		job('a', { name: 'part 10', created_at: 2 }),
		job('b', { name: 'Part 2', created_at: 3 }),
		job('c', { name: 'part 1', created_at: 1 })
	];

	it('sorts by date either way, or by name as a reader does', () => {
		expect(sortJobs(jobs, 'newest').map((j) => j.id)).toEqual(['b', 'a', 'c']);
		expect(sortJobs(jobs, 'oldest').map((j) => j.id)).toEqual(['c', 'a', 'b']);
		expect(sortJobs(jobs, 'name').map((j) => j.id)).toEqual(['c', 'b', 'a']);
	});
});
