import { describe, expect, it } from 'vitest';
import { Selection } from './selection.svelte.js';

describe('a selection of rows', () => {
	it('ticks and unticks a row', () => {
		const s = new Selection();
		s.toggle('a');
		s.toggle('b');
		expect(s.keys).toEqual(['a', 'b']);
		s.toggle('a');
		expect(s.keys).toEqual(['b']);
		expect(s.has('b')).toBe(true);
		expect(s.has('a')).toBe(false);
	});

	it('adds rows each once', () => {
		const s = new Selection();
		s.toggle('a');
		s.add(['a', 'b', 'c', 'b']);
		expect(s.keys).toEqual(['a', 'b', 'c']);
	});

	it('gives the ticked rows in the rows’ order', () => {
		const s = new Selection();
		s.add(['c', 'a']);
		const rows = [{ id: 'a' }, { id: 'b' }, { id: 'c' }];
		expect(s.of(rows, (r) => r.id)).toEqual([{ id: 'a' }, { id: 'c' }]);
	});

	it('ticks every row shown, then clears them all', () => {
		const s = new Selection();
		expect(s.allOf([])).toBe(false);
		s.toggle('b');
		s.toggleAll(['a', 'b']);
		expect(s.keys).toEqual(['a', 'b']);
		expect(s.allOf(['a', 'b'])).toBe(true);
		s.toggleAll(['a', 'b']);
		expect(s.keys).toEqual([]);
	});

	it('clears rows ticked but filtered out when all those shown are unticked', () => {
		const s = new Selection();
		s.add(['a', 'hidden']);
		s.toggleAll(['a']);
		expect(s.keys).toEqual([]);
	});

	it('forgets rows that no longer exist', () => {
		const s = new Selection();
		s.add(['a', 'b', 'c']);
		s.prune(new Set(['c', 'a', 'd']));
		expect(s.keys).toEqual(['a', 'c']);
		s.clear();
		expect(s.keys).toEqual([]);
	});
});
