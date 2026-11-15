// The readings open over the review's text: where the other set-ups, or the model under a
// correction, read differently, pointing at the words shows what each reads, and clicking them
// or stepping to them pins those readings until they are closed. The words of a difference are
// highlighted in both texts, the text shown and the one opened beside it, and the focus goes
// back to them when their readings close.

import { tick } from 'svelte';
import type { Entry } from './apparatus.js';
import { level, reveal } from './caret.js';

/** The words of a difference in a pane. */
export const wordsIn = (pane: HTMLElement | null, id: number) =>
	pane?.querySelector<HTMLElement>(`[data-entry="${id}"]`) ?? null;

export interface ReadingsPlaces {
	/** The differences of the text shown, in order. */
	entries: () => Entry[];
	/** The pane of the text shown. */
	textPane: () => HTMLElement | null;
	/** The pane of the text opened beside it, when open. */
	comparePane: () => HTMLElement | null;
}

/** The difference whose readings are open, the one last reached, and the one pointed at. */
export class Readings {
	/** The difference whose readings are open over the text. Pinned once clicked or stepped to, it
	 * stays until closed; only pointed at, it shows while the pointer is on it or its readings. */
	selected = $state<number | null>(null);
	pinned = $state(false);
	/** The difference last opened by a click or a step, or reached with the arrow keys, kept
	 * once its readings close, so the next step goes on from it rather than from the first. */
	stepped = $state<number | null>(null);
	/** The difference under the pointer, in either text: highlighted in both. */
	hovered = $state<number | null>(null);
	/** The text the pinned readings were opened from, where the focus goes back when they close. */
	#opener: HTMLElement | null = null;
	#hoverTimer: ReturnType<typeof setTimeout> | undefined;
	readonly #places: ReadingsPlaces;
	/** The difference whose readings are open, as an entry. */
	readonly open = $derived.by(
		() => this.#places.entries().find((e) => e.id === this.selected) ?? null
	);
	/** The difference that holds the texts' one tab stop: the one open, else the last reached. */
	readonly stop = $derived.by(() => (this.pinned ? this.selected : null) ?? this.stepped);

	constructor(places: ReadingsPlaces) {
		this.#places = places;
	}

	/** Shows a difference's readings over the text. */
	show(id: number, pin: boolean): void {
		clearTimeout(this.#hoverTimer);
		this.selected = id;
		this.pinned = pin;
		if (pin) this.stepped = id;
	}

	hide = (): void => {
		clearTimeout(this.#hoverTimer);
		this.selected = null;
		this.pinned = false;
	};

	/** Closes the readings and forgets the difference last reached, as on turning the page. */
	reset(): void {
		this.hide();
		this.stepped = null;
	}

	/** Keeps readings shown while the pointer is on them. */
	hold = (): void => {
		clearTimeout(this.#hoverTimer);
	};

	/** Pointing at highlighted words shows their readings after a moment, unless some are pinned. */
	pointAt = (e: PointerEvent, id: number): void => {
		this.hovered = id;
		if (e.pointerType !== 'mouse' || this.pinned) return;
		clearTimeout(this.#hoverTimer);
		this.#hoverTimer = setTimeout(() => this.show(id, false), this.selected === null ? 300 : 60);
	};

	/** Leaving the words or their readings closes readings that are not pinned. */
	pointAway = (): void => {
		this.hovered = null;
		if (this.pinned) return;
		clearTimeout(this.#hoverTimer);
		this.#hoverTimer = setTimeout(() => {
			if (!this.pinned) this.hide();
		}, 250);
	};

	/** Opens a difference and pins it, bringing its words into view in both texts, whichever
	 * text it was chosen in. Chosen from the keyboard, the focus goes into its readings. */
	async choose(id: number, from?: HTMLElement | null, keyboard = false): Promise<void> {
		this.show(id, true);
		this.#opener = from ?? this.#places.textPane();
		await tick();
		const textPane = this.#places.textPane();
		const comparePane = this.#places.comparePane();
		const words = wordsIn(textPane, id);
		const other = wordsIn(comparePane, id);
		const clicked = from ? wordsIn(from, id) : null;
		if (from && clicked) {
			// Chosen in one text: the other comes level with it, so both read the same place.
			const height = clicked.getBoundingClientRect().top - from.getBoundingClientRect().top;
			if (from === textPane) level(comparePane, other, height);
			else level(textPane, words, height);
		} else {
			reveal(textPane, words);
			reveal(comparePane, other);
		}
		words?.focus({ preventScroll: true });
		if (keyboard) await this.#focusReadings();
	}

	/** Takes the focus to the first reading to choose, once the readings are placed. */
	async #focusReadings(): Promise<void> {
		await tick();
		await new Promise((done) => requestAnimationFrame(done));
		const box = this.#places.textPane()?.querySelector<HTMLElement>('.readings');
		(box?.querySelector<HTMLElement>('.pick') ?? box?.querySelector<HTMLElement>('input'))?.focus({
			preventScroll: true
		});
	}

	/** Closes pinned readings, the focus back on the words they were opened from. */
	async close(): Promise<void> {
		const was = this.selected;
		const from = this.#opener;
		this.hide();
		if (was === null) return;
		await tick();
		(wordsIn(from, was) ?? wordsIn(this.#places.textPane(), was))?.focus({ preventScroll: true });
	}

	/**
	 * Closes the readings of difference `id` once one of them is used. When the focus was in
	 * them, it goes back to the words they were opened from; returns whether it was.
	 */
	leave(id: number): boolean {
		const textPane = this.#places.textPane();
		const focused = textPane?.querySelector('.readings')?.contains(document.activeElement);
		const from = this.#opener;
		this.hide();
		if (focused) (wordsIn(from, id) ?? wordsIn(textPane, id))?.focus({ preventScroll: true });
		return focused ?? false;
	}

	/** A click anywhere else closes pinned readings. */
	onPointerDown = (e: PointerEvent): void => {
		if (!this.pinned || (e.target as HTMLElement).closest('.readings, [data-entry], [data-step]'))
			return;
		this.hide();
	};

	/** Opens the next difference (`delta` 1) or the previous (-1), going round at either end. */
	step(delta: number, keyboard = false): void {
		const entries = this.#places.entries();
		if (entries.length === 0) return;
		const at = entries.findIndex((e) => e.id === (this.selected ?? this.stepped));
		const next = at < 0 ? (delta > 0 ? 0 : entries.length - 1) : at + delta;
		const target = entries[(next + entries.length) % entries.length];
		void this.choose(target.id, undefined, keyboard);
	}
}
