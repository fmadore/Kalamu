// Where the researcher is in a pane of the review: the caret or selection in the text as read,
// where a character of a text box stands, and scrolling a pane to bring words into view.

/** Where each line of a text starts, as offsets in it. */
export function lineStarts(text: string): number[] {
	const starts: number[] = [];
	let at = 0;
	for (const line of text.split('\n')) {
		starts.push(at);
		at += line.length + 1;
	}
	return starts;
}

/** Where the researcher is in the text as read, as offsets in `base`: the selection or the
 * caret left by a click when it is in view, else the first line in view; with how far below
 * the top of the pane that line stands. `pane` shows `base` one line per `.line`, its words in
 * the line's `.t`. */
export function placeInText(
	pane: HTMLElement | null,
	base: string
): { start: number; end: number; below: number } | null {
	if (!pane) return null;
	const rows = [...pane.querySelectorAll<HTMLElement>('.text.numbered > .line')];
	const box = pane.getBoundingClientRect();
	const starts = lineStarts(base);
	if (rows.length !== starts.length) return null;
	const rowOf = (node: Node) =>
		(node instanceof Element ? node : node.parentElement)?.closest<HTMLElement>('.line') ?? null;
	const offsetAt = (node: Node, offset: number) => {
		const row = rowOf(node);
		const i = row ? rows.indexOf(row) : -1;
		const words = row?.querySelector('.t');
		if (i < 0 || !words) return null;
		const before = document.createRange();
		before.setStart(words, 0);
		try {
			before.setEnd(node, offset);
		} catch {
			return starts[i];
		}
		const length = (starts[i + 1] ?? base.length + 1) - 1 - starts[i];
		return starts[i] + Math.min(before.toString().length, length);
	};
	const inView = (row: HTMLElement | null) => {
		const r = row?.getBoundingClientRect();
		return r !== undefined && r.bottom > box.top && r.top < box.bottom;
	};
	const chosen = getSelection();
	const range = chosen && chosen.rangeCount > 0 ? chosen.getRangeAt(0) : null;
	if (range && pane.contains(range.startContainer) && inView(rowOf(range.startContainer))) {
		const start = offsetAt(range.startContainer, range.startOffset);
		const end = offsetAt(range.endContainer, range.endOffset);
		const row = rowOf(range.startContainer);
		if (start !== null && row)
			return {
				start,
				end: end !== null && end >= start ? end : start,
				below: row.getBoundingClientRect().top - box.top
			};
	}
	const first = rows.findIndex((r) => r.getBoundingClientRect().bottom > box.top + 1);
	if (first <= 0) return null;
	return {
		start: starts[first],
		end: starts[first],
		below: rows[first].getBoundingClientRect().top - box.top
	};
}

/** How far below the top of a text box its character `at` stands, measured on a copy of the
 * box laid out the same way. */
export function caretTop(area: HTMLTextAreaElement, at: number): number {
	const style = getComputedStyle(area);
	const copy = document.createElement('div');
	for (const p of [
		'boxSizing',
		'width',
		'padding',
		'borderWidth',
		'borderStyle',
		'fontFamily',
		'fontSize',
		'fontWeight',
		'lineHeight',
		'letterSpacing',
		'wordSpacing',
		'tabSize'
	] as const)
		copy.style[p] = style[p];
	Object.assign(copy.style, {
		position: 'absolute',
		visibility: 'hidden',
		top: '0',
		left: '-9999px',
		whiteSpace: 'pre-wrap',
		overflowWrap: 'break-word'
	});
	copy.textContent = area.value.slice(0, at);
	const mark = document.createElement('span');
	mark.textContent = '​';
	copy.append(mark);
	document.body.append(copy);
	const top = mark.offsetTop;
	copy.remove();
	return top;
}

const still = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

/** Scrolls a pane so that an element in it is in view, a third of the way down. */
export function reveal(pane: HTMLElement | null, el: Element | Range | null): void {
	if (!pane || !el) return;
	const box = pane.getBoundingClientRect();
	const r = el.getBoundingClientRect();
	if (r.top >= box.top + 24 && r.bottom <= box.bottom - 24) return;
	pane.scrollTo({
		top: pane.scrollTop + r.top - box.top - box.height / 3,
		behavior: still() ? 'auto' : 'smooth'
	});
}

/** Scrolls a pane so that an element in it stands `height` below the pane's top, kept in
 * view. */
export function level(pane: HTMLElement | null, el: Element | null, height: number): void {
	if (!pane || !el) return;
	const box = pane.getBoundingClientRect();
	const at = Math.max(24, Math.min(height, box.height - 48));
	pane.scrollTo({
		top: pane.scrollTop + el.getBoundingClientRect().top - box.top - at,
		behavior: still() ? 'auto' : 'smooth'
	});
}

/**
 * Makes a text box as tall as its text, for a webview without CSS `field-sizing: content`
 * (WebKit before Safari 26.2, which macOS 14 and 15 ship): the pane around it then scrolls
 * the text, and a caret's place in the box is its place in the pane. Where the webview sizes
 * the box itself, nothing is done. An attachment: `{@attach growWithText}`.
 */
export function growWithText(area: HTMLTextAreaElement): (() => void) | undefined {
	if (typeof CSS !== 'undefined' && CSS.supports('field-sizing', 'content')) return;
	const fit = () => {
		// Shrunk first, so text taken out gives its room back, the panes around it kept where
		// they were scrolled; the borders are added back.
		const scrolled: [Element, number][] = [];
		for (let el = area.parentElement; el; el = el.parentElement) {
			if (el.scrollTop > 0) scrolled.push([el, el.scrollTop]);
		}
		area.style.height = 'auto';
		area.style.height = `${area.scrollHeight + area.offsetHeight - area.clientHeight}px`;
		for (const [el, top] of scrolled) el.scrollTop = top;
	};
	fit();
	area.addEventListener('input', fit);
	// A narrower or wider pane wraps the lines anew.
	let width = area.clientWidth;
	const resized = new ResizeObserver(() => {
		if (area.clientWidth === width) return;
		width = area.clientWidth;
		fit();
	});
	resized.observe(area);
	return () => {
		area.removeEventListener('input', fit);
		resized.disconnect();
	};
}
