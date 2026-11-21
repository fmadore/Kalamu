import { describe, expect, it } from 'vitest';
import { en } from './i18n/index.js';
import type { CollectionView } from './bindings/CollectionView.js';
import type { MapPoint } from './bindings/MapPoint.js';
import type { MeaningStatus } from './bindings/MeaningStatus.js';
import {
	MAX_GROUPS,
	WHOLE_MAP,
	canAsk,
	isOther,
	mapGroups,
	meaningStep,
	nextPoint,
	panned,
	spread,
	similarityText,
	usdText,
	zoomed
} from './meaning.js';

function status(over: Partial<MeaningStatus> = {}): MeaningStatus {
	const model = {
		id: 'embed',
		provider: 'gemini',
		name: 'Embed',
		dimensions: [768],
		max_chars: 8000,
		input_price: 0.15,
		local: false,
		multilingual: true
	};
	return {
		models: [model],
		choice: {
			project_id: 'p',
			space: { provider: 'gemini', model: 'embed', dimensions: 768 },
			chosen_at: 1,
			consented_at: 2,
			queries: 0,
			query_tokens: 0,
			query_cost_usd: null
		},
		model,
		blocked: null,
		counts: {
			pieces: 10,
			prepared: 10,
			missing: 0,
			changed: 0,
			chunks: 0,
			chars: 0,
			tokens: 0,
			skipped: 0
		},
		estimated_cost_usd: null,
		running: null,
		runs: [],
		prepared_at: 3,
		spaces: [],
		spent_usd: null,
		...over
	};
}

function point(id: string, x: number, y: number, over: Partial<MapPoint> = {}): MapPoint {
	return { document_id: id, title: id, x, y, collections: [], tags: [], chunks: 1, ...over };
}

describe('the set-up', () => {
	it('stands where the next thing to do is', () => {
		expect(meaningStep(status())).toBe('ready');
		expect(meaningStep(status({ models: [], choice: null, model: null }))).toBe('no_models');
		expect(meaningStep(status({ choice: null, model: null }))).toBe('choose');
		expect(meaningStep(status({ model: null }))).toBe('gone');
		expect(meaningStep(status({ blocked: 'provider.missing_key' }))).toBe('blocked');
		const unconsented = status();
		unconsented.choice!.consented_at = null;
		expect(meaningStep(unconsented)).toBe('consent');
		// A model run on this computer needs no consent.
		unconsented.model = { ...unconsented.model!, local: true };
		expect(meaningStep(unconsented)).toBe('ready');
		const changed = status();
		changed.counts = { ...changed.counts!, chunks: 3, changed: 1 };
		expect(meaningStep(changed)).toBe('prepare');
	});

	it('lets a question be asked once something is prepared', () => {
		expect(canAsk(status())).toBe(true);
		const none = status();
		none.counts = { ...none.counts!, prepared: 0, chunks: 12 };
		expect(canAsk(none)).toBe(false);
	});

	it('says a cost too small to write is less than a tenth of a cent', () => {
		expect(usdText(0.00015, en)).toBe('less than $0.001');
		expect(usdText(0.25, en)).toBe('$0.25');
		expect(usdText(0, en)).toBe('$0.000');
	});

	it('writes a similarity as a plain number, never a percentage', () => {
		expect(similarityText(0.8234, 'en')).toBe('0.82');
		expect(similarityText(0.8234, 'fr')).toBe('0,82');
	});
});

describe('the map', () => {
	const collections: CollectionView[] = [
		{ collection: { id: 'c1', project_id: 'p', name: 'Letters', created_at: 1 }, documents: 2 },
		{ collection: { id: 'c2', project_id: 'p', name: 'Bulletins', created_at: 1 }, documents: 1 }
	];

	it('groups points by first collection or first tag, each with its own shape', () => {
		const points = [
			point('a', 0, 0, { collections: ['c2', 'c1'], tags: ['Dakar'] }),
			point('b', 1, 0, { collections: ['c1'], tags: ['Thiès', 'Dakar'] }),
			point('c', 0, 1)
		];
		const byCollection = mapGroups(points, 'collection', collections);
		expect(byCollection.groupOf.get('a')?.label).toBe('Letters');
		expect(byCollection.groupOf.get('c')?.shape).toBe('hollow');
		expect(byCollection.legend.map((g) => [g.label, g.count])).toEqual([
			['Letters', 2],
			['', 1]
		]);
		const byTag = mapGroups(points, 'tag', collections);
		expect(byTag.groupOf.get('b')?.label).toBe('Dakar');
		expect(new Set(byTag.legend.map((g) => g.shape)).size).toBe(byTag.legend.length);
	});

	it('draws the largest groups apart and the rest as one', () => {
		const points = Array.from({ length: 10 }, (_, i) =>
			point(String(i), i / 10, 0, { tags: [`t${i}`] })
		);
		const { legend } = mapGroups(points, 'tag', []);
		expect(legend.length).toBe(MAX_GROUPS);
		expect(legend.filter(isOther)[0]?.count).toBe(10 - (MAX_GROUPS - 1));
	});

	it('sets apart points drawn on one another, and leaves the rest where they are', () => {
		// Nine issues of one newspaper in a tight cluster, a letter far from them.
		const issues = Array.from({ length: 9 }, (_, i) =>
			point(`issue-${i}`, 0.5 + (i % 3) * 0.002, 0.5 + Math.floor(i / 3) * 0.002)
		);
		const letter = point('letter', 0.1, 0.9);
		const gap = 0.04;
		const drawn = spread([...issues, letter], gap);
		expect(drawn.at(-1)).toEqual(letter);
		const ring = drawn.slice(0, 9);
		for (let a = 0; a < ring.length; a++) {
			for (let b = a + 1; b < ring.length; b++) {
				const apart = Math.hypot(ring[a].x - ring[b].x, ring[a].y - ring[b].y);
				expect(apart).toBeGreaterThanOrEqual(gap * 0.999);
			}
		}
		// Around where they sit, not far from it; nothing else of a point changes.
		for (const p of ring) expect(Math.hypot(p.x - 0.502, p.y - 0.502)).toBeLessThan(gap * 2);
		expect(ring.map((p) => p.document_id)).toEqual(issues.map((p) => p.document_id));
		// Zoomed in far enough, they stand at their own places.
		expect(spread(issues, 0.001)).toEqual(issues);
		// Points that do not touch, or a chain wider than a ring, are left as they are.
		const apart = [point('a', 0, 0), point('b', 0.5, 0)];
		expect(spread(apart, gap)).toEqual(apart);
		const chain = Array.from({ length: 6 }, (_, i) => point(`c${i}`, i * 0.035, 0));
		expect(spread(chain, gap)).toEqual(chain);
	});

	it('zooms around a point and pans, never past the whole map', () => {
		const closer = zoomed(WHOLE_MAP, 2, 0.5, 0.5);
		expect(closer.size).toBeCloseTo(0.55);
		expect(closer.x + closer.size / 2).toBeCloseTo(0.5);
		expect(zoomed(closer, 0.1, 0.5, 0.5).size).toBe(WHOLE_MAP.size);
		expect(panned(closer, 0.1, 0).x).toBeCloseTo(closer.x + 0.1);
	});

	it('moves with the arrow keys to the nearest point in that direction', () => {
		const points = [
			point('centre', 0.5, 0.5),
			point('right', 0.8, 0.55),
			point('far-right', 0.95, 0.5),
			point('up', 0.5, 0.9),
			point('left', 0.2, 0.4),
			point('down', 0.45, 0.1)
		];
		const centre = points[0];
		expect(nextPoint(points, centre, 'right')?.document_id).toBe('right');
		// Up on the map is a larger y.
		expect(nextPoint(points, centre, 'up')?.document_id).toBe('up');
		expect(nextPoint(points, centre, 'left')?.document_id).toBe('left');
		expect(nextPoint(points, centre, 'down')?.document_id).toBe('down');
		expect(nextPoint(points, points[2], 'right')).toBeNull();
	});
});
