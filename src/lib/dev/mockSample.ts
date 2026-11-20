// A sample first (#215) in the browser preview. `?long`: New job's first PDF has 40 pages, so
// it suggests trying a few first, and the job started is kept to be opened. `?sample`: a sample
// of three pages spread through a 40-page bulletin, read by Gemini, whose page offers the rest
// and another model; the jobs made from it are listed and opened. What was asked of the engine
// is kept on `window.__kalamuSample` for the tests. Development builds only.

import type { Job } from '../bindings/Job.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { NewJob } from '../bindings/NewJob.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import { job, setupA, source, summaries } from './mockJobs.js';
import { jobCommand } from './mockJobCommand.js';

declare global {
	interface Window {
		/** What the sample's page and New job asked of the engine, in order. */
		__kalamuSample?: { cmd: string; args: Record<string, unknown> }[];
	}
}

const SAMPLE_ID = '00000000-0000-0000-0000-0000000000s1';

const bulletin: SourceFile = {
	...source,
	id: '00000000-0000-0000-0000-0000000000f5',
	path: 'C:\\Archives\\Bulletin\\bulletin-1958-01.pdf',
	file_name: 'bulletin-1958-01.pdf',
	page_count: 40
};

const sampleJob: Job = {
	...job,
	id: SAMPLE_ID,
	name: 'Bulletin, January 1958',
	sources: [bulletin],
	setups: [setupA],
	settings: { ...job.settings, sample_pages: 3, sample_spread: true }
};

/** A job as the list shows it, `pieces` of them read. */
function summaryOf(j: Job, pieces: number, status: JobSummary['status']): JobSummary {
	return {
		...summaries[0],
		id: j.id,
		name: j.name,
		status,
		setup_count: j.setups.length,
		pieces_total: pieces * j.setups.length,
		pieces_finished: status === 'completed' ? pieces * j.setups.length : 0,
		pieces_truncated: 0,
		cost_usd: status === 'completed' ? 0.0011 : null,
		models: j.setups.map((s) => s.model),
		workflow: null
	};
}

/** The jobs made here, and how the list shows them. */
const made: Job[] = [];
const madeSummaries: JobSummary[] = [];

function keep(j: Job): Job {
	made.push(j);
	const pages = j.settings.sample_pages ?? j.sources[0]?.page_count ?? 1;
	madeSummaries.push(summaryOf(j, pages, 'pending'));
	return j;
}

function asked(cmd: string, args: Record<string, unknown>): void {
	(window.__kalamuSample ??= []).push({ cmd, args });
}

export function sampleCommand(cmd: string, a: Record<string, unknown>): unknown {
	const params = new URLSearchParams(location.search);
	const sample = params.has('sample');
	const long = params.has('long');
	if (!sample && !long) return undefined;
	const mine = (id: unknown) => made.find((j) => j.id === id);
	switch (cmd) {
		case 'inspect_files': {
			if (!long) return undefined;
			// Every PDF a bulletin of 40 pages, every photograph one page.
			return (a.paths as string[]).map((path, i) => {
				const image = /\.(jpg|png)$/i.test(path);
				return {
					path,
					source: {
						...source,
						id: `00000000-0000-0000-0000-0000000001${i}0`,
						path,
						file_name: path.split('\\').pop() ?? path,
						sha256: String(i).repeat(64),
						kind: image ? 'image' : 'pdf',
						page_count: image ? 1 : 40
					},
					warnings: [],
					error: null,
					pieces: null
				};
			});
		}
		case 'create_job': {
			if (!long) return undefined;
			asked(cmd, a);
			const request = a.job as NewJob;
			return keep({
				...job,
				id: `00000000-0000-0000-0000-0000000000n${made.length}`,
				name: request.name,
				sources: request.sources,
				settings: request.settings,
				status: 'pending'
			});
		}
		case 'list_jobs': {
			const listed = jobCommand(cmd, a) as JobSummary[];
			return [
				...madeSummaries,
				...(sample ? [summaryOf(sampleJob, 3, 'completed')] : []),
				...listed
			];
		}
		case 'get_job':
			if (sample && a.id === SAMPLE_ID) return sampleJob;
			return mine(a.id);
		case 'start_job': {
			const summary = madeSummaries.find((s) => s.id === a.id);
			if (!summary) return undefined;
			summary.status = 'running';
			return null;
		}
		case 'rest_estimate': {
			asked(cmd, a);
			const from = a.job === SAMPLE_ID ? sampleJob : mine(a.job);
			const setup = from?.setups.find((s) => s.id === a.setup);
			const usd = setup?.provider === 'anthropic' ? 0.52 : 0.04;
			return {
				estimate: {
					rows: [
						{
							provider: setup?.provider ?? 'gemini',
							model: setup?.model ?? '',
							pages: 37,
							merge: false,
							estimate: { usd, economy_usd: null }
						}
					],
					total_usd: usd,
					total_economy_usd: usd
				},
				cached_pages: 3
			};
		}
		case 'rest_of_sample': {
			asked(cmd, a);
			const from = a.job === SAMPLE_ID ? sampleJob : mine(a.job);
			if (!from) throw { id: 'job.not_found' };
			const setup = from.setups.find((s) => s.id === a.setup);
			if (!setup) throw { id: 'job.not_found' };
			return keep({
				...from,
				id: `00000000-0000-0000-0000-0000000000r${made.length}`,
				name: String(a.name),
				setups: [{ ...setup, id: `00000000-0000-0000-0000-00000000r${made.length}a0` }],
				settings: { ...from.settings, sample_pages: null, sample_spread: undefined },
				status: 'pending'
			});
		}
		case 'compare_sample': {
			asked(cmd, a);
			const from = a.job === SAMPLE_ID ? sampleJob : mine(a.job);
			if (!from) throw { id: 'job.not_found' };
			const added = {
				...setupA,
				id: `00000000-0000-0000-0000-00000000c${made.length}b0`,
				label: String(a.model),
				provider: String(a.provider),
				model: String(a.model)
			};
			return keep({
				...from,
				id: `00000000-0000-0000-0000-0000000000c${made.length}`,
				name: String(a.name),
				setups: [...from.setups, added],
				status: 'pending'
			});
		}
		default:
			return undefined;
	}
}
