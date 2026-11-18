import { describe, expect, it } from 'vitest';

import type { QueueItem } from './bindings/QueueItem.js';
import type { Signal } from './bindings/Signal.js';
import { en } from './i18n/en.js';
import {
	DEFAULT_FILTER,
	grouped,
	isKept,
	itemKey,
	keyOf,
	placeOf,
	REASONS,
	reasonText,
	parseSeed,
	sampleItems,
	SEED_MAX,
	shownItems
} from './queue.js';

function item(signals: Signal[], over: Partial<QueueItem> = {}): QueueItem {
	return {
		document_id: 'd1',
		document: 'Bulletin, May 1957',
		job_id: 'j1',
		job_name: 'Bulletin',
		task: 'text_extraction',
		source_id: 's1',
		setup_id: 'a',
		setup_label: 'flash',
		model: 'flash',
		preferred: false,
		piece: { kind: 'page', number: 2 },
		index: 1,
		revision: '3f9a1c07be52',
		answered_at: 0,
		reviewed: false,
		corrected: false,
		processing: signals.some((s) => s.kind === 'failed' || s.kind === 'not_read'),
		open: true,
		signals,
		decided_at: null,
		sample: false,
		kept: null,
		...over
	};
}

const failed = item([{ kind: 'failed', error: '503' }], { index: 0 });
const cut = item([{ kind: 'truncated', reason: 'limit' }, { kind: 'not_reviewed' }], { index: 1 });
const plain = item([{ kind: 'not_reviewed' }], { index: 2 });
const resolved = item([{ kind: 'disagrees', with: ['pro'], words: 3, fields: [] }], {
	index: 3,
	reviewed: true,
	open: false
});
const all = [failed, cut, plain, resolved];

describe('the review queue', () => {
	it('shows what is flagged and open by default, without pages only not reviewed', () => {
		expect(shownItems(all, DEFAULT_FILTER)).toEqual([failed, cut]);
		expect(shownItems(all, { ...DEFAULT_FILTER, resolved: true })).toEqual([failed, cut, resolved]);
		expect(shownItems(all, { reasons: ['not_reviewed'], resolved: false })).toEqual([cut, plain]);
	});

	it('lists every reason Rust has, each with words', () => {
		expect(REASONS).toEqual([
			'failed',
			'not_read',
			'truncated',
			'disagrees',
			'stale',
			'not_reviewed'
		]);
	});

	it('keeps processing problems apart from answers', () => {
		const { processing, answers } = grouped(all);
		expect(processing).toEqual([failed]);
		expect(answers).toEqual([cut, plain, resolved]);
	});

	it('says each reason in words', () => {
		expect(reasonText({ kind: 'failed', error: '503' }, en)).toBe('Failed: 503');
		expect(reasonText({ kind: 'not_read' }, en, true)).toBe('Not started yet');
		expect(reasonText({ kind: 'truncated', reason: 'limit' }, en)).toContain('length limit');
		expect(reasonText({ kind: 'truncated', reason: 'filter' }, en)).toBe(
			'Stopped by the provider’s filter: the end may be missing'
		);
		expect(reasonText({ kind: 'truncated', reason: 'other' }, en)).not.toContain('limit');
		expect(reasonText({ kind: 'disagrees', with: ['pro'], words: 1, fields: [] }, en)).toBe(
			'Reads differently from pro: 1 word'
		);
		expect(
			reasonText({ kind: 'disagrees', with: ['pro', 'mistral'], words: 0, fields: ['People'] }, en)
		).toBe('Answers differently from pro and mistral: People');
		expect(reasonText({ kind: 'stale', gone: true, unit: false }, en)).toContain('deleted');
		expect(reasonText({ kind: 'stale', gone: false, unit: true }, en)).toBe(
			'Read an item whose pages have changed since'
		);
	});

	it('names the place and keeps items apart by result and piece', () => {
		expect(placeOf(cut, en)).toBe('Page 2');
		expect(placeOf(item([{ kind: 'not_read' }], { piece: null, index: null }), en)).toBe(
			'The whole result'
		);
		expect(itemKey(cut)).not.toBe(itemKey(plain));
		expect(itemKey(item([{ kind: 'not_read' }], { index: null }))).toBe('j1/a/s1/all');
	});

	it('lists items kept aside apart, whatever the reasons shown, unless kept with an earlier answer', () => {
		const note = { note: 'Ask the archivist', kept_at: 0 };
		const keptPlain = item([{ kind: 'not_reviewed' }], {
			index: 4,
			kept: { ...note, current: true }
		});
		const keptEarlier = item([{ kind: 'truncated', reason: 'limit' }], {
			index: 5,
			kept: { ...note, current: false }
		});
		const list = [...all, keptPlain, keptEarlier];
		const shown = shownItems(list, DEFAULT_FILTER);
		// Only not reviewed, it would be hidden; kept aside, it stays in view.
		expect(shown).toContain(keptPlain);
		expect(isKept(keptPlain)).toBe(true);
		expect(isKept(keptEarlier)).toBe(false);
		const { answers, kept } = grouped(shown);
		expect(kept).toEqual([keptPlain]);
		// Kept with an earlier answer, the new one is listed with the others.
		expect(answers).toContain(keptEarlier);
		// Reviewed and resolved, it leaves the view with the other resolved items.
		const reviewed = { ...keptPlain, reviewed: true, open: false };
		expect(shownItems([reviewed], DEFAULT_FILTER)).toEqual([]);
	});

	it('lists the sample apart, never among what is flagged', () => {
		const drawn = item([], { index: 6, sample: true, reviewed: true, open: false });
		const drawnOpen = item([{ kind: 'not_reviewed' }], { index: 7, sample: true });
		const list = [...all, drawn, drawnOpen];
		expect(shownItems(list, { reasons: REASONS, resolved: true })).not.toContain(drawn);
		expect(shownItems(list, { reasons: REASONS, resolved: true })).not.toContain(drawnOpen);
		expect(sampleItems(list)).toEqual([drawn, drawnOpen]);
	});

	it('names an item by its result and piece to keep it aside', () => {
		expect(keyOf(cut)).toEqual({ job_id: 'j1', setup_id: 'a', source_id: 's1', index: 1 });
		expect(keyOf(item([{ kind: 'not_read' }], { piece: null, index: null })).index).toBeNull();
	});
});

describe('a seed typed by hand', () => {
	it('is six digits at most, or nothing for a new draw', () => {
		expect(SEED_MAX).toBe(999_999);
		expect(parseSeed('')).toBeNull();
		expect(parseSeed('   ')).toBeNull();
		expect(parseSeed('271828')).toBe(271_828);
		expect(parseSeed(' 42 ')).toBe(42);
		expect(parseSeed('000007')).toBe(7);
		expect(parseSeed('0')).toBe(0);
	});

	it('refuses what is not a seed rather than guessing', () => {
		for (const typed of ['1000000', '12.5', '-3', '1e3', '12 34', 'seed', '١٢٣']) {
			expect(parseSeed(typed)).toBe('invalid');
		}
	});
});
