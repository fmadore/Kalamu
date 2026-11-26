// Jobs on a server of the researcher's own in the browser preview, and the consent given to
// servers, as the shell keeps them: a server on the network is asked about once, and again once
// its address changes (`commands/servers.rs` withdraws it); a job made in New job with a server's
// model is kept, runs until it is paused (slow enough to change the server under it), and is
// refused, as `start_job` and `retry_failed` refuse it, while its server is not agreed to or
// once the server is removed. Providers of the four companies count as agreed to, so the other
// screens run unasked. Development builds only.

import type { Job } from '../bindings/Job.js';
import type { JobSummary } from '../bindings/JobSummary.js';
import type { NewJob } from '../bindings/NewJob.js';
import type { OwnServer } from '../bindings/OwnServer.js';
import { job, setupA, summaries } from './mockJobs.js';

/** The servers agreed to, by id, at the address they had then. */
const agreed = new Set<string>();
const jobs: Job[] = [];
const listed: JobSummary[] = [];

const thisComputer = (address: string) => {
	const host = new URL(address).hostname.replace(/^\[|\]$/g, '');
	return host === 'localhost' || host.startsWith('127.') || host === '::1';
};

/** The consent given to a server whose address changed, or that was removed, withdrawn. */
export function withdraw(server: string): void {
	agreed.delete(server);
}

/** The models of `server` that a running job uses: none, and it may be changed or removed. */
export function modelsInUse(server: string): string[] {
	const running = new Set(listed.filter((s) => s.status === 'running').map((s) => s.id));
	return [
		...new Set(
			jobs
				.filter((j) => running.has(j.id))
				.flatMap((j) => j.setups.filter((s) => s.provider === server).map((s) => s.model))
		)
	];
}

/** The jobs made here, as the list shows them. */
export const serverJobSummaries = (): JobSummary[] => listed;

/** The providers a job sends to, each once. */
const providersOf = (j: Job) => [...new Set(j.setups.map((s) => s.provider))];

/** The commands about servers' jobs and consent; `undefined` for any other, or another job.
 *  `server` finds a server by its id. */
export function serverJobCommand(
	cmd: string,
	a: Record<string, unknown>,
	server: (id: string) => OwnServer | undefined
): unknown {
	const mine = jobs.find((j) => j.id === a.id);
	const unagreed = (providers: string[]) =>
		providers.filter((p) => {
			const s = server(p);
			return s !== undefined && !thisComputer(s.address) && !agreed.has(p);
		});
	/** As the shell's `require_agreed`: a server removed, then consent. */
	const refuseUnagreed = (j: Job) => {
		for (const p of providersOf(j))
			if (p.startsWith('own-') && !server(p)) throw { id: 'server.removed' };
		const first = unagreed(providersOf(j))[0];
		if (first) throw { id: 'provider.consent', detail: first };
	};
	const summary = () => listed.find((s) => s.id === a.id)!;
	switch (cmd) {
		case 'providers_to_consent':
			return unagreed(a.providers as string[]);
		case 'consent_providers':
			for (const p of a.providers as string[]) agreed.add(p);
			return null;
		case 'create_job': {
			const request = a.job as NewJob;
			if (!request.setups.some((s) => server(s.provider))) return undefined;
			const made: Job = {
				...job,
				id: `00000000-0000-0000-0000-0000000000o${jobs.length}`,
				name: request.name,
				task: request.task,
				sources: request.sources,
				setups: request.setups.map((s, i) => ({
					...setupA,
					id: `00000000-0000-0000-0000-0000000000p${i}`,
					label: s.model,
					provider: s.provider,
					model: s.model
				})),
				status: 'pending'
			};
			jobs.push(made);
			listed.unshift({
				...summaries[0],
				id: made.id,
				name: made.name,
				task: made.task,
				status: 'pending',
				source_count: made.sources.length,
				setup_count: made.setups.length,
				pieces_total: 12 * made.setups.length,
				pieces_finished: 0,
				pieces_failed: 0,
				pieces_truncated: 0,
				cost_usd: null,
				models: made.setups.map((s) => s.model),
				workflow: null
			});
			return made;
		}
		case 'get_job':
			return mine ? { ...mine, status: summary().status } : undefined;
		case 'providers_ahead':
			return mine ? providersOf(mine) : undefined;
		// Runs until paused: a page or two read, the rest left for the slow server.
		case 'start_job':
		case 'retry_failed':
			if (!mine) return undefined;
			refuseUnagreed(mine);
			Object.assign(summary(), { status: 'running', pieces_finished: 2 });
			return null;
		case 'pause_job':
			if (!mine) return undefined;
			summary().status = 'paused';
			return null;
		default:
			return undefined;
	}
}
