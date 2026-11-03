import { describe, expect, it } from 'vitest';
import type { ProfileColumn } from './bindings/ProfileColumn.js';
import { draftProfile, hasFixedTexts, keyValue, moved, valueKey } from './profiles.js';

const col = (heading: string): ProfileColumn => ({ heading, value: { kind: 'title' } });

describe('export profiles', () => {
	it('turns each value into a key and back', () => {
		for (const value of [
			{ kind: 'title' },
			{ kind: 'tags' },
			{ kind: 'part', field: 'shelfmark' },
			{ kind: 'from', field: 'repository' },
			{ kind: 'status', task: 'analysis' },
			{ kind: 'model', task: 'contents' },
			{ kind: 'text', task: 'audio_transcription' }
		] as const) {
			expect(keyValue(valueKey(value))).toEqual(value);
		}
		expect(keyValue('fixed', { kind: 'fixed', text: 'Fonds AOF' })).toEqual({
			kind: 'fixed',
			text: 'Fonds AOF'
		});
		expect(keyValue('fixed', { kind: 'title' })).toEqual({ kind: 'fixed', text: '' });
		expect(keyValue('part:nothing')).toEqual({ kind: 'title' });
		// An analysis writes no text a column can take.
		expect(keyValue('text:analysis')).toEqual({ kind: 'title' });
		expect(keyValue('answer', { kind: 'answer', key: 'people' })).toEqual({
			kind: 'answer',
			key: 'people'
		});
		expect(keyValue('answer')).toEqual({ kind: 'answer', key: '' });
		// Switching to the dictionary's labels keeps the field.
		expect(keyValue('labelled', { kind: 'answer', key: 'people' })).toEqual({
			kind: 'labelled',
			key: 'people'
		});
	});

	it('moves a column within the list', () => {
		const columns = [col('a'), col('b'), col('c')];
		expect(moved(columns, 0, 1).map((c) => c.heading)).toEqual(['b', 'a', 'c']);
		expect(moved(columns, 2, 1)).toBe(columns);
		expect(moved(columns, 2, -5).map((c) => c.heading)).toEqual(['c', 'a', 'b']);
	});

	it('tells whether a fixed text would be shared', () => {
		const profile = draftProfile('Omeka', [col('a')]);
		expect(hasFixedTexts(profile)).toBe(false);
		profile.columns.push({ heading: 'o:item_set', value: { kind: 'fixed', text: ' ' } });
		expect(hasFixedTexts(profile)).toBe(false);
		profile.columns[1].value = { kind: 'fixed', text: 'Fonds AOF' };
		expect(hasFixedTexts(profile)).toBe(true);
	});
});
