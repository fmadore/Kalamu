import { describe, expect, it } from 'vitest';
import { agreementRows, formatKappa } from './agreement.js';
import type { JobAgreement } from './bindings/JobAgreement.js';
import { en, fr } from './i18n/index.js';

const setups = [{ id: 'one' }, { id: 'two' }, { id: 'three' }];

const agreement: JobAgreement = {
	fields: [
		{
			key: 'tone',
			label: 'Tone',
			measure: 'choice',
			pairs: [
				{ first: 'one', second: 'two', pieces: 40, agreement: 0.825, kappa: 0.6 },
				{ first: 'one', second: 'three', pieces: 12, agreement: 1, kappa: null },
				{ first: 'two', second: 'three', pieces: 0, agreement: null, kappa: null }
			]
		},
		{
			key: 'places',
			label: 'Places',
			measure: 'overlap',
			pairs: [{ first: 'one', second: 'two', pieces: 40, agreement: 2 / 3, kappa: null }]
		}
	]
};

describe('agreement between set-ups', () => {
	it('gives each field and pair a row, the field named on its first', () => {
		const rows = agreementRows(agreement, setups, en);
		expect(rows.map((r) => [r.label, r.span, r.pair.join('')])).toEqual([
			['Tone', 3, 'ab'],
			['', 0, 'ac'],
			['', 0, 'bc'],
			['Places', 1, 'ab']
		]);
	});

	it('says when kappa cannot be computed, does not apply, or nothing was compared', () => {
		const rows = agreementRows(agreement, setups, en);
		expect(rows.map((r) => [r.agreement, r.kappa])).toEqual([
			['82.5%', '0.60'],
			['100%', en.agreement.noKappa],
			[en.agreement.none, en.agreement.none],
			['66.7%', en.agreement.notApplicable]
		]);
	});

	it('writes kappa as each language writes numbers, never as minus zero', () => {
		expect(formatKappa(0.4, en)).toBe('0.40');
		expect(formatKappa(0.4, fr)).toBe('0,40');
		expect(formatKappa(-0.001, en)).toBe('0.00');
		expect(formatKappa(-0.25, en)).toBe('-0.25');
	});
});
