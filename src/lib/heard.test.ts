import { describe, expect, it } from 'vitest';
import { placeLines } from './heard.js';

const lines = [
	'Bonjour, merci de nous recevoir. Pouvez-vous vous présenter ?',
	"Je suis l'imam de la mosquée du quartier depuis 1998.",
	'Comment la communauté a-t-elle changé depuis ?'
];

const at = (text: string, spans: ReturnType<typeof placeLines>) =>
	spans.map((s) => (s ? text.slice(s.start, s.end) : null));

describe('the passage heard, found in the text shown', () => {
	it('finds each line past its speaker and time', () => {
		const text = [
			'[00:00:04] Speaker 1: Bonjour, merci de nous recevoir. Pouvez-vous vous présenter ?',
			"Imam Traoré: Je suis l'imam de la mosquée du quartier depuis 1998.",
			'Speaker 1: Comment la communauté a-t-elle changé depuis ?'
		].join('\n');
		expect(at(text, placeLines(text, lines))).toEqual([
			'Bonjour, merci de nous recevoir. Pouvez-vous vous présenter',
			"Je suis l'imam de la mosquée du quartier depuis 1998",
			'Comment la communauté a-t-elle changé depuis'
		]);
	});

	it('finds lines run together in one paragraph, in order', () => {
		const text = lines.join(' ');
		const spans = placeLines(text, lines).flatMap((s) => (s ? [s] : []));
		expect(spans).toHaveLength(3);
		for (let i = 1; i < spans.length; i++) {
			expect(spans[i].start).toBeGreaterThanOrEqual(spans[i - 1].end);
		}
		expect(at(text, placeLines(text, lines))[1]).toBe(
			"Je suis l'imam de la mosquée du quartier depuis 1998"
		);
	});

	it('finds a line whose opening was corrected by its last words', () => {
		const text = [
			lines[0],
			"Moi, je suis l'imam de la mosquée du quartier depuis 1998.",
			lines[2]
		].join('\n');
		const [, second, third] = at(text, placeLines(text, lines));
		expect(second).toContain('mosquée du quartier depuis 1998');
		expect(third).toBe('Comment la communauté a-t-elle changé depuis');
	});

	it('leaves out a line corrected away, and still finds the next', () => {
		const text = [lines[0], lines[2]].join('\n');
		const [first, second, third] = at(text, placeLines(text, lines));
		expect(first).not.toBeNull();
		expect(second).toBeNull();
		expect(third).toBe('Comment la communauté a-t-elle changé depuis');
	});

	it('finds nothing in an empty text or for an empty line', () => {
		expect(placeLines('', lines)).toEqual([null, null, null]);
		expect(placeLines('Bonjour', ['', 'Bonjour'])).toEqual([null, { start: 0, end: 7 }]);
	});
});
