// New job's estimate, asked of the engine again a moment after what is estimated last changed.
// The form keeps the answer; StartPanel shows it, and the pace step prices each pace with it.
// A text the model chosen cannot read is said where Start is (`tooLongOf`), and Start is off.

import type { Catalog } from './bindings/Catalog.js';
import type { Estimate } from './bindings/Estimate.js';
import type { EstimateRequest } from './bindings/EstimateRequest.js';
import { modelNameOf } from './catalogText.js';
import { messages, type Messages } from './i18n/index.js';
import { api } from './tauri.js';

/**
 * The sentence saying a text is too long for a model chosen, when the estimate names one: the
 * engine measured the text it would send (whole, or for an analysis that cites the text, its
 * longest passage) and would refuse the job for it (`job.text_too_long`). The first such text
 * is named, with how many others there are.
 */
export function tooLongOf(
	estimate: Estimate | null,
	catalog: Catalog,
	m: Messages = messages()
): string | null {
	const row = estimate?.rows.find((r) => r.too_long?.length);
	const [first, ...others] = row?.too_long ?? [];
	if (!row || !first) return null;
	const count = new Intl.NumberFormat(m.locale.tag);
	return m.newJob.tooLong(
		first.file,
		modelNameOf(catalog, row.provider, row.model),
		count.format(first.words),
		count.format(first.most_words),
		others.length
	);
}

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
