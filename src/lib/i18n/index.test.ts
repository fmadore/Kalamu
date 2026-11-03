import { describe, expect, it } from 'vitest';

import { en } from './en.js';
import { fr } from './fr.js';
import { placeholders, resolveLocale } from './index.js';
import { CREDIT_URLS } from '../credits.js';

/** Every key of a catalog, as dotted paths, with what each holds: text, a list or a function
 * of so many arguments. The type check catches a missing key; this also catches a list or a
 * function that changed shape. */
function shape(value: unknown, path = ''): string[] {
	if (typeof value === 'function') return [`${path} fn/${value.length}`];
	if (Array.isArray(value)) return [`${path} list`];
	if (typeof value === 'object' && value !== null) {
		return Object.entries(value).flatMap(([k, v]) => shape(v, path ? `${path}.${k}` : k));
	}
	return [`${path} ${typeof value}`];
}

/** Keys whose entries differ by design: English has no default prompts to translate, and each
 * language names its own languages the system lacks. */
const OWN = ['presets', 'languageNames'];
const shared = (catalog: object) =>
	shape(catalog).filter((key) => !OWN.some((own) => key.startsWith(`${own}.`)));

describe('the catalogs', () => {
	it('have the same keys, lists and functions in English and French', () => {
		expect(shared(fr)).toEqual(shared(en));
	});

	it('leave no French message empty where the English has text', () => {
		const blank = (catalog: object) =>
			Object.entries(flat(catalog))
				.filter(([, v]) => v === '')
				.map(([k]) => k);
		expect(blank(fr)).toEqual(blank(en));
	});
});

function flat(value: unknown, path = ''): Record<string, unknown> {
	if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
		return Object.assign(
			{},
			...Object.entries(value).map(([k, v]) => flat(v, path ? `${path}.${k}` : k))
		);
	}
	return { [path]: value };
}

describe('resolveLocale', () => {
	it('takes the language chosen in Settings', () => {
		expect(resolveLocale('fr', ['en-US'])).toBe('fr');
		expect(resolveLocale('en', ['fr-FR'])).toBe('en');
	});

	it('takes the first of the system’s languages Kalamu has, else English', () => {
		expect(resolveLocale('system', ['fr-CA', 'en-US'])).toBe('fr');
		expect(resolveLocale('system', ['de-DE', 'fr-FR'])).toBe('fr');
		expect(resolveLocale('system', ['en_GB', 'fr'])).toBe('en');
		expect(resolveLocale('system', ['de-DE'])).toBe('en');
		expect(resolveLocale('system', [])).toBe('en');
		expect(resolveLocale('xx', ['FR'])).toBe('fr');
	});
});

describe('placeholders', () => {
	it('cuts a message into its text and the names to set apart', () => {
		expect(placeholders('Kalamu uses {ffmpeg} and {opus}.')).toEqual([
			{ text: 'Kalamu uses ' },
			{ name: 'ffmpeg' },
			{ text: ' and ' },
			{ name: 'opus' },
			{ text: '.' }
		]);
		expect(placeholders('{a}{b}')).toEqual([{ name: 'a' }, { name: 'b' }]);
		expect(placeholders('No names.')).toEqual([{ text: 'No names.' }]);
	});

	it('finds a link and a name for every credit the message names', () => {
		for (const part of placeholders(en.settings.credits)) {
			if (!('name' in part)) continue;
			expect(CREDIT_URLS).toHaveProperty(part.name);
			expect(en.settings.creditNames).toHaveProperty(part.name);
		}
	});
});
