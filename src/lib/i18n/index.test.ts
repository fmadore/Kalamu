import { describe, expect, it } from 'vitest';

import { en } from './en.js';
import { placeholders } from './index.js';
import { CREDIT_URLS } from '../credits.js';

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
