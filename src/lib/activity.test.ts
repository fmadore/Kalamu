import { describe, expect, it } from 'vitest';
import type { JobSummary } from './bindings/JobSummary.js';
import { activityChoices, filterActivity, isFiltered, openFilter, summedCost } from './activity.js';

const DAY = 24 * 60 * 60 * 1000;
const now = 1_000 * DAY;

function job(id: string, options: Partial<JobSummary> = {}): JobSummary {
	return {
		id,
		name: id,
		task: 'text_extraction',
		status: 'completed',
		created_at: now,
		updated_at: now,
		source_count: 1,
		setup_count: 1,
		pieces_total: 1,
		pieces_finished: 1,
		pieces_failed: 0,
		pieces_truncated: 0,
		cost_usd: null,
		models: ['gemini-3.8-flash'],
		workflow: null,
		...options
	};
}

const jobs = [
	job('extract', { cost_usd: 0.25 }),
	job('people', {
		task: 'analysis',
		models: ['claude-sonnet-5-5'],
		workflow: 'Extract and analyse',
		cost_usd: 0.5,
		created_at: now - 10 * DAY
	}),
	job('compare', {
		models: ['gemini-3.8-flash', 'mistral-ocr-4'],
		status: 'paused',
		created_at: now - 100 * DAY
	}),
	job('whisper', {
		task: 'audio_transcription',
		models: ['large-v3-turbo'],
		created_at: now - 400 * DAY
	})
];
const ids = (list: JobSummary[]) => list.map((j) => j.id);

describe('filterActivity', () => {
	it('shows every job when no filter is set', () => {
		expect(isFiltered(openFilter)).toBe(false);
		expect(ids(filterActivity(jobs, openFilter, now))).toEqual(ids(jobs));
	});

	it('narrows by task, status, model and workflow together', () => {
		const filter = { ...openFilter, task: 'text_extraction' as const };
		expect(ids(filterActivity(jobs, filter, now))).toEqual(['extract', 'compare']);
		expect(ids(filterActivity(jobs, { ...filter, status: 'paused' }, now))).toEqual(['compare']);
		expect(ids(filterActivity(jobs, { ...openFilter, model: 'mistral-ocr-4' }, now))).toEqual([
			'compare'
		]);
		expect(
			ids(filterActivity(jobs, { ...openFilter, workflow: 'Extract and analyse' }, now))
		).toEqual(['people']);
		expect(isFiltered({ ...openFilter, model: 'x' })).toBe(true);
	});

	it('counts a period back from now', () => {
		const within = (period: 'week' | 'month' | 'year') =>
			ids(filterActivity(jobs, { ...openFilter, period }, now));
		expect(within('week')).toEqual(['extract']);
		expect(within('month')).toEqual(['extract', 'people']);
		expect(within('year')).toEqual(['extract', 'people', 'compare']);
	});
});

describe('activityChoices', () => {
	it('offers only what the jobs hold, each once', () => {
		expect(activityChoices(jobs)).toEqual({
			tasks: ['analysis', 'audio_transcription', 'text_extraction'],
			statuses: ['completed', 'paused'],
			models: ['claude-sonnet-5-5', 'gemini-3.8-flash', 'large-v3-turbo', 'mistral-ocr-4'],
			workflows: ['Extract and analyse']
		});
	});
});

describe('summedCost', () => {
	it('adds the priced jobs and counts the others', () => {
		expect(summedCost(jobs)).toEqual({ total: 0.75, unpriced: 2 });
	});

	it('has no total before any priced answer', () => {
		expect(summedCost(jobs.slice(2))).toEqual({ total: null, unpriced: 2 });
		expect(summedCost([])).toEqual({ total: null, unpriced: 0 });
	});
});
