// The text review's regions over the scan: whether they are shown (remembered on this
// computer), the regions of the page shown, read from the app, the one chosen, and how the
// choice is followed in the text pane.

import { tick } from 'svelte';
import type { PageRegions } from './bindings/PageRegions.js';
import type { ShownRegion } from './bindings/ShownRegion.js';
import { placeInText, reveal } from './caret.js';
import { clearPlace, markPlace, regionAt, type Level } from './regions.js';
import { keep, recall } from './stored.js';
import { api } from './tauri.js';

const SHOWN_KEY = 'kalamu.review.regions';

/** A region chosen: its granularity and place in that list; `from` says whether it was chosen
 * on the page or from the text, which brings it into view on the page. */
export interface Chosen {
	level: Level;
	index: number;
	from: 'page' | 'text';
}

export class Regions {
	readonly #jobId: string;
	readonly #sourceId: string;
	/** Whether the regions are drawn over the scan. */
	shown = $state(recall(SHOWN_KEY, (v) => (typeof v === 'boolean' ? v : null), false));
	/** The page's regions: null while they load, or for a source that has no pages. */
	page = $state<PageRegions | null>(null);
	chosen = $state<Chosen | null>(null);
	/** The granularity drawn now, which suits the zoom: set by the layer that draws them. */
	level = $state<Level>('blocks');
	/** The region chosen, at its own granularity. */
	readonly region: ShownRegion | null = $derived(
		this.chosen && this.page ? (this.page[this.chosen.level][this.chosen.index] ?? null) : null
	);
	/** Whether the page has any region to draw. */
	readonly any = $derived(!!this.page && this.page.blocks.length > 0);
	#asked = '';

	constructor(jobId: string, sourceId: string) {
		this.#jobId = jobId;
		this.#sourceId = sourceId;
	}

	toggle() {
		this.shown = !this.shown;
		keep(SHOWN_KEY, this.shown);
		if (!this.shown) this.chosen = null;
	}

	/** Reads the regions of piece `index` under a set-up, placed in `text`, the text shown; only
	 * the last asked for is kept. A piece of a recording or a text has none to ask for. */
	load(setupId: string | undefined, index: number, text: string, paged: boolean) {
		const asked = `${setupId}\u0000${index}\u0000${text}`;
		if (asked === this.#asked) return;
		const samePiece = this.#asked.startsWith(`${setupId}\u0000${index}\u0000`);
		this.#asked = asked;
		if (!samePiece) {
			this.page = null;
			this.chosen = null;
		}
		if (!setupId || !paged) return;
		api
			.pageRegions(this.#jobId, this.#sourceId, setupId, index)
			.then((page) => {
				if (this.#asked !== asked) return;
				this.page = page;
				// The text changed (a correction saved): the region stays chosen, its place read again.
				if (this.chosen && !page[this.chosen.level][this.chosen.index]) this.chosen = null;
			})
			.catch(() => {
				if (this.#asked === asked) this.page = null;
			});
	}

	choose(level: Level, index: number, from: Chosen['from'] = 'page') {
		this.chosen = { level, index, from };
	}

	/** Chooses the region drawn whose place in the text holds `offset`, where the app placed
	 * one there; otherwise leaves the choice as it was. */
	chooseAt(offset: number) {
		if (!this.shown || !this.page) return;
		const at = regionAt(this.page[this.level], offset);
		if (at >= 0) this.choose(this.level, at, 'text');
	}
}

/** What the text pane gives the regions: the pane, the text it shows, whether it is being
 * corrected, and what redraws its text (the mark is laid again after). */
export interface TextSide {
	pane: () => HTMLElement | null;
	text: () => string;
	editing: () => boolean;
	redrawn: () => unknown;
}

/**
 * Ties the regions to the text pane, from a component's set-up: the chosen region's place is
 * marked in the text where the app placed it, and brought into view when it was chosen on the
 * page; a click in the text chooses the region drawn whose place holds it, where there is one.
 * Nothing is marked while the text is being corrected.
 */
export function followRegions(regions: Regions, side: TextSide): void {
	let revealed: Chosen | null = null;
	$effect(() => {
		const chosen = regions.chosen;
		const span = side.editing() || !regions.shown ? null : (regions.region?.span ?? null);
		const pane = side.pane();
		void side.redrawn();
		if (!pane) return;
		let gone = false;
		void tick().then(() => {
			if (gone) return;
			const at = markPlace(pane, '.t', span);
			if (at && chosen?.from === 'page' && chosen !== revealed) reveal(pane, at);
			revealed = chosen;
		});
		return () => {
			gone = true;
			clearPlace();
		};
	});
	$effect(() => {
		const pane = side.pane();
		if (!pane || !regions.shown) return;
		const up = (e: PointerEvent) => {
			if (side.editing() || (e.target as Element).closest('button, textarea')) return;
			const chosen = getSelection();
			if (!chosen || chosen.rangeCount === 0 || !pane.contains(chosen.anchorNode)) return;
			const place = placeInText(pane, side.text());
			if (place) regions.chooseAt(place.start);
		};
		pane.addEventListener('pointerup', up);
		return () => pane.removeEventListener('pointerup', up);
	});
}
