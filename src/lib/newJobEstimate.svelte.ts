// New job's estimate, asked of the engine again a moment after what is estimated last changed.
// The form keeps the answer; StartPanel shows it, and the pace step prices each pace with it.

import type { Estimate } from './bindings/Estimate.js';
import type { EstimateRequest } from './bindings/EstimateRequest.js';
import { api } from './tauri.js';

/**
 * Follows what `request` reads: a quarter of a second after it last changed, asks the engine
 * what the job costs and gives the answer to `set`; nothing while there are no sources or no
 * chosen models. Called while a component is set up, so it ends with the component.
 */
export function followEstimate(
	request: () => EstimateRequest,
	set: (estimate: Estimate | null) => void
): void {
	let estimateTimer: ReturnType<typeof setTimeout> | null = null;
	/** Raised by each change to what is estimated, so an older, slower answer is dropped. */
	let estimateTicket = 0;
	$effect(() => {
		const req = request();
		if (estimateTimer) clearTimeout(estimateTimer);
		const ticket = ++estimateTicket;
		if (req.sources.length === 0 || req.setups.length === 0) {
			set(null);
			return;
		}
		estimateTimer = setTimeout(() => {
			api
				.estimateCost(req)
				.then((e) => {
					if (ticket === estimateTicket) set(e);
				})
				.catch(() => {
					if (ticket === estimateTicket) set(null);
				});
		}, 250);
	});
}
