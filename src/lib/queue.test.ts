import { describe, expect, it } from 'vitest';

import type { QueueItem } from './bindings/QueueItem.js';
import type { Signal } from './bindings/Signal.js';
import { en } from './i18n/en.js';
import {
	DEFAULT_FILTER,
	grouped,
	itemKey,
	placeOf,
	reasonCounts,
	reasonText,
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
		...over
	};
}

const failed = item([{ kind: 'failed', error: '503' }], { index: 0 });
const cut = item([{ kind: 'truncated' }, { kind: 'not_reviewed' }], { index: 1 });
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

	it('counts each reason whatever is shown', () => {
		const counts = reasonCounts(all);
		expect(counts.not_reviewed).toBe(2);
		expect(counts.failed).toBe(1);
		expect(counts.stale).toBe(0);
	});

	it('keeps processing problems apart from answers', () => {
		const { processing, answers } = grouped(all);
		expect(processing).toEqual([failed]);
		expect(answers).toEqual([cut, plain, resolved]);
	});

	it('says each reason in words', () => {
		expect(reasonText({ kind: 'failed', error: '503' }, en)).toBe('Failed: 503');
		expect(reasonText({ kind: 'not_read' }, en, true)).toBe('Not started yet');
		expect(reasonText({ kind: 'disagrees', with: ['pro'], words: 1, fields: [] }, en)).toBe(
			'Reads differently from pro: 1 word'
		);
		expect(
			reasonText({ kind: 'disagrees', with: ['pro', 'mistral'], words: 0, fields: ['People'] }, en)
		).toBe('Answers differently from pro and mistral: People');
		expect(reasonText({ kind: 'stale', gone: true }, en)).toContain('deleted');
	});

	it('names the place and keeps items apart by result and piece', () => {
		expect(placeOf(cut, en)).toBe('Page 2');
		expect(placeOf(item([{ kind: 'not_read' }], { piece: null, index: null }), en)).toBe(
			'The whole result'
		);
		expect(itemKey(cut)).not.toBe(itemKey(plain));
		expect(itemKey(item([{ kind: 'not_read' }], { index: null }))).toBe('j1/a/s1/all');
	});
});
