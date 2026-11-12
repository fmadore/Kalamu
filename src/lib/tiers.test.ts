import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import { acrossProviders, byTier, TIERS } from './tiers.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
const ids = (models: ModelInfo[]) => models.map((m) => m.id);
const of = (provider: string) => catalog.models.filter((m) => m.provider === provider);

describe('model tiers', () => {
	it('lists a provider’s models under their tiers, most capable first, in the catalogue’s order', () => {
		const groups = byTier(of('anthropic'));
		expect(groups.map((g) => g.tier)).toEqual(['top', 'balanced', 'light']);
		expect(ids(groups[0].models)).toEqual([
			'claude-fable-5-2',
			'claude-fable-5-1',
			'claude-opus-5-5'
		]);
		expect(ids(groups[2].models)).toEqual(['claude-haiku-5-5']);
	});

	it('leaves out a tier with no model', () => {
		expect(byTier(of('tesseract')).map((g) => g.tier)).toEqual(['light']);
	});

	it('lists every provider’s models by tier, and those on this computer apart', () => {
		const models = [...of('whisper'), ...of('gemini'), ...of('anthropic')];
		const groups = acrossProviders(models, catalog.providers);
		expect(groups.map((g) => g.key)).toEqual([...TIERS, 'local']);
		// Within a tier, each provider in the order given, its models in the catalogue's order.
		expect(ids(groups[0].models)).toEqual([
			'gemini-4-carbon',
			'gemini-4-argon',
			'gemini-3.1-pro-preview',
			'claude-fable-5-2',
			'claude-fable-5-1',
			'claude-opus-5-5'
		]);
		// No Whisper model is set against a cloud model: they have their own heading, most
		// capable first.
		expect(
			groups
				.slice(0, 3)
				.flatMap((g) => g.models)
				.some((m) => m.provider === 'whisper')
		).toBe(false);
		expect(ids(groups[3].models)).toEqual([
			'whisper-large-v3',
			'whisper-large-v3-turbo',
			'whisper-small',
			'whisper-base'
		]);
	});

	it('names no heading for computers when no model runs on one', () => {
		const groups = acrossProviders(of('openai'), catalog.providers);
		expect(groups.every((g) => g.tier !== null)).toBe(true);
	});
});
