import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Preset } from './bindings/Preset.js';
import { interviewRequest, withPrompt } from './interviewStart.svelte.js';
import { interviewPreset } from './dev/mockResearch.js';
import {
	firstChoice,
	keepChoice,
	keptPrompt,
	lastChoice,
	NEW_RESEARCH,
	quotesText,
	researchPrompts,
	suggestedName
} from './research.js';

const kept = (id: string, name: string, question?: string): Preset => ({
	...structuredClone(interviewPreset),
	id,
	name,
	builtin: false,
	modified: false,
	...(question === undefined ? {} : { question })
});

const summary: Preset = {
	id: 'analysis.summary',
	task: 'analysis',
	name: 'Summary',
	description: '',
	instructions: 'Summarise.',
	user_prompt: 'Read it.',
	builtin: true,
	modified: false,
	question: 'Not offered: it quotes nothing.',
	schema: {
		fields: [
			{ key: 'summary', label: 'Summary', instructions: '', kind: { type: 'text' }, reason: false }
		]
	}
};

describe('the descriptions the interview dialog offers', () => {
	it('are the prompts that quote the text and keep a description, by name', () => {
		const offered = researchPrompts([
			interviewPreset,
			summary,
			kept('user-2', 'Traders', 'Market traders and credit.'),
			kept('user-3', 'Empty', '   '),
			kept('user-4', 'No description'),
			kept('user-1', 'Apprentices', 'Learning a craft.')
		]);
		expect(offered.map((p) => p.name)).toEqual(['Apprentices', 'Traders']);
		expect(quotesText(interviewPreset.schema)).toBe(true);
		expect(quotesText(summary.schema)).toBe(false);
		expect(quotesText(undefined)).toBe(false);
	});

	it('are none on first use, so the dialog is one box', () => {
		expect(researchPrompts([interviewPreset, summary])).toEqual([]);
		expect(firstChoice([], null)).toBe(NEW_RESEARCH);
	});

	it('open on the one used last, else the first, else a new one', () => {
		const offered = [kept('user-1', 'A', 'a'), kept('user-2', 'B', 'b')];
		expect(firstChoice(offered, 'user-2')).toBe('user-2');
		expect(firstChoice(offered, NEW_RESEARCH)).toBe(NEW_RESEARCH);
		expect(firstChoice(offered, 'user-gone')).toBe('user-1');
		expect(firstChoice(offered, null)).toBe('user-1');
	});
});

describe('the choice remembered on this computer', () => {
	afterEach(() => vi.unstubAllGlobals());

	it('is the last one kept', () => {
		const items = new Map<string, string>();
		vi.stubGlobal('localStorage', {
			getItem: (k: string) => items.get(k) ?? null,
			setItem: (k: string, v: string) => void items.set(k, v)
		});
		expect(lastChoice()).toBeNull();
		keepChoice('user-2');
		expect(lastChoice()).toBe('user-2');
		localStorage.setItem('kalamu.interview.research', '{broken');
		expect(lastChoice()).toBeNull();
	});
});

describe('a description kept for the next interviews', () => {
	it('is named after the project, else its first words', () => {
		expect(suggestedName('Anything', ' Port workers ')).toBe('Port workers');
		expect(suggestedName('  How traders   find credit. ')).toBe('How traders find credit.');
		const long = suggestedName(
			'How market traders in a mid-sized town find credit, and what their families expect.'
		);
		expect(long).toBe('How market traders in a mid-sized town…');
		expect(long.length).toBeLessThanOrEqual(41);
	});

	it('is a copy of the interview prompt: same fields and instructions, its own name', () => {
		const copy = keptPrompt(interviewPreset, ' Traders ', '  Market traders and credit. ');
		expect(copy).toMatchObject({
			id: '',
			name: 'Traders',
			builtin: false,
			modified: false,
			question: 'Market traders and credit.',
			task: 'analysis',
			instructions: interviewPreset.instructions,
			user_prompt: interviewPreset.user_prompt
		});
		expect(copy.schema).toEqual(interviewPreset.schema);
		expect(copy.schema).not.toBe(interviewPreset.schema);
	});

	it('starts the job with its prompt in place of the ready-made one', () => {
		const request = {
			provider: 'gemini',
			model: 'm',
			preset_id: 'analysis.interview',
			label: null,
			options: { temperature: null, diarize: false, timestamps: false, speakers: null }
		};
		const chosen = withPrompt(request, 'user-1');
		expect(chosen.preset_id).toBe('user-1');
		expect('question' in chosen.options).toBe(false);
		// Typed and kept at once: the same words go as the question of the copy.
		expect(withPrompt(interviewRequest(request, 'Traders.'), 'user-1').options.question).toBe(
			'Traders.'
		);
		// A model that takes no prompt is sent none.
		expect(withPrompt({ ...request, preset_id: null }, 'user-1').preset_id).toBeNull();
	});
});
