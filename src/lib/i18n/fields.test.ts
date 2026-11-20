// The default prompts' fields in the interface language (#210): every field the defaults ship
// (`defaultFields.json`, written by the prompts crate's tests from the defaults themselves) has
// its French label, help and choices, and showing them changes nothing stored.

import { afterEach, describe, expect, it } from 'vitest';

import type { AnalysisSchema } from '../bindings/AnalysisSchema.js';
import type { Field } from '../bindings/Field.js';
import type { Setup } from '../bindings/Setup.js';
import { shownSetup } from '../review.svelte.js';
import shippedJson from './defaultFields.json';
import type { FieldText } from './en.js';
import { fr } from './fr.js';
import {
	choiceLabel,
	fieldLabel,
	localiseFields,
	localisePreset,
	localisePrompt,
	setLocale
} from './index.js';

const shipped = shippedJson as unknown as Record<string, Field[]>;

/** What a field lacks in French, as `preset › key: what`, nested fields included. */
function missing(id: string, fields: Field[], texts: Record<string, FieldText> | undefined) {
	const out: string[] = [];
	const walk = (fields: Field[], texts: Record<string, FieldText> | undefined, path: string) => {
		for (const field of fields) {
			const at = `${path} › ${field.key}`;
			const text = texts?.[field.key];
			if (!text?.label.trim()) out.push(`${at}: label`);
			if (field.instructions.trim() && !text?.help.trim()) out.push(`${at}: help`);
			const kind = field.kind;
			if (kind.type === 'choice' || kind.type === 'choices') {
				for (const value of kind.values) {
					if (!text?.values?.[value]?.trim()) out.push(`${at}: choice “${value}”`);
				}
			}
			if (kind.type === 'entries' || kind.type === 'cited') walk(kind.fields, text?.fields, at);
		}
	};
	walk(fields, texts, id);
	return out;
}

/** French texts left for a field, help or choice the defaults no longer ship. */
function stale(id: string, fields: Field[], texts: Record<string, FieldText> | undefined) {
	const out: string[] = [];
	const walk = (fields: Field[], texts: Record<string, FieldText> | undefined, path: string) => {
		for (const [key, text] of Object.entries(texts ?? {})) {
			const field = fields.find((f) => f.key === key);
			const at = `${path} › ${key}`;
			if (!field) {
				out.push(at);
				continue;
			}
			const kind = field.kind;
			const values = kind.type === 'choice' || kind.type === 'choices' ? kind.values : [];
			for (const value of Object.keys(text.values ?? {})) {
				if (!values.includes(value)) out.push(`${at}: choice “${value}”`);
			}
			const inner = kind.type === 'entries' || kind.type === 'cited' ? kind.fields : [];
			if (text.fields) walk(inner, text.fields, at);
		}
	};
	walk(fields, texts, id);
	return out;
}

const copy = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;

/** A schema as stored: the shipped fields of a default. */
const stored = (id: string): AnalysisSchema => ({ fields: copy(shipped[id] ?? []) });

/** What a schema stores and sends: everything but the words shown. */
function structure(fields: Field[]): unknown {
	return fields.map((f) => ({
		key: f.key,
		reason: f.reason,
		kind:
			f.kind.type === 'entries' || f.kind.type === 'cited'
				? { ...f.kind, fields: structure(f.kind.fields) }
				: f.kind
	}));
}

afterEach(() => setLocale('en'));

describe('the default fields in French', () => {
	it('are read from the defaults themselves', () => {
		expect(Object.keys(shipped)).toEqual(
			expect.arrayContaining(['analysis.entities', 'analysis.register', 'contents.book'])
		);
	});

	it('each have a label, help and every choice in French', () => {
		const lacking = Object.entries(shipped).flatMap(([id, fields]) =>
			missing(id, fields, fr.presets[id]?.fields)
		);
		expect(lacking).toEqual([]);
	});

	it('keep no French text for a field or choice no longer shipped', () => {
		const left = Object.entries(fr.presets).flatMap(([id, text]) =>
			text.fields ? stale(id, shipped[id] ?? [], text.fields) : []
		);
		expect(left).toEqual([]);
	});

	it('give one French name to each English name, as a dictionary or the list to check shows it', () => {
		const names = new Map<string, Set<string>>();
		const walk = (fields: Field[], texts: Record<string, FieldText> | undefined) => {
			for (const field of fields) {
				const text = texts?.[field.key];
				if (!text) continue;
				const seen = names.get(field.label) ?? new Set<string>();
				seen.add(text.label);
				names.set(field.label, seen);
				const kind = field.kind;
				if (kind.type === 'entries' || kind.type === 'cited') walk(kind.fields, text.fields);
			}
		};
		for (const [id, fields] of Object.entries(shipped)) walk(fields, fr.presets[id]?.fields);
		const split = [...names].filter(([, seen]) => seen.size > 1).map(([label]) => label);
		expect(split).toEqual([]);
	});
});

describe('localiseFields', () => {
	it('shows a default’s labels, help and choices in French, and nothing stored changes', () => {
		setLocale('fr');
		const schema = stored('analysis.tone');
		const before = JSON.stringify(schema);
		const shown = localiseFields('analysis.tone', schema);
		expect(JSON.stringify(schema)).toBe(before);
		expect(shown.fields.map((f) => f.label)).toEqual([
			'Sujet principal',
			'Tonalité',
			'Faits ou opinions'
		]);
		expect(shown.fields[1].instructions).toBe(
			'La façon dont le document présente son sujet principal.'
		);
		// The model still answers, and Kalamu still stores, the English choice.
		expect(structure(shown.fields)).toEqual(structure(schema.fields));
		const tone = shown.fields[1];
		expect(tone.kind).toEqual({
			type: 'choice',
			values: ['Positive', 'Neutral', 'Negative', 'Mixed']
		});
		expect(choiceLabel(tone, 'Mixed')).toBe('Mitigée');
		// An answer outside the choices is shown as given.
		expect(choiceLabel(tone, 'Ambivalent')).toBe('Ambivalent');
	});

	it('changes back to the stored English when the interface does', () => {
		setLocale('fr');
		const schema = stored('analysis.entities');
		expect(localiseFields('analysis.entities', schema).fields[0].label).toBe('Personnes');
		setLocale('en');
		const shown = localiseFields('analysis.entities', schema);
		expect(shown).toBe(schema);
		expect(shown.fields[0].label).toBe('People');
	});

	it('keeps what the researcher wrote, field by field, and another prompt as it is', () => {
		setLocale('fr');
		const schema = stored('analysis.summary_publication');
		schema.fields[0].label = 'Référence complète';
		const methods = schema.fields.find((f) => f.key === 'methods');
		if (methods?.kind.type !== 'choices') throw new Error('methods is a field of choices');
		methods.kind.values.push('Photo elicitation');
		const shown = localiseFields('analysis.summary_publication', schema);
		expect(shown.fields[0].label).toBe('Référence complète');
		expect(shown.fields[0].instructions).toBe(
			fr.presets['analysis.summary_publication'].fields?.reference.help
		);
		const shownMethods = shown.fields.find((f) => f.key === 'methods');
		if (!shownMethods) throw new Error('methods shown');
		expect(choiceLabel(shownMethods, 'Archival research')).toBe('Recherche en archives');
		expect(choiceLabel(shownMethods, 'Photo elicitation')).toBe('Photo elicitation');
		// A prompt of the researcher's own with the same fields, or none at all.
		expect(localiseFields('user-1', schema)).toBe(schema);
		expect(localiseFields(null, schema)).toBe(schema);
		expect(localiseFields('analysis.tone', undefined)).toBeUndefined();
	});

	it('reaches the fields of a repeating group and of entries that quote the text', () => {
		setLocale('fr');
		const register = localiseFields('analysis.register', stored('analysis.register'));
		const entries = register.fields[0];
		if (entries.kind.type !== 'entries') throw new Error('entries');
		const kind = entries.kind.fields.find((f) => f.key === 'kind_of_entry');
		expect(kind?.label).toBe('Nature de l’acte');
		expect(kind && choiceLabel(kind, 'Death or burial')).toBe('Décès ou inhumation');
		const interview = localiseFields('analysis.interview', stored('analysis.interview'));
		const quotations = interview.fields.find((f) => f.key === 'quotations');
		if (quotations?.kind.type !== 'cited') throw new Error('quotations');
		expect(quotations.kind.fields[0].label).toBe('Pourquoi c’est important');
	});

	it('reaches a table of contents’ fields inside the entries Kalamu wraps around them', () => {
		setLocale('fr');
		const own = stored('contents.book').fields;
		const title: Field = {
			key: 'title',
			label: 'Title',
			instructions: 'The full title.',
			kind: { type: 'text' },
			reason: false
		};
		const merged: AnalysisSchema = {
			fields: [
				{
					key: 'entries',
					label: 'Entries',
					instructions: 'One entry per article.',
					kind: { type: 'entries', fields: [title, ...own] },
					reason: false
				}
			]
		};
		const shown = localiseFields('contents.book', merged);
		const kind = shown.fields[0].kind;
		if (kind.type !== 'entries') throw new Error('entries');
		expect(kind.fields.map((f) => f.label)).toEqual(['Title', 'Niveau', 'Auteurs', 'Résumé']);
		expect(choiceLabel(kind.fields[1], 'Front or back matter')).toBe('Pages liminaires ou annexes');
	});

	it('leaves a job’s set-up as stored, shown in French for the review only', () => {
		setLocale('fr');
		const setup = {
			id: 's',
			label: 'gemini',
			provider: 'gemini',
			model: 'gemini',
			prompt: {
				preset_id: 'analysis.tone',
				preset_name: 'Tone of the document',
				template: '',
				text: 'sent',
				user_prompt: '',
				schema: stored('analysis.tone')
			},
			options: { temperature: null, diarize: false, timestamps: false, speakers: null }
		} as Setup;
		const before = JSON.stringify(setup);
		const shown = shownSetup(setup);
		expect(JSON.stringify(setup)).toBe(before);
		expect(shown.prompt.schema?.fields[1].label).toBe('Tonalité');
		expect(shown.prompt.text).toBe('sent');
		expect(localisePrompt({ preset_id: 'analysis.tone' })).toEqual({ preset_id: 'analysis.tone' });
	});

	it('names a field known only by its name, as a dictionary or the list to check gives it', () => {
		setLocale('fr');
		expect(fieldLabel('Places', 'places')).toBe('Lieux');
		expect(fieldLabel('Tone')).toBe('Tonalité');
		expect(fieldLabel('People named', 'people_named')).toBe('Personnes citées');
		expect(fieldLabel('Lieux de culte', 'places')).toBe('Lieux de culte');
		expect(fieldLabel('Places', 'sites')).toBe('Places');
		setLocale('en');
		expect(fieldLabel('Places', 'places')).toBe('Places');
	});

	it('gives a preset its French name without carrying its fields onto it', () => {
		setLocale('fr');
		const preset = localisePreset({
			id: 'analysis.tone',
			name: 'Tone of the document',
			description: '',
			builtin: true,
			modified: false
		});
		expect(preset).toEqual({
			id: 'analysis.tone',
			name: 'Tonalité du document',
			description: fr.presets['analysis.tone'].description,
			builtin: true,
			modified: false
		});
	});
});
