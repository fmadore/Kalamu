// New job's estimate, asked of the engine again a moment after what is estimated last changed.
// The form keeps the answer; StartPanel shows it, and the pace step prices each pace with it.
// A text the model chosen cannot read is said where Start is (`tooLongOf`), and Start is off.

import type { Catalog } from './bindings/Catalog.js';
import type { Estimate } from './bindings/Estimate.js';
import type { EstimateRequest } from './bindings/EstimateRequest.js';
import type { EstimateRow } from './bindings/EstimateRow.js';
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
 * What the economy choice says of the models that get no discount in it, one sentence per
 * reason the engine gives (`at_once`): a model without a batch service, an analysis that quotes
 * the text, a transcript translated line by line. A step after the others (`merge`) never goes
 * in a batch and is not named.
 */
export function notBatchedOf(
	rows: EstimateRow[],
	catalog: Catalog,
	m: Messages = messages()
): string[] {
	const named = (reason: EstimateRow['at_once'] | null) => [
		...new Set(
			rows
				.filter((r) => !r.merge && r.estimate.economy_usd === null)
				.filter((r) => (r.at_once ?? null) === reason)
				.map((r) => modelNameOf(catalog, r.provider, r.model))
		)
	];
	const list = (names: string[]) =>
		new Intl.ListFormat(m.locale.tag, { type: 'conjunction' }).format(names);
	const sentences: [string[], (names: string) => string][] = [
		[named(null), m.newJob.paceNotBatched],
		[named('quotes'), m.newJob.paceQuotes],
		[named('timed_lines'), m.newJob.paceTimedLines]
	];
	return sentences.filter(([names]) => names.length > 0).map(([names, say]) => say(list(names)));
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
