// The names of an analysis's list answers, changed from its review: a name's menu, opened from
// a click, Enter, a right click or the menu key, renames or removes it in the corrected answers
// of every set-up that found it; what the models missed is added at the end of a list. Each
// change can be taken back from its toast, and the focus, lost with a name or a toast's button,
// goes where the researcher can carry on. The names of an answer are one tab stop, ← and →
// going from one to the next (`ListAnswer` draws them).

import { tick } from 'svelte';
import { collate, type Witness } from './analysis.js';
import type { Correction } from './bindings/Correction.js';
import type { Field } from './bindings/Field.js';
import { messages } from './i18n/index.js';
import { rovingTarget } from './keys.js';
import type { Review } from './review.svelte.js';
import { toast } from './state.svelte.js';

/** The name whose menu is open, and where. */
export interface ItemMenuAt {
	key: string;
	value: string;
	x: number;
	y: number;
	/** A free name is renamed or removed; a choice only removed. */
	renamable: boolean;
}

/** What the review gives the names: its answers, the pane they are shown in, whether they are
 * being corrected, and a way to stop pointing at a name in the document. */
export interface ListedSide {
	review: Review;
	witnesses: () => Witness[];
	pane: () => HTMLElement | null;
	editing: () => boolean;
	unhover: () => void;
}

export class ListedNames {
	readonly #side: ListedSide;
	readonly #m = messages();
	/** The name whose menu is open, and where. */
	menu = $state<ItemMenuAt | null>(null);
	/** The name of each answer that holds its tab stop, once one was reached. */
	stops = $state<Record<string, string>>({});

	constructor(side: ListedSide) {
		this.#side = side;
	}

	/** Opens a name's menu: at the pointer for a right click, under the name otherwise. */
	openMenu(e: MouseEvent, f: Field) {
		if ((f.kind.type !== 'list' && f.kind.type !== 'choices') || this.#side.editing()) return;
		const item = (e.target as HTMLElement).closest<HTMLElement>('[data-item]');
		if (!item?.dataset.item) return;
		e.preventDefault();
		const pointer = e.type === 'contextmenu' && (e.clientX !== 0 || e.clientY !== 0);
		const at = item.getBoundingClientRect();
		this.menu = {
			key: f.key,
			value: item.dataset.item,
			x: pointer ? e.clientX : at.left,
			y: pointer ? e.clientY : at.bottom + 4,
			renamable: f.kind.type === 'list'
		};
	}

	focus(key: string, value: string) {
		this.#side
			.pane()
			?.querySelector<HTMLElement>(
				`[data-field="${CSS.escape(key)}"] [data-item="${CSS.escape(value)}"]`
			)
			?.focus();
	}

	rove(e: KeyboardEvent, key: string, values: string[], at: string) {
		if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
		const to = rovingTarget(values, at, e.key);
		if (to === null) return;
		e.preventDefault();
		this.stops[key] = to;
		this.focus(key, to);
	}

	/**
	 * Renames a name, or removes it when `to` is null, in the answers of every set-up that found
	 * it, each kept as reviewed or not as it was: one command, all the set-ups or none.
	 */
	async change(key: string, from: string, to: string | null) {
		const m = this.#m;
		this.#side.unhover();
		const listed = collate(key, this.#side.witnesses()).map((r) => r.value);
		/** Each set-up's corrections as they stood, to put back. */
		const changed = await this.#side.review.renameListed(0, key, from, to);
		if (changed === null) return;
		// A name removed can be put back from the toast, as it was in each set-up's answers.
		if (to === null && changed.length > 0) {
			toast(m.review.itemRemoved(from), {
				action: { label: m.common.undo, run: () => void this.#putBack(key, from, changed) }
			});
		} else if (changed.length > 0) {
			toast(m.review.saved);
		}
		// The focus, lost with the name it was on, goes to the name renamed or to the next one.
		await tick();
		if (document.activeElement && document.activeElement !== document.body) return;
		const now = collate(key, this.#side.witnesses()).map((r) => r.value);
		const next =
			to !== null && now.includes(to)
				? to
				: now[Math.min(Math.max(listed.indexOf(from), 0), now.length - 1)];
		if (next === undefined) return;
		this.stops[key] = next;
		this.focus(key, next);
	}

	/** Puts each set-up's corrected answers and review back as they were, and the focus, lost
	 * with the toast's button, on the name put back. */
	async #putBack(key: string, name: string, changed: Correction[]) {
		if (!(await this.#side.review.putBack(changed))) return;
		await tick();
		if (document.activeElement && document.activeElement !== document.body) return;
		if (!collate(key, this.#side.witnesses()).some((r) => r.value === name)) return;
		this.stops[key] = name;
		this.focus(key, name);
	}

	/** Adds what the researcher typed or chose to a list answer, in every set-up that read the
	 * document, and says what was added, with a way to take it back. */
	async add(key: string, text: string) {
		const m = this.#m;
		this.#side.unhover();
		const result = await this.#side.review.addListed(0, key, text);
		if (result && result.added.length > 0) {
			const before = result.before;
			toast(m.review.itemsAdded(result.added), {
				action: { label: m.common.undo, run: () => void this.#takeBack(key, before) }
			});
		}
		return result;
	}

	/** Takes back what was added: each set-up's corrected answers as they were, and the focus,
	 * lost with the toast's button, on the field's Add. */
	async #takeBack(key: string, before: Correction[]) {
		if (!(await this.#side.review.putBack(before))) return;
		await tick();
		if (document.activeElement && document.activeElement !== document.body) return;
		this.#side
			.pane()
			?.querySelector<HTMLElement>(`[data-field="${CSS.escape(key)}"] [data-add]`)
			?.focus();
	}
}
