// The texts a review compares with the text shown, and the one opened beside it: each other
// set-up's reading, the model's own text under a correction, and for a correction the text it
// corrected. Each is named by a siglum, as in an edition's apparatus. A translation's source is
// in another language, so it is shown beside the translation, never compared with it.

import { siglum } from './apparatus.js';
import type { PieceView } from './bindings/PieceView.js';
import type { Setup } from './bindings/Setup.js';
import type { Task } from './bindings/Task.js';
import type { Messages } from './i18n/index.js';
import { layout } from './review.svelte.js';

/** A text compared with the text shown. */
export interface Witness {
	/** `model`, `source`, or the set-up's id. */
	key: string;
	siglum: string;
	text: string;
}

/** A piece's text as the review shows it: the correction, else the model's. */
export function shownText(v: PieceView | undefined): string {
	return v?.corrected_text ?? v?.ai_text ?? '';
}

/**
 * The witnesses of a piece: the model itself when the first set-up's text was corrected, the
 * text a correction corrected, and each other set-up that answered. `viewOf` gives a set-up's
 * piece.
 */
export function witnessesOf(
	setups: Setup[],
	viewOf: (setup: Setup) => PieceView | undefined,
	task: Task | undefined,
	m: Messages
): Witness[] {
	const lead = setups[0] ? viewOf(setups[0]) : undefined;
	const out: Witness[] = [];
	if (lead?.corrected_text != null && lead.ai_text != null) {
		out.push({ key: 'model', siglum: siglum(0), text: lead.ai_text });
	}
	// A correction is compared with the text it corrected, so every change shows; a translation
	// is not, being in another language.
	if (task === 'text_correction' && lead?.source_text != null) {
		out.push({ key: 'source', siglum: m.review.originalSiglum, text: lead.source_text });
	}
	setups.slice(1).forEach((s, i) => {
		const v = viewOf(s);
		if (v?.ai_text != null) out.push({ key: s.id, siglum: siglum(i + 1), text: shownText(v) });
	});
	return out;
}

/** Who reads what a siglum names: the first set-up's model under a correction, the text
 * corrected, or another set-up. */
export function witnessName(s: string, setups: Setup[], m: Messages): string {
	if (s === siglum(0)) return m.review.modelText(setups[0]?.label ?? '');
	if (s === m.review.originalSiglum) return m.review.original;
	return setups[s.charCodeAt(0) - 97]?.label ?? s;
}

/** The witness opened beside the text: the one chosen, else the first other set-up, else the
 * first. */
export function comparedOf(witnesses: Witness[], key: string | null): Witness | undefined {
	return (
		witnesses.find((w) => w.key === key) ?? witnesses.find((w) => w.key !== 'model') ?? witnesses[0]
	);
}

/** The witness compared in the third pane, chosen by the researcher or opened on a match. */
export class Comparison {
	/** The witness chosen, by its key. */
	key = $state<string | null>(null);
	readonly #witnesses: () => Witness[];
	readonly compared = $derived.by(() => comparedOf(this.#witnesses(), this.key));
	/** Whether the third pane is open: asked for, with something to compare. */
	readonly open = $derived.by(() => layout.compare && this.#witnesses().length > 0);

	constructor(witnesses: () => Witness[]) {
		this.#witnesses = witnesses;
	}

	/** Opens the third pane on set-up `key`'s text. */
	show(key: string): void {
		this.key = key;
		if (!layout.compare) layout.toggleCompare();
	}
}
