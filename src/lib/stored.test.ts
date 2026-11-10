import { afterEach, describe, expect, it, vi } from 'vitest';
import { keep, keepFor, recall, recallFor } from './stored.js';

const isNumber = (v: unknown) => (typeof v === 'number' ? v : null);

function memory(): Storage {
	const items = new Map<string, string>();
	return {
		get length() {
			return items.size;
		},
		clear: () => items.clear(),
		getItem: (k) => items.get(k) ?? null,
		key: (i) => [...items.keys()][i] ?? null,
		removeItem: (k) => void items.delete(k),
		setItem: (k, v) => void items.set(k, v)
	};
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('what is remembered on this computer', () => {
	it('reads back what was kept', () => {
		vi.stubGlobal('localStorage', memory());
		keep('split', 0.4);
		expect(recall('split', isNumber, 0.5)).toBe(0.4);
	});

	it('gives the default for nothing kept, something unreadable or a value refused', () => {
		vi.stubGlobal('localStorage', memory());
		expect(recall('split', isNumber, 0.5)).toBe(0.5);
		localStorage.setItem('split', '{not json');
		expect(recall('split', isNumber, 0.5)).toBe(0.5);
		keep('split', 'wide');
		expect(recall('split', isNumber, 0.5)).toBe(0.5);
	});

	it('works without storage at all', () => {
		vi.stubGlobal('localStorage', undefined);
		expect(() => keep('split', 0.4)).not.toThrow();
		expect(recall('split', isNumber, 0.5)).toBe(0.5);
		expect(() => keepFor('turns', 's1', 90, 2)).not.toThrow();
		expect(recallFor('turns', 's1', isNumber)).toBeNull();
	});
});

describe('what is remembered for each of several things', () => {
	it('reads back each one, forgets one set to null, and refuses what it does not accept', () => {
		vi.stubGlobal('localStorage', memory());
		keepFor('turns', 's1', 90, 10);
		keepFor('turns', 's2', 'sideways', 10);
		expect(recallFor('turns', 's1', isNumber)).toBe(90);
		expect(recallFor('turns', 's2', isNumber)).toBeNull();
		expect(recallFor('turns', 's3', isNumber)).toBeNull();
		keepFor('turns', 's1', null, 10);
		expect(recallFor('turns', 's1', isNumber)).toBeNull();
	});

	it('keeps only the ones kept last', () => {
		vi.stubGlobal('localStorage', memory());
		keepFor('turns', 's1', 90, 2);
		keepFor('turns', 's2', 180, 2);
		// Kept again, s1 is now the most recent, so s2 goes when s3 comes.
		keepFor('turns', 's1', 270, 2);
		keepFor('turns', 's3', 90, 2);
		expect(recallFor('turns', 's1', isNumber)).toBe(270);
		expect(recallFor('turns', 's2', isNumber)).toBeNull();
		expect(recallFor('turns', 's3', isNumber)).toBe(90);
	});

	it('reads nothing from a value that is no list', () => {
		vi.stubGlobal('localStorage', memory());
		keep('turns', { s1: 90 });
		expect(recallFor('turns', 's1', isNumber)).toBeNull();
		keepFor('turns', 's1', 90, 2);
		expect(recallFor('turns', 's1', isNumber)).toBe(90);
	});
});
