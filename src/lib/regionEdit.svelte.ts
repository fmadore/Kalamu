// Drawing and adjusting the researcher's regions over a scan (#84): the box drawn with the
// pointer is kept at once; a box placed from the keyboard, or a region's box being moved or
// resized, is kept with Done (Enter) and put back as it was with Cancel (Escape). An engine's
// region keeps its own box: the researcher's is saved beside it (`adjusts`), and putting the
// engine's back removes theirs. Removing and putting back can be undone from the message that
// follows. Every mode ends by itself after one action, and the focus always lands somewhere.

import type { DrawnRegion } from './bindings/DrawnRegion.js';
import type { RegionDraft } from './bindings/RegionDraft.js';
import type { ShownRegion } from './bindings/ShownRegion.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import type { Rotation } from './pdf.js';
import { dragBox, keyStep, middleOf, sameBox, stepFor, type Grip } from './regionBox.js';
import { turnBox, unturnBox, type Box } from './regions.js';
import type { Regions } from './regions.svelte.js';
import { toast } from './state.svelte.js';
import { api } from './tauri.js';

/** What the box being adjusted belongs to: a new region, one of the researcher's own, or an
 * engine's region (by its id in the result `result`). */
export type Target =
	| { kind: 'new' }
	| { kind: 'own'; id: string; text: string }
	| { kind: 'engine'; id: string; result: string };

/** The part of the page in view and the reader's quarter turns, as the layer measures them. */
export interface Seen {
	view: Box | null;
	rotation: Rotation;
}

/** A box to save and what goes with it. */
interface Saving {
	id: string | null;
	box: Box;
	text: string;
	adjusts: RegionDraft['adjusts'];
}

const boxOf = (r: Box): Box => ({ x: r.x, y: r.y, width: r.width, height: r.height });

export class RegionEdit {
	readonly #regions: Regions;
	/** Nothing; the next drag on the page draws a box; or a box is being moved or resized. */
	mode = $state<'idle' | 'drawing' | 'adjusting'>('idle');
	/** The box being adjusted, on the page as read. */
	box = $state<Box | null>(null);
	target = $state<Target | null>(null);
	/** What was last said to a screen reader as the box moved or was kept. */
	said = $state('');
	/** The box as it was when adjusting began, to put back. */
	#from: Box | null = null;
	/** What the layer over the page measures, for a box placed in the middle of the view. */
	seen: (() => Seen) | null = null;

	constructor(regions: Regions) {
		this.#regions = regions;
	}

	/** Leaves any mode, keeping nothing. */
	reset() {
		this.mode = 'idle';
		this.box = null;
		this.target = null;
		this.#from = null;
	}

	/** *Draw a region* pressed: the next drag on the page draws a box. Pressed again, a box is
	 * placed in the middle of what is in view, to move and resize from the keyboard. */
	startDrawing() {
		if (this.mode === 'drawing') {
			const seen = this.seen?.();
			const box = seen ? unturnBox(middleOf(seen.view), seen.rotation) : middleOf(null);
			this.#begin(box, { kind: 'new' });
			return;
		}
		this.reset();
		this.#regions.chosen = null;
		this.mode = 'drawing';
		this.#say(messages().review.regionEdit.drawing);
	}

	/** Leaves drawing or adjusting, the box as it was; the focus goes back to the region, or to
	 * *Draw a region* for a box not yet kept. */
	cancel() {
		const target = this.target;
		const was = this.mode;
		this.reset();
		if (was !== 'adjusting') return;
		if (target?.kind === 'new' || !this.#regions.chosen) focusDraw();
		else this.#regions.focusing = true;
	}

	/** Starts moving and resizing the region chosen. */
	adjustChosen() {
		const regions = this.#regions;
		const region = regions.region;
		const chosen = regions.chosen;
		if (!region || !chosen) return;
		if (chosen.level === 'drawn') {
			this.#begin(boxOf(region), { kind: 'own', id: region.id, text: region.text });
		} else if (regions.page?.result) {
			this.#begin(boxOf(region), { kind: 'engine', id: region.id, result: regions.page.result });
		}
	}

	#begin(box: Box, target: Target) {
		this.mode = 'adjusting';
		this.box = box;
		this.#from = box;
		this.target = target;
	}

	/** The box adjusted with the pointer, given on the page as read. */
	setBox(box: Box) {
		if (this.mode === 'adjusting') this.box = box;
	}

	/** The box held by `grip` and moved `dx`, `dy` on the page as drawn (turned by `rotation`). */
	drag(from: Box, grip: Grip, dx: number, dy: number, rotation: Rotation) {
		this.setBox(unturnBox(dragBox(turnBox(from, rotation), grip, dx, dy), rotation));
	}

	/** An arrow key, with Shift or not, on the box being adjusted: moved or resized as the page
	 * is seen, by a step that suits `zoom`, and said in a word. False for another key. */
	nudge(key: string, shift: boolean, zoom: number, rotation: Rotation): boolean {
		const box = this.box;
		const step = keyStep(key, shift, stepFor(zoom));
		if (!box || !step) return false;
		this.drag(box, step.grip, step.dx, step.dy, rotation);
		this.#say(messages().review.regionEdit.said[step.said]);
		return true;
	}

	/** A box drawn with the pointer, on the page as read: kept at once, and chosen. */
	async drawn(box: Box) {
		this.reset();
		await this.#save({ id: null, box, text: '', adjusts: null }, null);
	}

	/** Keeps the box adjusted (Done, Enter); one left as it was is simply let go. */
	async done() {
		const [box, target, from] = [this.box, this.target, this.#from];
		if (this.mode !== 'adjusting' || !box || !target) return;
		if (target.kind !== 'new' && from && sameBox(box, from)) {
			this.reset();
			this.#regions.focusing = true;
			return;
		}
		const kept = await this.#save(
			target.kind === 'own'
				? { id: target.id, box, text: target.text, adjusts: null }
				: target.kind === 'engine'
					? { id: null, box, text: '', adjusts: { result: target.result, region: target.id } }
					: { id: null, box, text: '', adjusts: null },
			target.kind === 'engine' ? target.id : null
		);
		if (kept) this.reset();
	}

	/** Saves what the researcher wrote for one of their regions, when it changed. */
	async saveText(region: ShownRegion, text: string) {
		if (text === region.text) return;
		await this.#save({ id: region.id, box: boxOf(region), text, adjusts: null }, null, false);
	}

	/** Removes one of the researcher's regions; Undo, in the message that follows, brings it back. */
	async remove(region: ShownRegion) {
		const m = messages();
		try {
			const removed = await api.removeRegion(region.id);
			this.#regions.chosen = null;
			await this.#regions.reload();
			focusDraw();
			toast(m.review.regionEdit.removed, {
				action: { label: m.common.undo, run: () => void this.#restore(removed, removed.id) }
			});
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
	}

	/** Puts an engine's region's own box back: the researcher's box for it, which the page names
	 * (`adjusted_by`), is removed, and Undo brings it back. */
	async putBack(region: ShownRegion) {
		const regions = this.#regions;
		const ours = region.adjusted_by;
		if (!ours) return;
		const m = messages();
		try {
			const removed = await api.removeRegion(ours);
			await regions.reload();
			regions.chooseId(region.id);
			regions.focusing = true;
			toast(m.review.regionEdit.putBackDone, {
				action: { label: m.common.undo, run: () => void this.#restore(removed, region.id) }
			});
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
	}

	/** Brings back a region removed, chosen and given the focus. */
	async #restore(region: DrawnRegion, chosen: string) {
		try {
			await api.restoreRegion(region);
			await this.#regions.reload();
			if (this.#regions.chooseId(chosen)) this.#regions.focusing = true;
		} catch (e) {
			toast(describeError(e, messages()), { kind: 'error' });
		}
	}

	/** Saves a box, reads the page again and chooses the region kept (by `chosen`, an engine's
	 * region's id, or else the id it was kept under), giving it the focus when `focus`. Null when
	 * it failed, the reason shown. */
	async #save(draft: Saving, chosen: string | null, focus = true): Promise<DrawnRegion | null> {
		const regions = this.#regions;
		const place = regions.place;
		const m = messages();
		if (!place) return null;
		try {
			const kept = await api.saveRegion(draftOf(regions, place, draft));
			await regions.reload();
			if (regions.chooseId(chosen ?? kept.id) && focus) regions.focusing = true;
			this.#say(m.review.regionEdit.saved);
			return kept;
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
			return null;
		}
	}

	/** Says something to a screen reader: the same words twice are said twice. */
	#say(words: string) {
		this.said = this.said === words ? `${words} ` : words;
	}
}

/** A draft for the store: a box on the page `place` of the review's source. */
function draftOf(
	regions: Regions,
	place: { page: number; half: RegionDraft['half'] },
	d: Saving
): RegionDraft {
	return {
		id: d.id,
		job_id: regions.jobId,
		source_id: regions.sourceId,
		page: place.page,
		half: place.half,
		x: d.box.x,
		y: d.box.y,
		width: d.box.width,
		height: d.box.height,
		text: d.text,
		adjusts: d.adjusts
	};
}

/** Gives the focus to *Draw a region*, or to the switch beside it where it is gone. */
export function focusDraw() {
	if (typeof document === 'undefined') return;
	const to =
		document.querySelector<HTMLElement>('[data-region-draw]') ??
		document.querySelector<HTMLElement>('[data-region-toggle]');
	to?.focus();
}
