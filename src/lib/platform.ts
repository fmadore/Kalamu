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

/** Whether the wheel zooms rather than scrolls: with Ctrl on every system, as a trackpad pinch
 *  arrives, and with Command too on a Mac. */
export function zoomsWithWheel(e: WheelEvent, mac = isMac): boolean {
	return e.ctrlKey || (mac && e.metaKey);
}

/** The shortcut modifier alone, as each system writes it: "⌘" on a Mac, "Ctrl" elsewhere. */
export function modifier(mac = isMac): string {
	return mac ? '⌘' : messages().keys.ctrl;
}

/** A shortcut with that modifier, as each system writes it: "⌘N" on a Mac, "Ctrl N" elsewhere. */
export function shortcut(key: string, mac = isMac): string {
	return mac ? `⌘${key}` : `${messages().keys.ctrl} ${key}`;
}

/** A shortcut with Shift, as each system writes it: "⇧F" on a Mac, "Shift F" ("Maj F") elsewhere. */
export function shifted(key: string, mac = isMac): string {
	return mac ? `⇧${key}` : `${messages().keys.shift} ${key}`;
}

/** A shortcut with Alt, as each system writes it: "⌥←" on a Mac, "Alt ←" elsewhere. */
export function alted(key: string, mac = isMac): string {
	return mac ? `⌥${key}` : `${messages().keys.alt} ${key}`;
}

/** Whether a key goes back to the screen left: Alt and the left arrow, and ⌘[ on a Mac, as in
 *  the systems' own browsers. */
export function isBack(e: KeyboardEvent, mac = isMac): boolean {
	const only = (held: boolean) =>
		held && [e.ctrlKey, e.metaKey, e.altKey].filter(Boolean).length === 1;
	if (e.shiftKey) return false;
	if (mac && only(e.metaKey) && e.key === '[') return true;
	return only(e.altKey) && e.key === 'ArrowLeft';
}

/** How Back is written for this system. In a review Alt and the arrows turn the pages, so a
 *  Mac's ⌘[ is its only key there, and elsewhere there is none (`null`). */
export function backShortcut(inReview = false, mac = isMac): string | null {
	if (mac) return '⌘[';
	return inReview ? null : alted('←', mac);
}

/** What this system calls its keychain and the place the tray icon sits. */
export function osNames(mac = isMac) {
	return messages().os[mac ? 'macos' : 'windows'];
}
