// A passage of the text review chosen to be corrected by a model: where the selection stands in
// the text shown, the context sent around it, and whether it may be sent. The limits are the
// engine's (`core::passage_correction`), read from `coreLists.json` so the two cannot drift.

import lists from './coreLists.json';
import { lineStarts } from './caret.js';

/** The fewest letters or figures a passage holds, the most characters, and the characters of
 * context sent on each side. */
export const PASSAGE_LIMITS: { min: number; max: number; context: number } = lists.passage_limits;

/** A selection in the text shown, as offsets in it (UTF-16 units, as JavaScript counts), and
 * whether it was cut to the text because it went beyond it. */
export interface Chosen {
	start: number;
	end: number;
	cut: boolean;
}

/** What is sent: the passage, the context on each side, and where the passage starts. */
export interface Around {
	before: string;
	passage: string;
	after: string;
	at: number;
}

const isHigh = (c: number) => c >= 0xd800 && c <= 0xdbff;
const isLow = (c: number) => c >= 0xdc00 && c <= 0xdfff;

/**
 * The passage between `start` and `end` of `text` (one piece's text), with up to `context`
 * characters of the same text on each side, cut at a word's edge when cut at all, and never
 * through a character written in two units.
 */
export function passageAround(
	text: string,
	start: number,
	end: number,
	context = PASSAGE_LIMITS.context
): Around {
	let from = Math.max(0, start - context);
	if (from > 0 && isLow(text.charCodeAt(from))) from++;
	let before = text.slice(from, start);
	if (from > 0) before = before.replace(/^\S*\s+/u, '') || before;
	let to = Math.min(text.length, end + context);
	if (to < text.length && isHigh(text.charCodeAt(to - 1))) to--;
	let after = text.slice(end, to);
	if (to < text.length) after = after.replace(/\s+\S*$/u, '') || after;
	return { before, passage: text.slice(start, end), after, at: start };
}

/** Why a passage may not be sent: too few letters or figures, or too long; null when it may. */
export function passageProblem(passage: string): 'short' | 'long' | null {
	const letters = passage.match(/[\p{L}\p{N}]/gu)?.length ?? 0;
	if (letters < PASSAGE_LIMITS.min) return 'short';
	if (charCount(passage) > PASSAGE_LIMITS.max) return 'long';
	return null;
}

/** Characters as the engine counts them, a character written in two units once. */
export function charCount(text: string): number {
	return [...text].length;
}

/** `text` with the passage of `around` replaced by `with`, as the engine writes it. */
export function withPassage(text: string, around: Around, replacement: string): string {
	return text.slice(0, around.at) + replacement + text.slice(around.at + around.passage.length);
}

/**
 * The selection in the text shown, as offsets in `base`: null when nothing is selected there.
 * `pane` shows `base` one line per `.line`, its words in the line's `.t` (`MarkedText`). A
 * selection that starts or ends outside the text (in the header, the notes, the next pane) is
 * cut to the text and said to be.
 */
export function selectionInText(pane: HTMLElement | null, base: string): Chosen | null {
	if (!pane) return null;
	const root = pane.querySelector<HTMLElement>('.text.numbered');
	const chosen = getSelection();
	if (!root || !chosen || chosen.rangeCount === 0 || chosen.isCollapsed) return null;
	const range = chosen.getRangeAt(0);
	const rows = [...root.querySelectorAll<HTMLElement>(':scope > .line')];
	const starts = lineStarts(base);
	if (rows.length !== starts.length) return null;
	const whole = document.createRange();
	whole.selectNodeContents(root);
	// Entirely before or after the text: nothing of it is chosen.
	if (!range.intersectsNode(root)) return null;
	const offsetAt = (node: Node, offset: number, atEnd: boolean): number => {
		const where = whole.comparePoint(node, offset);
		if (where < 0) return 0;
		if (where > 0) return base.length;
		const row = (node instanceof Element ? node : node.parentElement)?.closest<HTMLElement>(
			'.line'
		);
		const i = row ? rows.indexOf(row) : -1;
		if (i < 0 || !row) return atEnd ? base.length : 0;
		const words = row.querySelector('.t');
		const length = (starts[i + 1] ?? base.length + 1) - 1 - starts[i];
		if (!words || !words.contains(node)) {
			// On a line's number, or between lines: the line's start, or its end.
			const before = words ? words.compareDocumentPosition(node) : 0;
			return before & Node.DOCUMENT_POSITION_FOLLOWING ? starts[i] + length : starts[i];
		}
		const upTo = document.createRange();
		upTo.setStart(words, 0);
		try {
			upTo.setEnd(node, offset);
		} catch {
			return starts[i];
		}
		return starts[i] + Math.min(upTo.toString().length, length);
	};
	const start = offsetAt(range.startContainer, range.startOffset, false);
	const end = offsetAt(range.endContainer, range.endOffset, true);
	if (end <= start) return null;
	const cut =
		whole.comparePoint(range.startContainer, range.startOffset) < 0 ||
		whole.comparePoint(range.endContainer, range.endOffset) > 0;
	return { start, end, cut };
}

/** Where the selection stands on screen, to open a menu under it from the keyboard. */
export function selectionBox(): DOMRect | null {
	const chosen = getSelection();
	if (!chosen || chosen.rangeCount === 0) return null;
	const rects = chosen.getRangeAt(0).getClientRects();
	return rects.length > 0 ? rects[rects.length - 1] : null;
}
