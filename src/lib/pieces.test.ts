import { describe, expect, it } from 'vitest';
import { en, fr } from './i18n/index.js';
import type { PieceView } from './bindings/PieceView.js';
import { clock, enginePages, pieceCost, pieceLabel, pieceNumber, typedPages } from './pieces.js';

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

describe('the halves in a list of pages', () => {
	it('names them in the interface language, and back as the engine reads them', () => {
		expect(typedPages('1, 3-5, 8 (left), 9 (right)', fr)).toBe('1, 3-5, 8 (gauche), 9 (droite)');
		expect(enginePages('1, 8 (Gauche), 9 droite', fr)).toBe('1, 8 (left), 9 right');
		expect(typedPages('8 (left)', en)).toBe('8 (left)');
		expect(enginePages('8 (left)', en)).toBe('8 (left)');
	});
});

describe('what a piece cost', () => {
	it('says a piece answered from the cache was not paid for again', () => {
		const view = (over: Partial<PieceView>) =>
			({ status: 'done', cost_usd: 0.003, ...over }) as PieceView;
		expect(pieceCost(view({}), en)).toBe(' · $0.003');
		expect(pieceCost(view({ status: 'cached' }), en)).toBe(' · From cache');
		expect(pieceCost(view({ cost_usd: null }), en)).toBe('');
		expect(pieceCost(undefined, en)).toBe('');
	});
});
