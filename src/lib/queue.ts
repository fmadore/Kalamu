// What the review queue shows of the engine's items: which reasons are on, each reason in
// words, and where an item opens. The items, their reasons and the counts come from Rust.

import type { QueueItem } from './bindings/QueueItem.js';
import type { Signal } from './bindings/Signal.js';
import type { Messages } from './i18n/index.js';
import { listNames } from './i18n/index.js';
import { pieceLabel } from './pieces.js';

export type Reason = Signal['kind'];

export const REASONS: Reason[] = [
	'failed',
	'not_read',
	'truncated',
	'disagrees',
	'stale',
	'not_reviewed'
];

/** The reasons shown, and whether items already reviewed stay in view. */
export interface QueueFilter {
	reasons: Reason[];
	resolved: boolean;
}

/** Everything flagged, without the long tail of pages that are only not reviewed yet. */
export const DEFAULT_FILTER: QueueFilter = {
	reasons: REASONS.filter((r) => r !== 'not_reviewed'),
	resolved: false
};

/** The items with one of the reasons shown, open ones only unless reviewed ones are asked for. */
export function shownItems(items: QueueItem[], filter: QueueFilter): QueueItem[] {
	return items.filter(
		(i) => (filter.resolved || i.open) && i.signals.some((s) => filter.reasons.includes(s.kind))
	);
}

/** How many items carry each reason, whatever is shown. */
export function reasonCounts(items: QueueItem[]): Record<Reason, number> {
	const counts = Object.fromEntries(REASONS.map((r) => [r, 0])) as Record<Reason, number>;
	for (const item of items) for (const s of item.signals) counts[s.kind]++;
	return counts;
}

/** One reason in plain words, with what was found. */
export function reasonText(signal: Signal, m: Messages, wholeResult = false): string {
	const d = m.queue.detail;
	switch (signal.kind) {
		case 'failed':
			return d.failed(signal.error);
		case 'not_read':
			return wholeResult ? d.neverStarted : d.notRead;
		case 'truncated':
			return d.truncated;
		case 'disagrees':
			return signal.fields.length > 0
				? d.disagreesFields(listNames(signal.with), listNames(signal.fields))
				: d.disagreesWords(listNames(signal.with), signal.words);
		case 'stale':
			if (signal.unit) return signal.gone ? d.unitGone : d.unitChanged;
			return signal.gone ? d.gone : d.stale;
		case 'not_reviewed':
			return d.notReviewed;
	}
}

/** Where in the document an item is: "Page 3", "Passage 2", or the result as a whole. */
export function placeOf(item: QueueItem, m: Messages): string {
	return item.piece ? pieceLabel(item.piece, m) : m.queue.wholeResult;
}

/** A stable key for an item, for lists and for finding it again after a reload. */
export function itemKey(item: QueueItem): string {
	return `${item.job_id}/${item.setup_id}/${item.source_id}/${item.index ?? 'all'}`;
}

/** Processing problems first, kept apart from the answers that need review. */
export function grouped(items: QueueItem[]): { processing: QueueItem[]; answers: QueueItem[] } {
	return {
		processing: items.filter((i) => i.processing),
		answers: items.filter((i) => !i.processing)
	};
}
