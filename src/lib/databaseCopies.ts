// The copies of the job database, as the interface offers them: what a launch did, said once;
// damage found by the check after launch, with the newest copy offered; and a copy opened in
// place of the database once the researcher agreed. What a copy is and which are kept is the
// store's; this only asks and tells.

import type { DatabaseCopies } from './bindings/DatabaseCopies.js';
import type { DatabaseCopy } from './bindings/DatabaseCopy.js';
import type { UnreadableJob } from './bindings/UnreadableJob.js';
import { ask } from './dialogs.js';
import { describeError } from './errors.js';
import { formatDate, listNames, messages } from './i18n/index.js';
import { leaveThen, toast } from './state.svelte.js';
import { api, on, type Unlisten } from './tauri.js';

/** The newest copy that can stand in for the database: a daily copy or one made before an
 *  update, never a file set aside. */
export function newestCopy(view: DatabaseCopies): DatabaseCopy | null {
	return view.copies.find((c) => c.kind !== 'set_aside') ?? null;
}

/** Asks before opening `copy` in place of the job database, then opens it: Kalamu restarts.
 *  `question` words the ask; the plain one by default. The restart does not pass through the
 *  window's closing guard (#271), so a draft not saved is asked about first, in the words
 *  leaving a screen or closing the window uses. */
export function openCopy(copy: DatabaseCopy, question?: (date: string) => string): void {
	leaveThen(() => void confirmAndOpen(copy, question));
}

async function confirmAndOpen(copy: DatabaseCopy, question?: (date: string) => string) {
	const m = messages();
	const date = formatDate(copy.made);
	const agreed = await ask((question ?? m.database.openAsk)(date), {
		ok: m.database.openOk,
		cancel: m.common.cancel
	});
	if (!agreed) return;
	try {
		await api.openDatabaseCopy(copy.name);
	} catch (e) {
		toast(describeError(e, m), { kind: 'error' });
	}
}

/** What this launch did to the database, said once. */
function tellLaunch(view: DatabaseCopies) {
	const m = messages();
	if (view.restored) {
		const copy = view.copies.find((c) => c.name === view.restored);
		toast(m.database.restored(copy ? formatDate(copy.made) : view.restored));
	}
	if (view.restore_failed) toast(m.database.restoreFailed, { kind: 'error' });
	if (view.started_empty) toast(m.database.startedEmpty, { kind: 'error' });
}

/** Damage the check found: the newest copy offered, once. */
function offerCopy(view: DatabaseCopies) {
	const m = messages();
	const copy = newestCopy(view);
	if (!copy) {
		toast(m.database.damagedNoCopy, { kind: 'error' });
		return;
	}
	openCopy(copy, m.database.damagedAsk);
}

/** Jobs left out of the list because this version cannot read them, named once. */
export function tellUnreadable(jobs: UnreadableJob[]) {
	if (jobs.length === 0) return;
	const m = messages();
	toast(m.database.unreadable(listNames(jobs.map((j) => j.name)), jobs.length), {
		kind: 'error'
	});
}

/** At start-up: says what opening the database did, and offers the newest copy if the check
 *  that runs after launch finds damage, whether it ends before or after this is called. */
export async function watchDatabase(): Promise<Unlisten> {
	let offered = false;
	const look = async (first: boolean) => {
		const view = await api.databaseCopies();
		if (first) tellLaunch(view);
		if (view.damaged && !offered) {
			offered = true;
			offerCopy(view);
		}
	};
	const unlisten = await on.databaseChecked(() => void look(false).catch(() => undefined));
	// Not awaited: a question about damage must not hold up the rest of start-up.
	void look(true).catch(() => undefined);
	return unlisten;
}
