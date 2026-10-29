import { describe, expect, it } from 'vitest';
import { columns, minShare, moveCut } from './review.svelte.js';

describe('the boundaries between the review’s panes', () => {
	it('moves a boundary, keeping each pane at its narrowest share', () => {
		expect(moveCut([0.5], 0, 0.4, 0.2)).toEqual([0.4]);
		expect(moveCut([0.5], 0, 0.05, 0.2)).toEqual([0.2]);
		expect(moveCut([0.5], 0, 0.95, 0.2)).toEqual([0.8]);
	});

	it('keeps the middle pane open between two boundaries', () => {
		expect(moveCut([0.33, 0.66], 0, 0.6, 0.2)).toEqual([0.46, 0.66]);
		expect(moveCut([0.33, 0.66], 1, 0.4, 0.2)).toEqual([0.33, 0.53]);
	});

	it('gives each pane its share of the grid', () => {
		expect(columns([0.25])).toBe('minmax(0, 0.25fr) minmax(0, 0.75fr)');
		expect(columns([0.25, 0.75])).toBe('minmax(0, 0.25fr) minmax(0, 0.5fr) minmax(0, 0.25fr)');
	});

	it('lets a pane be about 16rem wide, less in a narrow window', () => {
		expect(minShare(1300)).toBe(0.2);
		expect(minShare(600)).toBe(0.3);
		expect(minShare(0)).toBe(0.2);
	});
});
