import { afterEach, describe, expect, it, vi } from 'vitest';
import { keep, recall } from './stored.js';

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
	});
});
