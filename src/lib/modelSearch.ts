// Typing in a model list to find a model faster: every word typed must be found, without regard
// to case or accents, in the model's name, its id or its provider's name. The list's headings
// (tiers, offline engines, servers) stay, and one with no model left disappears.

import { searchKey } from './languages.js';
import type { ModelGroup } from './tiers.js';
import type { ModelInfo } from './bindings/ModelInfo.js';

/** The words typed, as compared: lower case, without accents. */
export function searchWords(query: string): string[] {
	return searchKey(query).split(/\s+/).filter(Boolean);
}

/** Whether every word is found in one of `fields` (a word may be part of one: "gem" finds
 *  "Gemini"). No word finds everything. */
export function matchesWords(words: string[], fields: string[]): boolean {
	const keys = fields.map(searchKey);
	return words.every((word) => keys.some((key) => key.includes(word)));
}

/** The groups with only the models the query finds, each in its order; a group left empty goes. */
export function findModels(
	groups: ModelGroup[],
	query: string,
	providerName: (model: ModelInfo) => string
): ModelGroup[] {
	const words = searchWords(query);
	if (words.length === 0) return groups;
	return groups
		.map((group) => ({
			...group,
			models: group.models.filter((m) => matchesWords(words, [m.name, m.id, providerName(m)]))
		}))
		.filter((group) => group.models.length > 0);
}
