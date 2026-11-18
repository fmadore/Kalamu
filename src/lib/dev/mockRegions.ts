// Where the sample newspaper's text sits on its scan (`sample-scan.jpg`), as `page_regions`
// answers for the first set-up: page 1 as Tesseract places it, every word on its own, in two
// columns; page 2 as Mistral OCR does, one block whose lines and words are estimated; page 3 not
// placed at all (the page-only fallback). Places in the text are found roughly as the app finds
// them (crates/core/src/regions.rs): by position while the text is the model's, and only where a
// region's words stand once after a correction. Each region has the id the app gives it (`r1`,
// `r1_l2`, `r1_l2_w3`), and the researcher's own regions (`mockDrawn.ts`) are laid over them as
// the engine lays them.

import type { Granularity } from '../bindings/Granularity.js';
import type { PageRegions } from '../bindings/PageRegions.js';
import type { ShownRegion } from '../bindings/ShownRegion.js';
import type { TextSpan } from '../bindings/TextSpan.js';
import { drawnOn, resultKey, withDrawn } from './mockDrawn.js';
import { job, pieces, setupA, source, texts } from './mockJobs.js';

/** The scan's size in pixels, which the boxes below are measured in. */
const W = 992;
const H = 1403;

/** A line of print: its left and right edges and its middle, in pixels, and how many words of
 * the text it holds. */
type PrintLine = [left: number, right: number, middle: number, words: number];

/** Page 1's blocks as Tesseract finds them: paragraphs, each column apart. */
const PAGE_ONE: PrintLine[][] = [
	[
		[112, 443, 286, 3],
		[235, 320, 318, 1]
	],
	[[297, 478, 355, 5]],
	[
		[107, 478, 393, 8],
		[77, 478, 423, 7],
		[77, 478, 453, 5],
		[77, 478, 483, 8],
		[77, 478, 513, 7],
		[77, 432, 543, 5]
	],
	[
		[107, 478, 584, 7],
		[77, 478, 614, 7],
		[77, 478, 644, 6],
		[77, 255, 674, 4]
	],
	[[145, 411, 716, 4]],
	[
		[107, 478, 754, 8],
		[77, 478, 784, 8],
		[77, 478, 814, 7]
	],
	[
		[514, 915, 284, 6],
		[514, 886, 314, 7]
	],
	[
		[544, 915, 355, 7],
		[514, 915, 385, 7],
		[514, 915, 415, 6],
		[514, 855, 445, 6]
	]
];

const region = (
	id: string,
	granularity: Granularity,
	x: number,
	y: number,
	width: number,
	height: number,
	text: string
): ShownRegion => ({
	id,
	x,
	y,
	width,
	height,
	text,
	granularity,
	origin: 'provider',
	span: null as TextSpan | null
});

/** Words set along a line's box, each as wide as its letters, numbered within line `lineId`. */
function setWords(
	lineId: string,
	words: string[],
	x: number,
	y: number,
	width: number,
	height: number
) {
	const total = words.reduce((n, w) => n + w.length + 1, -1);
	const unit = width / Math.max(total, 1);
	let at = x;
	return words.map((w, k) => {
		const r = region(`${lineId}_w${k + 1}`, 'word', at, y, w.length * unit, height, w);
		at += (w.length + 1) * unit;
		return r;
	});
}

type Placed = Pick<PageRegions, 'blocks' | 'lines' | 'words'>;

/** Page 1, placed word by word: blocks, lines and words, in reading order. */
function tesseractPage(text: string): Placed {
	const words = text.split(/\s+/).filter(Boolean);
	const blocks: ShownRegion[] = [];
	const lines: ShownRegion[] = [];
	const placed: ShownRegion[] = [];
	let next = 0;
	for (const [b, block] of PAGE_ONE.entries()) {
		const own: ShownRegion[] = [];
		for (const [l, [left, right, middle, count]] of block.entries()) {
			const held = words.slice(next, next + count);
			next += count;
			const [x, y, w, h] = [left / W, (middle - 12) / H, (right - left) / W, 24 / H];
			const id = `r${b + 1}_l${l + 1}`;
			own.push(region(id, 'line', x, y, w, h, held.join(' ')));
			placed.push(...setWords(id, held, x, y, w, h));
		}
		lines.push(...own);
		const x = Math.min(...own.map((l) => l.x));
		const y = own[0].y;
		const right = Math.max(...own.map((l) => l.x + l.width));
		const bottom = own[own.length - 1].y + own[own.length - 1].height;
		const joined = own.map((l) => l.text).join('\n');
		blocks.push(region(`r${b + 1}`, 'block', x, y, right - x, bottom - y, joined));
	}
	return { blocks, lines, words: placed };
}

/** Page 2, one block placed whole: its line and words estimated within it. */
function mistralPage(text: string): Placed {
	const [x, y, w, h] = [77 / W, 742 / H, 401 / W, 86 / H];
	const line: ShownRegion = { ...region('r1_l1', 'line', x, y, w, h, text), origin: 'estimated' };
	const words = setWords('r1_l1', text.split(/\s+/).filter(Boolean), x, y, w, h).map(
		(r): ShownRegion => ({ ...r, origin: 'estimated' })
	);
	return { blocks: [region('r1', 'block', x, y, w, h, text)], lines: [line], words };
}

/** Where a run of words stands in `text`, if it stands there exactly once. */
function onlyPlace(text: string, words: string): TextSpan | null {
	const escaped = words
		.split(/\s+/)
		.filter(Boolean)
		.map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
	if (escaped.length === 0) return null;
	const found = [...text.matchAll(new RegExp(`(?<!\\S)${escaped.join('\\s+')}(?!\\S)`, 'gu'))];
	return found.length === 1
		? { start: found[0].index, end: found[0].index + found[0][0].length }
		: null;
}

/** The model's text is the regions' words in order: each region takes the next of its words. */
function placeInOrder(text: string, list: ShownRegion[]) {
	const tokens = [...text.matchAll(/\S+/gu)];
	let at = 0;
	for (const r of list) {
		const n = r.text.split(/\s+/).filter(Boolean).length;
		const [first, last] = [tokens[at], tokens[at + n - 1]];
		r.span =
			n > 0 && first && last ? { start: first.index, end: last.index + last[0].length } : null;
		at += n;
	}
}

/** The regions `page_regions` gives for piece `index` under set-up `setupId`, the researcher's
 * own laid over them. */
export function mockRegions(jobId: string, setupId: string, index: number): PageRegions {
	const view = pieces.find((p) => p.setup_id === setupId && p.index === index);
	const corrected = view?.corrected_text != null;
	const ours = jobId === job.id && view?.source_id === source.id;
	const none: PageRegions = {
		blocks: [],
		lines: [],
		words: [],
		mapping: 'exact',
		corrected,
		result: ours && view?.status === 'done' ? resultKey(setupId, index) : null,
		drawn: []
	};
	if (!view || view.piece.kind !== 'page') return none;
	const drawn = drawnOn(jobId, view.source_id, view.piece.number, view.piece.half ?? null);
	if (!ours || setupId !== setupA.id) return withDrawn(none, drawn);
	const model = texts[index] ?? '';
	const page = index === 0 ? tesseractPage(model) : index === 1 ? mistralPage(model) : null;
	if (!page) return withDrawn(none, drawn);
	const shown = view.corrected_text ?? model;
	if (shown === model) {
		for (const list of [page.blocks, page.lines, page.words]) placeInOrder(shown, list);
		return withDrawn({ ...none, ...page, mapping: 'exact' }, drawn);
	}
	for (const r of [...page.blocks, ...page.lines, ...page.words]) {
		r.span = onlyPlace(model, r.text) ? onlyPlace(shown, r.text) : null;
	}
	return withDrawn({ ...none, ...page, mapping: 'unique' }, drawn);
}
