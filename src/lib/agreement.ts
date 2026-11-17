// Agreement between set-ups over a whole analysis, as the engine counts it (`job_agreement`),
// laid out for the job page's table: one row per field and pair of set-ups, each value said in
// the interface language. The counting itself is Rust's; this only writes it out.

import { siglum } from './apparatus.js';
import type { JobAgreement } from './bindings/JobAgreement.js';
import type { AgreementMeasure } from './bindings/AgreementMeasure.js';
import { formatPercent, type Messages } from './i18n/index.js';

/** One row of the table. */
export interface AgreementRow {
	key: string;
	/** The field's name, given on its first row only (the others fall under it). */
	label: string;
	/** How many rows the field spans: one per pair of set-ups. */
	span: number;
	measure: AgreementMeasure;
	/** The two set-ups' sigla, in the job's order. */
	pair: [string, string];
	agreement: string;
	kappa: string;
	pieces: number;
}

/** Kappa written to two decimals, as the interface language writes numbers: `0.40`, `0,40`. */
export function formatKappa(kappa: number, m: Messages): string {
	// A rounding to zero is no disagreement: never "-0.00".
	const value = Math.abs(kappa) < 0.005 ? 0 : kappa;
	return new Intl.NumberFormat(m.locale.tag, {
		minimumFractionDigits: 2,
		maximumFractionDigits: 2
	}).format(value);
}

/** The table's rows, field by field, each field's pairs in the job's order. */
export function agreementRows(
	agreement: JobAgreement,
	setups: { id: string }[],
	m: Messages
): AgreementRow[] {
	const letter = (id: string) =>
		siglum(
			Math.max(
				0,
				setups.findIndex((s) => s.id === id)
			)
		);
	return agreement.fields.flatMap((field) =>
		field.pairs.map((p, i) => {
			let kappa = '';
			if (field.measure === 'overlap') kappa = m.agreement.notApplicable;
			else if (p.pieces === 0) kappa = m.agreement.none;
			else kappa = p.kappa === null ? m.agreement.noKappa : formatKappa(p.kappa, m);
			return {
				key: field.key,
				label: i === 0 ? field.label : '',
				span: i === 0 ? field.pairs.length : 0,
				measure: field.measure,
				pair: [letter(p.first), letter(p.second)] as [string, string],
				agreement: p.agreement === null ? m.agreement.none : formatPercent(p.agreement),
				kappa,
				pieces: p.pieces
			};
		})
	);
}
