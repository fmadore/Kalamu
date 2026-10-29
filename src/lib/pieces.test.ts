import { describe, expect, it } from 'vitest';
import { en } from './i18n/index.js';
import { clock, pieceLabel, pieceNumber } from './pieces.js';

describe('the names of pieces', () => {
	it('names pages, their halves, passages and whole sources', () => {
		expect(pieceLabel({ kind: 'page', number: 3 }, en)).toBe('Page 3');
		expect(pieceLabel({ kind: 'page', number: 3, half: 'left' }, en)).toBe('Page 3, left half');
		expect(pieceLabel({ kind: 'passage', number: 2 }, en)).toBe('Passage 2');
		expect(pieceLabel({ kind: 'whole' }, en)).toBe('Whole document');
		expect(pieceLabel({ kind: 'whole' }, en, true)).toBe('Whole video');
	});

	it('names a piece of a recording by where it starts', () => {
		const piece = { kind: 'segment', index: 1, start_ms: 3_723_400, end_ms: 4_000_000 } as const;
		expect(pieceLabel(piece, en)).toBe('From 01:02:03');
		expect(clock(0)).toBe('00:00:00');
	});

	it('gives the strip a page’s own number, and other pieces their place', () => {
		expect(pieceNumber({ kind: 'page', number: 12 }, 0, en)).toBe('12');
		expect(pieceNumber({ kind: 'page', number: 12, half: 'right' }, 0, en)).toBe('12r');
		expect(pieceNumber({ kind: 'passage', number: 7 }, 2, en)).toBe('3');
	});
});
