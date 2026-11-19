// Regions read again by a pretend model in the browser preview, as the engine does it
// (`engine::region_reading`): checked as `core` checks a box, answered after a moment unless
// stopped, kept so the same request again costs nothing, and written only when accepted: into
// the text of a region the researcher drew (the preview's drawn regions) or in place of the
// region's words in the piece's text (its corrections). The "model" reads a marginal note, or
// mends the words it is shown. Development builds only.

import type { Correction } from '../bindings/Correction.js';
import type { ReadingEffect } from '../bindings/ReadingEffect.js';
import type { RegionAsk } from '../bindings/RegionAsk.js';
import type { RegionReading } from '../bindings/RegionReading.js';
import { locateWords, withReading } from '../regionReading.js';
import { PretendModel, fail, mend } from './mockAsking.js';
import { drawnOn } from './mockDrawn.js';
import type { FindPiece } from './mockPassages.js';

/** As `core::regions`: how far past an edge a box may reach, and the smallest box kept. */
const SLACK = 0.01;
const MIN_SIZE = 0.004;
/** What a reading costs here, when it is not free. */
const COST = 0.002;
/** What the pretend model reads in a region the researcher drew. */
const NOTE = 'Vu et approuvé, le 3 mai 1956';

const keyOf = (a: RegionAsk) =>
	JSON.stringify([
		a.provider,
		a.model,
		a.preset_id,
		a.source_id,
		a.page,
		a.half,
		a.x,
		a.y,
		a.width,
		a.height
	]);

// `?reading-slow`: the model takes long enough to be stopped.
const model = new PretendModel<RegionAsk, RegionReading>({
	key: keyOf,
	cost: COST,
	delay: 500,
	slow: 'reading-slow'
});

const shownOf = (find: FindPiece, setupId: string, sourceId: string, index: number) => {
	const piece = find(setupId, sourceId, index) ?? fail('job.nothing_read');
	return { piece, text: piece.corrected_text ?? piece.ai_text ?? '' };
};

/** The region the reading would write into, among the preview's drawn regions. */
const drawnOf = (a: RegionAsk) =>
	drawnOn(a.job_id, a.source_id, a.page, a.half).find((r) => r.id === a.region_id) ??
	fail('region.not_found');

function check(a: RegionAsk, find: FindPiece): void {
	const { x, y, width: w, height: h } = a;
	if (![x, y, w, h].every(Number.isFinite)) fail('region.not_finite');
	if (a.page < 1) fail('region.no_page');
	if (x < -SLACK || y < -SLACK || x + w > 1 + SLACK || y + h > 1 + SLACK) fail('region.outside');
	if (w < MIN_SIZE || h < MIN_SIZE) fail('region.too_small');
	if (a.target.kind === 'region') drawnOf(a);
	if (a.target.kind === 'text') {
		const { text } = shownOf(find, a.target.setup_id, a.source_id, a.target.index);
		if (!locateWords(text, a.target)) fail('region_reading.changed');
	}
}

function readingOf(a: RegionAsk): RegionReading {
	return {
		ask: a,
		status: 'asked',
		prompt: {
			preset_id: a.preset_id,
			preset_name: 'Printed document',
			template: '',
			text: '',
			user_prompt: ''
		},
		request: [],
		cache_key: keyOf(a),
		answer: null,
		cost_usd: null,
		cached: false,
		error: null,
		accepted: null,
		asked_at: Date.now(),
		answered_at: null,
		decided_at: null
	};
}

/** What the pretend model reads: the words shown, mended, or a marginal note. */
function readOf(a: RegionAsk): string {
	return a.target.kind === 'text' ? mend(a.target.was) : NOTE;
}

export function regionReadingCommand(
	cmd: string,
	a: Record<string, unknown>,
	find: FindPiece,
	local: (provider: string) => boolean
): unknown {
	switch (cmd) {
		case 'region_reading_estimate': {
			const ask = a.ask as RegionAsk;
			check(ask, find);
			return model.estimate(ask, local);
		}
		case 'ask_region_reading': {
			const ask = a.ask as RegionAsk;
			try {
				check(ask, find);
			} catch (e) {
				return Promise.reject(e);
			}
			return model.ask(
				ask,
				readingOf(ask),
				() => readOf(ask),
				(status, _answer, stored) => ({
					cost_usd: status === 'answered' && !stored && !local(ask.provider) ? COST : null
				})
			);
		}
		case 'cancel_region_reading':
			return model.cancel(String(a.id));
		case 'accept_region_reading': {
			const r = model.asked.get(String(a.id));
			if (!r || r.status !== 'answered') return fail('region_reading.not_waiting');
			const ask = r.ask;
			const text = String(a.text);
			let before: ReadingEffect;
			if (ask.target.kind === 'nowhere') return fail('region_reading.nowhere');
			if (ask.target.kind === 'region') {
				const region = drawnOf(ask);
				before = { kind: 'region', value: { ...region } };
				Object.assign(region, { text, updated_at: Date.now() });
			} else {
				const { setup_id: setupId, index } = ask.target;
				const { piece, text: shown } = shownOf(find, setupId, ask.source_id, index);
				const written = withReading(shown, ask.target, text) ?? fail('region_reading.changed');
				const correction: Correction = {
					job_id: ask.job_id,
					setup_id: setupId,
					source_id: ask.source_id,
					index,
					text: piece.corrected_text,
					reviewed: piece.reviewed,
					updated_at: 0
				};
				before = { kind: 'correction', value: correction };
				piece.corrected_text = written;
			}
			Object.assign(r, { status: 'accepted', accepted: text, decided_at: Date.now() });
			return before;
		}
		case 'discard_region_reading':
			return model.discard(String(a.id));
		case 'undo_region_reading': {
			const r = model.asked.get(String(a.id));
			const before = a.before as ReadingEffect;
			if (!r || r.status !== 'accepted') return fail('region_reading.not_waiting');
			if (before.kind === 'region') {
				const region = drawnOn(
					before.value.job_id,
					before.value.source_id,
					before.value.page,
					before.value.half
				).find((d) => d.id === before.value.id);
				if (region) Object.assign(region, before.value);
			} else {
				const c = before.value;
				const piece = find(c.setup_id, c.source_id, c.index);
				if (piece) {
					piece.corrected_text = c.text;
					piece.reviewed = c.reviewed;
				}
			}
			r.status = 'undone';
			return null;
		}
		default:
			return undefined;
	}
}
