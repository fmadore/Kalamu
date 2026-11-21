// The search by meaning and the map, as the interface shows them: where the set-up stands, a
// similarity written soberly, and the map's groups, shapes, scale and keyboard moves. The work
// itself (chunks, vectors, ranking, the projection) is done in Rust (`kalamu-search`,
// `kalamu-engine`). Pure, so it is tested without the app.

import type { CollectionView } from './bindings/CollectionView.js';
import type { MapPoint } from './bindings/MapPoint.js';
import type { MeaningPassage } from './bindings/MeaningPassage.js';
import type { MeaningRequest } from './bindings/MeaningRequest.js';
import type { MeaningStatus } from './bindings/MeaningStatus.js';
import { formatUsd, type Messages } from './i18n/index.js';
import type { ReviewFind } from './state.svelte.js';

/** The search by meaning a project opens with: every document, the preferred readings. */
export function blankMeaningRequest(): MeaningRequest {
	return {
		query: '',
		collection: null,
		tag: null,
		task: null,
		review: 'any',
		results: 'preferred'
	};
}

/** What the review opens at for a passage: its piece, and its opening words among those the
 * review's search finds there, as a word search's match opens. */
export function meaningFind(passage: MeaningPassage): ReviewFind {
	return {
		query: passage.hit.term,
		setupId: passage.setup_id,
		index: passage.hit.index,
		nth: passage.hit.nth,
		field: null,
		entry: null
	};
}

/** Dollars as shown: an amount too small for a tenth of a cent is said to be less than one,
 * rather than written as nothing. */
export function usdText(value: number, m: Messages): string {
	return value > 0 && value < 0.001 ? m.meaning.setup.under(formatUsd(0.001)) : formatUsd(value);
}

/** A similarity as shown: two decimals, never a percentage, which would read as a probability. */
export function similarityText(value: number, locale: string): string {
	return new Intl.NumberFormat(locale, {
		minimumFractionDigits: 2,
		maximumFractionDigits: 2
	}).format(value);
}

/** Where the set-up of a project's search by meaning stands, for what it shows first. */
export type MeaningStep =
	/** No model is offered in this copy. */
	| 'no_models'
	/** No model chosen for the project. */
	| 'choose'
	/** The project's model is no longer offered. */
	| 'gone'
	/** The model cannot run now: not downloaded, or no key. */
	| 'blocked'
	/** A cloud model the researcher has not agreed to send the text to. */
	| 'consent'
	/** A preparation is running. */
	| 'running'
	/** Nothing prepared yet, or something left to prepare: the estimate and the button. */
	| 'prepare'
	/** Everything read is prepared and current. */
	| 'ready';

export function meaningStep(status: MeaningStatus): MeaningStep {
	if (status.running) return 'running';
	if (status.models.length === 0 && !status.choice) return 'no_models';
	if (!status.choice) return 'choose';
	if (!status.model) return 'gone';
	if (status.blocked) return 'blocked';
	if (!status.model.local && status.choice.consented_at === null) return 'consent';
	if (status.counts && status.counts.chunks > 0) return 'prepare';
	return 'ready';
}

/** Whether a search can be asked: something is prepared and the model can embed the question. */
export function canAsk(status: MeaningStatus): boolean {
	const step = meaningStep(status);
	return (
		(step === 'ready' || step === 'prepare' || step === 'running') &&
		!!status.counts &&
		status.counts.prepared > 0
	);
}

/** The shapes a map's groups are drawn with, so a group is never told by its colour alone. */
export const SHAPES = ['circle', 'square', 'triangle', 'diamond', 'cross', 'ring'] as const;
export type Shape = (typeof SHAPES)[number] | 'hollow';

/** How the map's points are grouped. */
export type MapGrouping = 'collection' | 'tag';

export interface MapGroup {
	/** A collection's id, a tag, or empty for the documents in none. */
	key: string;
	/** The collection's name or the tag; empty for none, which the interface names. */
	label: string;
	/** From 0, the palette's place; -1 for the documents in no group. */
	tone: number;
	shape: Shape;
	count: number;
}

/** The most groups drawn apart; the rest are drawn as one, "other". */
export const MAX_GROUPS = SHAPES.length;
const OTHER = '\u0000other';

/**
 * Each point's group and the legend: by collection, a document's first collection in the
 * project's order; by tag, its first tag in alphabetical order. The largest groups are drawn
 * apart, each with its own shape and tone; the rest share one, and documents in none are hollow.
 */
export function mapGroups(
	points: MapPoint[],
	by: MapGrouping,
	collections: CollectionView[]
): { legend: MapGroup[]; groupOf: Map<string, MapGroup> } {
	const order = collections.map((c) => c.collection.id);
	const name = new Map(collections.map((c) => [c.collection.id, c.collection.name]));
	const keyOf = (p: MapPoint): string => {
		if (by === 'collection') {
			const mine = order.filter((id) => p.collections.includes(id));
			return mine[0] ?? '';
		}
		return [...p.tags].sort((a, b) => a.localeCompare(b))[0] ?? '';
	};
	const counts = new Map<string, number>();
	for (const p of points) counts.set(keyOf(p), (counts.get(keyOf(p)) ?? 0) + 1);
	const named = [...counts.keys()]
		.filter((k) => k !== '')
		.sort((a, b) => (counts.get(b) ?? 0) - (counts.get(a) ?? 0) || a.localeCompare(b));
	const kept = named.length > MAX_GROUPS ? named.slice(0, MAX_GROUPS - 1) : named;
	const groups = new Map<string, MapGroup>();
	kept.forEach((key, i) =>
		groups.set(key, {
			key,
			label: by === 'collection' ? (name.get(key) ?? key) : key,
			tone: i,
			shape: SHAPES[i],
			count: counts.get(key) ?? 0
		})
	);
	const rest = named.slice(kept.length);
	if (rest.length > 0) {
		groups.set(OTHER, {
			key: OTHER,
			label: '',
			tone: MAX_GROUPS - 1,
			shape: SHAPES[MAX_GROUPS - 1],
			count: rest.reduce((n, k) => n + (counts.get(k) ?? 0), 0)
		});
	}
	if (counts.has('')) {
		groups.set('', { key: '', label: '', tone: -1, shape: 'hollow', count: counts.get('') ?? 0 });
	}
	const groupOf = new Map<string, MapGroup>();
	for (const p of points) {
		const key = keyOf(p);
		const group = groups.get(key) ?? groups.get(OTHER);
		if (group) groupOf.set(p.document_id, group);
	}
	return { legend: [...groups.values()], groupOf };
}

/** Whether a legend entry stands for the groups not drawn apart. */
export function isOther(group: MapGroup): boolean {
	return group.key === OTHER;
}

/** The part of the map shown: its top-left corner and its size, in the map's own units (0 to 1). */
export interface MapWindow {
	x: number;
	y: number;
	size: number;
}

export const WHOLE_MAP: MapWindow = { x: -0.05, y: -0.05, size: 1.1 };
const MIN_SIZE = 0.02;

/** The window zoomed by `factor` (above 1 closer) around the point (`cx`, `cy`). */
export function zoomed(view: MapWindow, factor: number, cx: number, cy: number): MapWindow {
	const size = Math.min(Math.max(view.size / factor, MIN_SIZE), WHOLE_MAP.size);
	const k = size / view.size;
	return { x: cx - (cx - view.x) * k, y: cy - (cy - view.y) * k, size };
}

/** The window moved by (`dx`, `dy`) in the map's units. */
export function panned(view: MapWindow, dx: number, dy: number): MapWindow {
	return { ...view, x: view.x + dx, y: view.y + dy };
}

/** A point's place on the drawing: the map's y grows upwards, the drawing's downwards. */
export function placed(p: { x: number; y: number }): { x: number; y: number } {
	return { x: p.x, y: 1 - p.y };
}

export type Direction = 'left' | 'right' | 'up' | 'down';

/**
 * The point an arrow key moves to from `from`: the nearest one lying in that direction (within
 * 45 degrees either side), nearer ones along the direction first; none at the edge.
 */
export function nextPoint(
	points: MapPoint[],
	from: MapPoint,
	direction: Direction
): MapPoint | null {
	const a = placed(from);
	let best: MapPoint | null = null;
	let bestScore = Infinity;
	for (const p of points) {
		if (p.document_id === from.document_id) continue;
		const b = placed(p);
		const dx = b.x - a.x;
		const dy = b.y - a.y;
		const along = { left: -dx, right: dx, up: -dy, down: dy }[direction];
		const across = direction === 'left' || direction === 'right' ? Math.abs(dy) : Math.abs(dx);
		if (along <= 0 || across > along) continue;
		const score = along + 2 * across;
		if (score < bestScore) {
			best = p;
			bestScore = score;
		}
	}
	return best;
}

/**
 * The points as drawn: those closer than `gap` to each other (marks that would lie on one
 * another at this zoom) set apart in a small ring around where they sit together, in the order
 * they lie around it, so each can be seen, pointed at and read; any other where it is. The ring
 * shrinks as the map is zoomed in, `gap` with it, until the points stand at their own places.
 * A chain of points spread wider than a ring would be is left as it is. Only the drawing moves:
 * the places stay the model's.
 */
export function spread<P extends { document_id: string; x: number; y: number }>(
	points: P[],
	gap: number
): P[] {
	if (gap <= 0 || points.length < 2) return points;
	// Points within `gap` of each other, joined into groups.
	const group = points.map((_, i) => i);
	const root = (i: number): number => (group[i] === i ? i : (group[i] = root(group[i])));
	for (let i = 0; i < points.length; i++) {
		for (let j = i + 1; j < points.length; j++) {
			if (Math.hypot(points[i].x - points[j].x, points[i].y - points[j].y) < gap) {
				group[root(j)] = root(i);
			}
		}
	}
	const members = new Map<number, number[]>();
	points.forEach((_, i) => members.set(root(i), [...(members.get(root(i)) ?? []), i]));
	const out = [...points];
	for (const together of members.values()) {
		const n = together.length;
		if (n < 2) continue;
		const cx = together.reduce((s, i) => s + points[i].x, 0) / n;
		const cy = together.reduce((s, i) => s + points[i].y, 0) / n;
		// Neighbours a gap apart round the ring.
		const ring = gap / (2 * Math.sin(Math.PI / n));
		const wide = Math.max(...together.map((i) => Math.hypot(points[i].x - cx, points[i].y - cy)));
		if (wide > ring) continue;
		const order = [...together].sort(
			(a, b) =>
				Math.atan2(points[a].y - cy, points[a].x - cx) -
					Math.atan2(points[b].y - cy, points[b].x - cx) ||
				points[a].document_id.localeCompare(points[b].document_id)
		);
		order.forEach((i, k) => {
			const angle = Math.PI / 2 + (2 * Math.PI * k) / n;
			out[i] = { ...points[i], x: cx + ring * Math.cos(angle), y: cy + ring * Math.sin(angle) };
		});
	}
	return out;
}
