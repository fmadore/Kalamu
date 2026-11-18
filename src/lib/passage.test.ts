import { describe, expect, it } from 'vitest';
import {
	PASSAGE_LIMITS,
	charCount,
	passageAround,
	passageProblem,
	withPassage
} from './passage.js';

const TEXT = 'Le Gouverneur général a reçu ce matin deux cents fideles de la ville.';

describe('a passage chosen for a model to correct', () => {
	it('takes the engine’s limits', () => {
		expect(PASSAGE_LIMITS).toEqual({ min: 3, max: 4000, context: 300 });
	});

	it('sends the whole text around a short passage as context', () => {
		const at = TEXT.indexOf('deux');
		const around = passageAround(TEXT, at, at + 'deux cents fideles'.length);
		expect(around).toEqual({
			before: 'Le Gouverneur général a reçu ce matin ',
			passage: 'deux cents fideles',
			after: ' de la ville.',
			at
		});
		expect(withPassage(TEXT, around, 'deux cents fidèles')).toBe(
			'Le Gouverneur général a reçu ce matin deux cents fidèles de la ville.'
		);
	});

	it('cuts a longer context at a word’s edge', () => {
		const at = TEXT.indexOf('deux');
		const around = passageAround(TEXT, at, at + 4, 12);
		// Twelve characters before are “e ce matin ” after a part of a word: cut to the word.
		expect(around.before).toBe('ce matin ');
		expect(around.after).toBe(' cents');
	});

	it('never cuts a character written in two units', () => {
		const text = `${'𝔄'.repeat(3)} mot`;
		const at = text.indexOf('mot');
		const around = passageAround(text, at, at + 3, 4);
		expect(around.before.charCodeAt(0)).not.toBe(0xdd04);
		expect([...around.before].every((c) => c === '𝔄' || c === ' ')).toBe(true);
	});

	it('refuses a passage too short or too long', () => {
		expect(passageProblem(' a, ')).toBe('short');
		expect(passageProblem('abc')).toBeNull();
		expect(passageProblem('ابن')).toBeNull();
		expect(passageProblem('a'.repeat(PASSAGE_LIMITS.max + 1))).toBe('long');
		// Counted in characters, as the engine counts them.
		expect(passageProblem('𝔄'.repeat(PASSAGE_LIMITS.max))).toBeNull();
		expect(charCount('𝔄é')).toBe(2);
	});
});
