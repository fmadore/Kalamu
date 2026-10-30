import { describe, expect, it } from 'vitest';
import type { ExportFormat } from './bindings/ExportFormat.js';
import { FORMATS, filterTerms, SEE_ALSO, TERMS } from './glossary.js';
import { en } from './i18n/index.js';

describe('glossary', () => {
	it('lists every term the interface defines, once, each with a definition', () => {
		expect([...TERMS].sort()).toEqual(Object.keys(en.glossary.terms).sort());
		for (const t of TERMS) {
			expect(en.glossary.terms[t].word.length).toBeGreaterThan(0);
			expect(en.glossary.terms[t].text.length).toBeGreaterThan(40);
		}
	});

	it('points only to terms it defines, never to the term itself', () => {
		for (const [term, see] of Object.entries(SEE_ALSO)) {
			for (const other of see ?? []) {
				expect(TERMS).toContain(other);
				expect(other).not.toBe(term);
			}
		}
	});

	it('describes every export format and the provenance file', () => {
		const all: ExportFormat[] = [
			'txt',
			'markdown',
			'docx',
			'json',
			'csv',
			'srt',
			'vtt',
			'searchable_pdf'
		];
		expect(FORMATS).toEqual([...all, 'provenance']);
		for (const f of FORMATS) expect(en.glossary.formats[f].length).toBeGreaterThan(20);
	});

	it('finds terms by their word or definition, forgiving capitals and accents', () => {
		expect(filterTerms(en, '')).toEqual(TERMS);
		expect(filterTerms(en, 'whisper')).toContain('whisper');
		expect(filterTerms(en, 'HTR')).toContain('ocr');
		expect(filterTerms(en, 'siglum')).toEqual(['setup']);
		expect(filterTerms(en, 'zzzz')).toEqual([]);
	});
});
