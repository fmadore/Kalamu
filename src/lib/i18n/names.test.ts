// The default prompts' names, and the names the workflow examples give, in the interface
// language: a job's prompt in a review's header, a workflow made from an example in English
// and read in French. Only what Kalamu wrote is shown in another language; what the researcher
// wrote stays as written, and nothing stored changes.

import { readFileSync } from 'node:fs';
import { afterEach, describe, expect, it } from 'vitest';

import shippedNames from './defaultNames.json';
import { en } from './en.js';
import { fr } from './fr.js';
import { localisePrompt, presetName, setLocale, shownName } from './index.js';

afterEach(() => setLocale('en'));

describe('default prompts’ names', () => {
	it('are listed as the prompts crate ships them, each with its French name', () => {
		// `defaultNames.json` is written by hand: it must say what `defaults.rs` says.
		const source = readFileSync('crates/prompts/src/defaults.rs', 'utf8');
		const shipped = Object.fromEntries(
			[...source.matchAll(/id: "([^"]+)",\s*task: [^,]+,\s*name: "([^"]+)"/g)].map((m) => [
				m[1],
				m[2]
			])
		);
		const dictionary = readFileSync('crates/prompts/src/dictionary.rs', 'utf8');
		const merges = /id: DICTIONARY_MERGES,\s*task: [^,]+,\s*name: "([^"]+)"/.exec(dictionary);
		expect(Object.keys(shipped).length).toBeGreaterThan(10);
		expect(shippedNames).toEqual({ ...shipped, 'dictionary.merges': merges?.[1] });
		for (const id of Object.keys(shippedNames)) expect(fr.presets[id]?.name, id).toBeTruthy();
	});

	it('show a job’s prompt in the interface language, and a name of the researcher’s as written', () => {
		setLocale('fr');
		expect(presetName('analysis.entities', 'People, places and organisations')).toBe(
			'Personnes, lieux et organisations'
		);
		expect(presetName('analysis.interview', 'Interview: themes and quotations')).toBe(
			fr.presets['analysis.interview'].name
		);
		expect(presetName('analysis.entities', 'Mes personnes')).toBe('Mes personnes');
		expect(presetName(null, 'People, places and organisations')).toBe(
			'People, places and organisations'
		);
		setLocale('en');
		// A job made in French, read in English.
		expect(presetName('analysis.entities', 'Personnes, lieux et organisations')).toBe(
			'People, places and organisations'
		);
		const prompt = {
			preset_id: 'analysis.entities',
			preset_name: 'Personnes, lieux et organisations'
		};
		expect(localisePrompt(prompt).preset_name).toBe('People, places and organisations');
		expect(prompt.preset_name).toBe('Personnes, lieux et organisations');
	});
});

describe('workflow and step names', () => {
	it('show an example made in English in French, step by step', () => {
		setLocale('fr');
		expect(shownName('Read the text')).toBe('Lire le texte');
		expect(shownName('People, places and organisations')).toBe('Personnes, lieux et organisations');
		expect(shownName(en.workflows.examples.letters.name)).toBe(fr.workflows.examples.letters.name);
		expect(shownName('Translate the text')).toBe('Traduire le texte');
		// A step added from a default prompt carries its name.
		expect(shownName('Tone of the document')).toBe('Tonalité du document');
		// What the researcher named stays.
		expect(shownName('Newspapers')).toBe('Newspapers');
		setLocale('en');
		expect(shownName('Lire le texte')).toBe('Read the text');
		expect(shownName('Read the text')).toBe('Read the text');
	});

	it('pair every example’s steps across the languages', () => {
		for (const [kind, words] of Object.entries(en.workflows.examples)) {
			const french = fr.workflows.examples[kind as keyof typeof en.workflows.examples];
			expect(french.steps.length, kind).toBe(words.steps.length);
		}
	});
});
