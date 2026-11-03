import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { LanguageInfo } from './bindings/LanguageInfo.js';
import {
	findLanguages,
	modelsNeedingTwoLetters,
	modelsTakingOne,
	modelsWithout,
	modelsWithoutTimes,
	offeredLanguages,
	takesLanguage
} from './languages.js';

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

	it('names the transcription models that take one language, or only two-letter codes', () => {
		const chosen = [
			model('gpt-transcribe'),
			model('voxtral-mini-transcribe-3-0'),
			model('whisper-base')
		];
		expect(modelsTakingOne(1, chosen)).toEqual([]);
		expect(modelsTakingOne(2, chosen)).toEqual([
			model('voxtral-mini-transcribe-3-0')?.name,
			model('whisper-base')?.name
		]);
		expect(modelsWithoutTimes(chosen)).toEqual([model('voxtral-mini-transcribe-3-0')?.name]);
		expect(modelsNeedingTwoLetters('fr', chosen)).toEqual([]);
		expect(modelsNeedingTwoLetters('mos', chosen)).toEqual([model('gpt-transcribe')?.name]);
		expect(modelsTakingOne(3, [model('gemini-3.5-transcribe')])).toEqual([]);
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
