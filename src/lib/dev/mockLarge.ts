// Thousands of rows, for timing the long tables and a large job's page in the mock
// (`?mock&rows=3000`): jobs on the Jobs page, documents in the sample project, names in its
// dictionary and files in the sample job, added to what the mock answers. Also the engine's
// counting of a job's pieces (`job_progress`), which the mock answers like the engine.

import type { DictionaryView } from '../bindings/DictionaryView.js';
import type { Job } from '../bindings/Job.js';
import type { JobProgress } from '../bindings/JobProgress.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { PieceStatus } from '../bindings/PieceStatus.js';
import type { ProjectView } from '../bindings/ProjectView.js';

/** How many rows `?rows=` asks for, at most 50,000; none without it. */
export function largeRows(search = location.search): number {
	const n = Math.floor(Number(new URLSearchParams(search).get('rows')));
	return Number.isFinite(n) && n > 0 ? Math.min(n, 50_000) : 0;
}

type Counted = {
	setup_id: string;
	source_id: string;
	status: PieceStatus;
	error: string | null;
};

/** The engine's `job_progress`: the pieces held over the whole job, and each of `sources`
 * counted under every set-up, at most three different errors listed. */
export function progressOf(pieces: readonly Counted[], sources: readonly string[]): JobProgress {
	const wanted = new Set(sources);
	const rows = new Map<string, JobProgress['files'][number]>();
	let held = 0;
	for (const p of pieces) {
		if (p.status === 'submitted') held++;
		if (!wanted.has(p.source_id)) continue;
		const key = `${p.source_id}/${p.setup_id}`;
		let row = rows.get(key);
		if (!row) {
			row = {
				source_id: p.source_id,
				setup_id: p.setup_id,
				total: 0,
				done: 0,
				failed: 0,
				errors: []
			};
			rows.set(key, row);
		}
		row.total++;
		if (p.status === 'done' || p.status === 'cached') row.done++;
		if (p.status === 'failed') row.failed++;
		if (p.error && row.errors.length < 3 && !row.errors.includes(p.error)) row.errors.push(p.error);
	}
	return { held, files: [...rows.values()] };
}

const bigId = (kind: string, i: number) => `${kind}-${String(i).padStart(6, '0')}`;

/** The pieces of the sample job's added files: one page each, under each set-up. */
function largePieces(job: Job, n: number): Counted[] {
	return Array.from({ length: n }, (_, i) =>
		job.setups.map((setup, s) => ({
			setup_id: setup.id,
			source_id: bigId('source', i),
			status: (i % 50 === 7 && s === 1
				? 'failed'
				: i % 3 === 0
					? 'pending'
					: 'done') as PieceStatus,
			error: i % 50 === 7 && s === 1 ? 'The model refused the page.' : null
		}))
	).flat();
}

/** `result`, the mock's answer to `cmd`, with `n` rows added where it has a long list. */
export function enlarge(
	cmd: string,
	a: Record<string, unknown>,
	result: unknown,
	n: number,
	sample: Job
): unknown {
	if (n === 0) return result;
	switch (cmd) {
		case 'list_jobs': {
			const jobs = result as JobSummary[];
			const first = jobs[0];
			if (!first) return result;
			return [
				...jobs,
				...Array.from({ length: n }, (_, i) => ({
					...first,
					id: bigId('job', i),
					name: `Fonds ${i + 1}`,
					created_at: first.created_at - (i + 1) * 60_000
				}))
			];
		}
		case 'get_job': {
			if (a.id !== sample.id) return result;
			const job = result as Job;
			const first = job.sources[0];
			return {
				...job,
				sources: [
					...job.sources,
					...Array.from({ length: n }, (_, i) => ({
						...first,
						id: bigId('source', i),
						file_name: `page-${String(i + 1).padStart(5, '0')}.pdf`,
						page_count: 1
					}))
				]
			};
		}
		case 'job_progress': {
			if (a.id !== sample.id) return result;
			const more = progressOf(largePieces(sample, n), a.sources as string[]);
			const progress = result as JobProgress;
			return { held: progress.held + more.held, files: [...progress.files, ...more.files] };
		}
		case 'project_view': {
			const view = result as ProjectView;
			const first = view.documents[0];
			if (!first) return result;
			return {
				...view,
				documents: [
					...view.documents,
					...Array.from({ length: n }, (_, i) => ({
						...first,
						document: {
							...first.document,
							id: bigId('document', i),
							title: `Letter ${i + 1}`,
							tags: []
						},
						collections: []
					}))
				]
			};
		}
		case 'dictionary_view': {
			const view = result as DictionaryView;
			// Added to the field shown first.
			const field = view.namespaces[0]?.key;
			const first = view.rows.find((r) => r.namespace === field);
			if (!first) return result;
			return {
				...view,
				namespaces: view.namespaces.map((ns) =>
					ns.key === field ? { ...ns, rows: ns.rows + n } : ns
				),
				rows: [
					...view.rows,
					...Array.from({ length: n }, (_, i) => ({
						...first,
						key: bigId('name', i),
						entry: null,
						label: `Name ${i + 1}`,
						forms: [{ ...first.forms[0], form: `Name ${i + 1}` }],
						apart: [],
						similar: []
					}))
				]
			};
		}
		default:
			return result;
	}
}
