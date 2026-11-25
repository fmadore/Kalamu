// Closing the window in the browser preview, as the desktop app's shell does it (`lib.rs`):
// while changes are not saved (`set_draft_open`), the interface is asked first with the
// `close-requested` event; otherwise the window closes. The tests close it through
// `window.__kalamuCloseWindow()` and read what came of it on `window.__kalamuWindow`. No job
// runs in the preview, so the event never asks about jobs. A failure the interface sends to the
// log (`log_interface_error`) is kept on `window.__kalamuLoggedErrors`, for the tests to read.
// Development builds only.

import { emit } from '@tauri-apps/api/event';

declare global {
	interface Window {
		/** Asks to close the window, as its close button does. */
		__kalamuCloseWindow?: () => void;
		/** Whether changes are not saved, as the interface last said; and what closing did:
		 * whether the window closed at once, or the app quit once asked. */
		__kalamuWindow?: { draft: boolean; closed: boolean; quit: boolean };
		/** The failures the interface sent to the log, as it sent them. */
		__kalamuLoggedErrors?: unknown[];
	}
}

const outcome = () => (window.__kalamuWindow ??= { draft: false, closed: false, quit: false });

/** Makes the window's close button available to the tests. */
export function installClose(): void {
	window.__kalamuCloseWindow = () => {
		if (outcome().draft) void emit('close-requested', false);
		else outcome().closed = true;
	};
}

export function windowCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'set_draft_open':
			outcome().draft = a.open === true;
			return null;
		case 'quit_app':
			outcome().quit = true;
			return null;
		case 'log_interface_error':
			(window.__kalamuLoggedErrors ??= []).push(a.error);
			return null;
		default:
			return undefined;
	}
}
