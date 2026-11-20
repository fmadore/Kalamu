// Which of a project's documents speak of the researcher's question (#51): the setup the
// screening runs with, the documents' filter by what an analysis answered (any analysis with a
// choice, never one known by its prompt's name), and the researcher's own decision, kept as one
// of two tags so the model's answer is never changed. Pure, so it is tested without the app.

import type { ChoiceAnswers } from './bindings/ChoiceAnswers.js';
import type { Estimate } from './bindings/Estimate.js';
import type { Catalog } from './bindings/Catalog.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import { modelNameOf } from './catalogText.js';
import { choiceLabel, fieldLabel, listNames, localiseFields, type Messages } from './i18n/index.js';

/** The ready-made prompt that reads a document against the researcher's question. */
export const SCREENING_PRESET = 'analysis.screening';

/** The setup a screening runs with: the draft's, with the question typed (left out when empty,
 * as the engine's cache keys expect). */
export function screeningRequest(request: SetupRequest, question: string): SetupRequest {
	const asked = question.trim();
	return asked ? { ...request, options: { ...request.options, question: asked } } : request;
}

/** The documents the model chosen cannot read even in parts, one passage of each alone too long
 * for it, as the estimate measured the texts it would send: they are left out of the job, and
 * the estimate has priced the others only. The sentence names them and the model; null when
 * every document can be read. */
export function tooLongOf(
	estimate: Estimate | null,
	catalog: Catalog,
	m: Messages
): { sources: string[]; said: string } | null {
	const row = estimate?.rows.find((r) => r.too_long?.length);
	const long = row?.too_long ?? [];
	if (!row || long.length === 0) return null;
	return {
		sources: long.map((l) => l.source),
		said: m.screening.tooLong(
			listNames(long.map((l) => l.file)),
			modelNameOf(catalog, row.provider, row.model),
			long.length
		)
	};
}

/** The filter's value for one answer of one choice: the analysis, the field and the value. */
export function answerKey(group: ChoiceAnswers, value: string): string {
	return JSON.stringify([group.job_id, group.setup_id, group.field.key, value]);
}

/** The choice and the value a filter's value names, or null when none of these does. */
export function chosenAnswer(
	groups: ChoiceAnswers[],
	key: string
): { group: ChoiceAnswers; value: string } | null {
	if (!key) return null;
	for (const group of groups)
		for (const value of valuesOf(group))
			if (answerKey(group, value) === key) return { group, value };
	return null;
}

/** The values a choice was answered with, in the order its field lists them, any other (one the
 * field no longer lists) after them. */
function valuesOf(group: ChoiceAnswers): string[] {
	const listed = group.field.kind.type === 'choice' ? group.field.kind.values : [];
	const given = new Set(group.answers.map((a) => a.value));
	return [...listed.filter((v) => given.has(v)), ...[...given].filter((v) => !listed.includes(v))];
}

/** One option of the filter. */
export interface AnswerOption {
	key: string;
	label: string;
}

/** The filter's options, a group per analysis's choice: named by the question it was asked
 * with, or else by its field and its job; each answer given in the interface's words, with how
 * many documents it was given to. */
export function answerOptions(
	groups: ChoiceAnswers[],
	m: Messages
): { title: string; options: AnswerOption[] }[] {
	return groups.map((group) => {
		const field =
			localiseFields(group.preset_id, { fields: [group.field] })?.fields[0] ?? group.field;
		return {
			title:
				group.question ??
				m.screening.group(fieldLabel(group.field.label, group.field.key), group.job_name),
			options: valuesOf(group).map((value) => ({
				key: answerKey(group, value),
				label: m.screening.answerCount(
					choiceLabel(field, value),
					group.answers.filter((a) => a.value === value).length
				)
			}))
		};
	});
}

/** The documents given that answer, or null when the filter is off. */
export function answeredDocuments(groups: ChoiceAnswers[], key: string): Set<string> | null {
	const chosen = chosenAnswer(groups, key);
	if (!chosen) return null;
	return new Set(
		chosen.group.answers.filter((a) => a.value === chosen.value).map((a) => a.document)
	);
}

/** The name an analysis's table is offered under: the project's and the job's, without what
 * Windows or macOS refuse in a file name, and not too long to read in the save dialog. */
export function tableName(project: string, group: ChoiceAnswers): string {
	return `${project} - ${group.job_name}`
		.replace(/[<>:"/\\|?*\p{Cc}]/gu, ' ')
		.replace(/\s+/g, ' ')
		.slice(0, 80)
		.replace(/[\s.]+$/, '');
}

/** The researcher's decision on a document, as tags: the one given, and the other, taken off. */
export function decisionTags(keep: boolean, m: Messages): { add: string; remove: string } {
	const [kept, aside] = [m.screening.keptTag, m.screening.setAsideTag];
	return keep ? { add: kept, remove: aside } : { add: aside, remove: kept };
}
