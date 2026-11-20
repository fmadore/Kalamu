// The native questions and pickers: files and folders to read or write, consent before a
// cloud provider is sent files, the folders the Mac App Store's build must be granted, and
// yes/no questions. Outside the desktop app (a plain `npm run dev`) they answer as a browser can.

import type { Catalog } from './bindings/Catalog.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { Task } from './bindings/Task.js';
import { providerNameOf } from './catalogText.js';
import { listNames, messages } from './i18n/index.js';
import { documentExtensions, taskExtensions } from './newJob.js';
import { baseName } from './paths.js';
import { api, isTauri, PRIVACY_URL } from './tauri.js';

/** Files with these extensions, several at once. */
async function pickSources(extensions: string[]): Promise<string[]> {
	if (!isTauri()) return [];
	const { open } = await import('@tauri-apps/plugin-dialog');
	const picked = await open({
		multiple: true,
		filters: [{ name: messages().common.sources, extensions }]
	});
	return picked === null ? [] : Array.isArray(picked) ? picked : [picked];
}

/** The files a task reads. */
export function pickFiles(task: Task): Promise<string[]> {
	return pickSources(taskExtensions(task));
}

/** Files of any kind Kalamu reads, for a project's documents. */
export function pickDocuments(): Promise<string[]> {
	return pickSources(documentExtensions());
}

export async function pickFolder(): Promise<string | null> {
	if (!isTauri()) return null;
	const { open } = await import('@tauri-apps/plugin-dialog');
	const picked = await open({ directory: true, multiple: false });
	return typeof picked === 'string' ? picked : null;
}

/** Where to write a JSON file named `name`: a preset, a profile, a dictionary, a project's
 *  descriptions or a pipeline's record, some with a CSV or Markdown twin written beside it. */
export async function pickJsonSave(name: string): Promise<string | null> {
	if (!isTauri()) return null;
	const { save } = await import('@tauri-apps/plugin-dialog');
	return save({ defaultPath: `${name}.json`, filters: [{ name: 'JSON', extensions: ['json'] }] });
}

/** Where to write a table named `name`: an Excel workbook, its CSV twin written beside it. */
export async function pickTableSave(name: string): Promise<string | null> {
	if (!isTauri()) return null;
	const { save } = await import('@tauri-apps/plugin-dialog');
	return save({ defaultPath: `${name}.xlsx`, filters: [{ name: 'Excel', extensions: ['xlsx'] }] });
}

export async function pickPresetFile(): Promise<string | null> {
	if (!isTauri()) return null;
	const { open } = await import('@tauri-apps/plugin-dialog');
	const picked = await open({ multiple: false, filters: [{ name: 'JSON', extensions: ['json'] }] });
	return typeof picked === 'string' ? picked : null;
}

/** Where results go beside these sources: each file's folder, or, for text frozen from an
 *  earlier result, its original's. YouTube videos have none. */
export function besidePlaces(sources: SourceFile[]): string[] {
	return sources
		.filter((s) => s.kind !== 'youtube')
		.map((s) => s.derived_from?.original_path ?? s.path);
}

/** In the Mac App Store's build, asks the researcher for each folder results will be written to
 *  beside these files, since macOS grants a picked file but not its folder. False when they
 *  decline; throws when another folder was chosen. Elsewhere there is nothing to ask. */
export async function askForFolders(paths: string[]): Promise<boolean> {
	if (!isTauri() || paths.length === 0) return true;
	const m = messages();
	const { ask, open } = await import('@tauri-apps/plugin-dialog');
	const nameOf = (folder: string) => baseName(folder) || folder;
	for (const folder of await api.foldersToAsk(paths)) {
		const go = await ask(m.access.ask(nameOf(folder)), {
			kind: 'info',
			okLabel: m.access.choose,
			cancelLabel: m.common.cancel
		});
		if (!go) return false;
		const picked = await open({ directory: true, multiple: false, defaultPath: folder });
		if (typeof picked !== 'string') return false;
		await api.keepAccess([picked]);
	}
	const missing = await api.foldersToAsk(paths);
	if (missing.length > 0) {
		throw new Error(m.access.missing(nameOf(missing[0])));
	}
	return true;
}

/** The providers a job's set-ups send to, the step that combines included. */
export function providersOf(setups: SetupRequest[]): string[] {
	return [...new Set(setups.flatMap((s) => [s.provider, ...(s.merge ? [s.merge.provider] : [])]))];
}

/**
 * Asks, once for each cloud provider, before the first job, run or proposal sends the
 * researcher's files to it (App Review Guideline 5.1.2(i), and plain courtesy on Windows too):
 * which provider, what is sent, and where the privacy policy is. Offline engines are never asked
 * about. False when the researcher declines; Rust refuses the work until they agree.
 */
export async function askConsent(providers: string[], catalog: Catalog | null): Promise<boolean> {
	if (providers.length === 0) return true;
	const needed = await api.providersToConsent(providers);
	if (needed.length === 0) return true;
	const m = messages();
	const names = listNames(needed.map((id) => (catalog ? providerNameOf(catalog, id) : id)));
	const message = m.consent.ask(names, PRIVACY_URL);
	let go: boolean;
	if (isTauri()) {
		const { ask } = await import('@tauri-apps/plugin-dialog');
		go = await ask(message, {
			title: m.consent.title,
			kind: 'warning',
			okLabel: m.consent.send,
			cancelLabel: m.common.cancel
		});
	} else {
		go = window.confirm(message);
	}
	if (!go) return false;
	await api.consentProviders(needed);
	return true;
}

/** What a job or a workflow run asks before it starts: consent for each cloud provider its
 *  set-ups send to, then, in the Mac App Store's build, the folders beside the sources that
 *  results go to. False when the researcher declines either. */
export async function mayStart(
	setups: SetupRequest[],
	catalog: Catalog | null,
	sources: SourceFile[],
	beside: boolean
): Promise<boolean> {
	if (!(await askConsent(providersOf(setups), catalog))) return false;
	return !beside || askForFolders(besidePlaces(sources));
}

/** A yes/no question in a native dialog, its buttons named by `labels` when given. */
export async function ask(
	message: string,
	labels?: { ok: string; cancel: string }
): Promise<boolean> {
	if (!isTauri()) return window.confirm(message);
	const { ask } = await import('@tauri-apps/plugin-dialog');
	return ask(message, {
		kind: 'warning',
		...(labels ? { okLabel: labels.ok, cancelLabel: labels.cancel } : {})
	});
}
