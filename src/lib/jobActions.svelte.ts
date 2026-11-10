// Renaming and deleting a job, the same from the sidebar, the Jobs page, a job's menu and its
// own page.

import type { JobSummary } from './bindings/JobSummary.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import { app, go, refreshJobs, toast } from './state.svelte.js';
import { api, ask } from './tauri.js';

/**
 * Renames job `id` to the name typed, without spaces at its ends, unless that is empty or its
 * name already; the list of jobs is read again. Returns whether it was renamed; throws what the
 * core reports.
 */
export async function renameJob(id: string, typed: string): Promise<boolean> {
	const name = typed.trim();
	const job = app.jobs.find((j) => j.id === id);
	if (!name || !job || name === job.name) return false;
	await api.renameJob(id, name);
	await refreshJobs();
	return true;
}

/**
 * Asks, then deletes job `id`: a screen that shows it goes back to the jobs, and the list of
 * jobs is read again. Returns false when the researcher kept it; throws what the core reports.
 */
export async function deleteJob(id: string): Promise<boolean> {
	if (!(await ask(messages().jobs.deleteConfirm))) return false;
	await api.deleteJob(id);
	const shown = app.view.name === 'jobs' || app.view.name === 'review' ? app.view.jobId : null;
	if (shown === id) go({ name: 'jobs' });
	await refreshJobs();
	return true;
}

/**
 * A job renamed in its row of a list: Enter or leaving the box renames it, Escape gives up. What
 * goes wrong is said in a toast, the list staying as it was.
 */
export class RowRename {
	/** The job being renamed, and the name typed so far. */
	editing = $state<{ id: string; name: string } | null>(null);

	start(job: JobSummary): void {
		this.editing = { id: job.id, name: job.name };
	}

	commit = async (): Promise<void> => {
		const typed = this.editing;
		this.editing = null;
		if (!typed) return;
		try {
			await renameJob(typed.id, typed.name);
		} catch (e) {
			toast(describeError(e, messages()), { kind: 'error' });
		}
	};

	keys = (e: KeyboardEvent): void => {
		if (e.key === 'Enter') void this.commit();
		else if (e.key === 'Escape') this.editing = null;
	};
}

/** Puts the caret in a name box as it opens, the name selected. */
export function focusName(input: HTMLInputElement): void {
	input.focus();
	input.select();
}
