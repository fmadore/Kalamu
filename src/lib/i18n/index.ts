// The interface language. English only for now; a new language is a catalog typed `Messages`
// added to CATALOGS, nothing else.

import { en, type Messages } from './en.js';

export const LOCALES = ['en'] as const;
export type Locale = (typeof LOCALES)[number];

const CATALOGS: Record<Locale, Messages> = { en };

let current: Locale = 'en';

/** The catalog for the interface language. */
export function messages(): Messages {
	return CATALOGS[current];
}

export function setLocale(locale: Locale): void {
	current = locale;
	if (typeof document !== 'undefined') document.documentElement.lang = CATALOGS[locale].locale.tag;
}

/** Names joined as the interface language joins them: "Whisper Base and Mistral OCR". */
export function listNames(names: string[]): string {
	return new Intl.ListFormat(CATALOGS[current].locale.tag, { type: 'conjunction' }).format(names);
}

/** A language's name in the interface language, from its ISO code (`mos` → "Mossi"). */
export function languageName(code: string, fallback?: string): string {
	try {
		const names = new Intl.DisplayNames([CATALOGS[current].locale.tag], { type: 'language' });
		const name = names.of(code);
		if (name && name !== code) return name;
	} catch {
		// An invalid code: fall through to the catalogue's own name.
	}
	return fallback ?? code;
}

export function formatUsd(value: number): string {
	const digits = value < 0.1 ? 3 : 2;
	return new Intl.NumberFormat(CATALOGS[current].locale.tag, {
		style: 'currency',
		currency: 'USD',
		minimumFractionDigits: digits,
		maximumFractionDigits: digits
	}).format(value);
}

export function formatDateTime(ms: number): string {
	return new Date(ms).toLocaleString(CATALOGS[current].locale.tag, {
		dateStyle: 'medium',
		timeStyle: 'short'
	});
}

export function formatPercent(value: number): string {
	return new Intl.NumberFormat(CATALOGS[current].locale.tag, {
		style: 'percent',
		maximumFractionDigits: 1
	}).format(value);
}

export { en };
export type { Messages };
