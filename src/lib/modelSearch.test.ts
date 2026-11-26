import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import { findModels, matchesWords, searchWords } from './modelSearch.js';
import { acrossProviders } from './tiers.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
const providerName = (m: ModelInfo) =>
	catalog.providers.find((p) => p.id === m.provider)?.name ?? '';
const of = (...providers: string[]) => catalog.models.filter((m) => providers.includes(m.provider));
const ids = (groups: ReturnType<typeof findModels>) =>
	groups.flatMap((g) => g.models.map((m) => m.id));

describe('typing in a model list', () => {
	it('compares words without case or accents', () => {
		expect(searchWords('  Gémini   FLASH ')).toEqual(['gemini', 'flash']);
		expect(matchesWords(['eleve'], ['Modèle élevé'])).toBe(true);
		expect(matchesWords(searchWords('ÉLÈVE'), ['eleve'])).toBe(true);
	});

	it('needs every word, each in the name, the id or the provider’s name', () => {
		expect(
			matchesWords(
				['claude', 'haiku'],
				['Claude Haiku 5.5', 'claude-haiku-5-5', 'Anthropic Claude']
			)
		).toBe(true);
		// A word found in the provider's name alone counts.
		expect(
			matchesWords(
				['anthropic', 'haiku'],
				['Claude Haiku 5.5', 'claude-haiku-5-5', 'Anthropic Claude']
			)
		).toBe(true);
		expect(
			matchesWords(
				['claude', 'gemini'],
				['Claude Haiku 5.5', 'claude-haiku-5-5', 'Anthropic Claude']
			)
		).toBe(false);
		// Nothing typed finds everything.
		expect(matchesWords([], ['anything'])).toBe(true);
	});

	it('keeps the headings that still hold a model, and the models in their order', () => {
		const groups = acrossProviders(of('anthropic', 'gemini', 'tesseract'), catalog.providers);
		expect(findModels(groups, '', providerName)).toBe(groups);
		const found = findModels(groups, 'claude', providerName);
		expect(found.every((g) => g.models.every((m) => m.provider === 'anthropic'))).toBe(true);
		expect(found.map((g) => g.key)).not.toContain('local');
		expect(ids(found)).toEqual(ids(groups).filter((id) => id.startsWith('claude')));
		// By the id, as the server or the catalogue writes it.
		expect(ids(findModels(groups, 'haiku-5-5', providerName))).toEqual(['claude-haiku-5-5']);
		// On this computer: Tesseract's heading alone.
		expect(findModels(groups, 'tessér', providerName).map((g) => g.key)).toEqual(['local']);
		expect(findModels(groups, 'nothing like this', providerName)).toEqual([]);
	});
});
