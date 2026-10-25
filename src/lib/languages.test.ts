import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { LanguageInfo } from './bindings/LanguageInfo.js';
import { findLanguages, modelsWithout, offeredLanguages, takesLanguage } from './languages.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
const model = (id: string) => catalog.models.find((m) => m.id === id);

describe('languages offered in New job', () => {
	it('lists every language for models that read any, and narrows to what Whisper knows', () => {
		expect(offeredLanguages(catalog, [model('gemini-3.8-flash')])).toHaveLength(
			catalog.languages.length
		);
		const whisper = offeredLanguages(catalog, [model('gemini-3.8-flash'), model('whisper-base')]);
		expect(whisper.length).toBeLessThan(catalog.languages.length);
		expect(whisper.map((l) => l.code)).toContain('fr');
	});

	it('names the models that cannot use a language, and leaves out those that take none', () => {
		const hausa = modelsWithout('ha', [model('whisper-base'), model('claude-sonnet-5-5')]);
		expect(hausa.length <= 1).toBe(true);
		expect(takesLanguage(model('mistral-ocr-4-1'))).toBe(false);
		expect(modelsWithout('xx', [model('mistral-ocr-4-1')])).toEqual([]);
	});

	it('sorts by name and finds a language by name without accents, English name or code', () => {
		const langs: LanguageInfo[] = [
			{ code: 'mos', name: 'Mooré' },
			{ code: 'fr', name: 'French' },
			{ code: 'ar', name: 'Arabic' }
		];
		const fr = (l: LanguageInfo) => ({ mos: 'mooré', fr: 'français', ar: 'arabe' })[l.code] ?? '';
		expect(findLanguages(langs, '', fr, 'fr').map((l) => l.code)).toEqual(['ar', 'fr', 'mos']);
		expect(findLanguages(langs, 'moore', fr, 'fr').map((l) => l.code)).toEqual(['mos']);
		expect(findLanguages(langs, 'French', fr, 'fr').map((l) => l.code)).toEqual(['fr']);
		expect(findLanguages(langs, 'AR', fr, 'fr').map((l) => l.code)).toEqual(['ar']);
	});
});
