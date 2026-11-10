import { describe, expect, it } from 'vitest';
import type { ExportFormat } from './bindings/ExportFormat.js';
import { FORMATS, filterTerms, keyboard, SEE_ALSO, TERMS } from './glossary.js';
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
			'searchable_pdf',
			'alto'
		];
		expect(FORMATS).toEqual([...all, 'provenance']);
		for (const f of FORMATS) expect(en.glossary.formats[f].length).toBeGreaterThan(20);
	});

	it('finds terms by their word or definition, forgiving capitals and accents', () => {
		const all = filterTerms(en, '');
		expect([...all].sort()).toEqual([...TERMS].sort());
		const words = all.map((t) => en.glossary.terms[t].word.toLowerCase());
		expect(words).toEqual([...words].sort((a, b) => a.localeCompare(b)));
		expect(filterTerms(en, 'whisper')).toContain('whisper');
		expect(filterTerms(en, 'HTR')).toContain('ocr');
		expect(filterTerms(en, 'siglum')).toEqual(['setup']);
		expect(filterTerms(en, 'zzzz')).toEqual([]);
	});

	it('lists the shortcuts by where they work, written for each system', () => {
		for (const mac of [false, true]) {
			const groups = keyboard(en, mac);
			expect(groups.map((g) => g.place)).toEqual(Object.keys(en.glossary.keyPlaces));
			for (const g of groups) {
				for (const row of g.rows) {
					expect(row.does.length).toBeGreaterThan(5);
					expect(row.keys.every((k) => k.length > 0)).toBe(true);
				}
			}
		}
		const keys = (mac: boolean) => keyboard(en, mac).flatMap((g) => g.rows.flatMap((r) => r.keys));
		expect(keys(false)).toContain('Ctrl N');
		expect(keys(false)).toContain('Alt ←');
		expect(keys(false)).not.toContain('⌘[');
		expect(keys(true)).toContain('⌘N');
		expect(keys(true)).toContain('⌘[');
		expect(keys(true).some((k) => k.includes('Ctrl'))).toBe(false);
	});
});
