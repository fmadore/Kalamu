import { describe, expect, it } from 'vitest';
import type { PieceView } from './bindings/PieceView.js';
import type { Setup } from './bindings/Setup.js';
import { comparedOf, shownText, witnessesOf, witnessName } from './compare.svelte.js';
import { pieces } from './dev/mockJobs.js';
import { en } from './i18n/en.js';

const setup = (id: string, label: string) => ({ id, label }) as Setup;
const setups = [setup('a', 'flash'), setup('b', 'mistral'), setup('c', 'claude')];
const view = (over: Partial<PieceView>) =>
	({ ...pieces[0], corrected_text: null, source_text: null, ...over }) as PieceView;

describe('the texts a review compares', () => {
	it('compares the text shown with each other set-up that answered', () => {
		const views: Record<string, PieceView | undefined> = {
			a: view({ ai_text: 'one' }),
			b: view({ ai_text: 'two', corrected_text: 'two, corrected' }),
			c: view({ ai_text: null })
		};
		expect(witnessesOf(setups, (s) => views[s.id], 'text_extraction', en)).toEqual([
			{ key: 'b', siglum: 'b', text: 'two, corrected' }
		]);
	});

	it('adds the model’s own text under a correction, and the text a correction corrected', () => {
		const lead = view({ ai_text: 'model', corrected_text: 'mine', source_text: 'original' });
		expect(witnessesOf(setups.slice(0, 1), () => lead, 'text_correction', en)).toEqual([
			{ key: 'model', siglum: 'a', text: 'model' },
			{ key: 'source', siglum: 'O', text: 'original' }
		]);
		expect(shownText(lead)).toBe('mine');
		expect(shownText(undefined)).toBe('');
	});

	it('shows a translation beside its source, never compared with it', () => {
		const lead = view({ ai_text: 'model', corrected_text: 'mine', source_text: 'original' });
		expect(witnessesOf(setups.slice(0, 1), () => lead, 'translation', en)).toEqual([
			{ key: 'model', siglum: 'a', text: 'model' }
		]);
	});

	it('names who reads what each siglum names', () => {
		expect(witnessName('a', setups, en)).toBe('flash, model output');
		expect(witnessName('O', setups, en)).toBe('Original text');
		expect(witnessName('c', setups, en)).toBe('claude');
		expect(witnessName('z', setups, en)).toBe('z');
	});

	it('opens the witness chosen, else another set-up before the model', () => {
		const witnesses = [
			{ key: 'model', siglum: 'a', text: '' },
			{ key: 'b', siglum: 'b', text: '' }
		];
		expect(comparedOf(witnesses, 'model')?.key).toBe('model');
		expect(comparedOf(witnesses, null)?.key).toBe('b');
		expect(comparedOf(witnesses.slice(0, 1), 'gone')?.key).toBe('model');
		expect(comparedOf([], null)).toBeUndefined();
	});
});
