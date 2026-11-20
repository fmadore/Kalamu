import { afterEach, describe, expect, it } from 'vitest';

import { guideUrl } from './guide.js';
import { setLocale } from './i18n/index.js';

describe('guideUrl', () => {
	afterEach(() => setLocale('en'));

	it('opens the English guide in English', () => {
		setLocale('en');
		expect(guideUrl()).toBe('https://fmadore.github.io/Kalamu/guide/');
	});

	it('opens the French guide in French', () => {
		setLocale('fr');
		expect(guideUrl()).toBe('https://fmadore.github.io/Kalamu/guide/fr/');
	});
});
