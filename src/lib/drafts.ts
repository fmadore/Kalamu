// What was typed in a dialog closed before it started anything (by Escape, Cancel or a click
// elsewhere), kept for its next opening while the app runs, so a key pressed once too often
// never loses a research question. Kept in memory only: nothing is written to the disk, and
// the draft goes once the dialog has started its job.

const drafts = new Map<string, unknown>();

/** Keeps a dialog's draft under its key; `null` forgets it, as when nothing was typed. */
export function keepDraft<T>(key: string, draft: T | null): void {
	if (draft === null) drafts.delete(key);
	else drafts.set(key, structuredClone(draft));
}

/** The draft kept under a key, if any. It stays kept until replaced or forgotten. */
export function draftOf<T>(key: string): T | null {
	const draft = drafts.get(key);
	return draft === undefined ? null : (structuredClone(draft) as T);
}

/** Forgets a dialog's draft, once what it was typed for has started. */
export function dropDraft(key: string): void {
	drafts.delete(key);
}
