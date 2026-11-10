// Words found by a search, marked where they stand on the page through the browser's own
// highlights (`::highlight(found)` in app.css), so the text's elements stay as they are.

import { find } from './search.js';

export const supported = () => typeof CSS !== 'undefined' && 'highlights' in CSS;

/** Takes the marks off. */
export function clearFound(): void {
	if (!supported()) return;
	CSS.highlights.delete('found');
	CSS.highlights.delete('found-now');
}

/** The text of the `blocks` under `root`, each block read as a line of its own, and a range
 * over any stretch of it, as it stands among the page's text nodes. */
export function located(
	root: ParentNode,
	blocks: string
): { text: string; range: (start: number, end: number) => Range | null } {
	const nodes: { node: Text; at: number }[] = [];
	let text = '';
	for (const block of root.querySelectorAll(blocks)) {
		if (text) text += '\n';
		const walk = document.createTreeWalker(block, NodeFilter.SHOW_TEXT);
		for (let n = walk.nextNode(); n; n = walk.nextNode()) {
			nodes.push({ node: n as Text, at: text.length });
			text += (n as Text).data;
		}
	}
	/** The text node and offset in it of a place in the text; an end belongs to the node before. */
	function point(offset: number, end: boolean): [Text, number] {
		const hit =
			nodes.find(({ node, at }) =>
				end ? offset > at && offset <= at + node.length : offset >= at && offset < at + node.length
			) ?? nodes[nodes.length - 1];
		return [hit.node, Math.min(offset - hit.at, hit.node.length)];
	}
	return {
		text,
		range(start, end) {
			if (nodes.length === 0) return null;
			const range = new Range();
			range.setStart(...point(start, false));
			range.setEnd(...point(end, true));
			return range;
		}
	};
}

/** Marks every place `query` is found in the text of the `blocks` under `root`, each block read
 * as a line of its own, and the `current`-th (from 0) as the one gone to, which it returns. */
export function markFound(
	root: ParentNode,
	blocks: string,
	query: string,
	current = -1
): Range | null {
	clearFound();
	if (!supported()) return null;
	const { text, range } = located(root, blocks);
	const ranges = find(text, query).flatMap((m) => range(m.start, m.end) ?? []);
	if (ranges.length === 0) return null;
	CSS.highlights.set('found', new Highlight(...ranges));
	const now = ranges[current] ?? null;
	if (now) CSS.highlights.set('found-now', new Highlight(now));
	return now;
}
