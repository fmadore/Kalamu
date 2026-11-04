// The job list, as the sidebar groups it and the Jobs page finds and sorts it. What a job is and
// how far it got come from the engine (`JobSummary`); this only chooses which to show, in what
// order.

import type { JobStatus } from './bindings/JobStatus.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { Task } from './bindings/Task.js';
import { folded } from './search.js';

/** How many finished jobs the sidebar lists before its link to the Jobs page. */
export const FINISHED_SHOWN = 8;

/** Jobs that stopped before the end and can be resumed. */
export const UNFINISHED: JobStatus[] = ['interrupted', 'paused', 'pending'];

export type GroupKey = 'waiting' | 'running' | 'unfinished' | 'finished';

/** Which sidebar group a job belongs to: workflow steps waiting for the researcher first, then
 *  what runs, what stopped early, and the rest. */
export function groupOf(job: JobSummary, waiting: Record<string, unknown>): GroupKey {
	if (waiting[job.id]) return 'waiting';
	if (job.status === 'running') return 'running';
	if (UNFINISHED.includes(job.status)) return 'unfinished';
	return 'finished';
}

/** The finished jobs the sidebar lists: the most recent `limit`, and the job open in the main
 *  area when it is older, so the one being read or reviewed is never hidden. `jobs` is
 *  newest first, as the engine lists them. */
export function finishedShown(
	jobs: JobSummary[],
	openId: string | undefined,
	limit = FINISHED_SHOWN
): JobSummary[] {
	const shown = jobs.slice(0, limit);
	const open = jobs.findIndex((j) => j.id === openId);
	if (open >= limit) shown.push(jobs[open]);
	return shown;
}

/** What the Jobs page's status filter can choose: one of the engine's states, or the jobs
 *  waiting for the researcher. */
export type StatusFilter = '' | 'waiting' | JobStatus;

export interface JobFilter {
	query: string;
	task: Task | '';
	status: StatusFilter;
}

/** The jobs whose name, workflow or source files hold every word typed (capitals and accents
 *  ignored), of the task and status chosen (any with `''`). */
export function shownJobs(
	jobs: JobSummary[],
	filter: JobFilter,
	sources: ReadonlyMap<string, string[]>,
	waiting: Record<string, unknown>
): JobSummary[] {
	const words = folded(filter.query).split(/\s+/).filter(Boolean);
	return jobs.filter((job) => {
		if (filter.task && job.task !== filter.task) return false;
		if (filter.status === 'waiting' && !waiting[job.id]) return false;
		if (filter.status && filter.status !== 'waiting' && job.status !== filter.status) {
			return false;
		}
		if (words.length === 0) return true;
		const names = sources.get(job.id) ?? [];
		const haystack = folded(`${job.name} ${job.workflow ?? ''} ${names.join(' ')}`);
		return words.every((w) => haystack.includes(w));
	});
}

export type JobSort = 'newest' | 'oldest' | 'name';

/** Newest or oldest first, or by name as a reader sorts (`part 2` before `part 10`). */
export function sortJobs(jobs: JobSummary[], by: JobSort): JobSummary[] {
	const name = (a: JobSummary, b: JobSummary) =>
		a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
	const sorted = [...jobs];
	switch (by) {
		case 'newest':
			return sorted.sort((a, b) => b.created_at - a.created_at || name(a, b));
		case 'oldest':
			return sorted.sort((a, b) => a.created_at - b.created_at || name(a, b));
		case 'name':
			return sorted.sort((a, b) => name(a, b) || b.created_at - a.created_at);
	}
}
