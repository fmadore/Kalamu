// The interface language. A new language is a catalog typed `Messages` added to LOCALES and
// CATALOGS, nothing else: Settings lists it, and the system's language picks it.

import type { AnalysisSchema } from '../bindings/AnalysisSchema.js';
import type { Field } from '../bindings/Field.js';
import shippedFields from './defaultFields.json';
import shippedNames from './defaultNames.json';
import { en, type FieldText, type Messages } from './en.js';
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

/** Every language's catalog, for what must be read whatever language wrote it (the screening's
 * Keep and Set aside tags). */
export function everyCatalog(): Messages[] {
	return LOCALES.map((l) => CATALOGS[l]);
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

const collators = new Map<string, Intl.Collator>();

/**
 * Text in the order a reader of the interface language expects ("Éric" beside "Eric", not after
 * "Zoé"). Each list states how it differs from the plain order: `numeric` puts `part 2` before
 * `part 10`, `sensitivity: 'base'` sets capitals and accents aside. `tag` fixes the language
 * instead, for a catalog given explicitly.
 */
export function readerOrder(
	options: Intl.CollatorOptions = {},
	tag: string = CATALOGS[current].locale.tag
): Intl.Collator {
	const key = `${tag} ${JSON.stringify(options)}`;
	let collator = collators.get(key);
	if (!collator) {
		collator = new Intl.Collator(tag, options);
		collators.set(key, collator);
	}
	return collator;
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
	return text && preset.builtin && !preset.modified
		? { ...preset, name: text.name, description: text.description }
		: preset;
}

/** The default prompts' fields as shipped, written by the prompts crate's tests. */
const SHIPPED = shippedFields as unknown as Record<string, Field[]>;

/**
 * A field as shown: its label and help in the interface language, and its choices' labels by
 * the value stored (`choiceLabel`). Never sent back to the app.
 */
export type ShownField = Field & { choiceLabels?: Record<string, string> };

function shownFields(
	fields: Field[],
	shipped: Field[],
	texts: Record<string, FieldText> | undefined
): Field[] {
	if (!texts) return fields;
	return fields.map((field) => {
		const was = shipped.find((s) => s.key === field.key);
		const text = texts[field.key];
		const kind = field.kind;
		// A group Kalamu wraps around the preset's fields (a table of contents' entries, with
		// their title and pages): the preset's fields are inside.
		if (!was) {
			return kind.type === 'entries' || kind.type === 'cited'
				? { ...field, kind: { ...kind, fields: shownFields(kind.fields, shipped, texts) } }
				: field;
		}
		if (!text) return field;
		const shown: ShownField = { ...field };
		if (field.label === was.label) shown.label = text.label;
		if (field.instructions === was.instructions) shown.instructions = text.help;
		const before = was.kind;
		if (
			(kind.type === 'choice' || kind.type === 'choices') &&
			before.type === kind.type &&
			text.values
		) {
			const labels: Record<string, string> = {};
			for (const value of kind.values) {
				const label = text.values[value];
				if (label && before.values.includes(value)) labels[value] = label;
			}
			shown.choiceLabels = labels;
		}
		if ((kind.type === 'entries' || kind.type === 'cited') && before.type === kind.type) {
			shown.kind = { ...kind, fields: shownFields(kind.fields, before.fields, text.fields) };
		}
		return shown;
	});
}

/**
 * A default prompt's fields in the interface language, for the screen only: each label, help
 * and choice that is, word for word, the default's shipped English (`defaultFields.json`) is
 * shown in the language's words, and anything the researcher wrote stays as written. Keys,
 * kinds and choice values are untouched, so what is stored, cached and sent never changes. The
 * id is a job's frozen prompt's `preset_id` or a preset's own; another id changes nothing.
 */
export function localiseFields<S extends AnalysisSchema | null | undefined>(
	presetId: string | null | undefined,
	schema: S
): S {
	const shipped = presetId ? SHIPPED[presetId] : undefined;
	const texts = presetId ? CATALOGS[current].presets[presetId]?.fields : undefined;
	if (!schema || !shipped || !texts) return schema;
	return { ...schema, fields: shownFields(schema.fields, shipped, texts) };
}

/** The default prompts' names as the core ships them, in English, by id. */
const SHIPPED_NAMES = shippedNames as Record<string, string>;

/**
 * A default prompt's name as a job froze it, for the screen only: in the interface language
 * when it is, word for word, the default's shipped English name (or its name in another
 * language Kalamu has); a name the researcher wrote stays as written.
 */
export function presetName(presetId: string | null | undefined, name: string): string {
	if (!presetId) return name;
	const shipped = SHIPPED_NAMES[presetId];
	const isDefault =
		name === shipped || LOCALES.some((l) => CATALOGS[l].presets[presetId]?.name === name);
	if (!isDefault) return name;
	return CATALOGS[current].presets[presetId]?.name ?? shipped ?? name;
}

/** A frozen prompt with its name and fields as `presetName` and `localiseFields` show them. */
export function localisePrompt<
	P extends { preset_id: string | null; preset_name?: string; schema?: AnalysisSchema }
>(prompt: P): P {
	const named =
		prompt.preset_name === undefined
			? prompt
			: { ...prompt, preset_name: presetName(prompt.preset_id, prompt.preset_name) };
	return named.schema ? { ...named, schema: localiseFields(named.preset_id, named.schema) } : named;
}

/**
 * A workflow's, a run's or a step's name as shown: in the interface language when it is, word
 * for word, one Kalamu wrote, in any language it has (an example's name or step, as when the
 * workflow was made from the example in English and is read in French, or a default prompt's
 * name, which a step added from a prompt takes); any other as written. Never stored.
 */
export function shownName(name: string): string {
	const here = CATALOGS[current];
	for (const l of LOCALES) {
		const examples = CATALOGS[l].workflows.examples;
		for (const [kind, words] of Object.entries(examples) as [
			keyof typeof examples,
			(typeof examples)[keyof typeof examples]
		][]) {
			if (words.name === name) return here.workflows.examples[kind].name;
			const at = words.steps.indexOf(name);
			if (at >= 0) return here.workflows.examples[kind].steps[at] ?? name;
		}
	}
	for (const [id, shipped] of Object.entries(SHIPPED_NAMES)) {
		if (shipped === name || LOCALES.some((l) => CATALOGS[l].presets[id]?.name === name)) {
			return here.presets[id]?.name ?? shipped;
		}
	}
	return name;
}

/** A choice as shown, from the value stored: its label in the interface language for a
 * default's own choice (`localiseFields`), any other as it is. */
export function choiceLabel(field: Field, value: string): string {
	return (field as ShownField).choiceLabels?.[value] ?? value;
}

/**
 * A field's name where only its name (and key) is known, as in a project's dictionary or the
 * list of what to check: in the interface language when it is, word for word, the label of a
 * default field (of that key), else as it is.
 */
export function fieldLabel(label: string, key?: string): string {
	const find = (fields: Field[], texts: Record<string, FieldText> | undefined): string | null => {
		for (const field of fields) {
			const text = texts?.[field.key];
			if (!text) continue;
			if (field.label === label && (key === undefined || field.key === key)) return text.label;
			const kind = field.kind;
			if (kind.type === 'entries' || kind.type === 'cited') {
				const inner = find(kind.fields, text.fields);
				if (inner) return inner;
			}
		}
		return null;
	};
	for (const [id, fields] of Object.entries(SHIPPED)) {
		const found = find(fields, CATALOGS[current].presets[id]?.fields);
		if (found) return found;
	}
	return label;
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
