// The player's place followed in a recording's transcript, from the text review's set-up: the
// passage the player is at is marked in the text, drawn again with the text, and kept in view
// while it plays, unless the researcher has just scrolled the text away (`heard.ts` finds and
// marks it).

import { tick } from 'svelte';
import { reveal } from './caret.js';
import { clearHeard, markHeard, placeLines } from './heard.js';
import type { TextSide } from './regions.svelte.js';

/** How long after the researcher scrolled the text the player's place is followed again. */
const PAUSE_MS = 4_000;

/** What the review gives the player's place: the text pane, besides whether a recording is
 * shown, the player's turns, and the turn it is at (from 0, -1 for none) and whether it plays. */
export interface Listening extends TextSide {
	recording: () => boolean;
	turns: () => string[];
	heard: () => { line: number; playing: boolean };
}

export function followHeard(side: Listening): void {
	/** Where each of the player's turns stands in the text shown, corrections included. */
	const placed = $derived(side.recording() ? placeLines(side.text(), side.turns()) : []);
	/** When the researcher last scrolled the text. */
	let scrolledAt = 0;
	$effect(() => {
		const pane = side.pane();
		if (!pane) return;
		const scrolled = () => (scrolledAt = Date.now());
		const events = ['wheel', 'touchmove', 'keydown'] as const;
		for (const name of events) pane.addEventListener(name, scrolled, { passive: true });
		return () => events.forEach((name) => pane.removeEventListener(name, scrolled));
	});
	$effect(() => {
		const heard = side.heard();
		const span = side.editing() ? null : (placed[heard.line] ?? null);
		const playing = heard.playing;
		const pane = side.pane();
		void side.redrawn();
		if (!pane || !side.recording()) return;
		let gone = false;
		void tick().then(() => {
			if (gone) return;
			const at = markHeard(pane, '.t', span);
			if (at && playing && Date.now() - scrolledAt > PAUSE_MS) reveal(pane, at);
		});
		return () => {
			gone = true;
			clearHeard();
		};
	});
}
