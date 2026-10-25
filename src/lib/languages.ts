// Which languages New job offers, in what order, and how a typed search finds them. Kept out of
// the components so it can be tested on its own.

import type { Catalog } from './bindings/Catalog.js';
import type { LanguageInfo } from './bindings/LanguageInfo.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import { languageName } from './i18n/index.js';

/** Mistral OCR reads any language and takes none; a missing flag means the model takes one. */
export function takesLanguage(model: ModelInfo | undefined): boolean {
	return model?.takes_language !== false;
}

/**
 * The languages every chosen model can use. Models that read any language leave the whole
 * catalogue; a model with its own list (Whisper) narrows it.
 */
export function offeredLanguages(
	catalog: Catalog,
	models: (ModelInfo | undefined)[]
): LanguageInfo[] {
	const lists = models.flatMap((m) =>
		m && takesLanguage(m) && Array.isArray(m.languages) ? [m.languages] : []
	);
	if (lists.length === 0) return catalog.languages;
	const [first, ...rest] = lists;
	const codes = first.filter((code) => rest.every((list) => list.includes(code)));
	return codes.map(
		(code) => catalog.languages.find((l) => l.code === code) ?? { code, name: code }
	);
}

/** A language's name in the interface language, or the catalogue's when the system has none. */
export function languageLabel(catalog: Catalog, code: string): string {
	return languageName(code, catalog.languages.find((l) => l.code === code)?.name);
}

/** The chosen models that cannot use a language, by name. */
export function modelsWithout(code: string, models: (ModelInfo | undefined)[]): string[] {
	return models.flatMap((m) =>
		m && takesLanguage(m) && Array.isArray(m.languages) && !m.languages.includes(code)
			? [m.name]
			: []
	);
}

/** Lower case without accents, so "moore" finds "Mooré" and "e" finds "É". */
export function searchKey(text: string): string {
	return text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
}

/**
 * Languages sorted by their name in the interface language, filtered by a typed search that
 * matches that name, the English name or the code.
 */
export function findLanguages(
	languages: LanguageInfo[],
	query: string,
	name: (l: LanguageInfo) => string,
	locale: string
): LanguageInfo[] {
	const q = searchKey(query.trim());
	const collator = new Intl.Collator(locale, { sensitivity: 'base' });
	return languages
		.filter(
			(l) =>
				q === '' ||
				searchKey(name(l)).includes(q) ||
				searchKey(l.name).includes(q) ||
				l.code.toLowerCase() === q
		)
		.sort((a, b) => collator.compare(name(a), name(b)));
}
