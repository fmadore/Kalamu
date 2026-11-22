// What the review queue shows of the engine's items: which reasons are on, each reason in
// words, and where an item opens. The items, their reasons, the sample drawn at random and the
// counts come from Rust.

import type { QueueItem } from './bindings/QueueItem.js';
import type { QueueItemKey } from './bindings/QueueItemKey.js';
import type { Signal } from './bindings/Signal.js';
import lists from './coreLists.json';
import type { Messages } from './i18n/index.js';
import { fieldLabel, listNames } from './i18n/index.js';
import { pieceLabel } from './pieces.js';
import type { View } from './state.svelte.js';

export type Reason = Signal['kind'];

/** Every reason, in the order Rust lists them (`Signal::KINDS`, written by `cargo test`). */
export const REASONS = lists.queue_reasons as Reason[];

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

/** Kept aside by the researcher with the answer it holds now; one kept with an earlier answer
 * is listed with the others again. */
export const isKept = (item: QueueItem): boolean => item.kept?.current === true;

/** The items with one of the reasons shown, open ones only unless reviewed ones are asked for.
 * Items kept aside stay in view whatever the reasons shown; the sample is listed apart. */
export function shownItems(items: QueueItem[], filter: QueueFilter): QueueItem[] {
	return items.filter(
		(i) =>
			!i.sample &&
			(filter.resolved || i.open) &&
			(isKept(i) || i.signals.some((s) => filter.reasons.includes(s.kind)))
	);
}

/** The pages drawn at random, whatever the filters: listed for the sample, not for a problem. */
export function sampleItems(items: QueueItem[]): QueueItem[] {
	return items.filter((i) => i.sample);
}

/** The most pages a sample holds, and the largest seed (`MAX_SAMPLE`, `MAX_SEED` in Rust). */
export const SAMPLE_MAX: number = lists.sample_limits.size;
export const SEED_MAX: number = lists.sample_limits.seed;

/**
 * A seed as the researcher typed it, copied from a manifest or an article: `null` when the box
 * is empty (a new draw), the number when it is digits only and six of them at most, `'invalid'`
 * otherwise. Spaces around it are ignored.
 */
export function parseSeed(typed: string): number | null | 'invalid' {
	const text = typed.trim();
	if (text === '') return null;
	if (!/^\d+$/.test(text)) return 'invalid';
	const seed = Number(text);
	return seed <= SEED_MAX ? seed : 'invalid';
}

/** What names an item to keep it aside or put it back. */
export function keyOf(item: QueueItem): QueueItemKey {
	return {
		job_id: item.job_id,
		setup_id: item.setup_id,
		source_id: item.source_id,
		index: item.index
	};
}

/** The review an item opens: its own page, and its own set-up shown first, so that the answer
 * listed is the one read, corrected and marked rather than the job's first set-up's. */
export function reviewOf(item: QueueItem): Extract<View, { name: 'review' }> {
	return {
		name: 'review',
		jobId: item.job_id,
		sourceId: item.source_id,
		index: item.index ?? undefined,
		setupId: item.setup_id
	};
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
			return signal.reason === 'filter'
				? d.truncatedFilter
				: signal.reason === 'other'
					? d.truncatedOther
					: d.truncated;
		case 'disagrees':
			return signal.fields.length > 0
				? d.disagreesFields(
						listNames(signal.with),
						listNames(signal.fields.map((label) => fieldLabel(label)))
					)
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

/** Processing problems first, kept apart from the answers that need review; then the items the
 * researcher kept aside. */
export function grouped(items: QueueItem[]): {
	processing: QueueItem[];
	answers: QueueItem[];
	kept: QueueItem[];
} {
	const listed = items.filter((i) => !isKept(i));
	return {
		processing: listed.filter((i) => i.processing),
		answers: listed.filter((i) => !i.processing),
		kept: items.filter(isKept)
	};
}
