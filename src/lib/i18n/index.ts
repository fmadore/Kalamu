// The interface language. A new language is a catalog typed `Messages` added to LOCALES and
// CATALOGS, nothing else: Settings lists it, and the system's language picks it.

import { en, type Messages } from './en.js';
import { fr } from './fr.js';

export const LOCALES = ['en', 'fr'] as const;
export type Locale = (typeof LOCALES)[number];

const CATALOGS: Record<Locale, Messages> = { en, fr };

let current: Locale = 'en';

/** The interface language in use. */
export function locale(): Locale {
	return current;
}

/** Each language Kalamu offers, by its own name ("Français"), for the choice in Settings. */
export function localeNames(): { locale: Locale; name: string }[] {
	return LOCALES.map((l) => ({ locale: l, name: CATALOGS[l].locale.name }));
}

/**
 * The language the setting asks for: `en` or `fr` as chosen, or for `system` the first of the
 * system's languages (`fr-CA`, `de-DE`…, in order of preference) that Kalamu has, else English.
 */
export function resolveLocale(setting: string, system: readonly string[]): Locale {
	if ((LOCALES as readonly string[]).includes(setting)) return setting as Locale;
	for (const tag of system) {
		const base = tag.toLowerCase().split(/[-_]/)[0];
		const found = LOCALES.find((l) => l === base);
		if (found) return found;
	}
	return 'en';
}

/** The catalog for the interface language. */
export function messages(): Messages {
	return CATALOGS[current];
}

export function setLocale(locale: Locale): void {
	current = locale;
	if (typeof document !== 'undefined') document.documentElement.lang = CATALOGS[locale].locale.tag;
}

/**
 * A message with `{name}` placeholders, cut into its plain text and the names, so a component
 * can set each name as a link or a mark: "uses {ffmpeg} under" → text, `ffmpeg`, text.
 */
export function placeholders(message: string): ({ text: string } | { name: string })[] {
	return message
		.split(/\{(\w+)\}/)
		.map((part, i) => (i % 2 === 1 ? { name: part } : { text: part }))
		.filter((part) => !('text' in part) || part.text !== '');
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
	return fallback ?? CATALOGS[current].languageNames[code] ?? code;
}

/**
 * A default prompt's name and description in the interface language. The defaults are written
 * in English by the core; the user's own prompts, and defaults they edited, stay as written.
 */
export function localisePreset<
	P extends { id: string; name: string; description: string; builtin: boolean; modified: boolean }
>(preset: P): P {
	const text = CATALOGS[current].presets[preset.id];
	return text && preset.builtin && !preset.modified ? { ...preset, ...text } : preset;
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

export function formatDate(ms: number): string {
	return new Date(ms).toLocaleDateString(CATALOGS[current].locale.tag, { dateStyle: 'medium' });
}

/** A calendar day written `2026-11-02` (the model catalogue's date), as the interface language
 * writes it; anything else as it is. Read as a local day, so no time zone moves it. */
export function formatDay(iso: string): string {
	const day = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
	if (!day) return iso;
	return formatDate(new Date(Number(day[1]), Number(day[2]) - 1, Number(day[3])).getTime());
}

export function formatPercent(value: number): string {
	return new Intl.NumberFormat(CATALOGS[current].locale.tag, {
		style: 'percent',
		maximumFractionDigits: 1
	}).format(value);
}

export { en, fr };
export type { Messages };
