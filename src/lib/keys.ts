// What the keyboard does where the pointer would: moving between the marks of a text that hold
// one tab stop between them, panning a zoomed page, and telling whether a key belongs to the
// field or pane it was pressed in rather than to the window's shortcuts.

import type { Attachment } from 'svelte/attachments';

/** The place a key moves to among marks that share one tab stop, in the order they stand:
 * the next or previous with → and ←, the first or last with Home and End; null for any other
 * key, and at either end, where the focus stays. */
export function rovingTarget<T>(order: readonly T[], at: T, key: string): T | null {
	const i = order.indexOf(at);
	if (i < 0 || order.length === 0) return null;
	const to =
		key === 'ArrowRight'
			? i + 1
			: key === 'ArrowLeft'
				? i - 1
				: key === 'Home'
					? 0
					: key === 'End'
						? order.length - 1
						: -1;
	if (to < 0 || to >= order.length || to === i) return null;
	return order[to];
}

/** The mark that holds the tab stop: the one wanted when it is in the text, else the first. */
export function rovingStop<T>(order: readonly T[], wanted: T | null | undefined): T | null {
	if (wanted !== null && wanted !== undefined && order.includes(wanted)) return wanted;
	return order[0] ?? null;
}

/** How far an arrow key moves a zoomed page, in pixels: a short step, or most of the pane with
 * Shift; null for any other key. */
export function panStep(
	key: string,
	shift: boolean,
	pane: { width: number; height: number }
): { x: number; y: number } | null {
	const across = shift ? Math.round(pane.width * 0.8) : 48;
	const down = shift ? Math.round(pane.height * 0.8) : 48;
	switch (key) {
		case 'ArrowLeft':
			return { x: -across, y: 0 };
		case 'ArrowRight':
			return { x: across, y: 0 };
		case 'ArrowUp':
			return { x: 0, y: -down };
		case 'ArrowDown':
			return { x: 0, y: down };
		default:
			return null;
	}
}

/** Where typing goes: a key pressed there is the field's, never a shortcut's. */
export const FIELDS = 'input, textarea, select, [contenteditable]:not([contenteditable="false"])';

/** Whether an element scrolls by itself, with the keyboard once it has the focus: what it holds
 * is taller than it, and its overflow lets it scroll. */
export function scrollsItself(
	box: { scrollHeight: number; clientHeight: number },
	overflowY: string
): boolean {
	return (
		(overflowY === 'auto' || overflowY === 'scroll') && box.scrollHeight > box.clientHeight + 1
	);
}

/** Gives the focus back to what had it when a menu opened, if closing the menu left the focus
 * nowhere (on the page's body, or in the menu); a choice that took it elsewhere keeps it. */
export function giveFocusBack(opener: Element | null, closed: Element | null): void {
	const now = document.activeElement;
	if (now && now !== document.body && !closed?.contains(now)) return;
	if (opener instanceof HTMLElement && opener.isConnected) opener.focus({ preventScroll: true });
}

/** What the Tab key reaches inside an element. */
const TAB_STOPS = [
	'a[href]',
	'button:not(:disabled)',
	'input:not(:disabled):not([type="hidden"])',
	'select:not(:disabled)',
	'textarea:not(:disabled)',
	'[tabindex]'
]
	.map((s) => `${s}:not([tabindex="-1"])`)
	.join(', ');

/**
 * Lets the keyboard scroll a pane in every webview: WebKit scrolls only what has the focus, and
 * a pane holding nothing the Tab key reaches never has it. Such a pane, once what it holds is
 * taller than it, becomes a tab stop named `label`; a pane with a button or a field in it is
 * scrolled from there, and adds no stop.
 */
export function scrollableByKeys(label: string): Attachment<HTMLElement> {
	return (pane) => {
		let frame = 0;
		const update = () => {
			cancelAnimationFrame(frame);
			frame = requestAnimationFrame(() => {
				const needed =
					scrollsItself(pane, getComputedStyle(pane).overflowY) &&
					pane.querySelector(TAB_STOPS) === null;
				if (needed) {
					pane.tabIndex = 0;
					pane.setAttribute('role', 'group');
					pane.setAttribute('aria-label', label);
				} else if (document.activeElement !== pane) {
					pane.removeAttribute('tabindex');
					pane.removeAttribute('role');
					pane.removeAttribute('aria-label');
				}
			});
		};
		update();
		const content = new MutationObserver(update);
		content.observe(pane, {
			childList: true,
			subtree: true,
			attributes: true,
			attributeFilter: ['tabindex', 'disabled']
		});
		const size = new ResizeObserver(update);
		size.observe(pane);
		return () => {
			cancelAnimationFrame(frame);
			content.disconnect();
			size.disconnect();
		};
	};
}

/** Whether Page Up and Page Down are left to where the focus is, to scroll as usual, rather
 * than turning the review's page: in a field, or on a pane focused to be scrolled. */
export function pageKeysStay(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	if (target.closest(FIELDS)) return true;
	return scrollsItself(target, getComputedStyle(target).overflowY);
}
