// What a researcher wrote about their work, kept as a prompt of their own so that twenty
// interviews of one project ask it once: "Analyse this interview" offers the analysis prompts
// that quote the text and keep a description (`Preset.question`), the one used last chosen
// again, and keeps a new description as a copy of the interview prompt under a short name.

import type { Preset } from './bindings/Preset.js';
import type { AnalysisSchema } from './bindings/AnalysisSchema.js';
import { keep, recall } from './stored.js';

/** The choice of a description typed anew, beside the kept ones. */
export const NEW_RESEARCH = 'new';
const LAST_KEY = 'kalamu.interview.research';
/** A name made from a description: its first words, up to about this many characters. */
const NAME_LENGTH = 40;

/** Whether an analysis prompt's fields hold entries that quote the text. */
export function quotesText(schema: AnalysisSchema | undefined): boolean {
	return !!schema?.fields.some((f) => f.kind.type === 'cited');
}

/** The prompts that keep a description of the researcher's work and quote the text, by name:
 * those the dialog offers. */
export function researchPrompts(presets: Preset[]): Preset[] {
	return presets
		.filter((p) => p.task === 'analysis' && quotesText(p.schema) && !!p.question?.trim())
		.sort((a, b) => a.name.localeCompare(b.name));
}

/** The choice the dialog opens on: the one used last when it is still there, else the first
 * kept description, else a new one. */
export function firstChoice(kept: Preset[], last: string | null): string {
	if (last === NEW_RESEARCH || kept.some((p) => p.id === last)) return last!;
	return kept[0]?.id ?? NEW_RESEARCH;
}

export const lastChoice = (): string | null =>
	recall(LAST_KEY, (v) => (typeof v === 'string' ? v : null), null);

export const keepChoice = (choice: string): void => keep(LAST_KEY, choice);

/** A name for a description kept: the project's, when the interview belongs to one, else the
 * description's first words. */
export function suggestedName(description: string, project?: string): string {
	if (project?.trim()) return project.trim();
	const words = description.trim().replace(/\s+/g, ' ');
	if (words.length <= NAME_LENGTH) return words;
	const cut = words.slice(0, NAME_LENGTH);
	const space = cut.lastIndexOf(' ');
	return `${(space > NAME_LENGTH / 2 ? cut.slice(0, space) : cut).replace(/[\s,;:.]+$/, '')}…`;
}

/** The interview prompt copied as the researcher's own, keeping their description under
 * `name`: same fields, same instructions. */
export function keptPrompt(base: Preset, name: string, description: string): Preset {
	return {
		...structuredClone(base),
		id: '',
		name: name.trim(),
		builtin: false,
		modified: false,
		question: description.trim()
	};
}
