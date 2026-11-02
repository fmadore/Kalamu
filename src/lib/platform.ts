// The one place in the interface that knows which operating system it runs on: the modifier
// key shortcuts use, how a shortcut is written, and what the system calls the places Kalamu
// uses. Components ask here rather than testing the OS themselves.

import { messages } from './i18n/index.js';

/** Whether the interface runs on a Mac, from the webview's user agent (WebKit on macOS). */
export const isMac =
	typeof navigator !== 'undefined' && /Macintosh|Mac OS X/.test(navigator.userAgent);

/** Whether the shortcut modifier is held: Command on a Mac, Ctrl elsewhere. */
export function primary(e: KeyboardEvent | MouseEvent, mac = isMac): boolean {
	return mac ? e.metaKey : e.ctrlKey;
}

/** A shortcut with that modifier, as each system writes it: "⌘N" on a Mac, "Ctrl N" elsewhere. */
export function shortcut(key: string, mac = isMac): string {
	return mac ? `⌘${key}` : `${messages().keys.ctrl} ${key}`;
}

/** What this system calls its keychain and the place the tray icon sits. */
export function osNames(mac = isMac) {
	return messages().os[mac ? 'macos' : 'windows'];
}
