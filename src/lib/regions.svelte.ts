// The regions over the scan in a review: whether they are shown (remembered on this computer),
// the regions of the page shown, read from the app, the one chosen, and how the choice is
// followed in the text pane. In the text review a page has the engine's regions and the
// researcher's own (`load`); in the analysis, table-of-contents and records reviews only the
// researcher's (`loadDrawn`). Drawing and adjusting them is `RegionEdit`'s.

import { tick } from 'svelte';
import type { PageHalf } from './bindings/PageHalf.js';
import type { PageRegions } from './bindings/PageRegions.js';
import type { PieceRef } from './bindings/PieceRef.js';
import type { ShownRegion } from './bindings/ShownRegion.js';
import { placeInText, reveal } from './caret.js';
import { RegionEdit } from './regionEdit.svelte.js';
import { clearPlace, markPlace, regionAt, type Level } from './regions.js';
import { keep, recall } from './stored.js';
import { api } from './tauri.js';

const SHOWN_KEY = 'kalamu.review.regions';

/** Where a region is chosen from: the engine's at one granularity, or the researcher's own. */
export type List = Level | 'drawn';

/** A region chosen: its list and place in it; `from` says whether it was chosen on the page or
 * from the text, which brings it into view on the page. */
export interface Chosen {
	level: List;
	index: number;
	from: 'page' | 'text';
}

/** A page of a source, where the researcher's regions on it are kept: its number from 1, and
 * the half of a split spread. */
export interface Place {
	page: number;
	half: PageHalf | null;
}

/** The page a piece is, or null for a piece that is not a page (a whole document, a passage). */
export function placeOf(piece: PieceRef | null | undefined): Place | null {
	return piece?.kind === 'page' ? { page: piece.number, half: piece.half ?? null } : null;
}

export class Regions {
	readonly jobId: string;
	readonly sourceId: string;
	/** Whether the regions are drawn over the scan. */
	shown = $state(recall(SHOWN_KEY, (v) => (typeof v === 'boolean' ? v : null), false));
	/** The page's regions: null while they load, or for a source that has no pages. */
	page = $state<PageRegions | null>(null);
	chosen = $state<Chosen | null>(null);
	/** The granularity drawn now, which suits the zoom: set by the layer that draws them. */
	level = $state<Level>('blocks');
	/** Where the researcher's regions on the page shown are kept; null where none can be drawn. */
	place = $state<Place | null>(null);
	/** In the text review, the set-up and piece the engine's regions are read from. */
	reading = $state<{ setupId: string; index: number } | null>(null);
	/** Whether the page can hold the engine's regions: the text review's. */
	engine = $state(false);
	/** Asked from a region's card to read that region again; not offered when not given. */
	onread = $state<((region: ShownRegion) => void) | undefined>(undefined);
	/** Set to give the chosen region the focus once it is drawn again (after a save, a removal
	 * put back); the layer that draws it does, and clears it. */
	focusing = $state(false);
	/** Drawing and adjusting a region. */
	readonly edit: RegionEdit;
	/** The region chosen, in its own list. */
	readonly region: ShownRegion | null = $derived(
		this.chosen && this.page ? (this.page[this.chosen.level][this.chosen.index] ?? null) : null
	);
	/** Whether the engine placed any region on the page. */
	readonly any = $derived(!!this.page && this.page.blocks.length > 0);
	/** Whether the page has any region at all, the engine's or the researcher's. */
	readonly anyAtAll = $derived(this.any || (this.page?.drawn.length ?? 0) > 0);
	/** Whether the researcher can draw on the page shown. */
	readonly drawable = $derived(this.page !== null && this.place !== null);
	#asked = '';
	#fetch: (() => Promise<PageRegions>) | null = null;

	constructor(jobId: string, sourceId: string) {
		this.jobId = jobId;
		this.sourceId = sourceId;
		this.edit = new RegionEdit(this);
	}

	toggle() {
		this.shown = !this.shown;
		keep(SHOWN_KEY, this.shown);
		if (!this.shown) {
			this.chosen = null;
			this.edit.reset();
		}
	}

	/** Reads the regions of piece `index` under a set-up, placed in `text`, the text shown; only
	 * the last asked for is kept. A piece of a recording or a text has none to ask for. `piece`
	 * says which page it is, where the researcher's own regions are drawn. */
	load(
		setupId: string | undefined,
		index: number,
		text: string,
		paged: boolean,
		piece?: PieceRef | null
	) {
		const place = paged ? placeOf(piece) : null;
		const prefix = `${setupId}\u0000${index}\u0000`;
		this.engine = true;
		this.reading = setupId && paged ? { setupId, index } : null;
		this.#ask(
			`${prefix}${text}\u0000${place?.page}`,
			prefix,
			place,
			setupId && paged ? () => api.pageRegions(this.jobId, this.sourceId, setupId, index) : null
		);
	}

	/** Reads the researcher's own regions on a page with no engine's regions (an analysis's, a
	 * table of contents', a page of records); none where nothing can be drawn. */
	loadDrawn(place: Place | null) {
		const key = place ? `drawn\u0000${place.page}\u0000${place.half}` : 'drawn\u0000';
		this.engine = false;
		this.reading = null;
		this.#ask(
			key,
			key,
			place,
			place ? () => api.drawnRegions(this.jobId, this.sourceId, place.page, place.half) : null
		);
	}

	/** Asks for a page's regions; a new page lets the choice and any drawing go. */
	#ask(
		asked: string,
		samePrefix: string,
		place: Place | null,
		fetch: (() => Promise<PageRegions>) | null
	) {
		if (asked === this.#asked) return;
		const samePiece = this.#asked.startsWith(samePrefix);
		this.#asked = asked;
		this.place = place;
		this.#fetch = fetch;
		if (!samePiece) {
			this.page = null;
			this.chosen = null;
			this.edit.reset();
		}
		if (fetch) void this.#read(asked, fetch);
	}

	async #read(asked: string, fetch: () => Promise<PageRegions>) {
		try {
			const page = await fetch();
			if (this.#asked !== asked) return;
			this.page = page;
			// The text changed (a correction saved): the region stays chosen, its place read again.
			if (this.chosen && !page[this.chosen.level][this.chosen.index]) this.chosen = null;
		} catch {
			if (this.#asked === asked) this.page = null;
		}
	}

	/** Reads the page's regions again, after one was saved, removed or put back. */
	async reload(): Promise<void> {
		if (this.#fetch) await this.#read(this.#asked, this.#fetch);
	}

	choose(level: List, index: number, from: Chosen['from'] = 'page') {
		this.chosen = { level, index, from };
	}

	/** Chooses the region with this id, at the granularity drawn first; false when the page has
	 * none. */
	chooseId(id: string): boolean {
		const page = this.page;
		if (!page) return false;
		const lists: List[] = [this.level, 'drawn', 'blocks', 'lines', 'words'];
		for (const level of lists) {
			const index = page[level].findIndex((r) => r.id === id);
			if (index >= 0) {
				this.choose(level, index);
				return true;
			}
		}
		return false;
	}

	/** Chooses the region drawn whose place in the text holds `offset`, where the app placed
	 * one there; otherwise leaves the choice as it was. */
	chooseAt(offset: number) {
		if (!this.shown || !this.page || this.edit.mode !== 'idle') return;
		const at = regionAt(this.page[this.level], offset);
		if (at >= 0) this.choose(this.level, at, 'text');
	}
}

/** The researcher's own regions on the scan page a review shows, from a component's set-up:
 * read again whenever `place` gives another page; none where it gives none (a text, a
 * recording). Offered to the review's frame while `regions.place` is set. */
export function ownRegions(jobId: string, sourceId: string, place: () => Place | null): Regions {
	const regions = new Regions(jobId, sourceId);
	$effect(() => regions.loadDrawn(place()));
	return regions;
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
