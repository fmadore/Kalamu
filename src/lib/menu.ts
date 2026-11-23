// Where the keyboard goes in a menu opened from a job or a name (`ContextMenu`). Pure, so it is
// tested without the app.

/**
 * The item a key moves to among `count` items, from the one at `at` (-1 when none has the
 * focus): ↓ and ↑ go round, from the last to the first and back; Home and End go to the first
 * and last. -1 for any other key, which the menu leaves alone.
 */
export function menuTarget(count: number, at: number, key: string): number {
	if (key === 'ArrowDown') return (at + 1) % count;
	if (key === 'ArrowUp') return (at - 1 + count) % count;
	if (key === 'Home') return 0;
	if (key === 'End') return count - 1;
	return -1;
}

/** Whether a key opens the menu of what has the focus: the menu key, or Shift+F10. */
export function isMenuKey(e: KeyboardEvent): boolean {
	return e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10');
}

/**
 * Opens a menu from a right click, at the pointer, or from the keyboard (a `contextmenu` event
 * with no place), where `open` decides. WebKit on macOS opens the menu as the button goes down:
 * the menu then waits for it to come up, or that release would close it again at once.
 */
export function openFromClick(e: MouseEvent, open: (at: { x: number; y: number } | null) => void) {
	e.preventDefault();
	const at = e.clientX === 0 && e.clientY === 0 ? null : { x: e.clientX, y: e.clientY };
	if (e.buttons === 0) {
		open(at);
		return;
	}
	addEventListener('pointerup', () => setTimeout(() => open(at)), { once: true });
}

/** Copies `text`; a webview that refuses the clipboard to a script still copies a selection. */
export async function copyText(text: string): Promise<void> {
	try {
		await navigator.clipboard.writeText(text);
	} catch {
		document.execCommand('copy');
	}
}
