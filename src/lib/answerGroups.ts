// A question asked under two versions of its prompt gives two groups of answers (#241), kept
// apart in the filter and the exported table. Named by the question alone, they would look the
// same: each is told apart by when it was first asked, and the asking again of documents added
// since is offered for the newest version only, as a question asked again now is asked with the
// prompt as it stands. Pure, so it is tested without the app.

import type { ChoiceAnswers } from './bindings/ChoiceAnswers.js';

/** The analysis a group is headed by: the fields of one set-up share it. */
function head(group: ChoiceAnswers): string {
	return `${group.job_id}/${group.setup_id}`;
}

/** The filter's group titles, each followed by when its group was first asked (`since`, given
 * the date) where another group of another analysis has the same title; the others unchanged. */
export function distinctTitles(
	titles: string[],
	groups: ChoiceAnswers[],
	since: (asked: number) => string
): string[] {
	return titles.map((title, i) => {
		const heads = new Set(groups.filter((_, j) => titles[j] === title).map(head));
		return heads.size > 1 ? `${title} · ${since(groups[i].first_asked)}` : title;
	});
}

/** Whether a group is an older version of a question asked again since with the same model, its
 * prompt changed: `groups` come newest first. */
export function supersededAt(groups: ChoiceAnswers[], i: number): boolean {
	const group = groups[i];
	return (
		group.question !== null &&
		groups
			.slice(0, i)
			.some(
				(o) =>
					o.question === group.question &&
					o.provider === group.provider &&
					o.model === group.model &&
					o.prompt !== group.prompt
			)
	);
}
