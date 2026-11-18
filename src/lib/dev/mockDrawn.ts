// Regions the researcher draws over a scan in the browser preview, or whose box they adjust,
// kept in memory as the engine keeps them (`engine::regions::drawn`): checked as `core` checks
// a box, laid over the engine's regions, removed and put back by their id. A box adjusting a
// region of a reading shown under another set-up is that set-up's; one adjusting a reading no
// set-up shows is shown as drawn. Development builds only.

import type { AdjustedRegion } from '../bindings/AdjustedRegion.js';
import type { DrawnRegion } from '../bindings/DrawnRegion.js';
import type { PageHalf } from '../bindings/PageHalf.js';
import type { PageRegions } from '../bindings/PageRegions.js';
import type { RegionDraft } from '../bindings/RegionDraft.js';
import type { ShownRegion } from '../bindings/ShownRegion.js';
import { job, pieces, source } from './mockJobs.js';

/** As `core::regions`: how far past an edge a box is clamped, and the smallest box kept. */
const SLACK = 0.01;
const MIN_SIZE = 0.004;

const regions: DrawnRegion[] = [];

/** The stored result's cache key a piece's regions come from, here made up from its place. */
export const resultKey = (setupId: string, index: number) => `mock-result-${setupId}-${index}`;

const fail = (id: string): never => {
	throw { id };
};

/** The readings the sample job's set-ups show of page `page` (or that half). */
function readings(page: number, half: PageHalf | null): Set<string> {
	return new Set(
		pieces
			.filter(
				(p) =>
					p.source_id === source.id &&
					p.status === 'done' &&
					p.piece.kind === 'page' &&
					p.piece.number === page &&
					(p.piece.half ?? null) === half
			)
			.map((p) => resultKey(p.setup_id, p.index))
	);
}

/** The researcher's regions on a page, in the order they were made. */
export function drawnOn(
	jobId: string,
	sourceId: string,
	page: number,
	half: PageHalf | null
): DrawnRegion[] {
	return regions.filter(
		(r) => r.job_id === jobId && r.source_id === sourceId && r.page === page && r.half === half
	);
}

const shownOf = (r: DrawnRegion): ShownRegion => ({
	id: r.id,
	x: r.x,
	y: r.y,
	width: r.width,
	height: r.height,
	text: r.text,
	granularity: 'block',
	origin: 'manual',
	span: null
});

/** `page` with `drawn` laid over it, as the engine lays them: a box adjusting the reading
 * shown takes its region's place; one adjusting another reading a set-up still shows is left
 * out; any other is among `drawn`. */
export function withDrawn(page: PageRegions, drawn: DrawnRegion[]): PageRegions {
	const out: PageRegions = { ...page, drawn: [...page.drawn] };
	for (const r of drawn) {
		if (
			r.adjusts &&
			r.adjusts.result !== page.result &&
			readings(r.page, r.half).has(r.adjusts.result)
		) {
			continue;
		}
		const adjusts = r.adjusts;
		const lists = [out.blocks, out.lines, out.words];
		const at =
			adjusts && adjusts.result === page.result
				? lists.find((l) => l.some((s) => s.id === adjusts.region))
				: undefined;
		if (at && adjusts) {
			const i = at.findIndex((s) => s.id === adjusts.region);
			at[i] = { ...at[i], x: r.x, y: r.y, width: r.width, height: r.height, origin: 'manual' };
		} else {
			out.drawn.push(shownOf(r));
		}
	}
	return out;
}

/** A draft's box made tidy, or refused, as `core::regions::check` does. */
function checked(d: RegionDraft): RegionDraft {
	if (![d.x, d.y, d.width, d.height].every(Number.isFinite)) fail('region.not_finite');
	if (d.page < 1) fail('region.no_page');
	const [x0, x1] = [Math.min(d.x, d.x + d.width), Math.max(d.x, d.x + d.width)];
	const [y0, y1] = [Math.min(d.y, d.y + d.height), Math.max(d.y, d.y + d.height)];
	if (x0 < -SLACK || y0 < -SLACK || x1 > 1 + SLACK || y1 > 1 + SLACK) fail('region.outside');
	const [left, top, right, bottom] = [
		Math.max(x0, 0),
		Math.max(y0, 0),
		Math.min(x1, 1),
		Math.min(y1, 1)
	];
	if (right - left < MIN_SIZE || bottom - top < MIN_SIZE) fail('region.too_small');
	return { ...d, x: left, y: top, width: right - left, height: bottom - top };
}

/** The sample job's newspaper, the only source the preview draws on. */
function onPage(jobId: string, sourceId: string, page: number, half: PageHalf | null) {
	if (jobId !== job.id || sourceId !== source.id) fail('job.not_found');
	if (page > (source.page_count ?? 0) || half !== null) fail('region.no_page');
}

function save(draft: RegionDraft): DrawnRegion {
	const d = checked(draft);
	onPage(d.job_id, d.source_id, d.page, d.half);
	const adjusts: AdjustedRegion | null = d.adjusts;
	// The reading must still be shown; whether it holds the region is not checked here.
	if (adjusts && !readings(d.page, d.half).has(adjusts.result)) fail('region.adjusts');
	const existing = d.id
		? (regions.find((r) => r.id === d.id) ?? fail('region.not_found'))
		: adjusts
			? regions.find(
					(r) => r.adjusts?.result === adjusts.result && r.adjusts.region === adjusts.region
				)
			: undefined;
	const now = Date.now();
	const region: DrawnRegion = {
		id: existing?.id ?? crypto.randomUUID(),
		job_id: d.job_id,
		source_id: d.source_id,
		page: d.page,
		half: d.half,
		x: d.x,
		y: d.y,
		width: d.width,
		height: d.height,
		text: d.text,
		adjusts,
		source_hash: source.sha256,
		created_at: existing?.created_at ?? now,
		updated_at: now
	};
	const at = regions.findIndex((r) => r.id === region.id);
	if (at >= 0) regions[at] = region;
	else regions.push(region);
	return region;
}

export function regionCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'save_region':
			return save(a.draft as RegionDraft);
		case 'remove_region': {
			const at = regions.findIndex((r) => r.id === a.id);
			if (at < 0) fail('region.not_found');
			return regions.splice(at, 1)[0];
		}
		case 'restore_region': {
			const region = a.region as DrawnRegion;
			onPage(region.job_id, region.source_id, region.page, region.half);
			regions.push(region);
			return region;
		}
		case 'drawn_regions': {
			const [jobId, sourceId] = [String(a.jobId), String(a.sourceId)];
			const half = (a.half ?? null) as PageHalf | null;
			const page: PageRegions = {
				blocks: [],
				lines: [],
				words: [],
				mapping: 'exact',
				corrected: false,
				result: null,
				drawn: []
			};
			return withDrawn(page, drawnOn(jobId, sourceId, Number(a.page), half));
		}
		default:
			return undefined;
	}
}
