import { afterEach, describe, expect, it } from 'vitest';

import { guideUrl } from './guide.js';
import { setLocale } from './i18n/index.js';

describe('guideUrl', () => {
	afterEach(() => setLocale('en'));

	it('opens the English guide in English', () => {
		setLocale('en');
		expect(guideUrl()).toBe('https://kalamu.frederickmadore.com/guide/');
	});

	it('opens the French guide in French', () => {
		setLocale('fr');
		expect(guideUrl()).toBe('https://kalamu.frederickmadore.com/guide/fr/');
	});

	it('opens a page of the guide in the interface language', () => {
		setLocale('en');
		expect(guideUrl('other-servers')).toBe(
			'https://kalamu.frederickmadore.com/guide/other-servers.html'
		);
		setLocale('fr');
		expect(guideUrl('other-servers')).toBe(
			'https://kalamu.frederickmadore.com/guide/fr/other-servers.html'
		);
	});
});
