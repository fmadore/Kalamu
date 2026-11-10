import { describe, expect, it } from 'vitest';
import { nextRows, PAGE_ROWS, Paging, rowIndex } from './paging.svelte.js';
import { largeRows, progressOf } from './dev/mockLarge.js';

describe('a long table drawn a page at a time', () => {
	it('draws the first page, then more on asking, and starts again when reset', () => {
		const all = Array.from({ length: 450 }, (_, i) => i);
		const paging = new Paging();
		expect(paging.of(all)).toHaveLength(PAGE_ROWS);
		expect(paging.next(all.length)).toBe(PAGE_ROWS);
		paging.more();
		expect(paging.of(all)).toHaveLength(400);
		expect(paging.next(all.length)).toBe(50);
		paging.more();
		expect(paging.of(all)).toBe(all);
		expect(paging.next(all.length)).toBe(0);
		paging.reset();
		expect(paging.of(all)).toHaveLength(PAGE_ROWS);
	});

	it('says how many rows more would add, and where a row stands under the header', () => {
		expect(nextRows(10, 200)).toBe(0);
		expect(nextRows(250, 200)).toBe(50);
		expect(nextRows(1000, 200, 100)).toBe(100);
		expect(rowIndex(0)).toBe(2);
	});
});

describe("the mock's large sample", () => {
	it('is asked for in the address, within a limit', () => {
		expect(largeRows('?mock')).toBe(0);
		expect(largeRows('?mock&rows=3000')).toBe(3000);
		expect(largeRows('?rows=1e9')).toBe(50_000);
		expect(largeRows('?rows=nope')).toBe(0);
	});

	it("counts a job's pieces as the engine does", () => {
		const piece = (setup: string, source: string, status: string, error: string | null = null) =>
			({ setup_id: setup, source_id: source, status, error }) as const;
		const progress = progressOf(
			[
				piece('a', 'one', 'done'),
				piece('a', 'one', 'cached'),
				piece('a', 'one', 'failed', 'refused'),
				piece('a', 'one', 'failed', 'refused'),
				piece('b', 'one', 'submitted'),
				piece('a', 'two', 'submitted')
			] as Parameters<typeof progressOf>[0],
			['one']
		);
		expect(progress.held).toBe(2);
		expect(progress.files).toEqual([
			{ source_id: 'one', setup_id: 'a', total: 4, done: 2, failed: 2, errors: ['refused'] },
			{ source_id: 'one', setup_id: 'b', total: 1, done: 0, failed: 0, errors: [] }
		]);
	});
});
