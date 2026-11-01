// A project's activity: which of its jobs the filters show, the choices each filter offers, and
// what the jobs shown cost together. Pure, so it is tested without the app.

import type { JobStatus } from './bindings/JobStatus.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { Task } from './bindings/Task.js';

export type Period = 'any' | 'week' | 'month' | 'year';

/** An empty string leaves that filter open. */
export interface ActivityFilter {
	task: Task | '';
	status: JobStatus | '';
	model: string;
	workflow: string;
	period: Period;
}

export const openFilter: ActivityFilter = {
	task: '',
	status: '',
	model: '',
	workflow: '',
	period: 'any'
};

const DAY = 24 * 60 * 60 * 1000;
const periods: Record<Exclude<Period, 'any'>, number> = {
	week: 7 * DAY,
	month: 30 * DAY,
	year: 365 * DAY
};

/** Whether any filter narrows the list. */
export function isFiltered(filter: ActivityFilter): boolean {
	return (
		filter.task !== '' ||
		filter.status !== '' ||
		filter.model !== '' ||
		filter.workflow !== '' ||
		filter.period !== 'any'
	);
}

/** The jobs every filter lets through, in the order given; a period counts back from `now`
 *  to when the job was created. */
export function filterActivity(
	jobs: JobSummary[],
	filter: ActivityFilter,
	now: number = Date.now()
): JobSummary[] {
	const since = filter.period === 'any' ? null : now - periods[filter.period];
	return jobs.filter(
		(job) =>
			(filter.task === '' || job.task === filter.task) &&
			(filter.status === '' || job.status === filter.status) &&
			(filter.model === '' || job.models.includes(filter.model)) &&
			(filter.workflow === '' || job.workflow === filter.workflow) &&
			(since === null || job.created_at >= since)
	);
}

/** What each filter can choose among: only what the jobs hold, each once, sorted. */
export function activityChoices(jobs: JobSummary[]): {
	tasks: Task[];
	statuses: JobStatus[];
	models: string[];
	workflows: string[];
} {
	const sorted = <T extends string>(values: Iterable<T>) =>
		[...new Set(values)].sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
	return {
		tasks: sorted(jobs.map((j) => j.task)),
		statuses: sorted(jobs.map((j) => j.status)),
		models: sorted(jobs.flatMap((j) => j.models)),
		workflows: sorted(jobs.flatMap((j) => (j.workflow === null ? [] : [j.workflow])))
	};
}

/** What the jobs cost together, `null` when none has a priced answer yet, and how many of
 *  them have none (offline jobs, or jobs not yet answered), so a sum is never read as whole
 *  when it is not. */
export function summedCost(jobs: JobSummary[]): { total: number | null; unpriced: number } {
	let total: number | null = null;
	let unpriced = 0;
	for (const job of jobs) {
		if (job.cost_usd === null) unpriced += 1;
		else total = (total ?? 0) + job.cost_usd;
	}
	return { total, unpriced };
}
