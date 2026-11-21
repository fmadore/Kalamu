// A project saved as one file and opened on another computer (#213): where the file goes and
// which file is opened, asked through the system's own dialogs. What the file holds and how it
// is checked are Rust's.

import { messages } from './i18n/index.js';
import { isTauri } from './tauri.js';

/** A project file's extension, without its dot, as Rust writes it. */
export const PROJECT_FILE_EXTENSION = 'kalamu';

function filters() {
	return [{ name: messages().projectFile.fileType, extensions: [PROJECT_FILE_EXTENSION] }];
}

/** Where to save a project's file, `name` (with its extension) offered. */
export async function pickProjectFileSave(name: string): Promise<string | null> {
	if (!isTauri()) return null;
	const { save } = await import('@tauri-apps/plugin-dialog');
	return save({ defaultPath: name, filters: filters() });
}

/** A project file to open. */
export async function pickProjectFile(): Promise<string | null> {
	if (!isTauri()) return null;
	const { open } = await import('@tauri-apps/plugin-dialog');
	const picked = await open({ multiple: false, filters: filters() });
	return typeof picked === 'string' ? picked : null;
}
