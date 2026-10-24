import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import { en } from './i18n/index.js';
import {
	acceptsPrompt,
	modelInfo,
	newDraft,
	suggestName,
	toRequest,
	warningText
} from './newJob.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;

describe('new job form', () => {
	it('starts with the recommended model the user has a key for', () => {
		const d = newDraft(catalog, 'text_extraction', 'p', () => true);
		expect(d.model).toBe('gemini-3.8-flash');
		const claudeOnly = newDraft(catalog, 'text_extraction', 'p', (p) => p === 'anthropic');
		expect(claudeOnly.provider).toBe('anthropic');
	});

	it('suggests a different model for a comparison', () => {
		const first = newDraft(catalog, 'text_extraction', 'p', () => true);
		const second = newDraft(catalog, 'text_extraction', 'p', () => true, [
			`${first.provider}/${first.model}`
		]);
		expect(`${second.provider}/${second.model}`).not.toBe(`${first.provider}/${first.model}`);
	});

	it('sends no prompt to models that take none', () => {
		const ocr = {
			...newDraft(catalog, 'text_extraction', 'preset', () => true),
			provider: 'mistral',
			model: 'mistral-ocr-4-1'
		};
		expect(acceptsPrompt(modelInfo(catalog, ocr))).toBe(false);
		expect(toRequest(catalog, 'text_extraction', ocr).preset_id).toBeNull();
		const flash = newDraft(catalog, 'text_extraction', 'preset', () => true);
		expect(toRequest(catalog, 'text_extraction', flash).preset_id).toBe('preset');
		expect(toRequest(catalog, 'text_extraction', flash).options.diarize).toBe(false);
	});

	it('words input warnings and names jobs after what was added', () => {
		const text = warningText({ source_id: 'x', kind: 'double_page_spread', pages: [2, 3] }, en);
		expect(text).toContain('page 2, 3');
		expect(
			suggestName(
				['C:\\Archives\\Bulletin'],
				['C:\\Archives\\Bulletin\\a.pdf', 'C:\\Archives\\Bulletin\\b.pdf']
			)
		).toBe('Bulletin');
		expect(suggestName(['C:\\x\\lettre.jpg'], ['C:\\x\\lettre.jpg'])).toBe('lettre.jpg');
	});
});
