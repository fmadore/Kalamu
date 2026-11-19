// The interview prompt and the researcher's own prompts for the browser preview, kept in
// memory: saved, duplicated and deleted as the app's library does, a description of the
// research kept trimmed on an analysis only. `?research` starts with two kept descriptions, as
// a researcher with two projects would have. Development builds only.

import type { Preset } from '../bindings/Preset.js';
import { interviewSchema } from './mockInterview.js';

/** The ready-made prompt that reads an interview for its themes and quotations. */
export const interviewPreset: Preset = {
	id: 'analysis.interview',
	task: 'analysis',
	name: 'Interview: themes and quotations',
	description:
		'A transcript read for your research: a short summary, its themes, keywords and the passages worth quoting, each quotation found word for word in the transcript.',
	instructions: '# Interview: themes and quotations\n\nYou read an interview for a researcher…',
	user_prompt:
		'Read the interview above and fill in each field, following the instructions given. Quote only words that stand in the transcript, exactly as written, each with the number of its turn.',
	builtin: true,
	modified: false,
	schema: interviewSchema
};

const kept = (id: string, name: string, question: string): Preset => ({
	...structuredClone(interviewPreset),
	id,
	name,
	builtin: false,
	modified: false,
	question
});

// Read in the browser only: the unit tests import this file without a page.
const twoKept =
	typeof location !== 'undefined' && new URLSearchParams(location.search).has('research');
let own: Preset[] = twoKept
	? [
			kept(
				'user-research-1',
				'Market traders',
				'How market traders in a mid-sized town find credit, and what their families expect in return.'
			),
			kept(
				'user-research-2',
				'Crafts apprenticeship',
				'How young people learn a craft from a master, and when they are allowed to work alone.'
			)
		]
	: [];
let edited: Preset | null = null;
let next = 1;

/** The interview prompt as edited, if it was, and the researcher's own prompts by name. */
export function researchPresets(): { interview: Preset; own: Preset[] } {
	return {
		interview: edited ?? interviewPreset,
		own: [...own].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
	};
}

/** A prompt as the library keeps it: an analysis's description trimmed, none when empty. */
function settled(preset: Preset): Preset {
	const question = preset.question?.trim();
	const out = { ...preset };
	delete out.question;
	return question && preset.task === 'analysis' ? { ...out, question } : out;
}

export function researchCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'save_preset': {
			const preset = settled(structuredClone(a.preset as Preset));
			// Another ready-made prompt edited: answered with nothing, as before.
			if (preset.builtin && preset.id !== interviewPreset.id) return undefined;
			if (!preset.name.trim() || !preset.instructions.trim()) throw { id: 'prompt.invalid' };
			if (preset.id === interviewPreset.id) {
				edited = { ...preset, builtin: true, modified: true };
				return edited;
			}
			const saved = {
				...preset,
				id: own.some((p) => p.id === preset.id) ? preset.id : `user-mock-${next++}`,
				builtin: false,
				modified: false
			};
			own = [...own.filter((p) => p.id !== saved.id), saved];
			return saved;
		}
		case 'duplicate_preset': {
			const from = [edited ?? interviewPreset, ...own].find((p) => p.id === a.id);
			if (!from) return undefined;
			return researchCommand('save_preset', {
				preset: { ...from, id: '', name: String(a.name), builtin: false, modified: false }
			});
		}
		case 'delete_preset':
			if (a.id === interviewPreset.id) edited = null;
			else if (own.some((p) => p.id === a.id)) own = own.filter((p) => p.id !== a.id);
			else return undefined;
			return null;
		default:
			return undefined;
	}
}
