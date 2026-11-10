// Where the researcher has been, for Back: the screens left, most recent last. Pure, so it is
// tested without the app; `state.svelte.ts` keeps the list and decides when it changes.

import type { View } from './state.svelte.js';

/** Screens kept to go back through; the oldest goes first. */
export const HISTORY = 30;

/** What names a screen: the kind of view and the ids it opens. Within one place the rest (a
 * collection shown, a project's pane, the page of a review) is that screen's own state, so a
 * change to it replaces the place rather than adding one to go back through. */
function placeOf(view: View): string {
	const ids: (string | undefined)[] = [];
	switch (view.name) {
		case 'jobs':
			ids.push(view.jobId);
			break;
		case 'review':
			ids.push(view.jobId, view.sourceId);
			break;
		case 'projects':
			ids.push(view.projectId, view.documentId);
			break;
		case 'workflows':
			ids.push(view.workflowId, view.runId);
			break;
		case 'prompts':
			ids.push(view.presetId);
			break;
		case 'new':
		case 'allJobs':
		case 'settings':
		case 'glossary':
			break;
	}
	return [view.name, ...ids.map((id) => id ?? '')].join('/');
}

/** Whether two views are the same place. */
export function samePlace(a: View, b: View): boolean {
	return placeOf(a) === placeOf(b);
}

/** How a screen left is kept to come back to: New job as the form was left, since an opening
 * with files or a project would start it afresh. */
export function kept(view: View): View {
	return view.name === 'new' ? { name: 'new' } : view;
}

/** The screens left once `from` is left for `to`: unchanged when `to` is the same place. */
export function leaving(past: View[], from: View, to: View, limit = HISTORY): View[] {
	if (samePlace(from, to)) return past;
	const next = [...past, kept(from)];
	return next.length > limit ? next.slice(next.length - limit) : next;
}
