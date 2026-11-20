import { describe, expect, it } from 'vitest';
import type { SourceFile } from './bindings/SourceFile.js';
import { jobSettings } from './newJob.js';
import { longest, offersSample, SAMPLE_OVER } from './newJobSample.js';

const doc = (pages: number | null, kind: SourceFile['kind'] = 'pdf') =>
	({ kind, page_count: pages }) as SourceFile;

describe('the suggestion to try a few pages first', () => {
	it('is made for a text extraction of a long document only', () => {
		expect(offersSample('text_extraction', true, [doc(40)])).toBe(true);
		expect(offersSample('text_extraction', true, [doc(3), doc(SAMPLE_OVER + 1)])).toBe(true);
		// A short document, or many short ones, changes nothing.
		expect(offersSample('text_extraction', true, [doc(SAMPLE_OVER)])).toBe(false);
		expect(offersSample('text_extraction', true, Array(30).fill(doc(1, 'image')))).toBe(false);
		expect(offersSample('text_extraction', true, [])).toBe(false);
		// Other tasks keep the results step's sample of first pages.
		expect(offersSample('contents', true, [doc(40)])).toBe(false);
		expect(offersSample('text_extraction', false, [doc(40)])).toBe(false);
	});

	it('counts the pages of the longest document', () => {
		expect(longest([doc(3), doc(40), doc(null)])).toBe(40);
		expect(longest([])).toBe(0);
	});

	it('asks for spread pages only with a sample of a task read page by page', () => {
		const form = {
			languageMode: 'auto' as const,
			languages: [],
			languagesTaken: false,
			sample: true,
			samplePages: 3,
			sampleSpread: true,
			paged: true,
			lineBreaks: 'running_text' as const,
			linesTaken: false,
			pace: 'fast' as const,
			pagesPerCall: undefined
		};
		expect(jobSettings(form)).toMatchObject({ sample_pages: 3, sample_spread: true });
		expect(jobSettings({ ...form, sampleSpread: false })).not.toHaveProperty('sample_spread');
		expect(jobSettings({ ...form, sample: false })).toMatchObject({ sample_pages: null });
		expect(jobSettings({ ...form, sample: false })).not.toHaveProperty('sample_spread');
		expect(jobSettings({ ...form, paged: false })).not.toHaveProperty('sample_spread');
	});
});
