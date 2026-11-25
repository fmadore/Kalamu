<script lang="ts">
	// Over the top of the lightbox while the regions are shown. For an engine's region chosen: its
	// own text, as the model read it, and whether its place is marked in the text, or why not;
	// *Move or resize*, and once the researcher gave it a box, *Put back the model's box*. For one
	// of the researcher's own: what is written there, which they may type (saved when the field
	// is left, or with Ctrl Enter; ⌘ on a Mac), *Move or resize* and *Remove*. While a box is
	// drawn or adjusted, what to do and how to leave. On a page with no region, what to do
	// instead. *Read this region again…* is offered where the review gives a way to. When the
	// card would cover the region it speaks of (one drawn near the top of the page), it goes to
	// the foot of the scan. The researcher can drag it up or down out of the way, by its grip or
	// its empty parts (the grip's arrow keys move it too); a double click puts it back.
	import { tick } from 'svelte';
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { isMac, primary, shortcut } from '#lib/platform.js';
	import { noteCovers } from '#lib/regionBox.js';
	import { focusDraw } from '#lib/regionEdit.svelte.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions }: { regions: Regions } = $props();
	const m = messages();
	const e = $derived(m.review.regionEdit);
	const edit = $derived(regions.edit);

	const region = $derived(regions.region);
	const own = $derived(regions.own);
	const adjusted = $derived(regions.adjusted);
	const adjustable = $derived(regions.adjustable);
	const status = $derived(
		!region || own
			? ''
			: region.span
				? m.review.regionMarked
				: regions.page?.corrected
					? m.review.regionNotPlacedCorrected
					: m.review.regionNotPlaced
	);
	/** What to say on a page with no region to choose. */
	const fallback = $derived.by(() => {
		const page = regions.page;
		if (!page || region || regions.anyAtAll) return '';
		if (regions.engine) return regions.drawable ? e.noPositions : m.review.regionsFallback;
		return regions.drawable ? e.noneYet : '';
	});

	let note = $state<HTMLElement | null>(null);
	/** Whether the card sits at the foot of the scan, clear of the region chosen. */
	let atFoot = $state(false);
	/** Where the researcher dragged the card, from the top of the pane; kept while the review is
	 * open, so it stays out of the way as other regions are chosen. */
	let movedTo = $state<number | null>(null);
	/** What keeps a press for itself rather than moving the card. */
	const FIELDS = 'button, textarea, input, select, a, .said';
	/** A drag of the card under way: where the pointer and the card started. */
	let drag: { pointer: number; y: number; from: number; moving: boolean } | null = null;

	/** The card's top kept inside the pane, under its bar and above its foot. */
	function within(top: number): number {
		const pane = note?.closest<HTMLElement>('.pane');
		if (!note || !pane) return top;
		const bar = (pane.querySelector<HTMLElement>('.pane-bar')?.offsetHeight ?? 0) + 4;
		const lowest = Math.max(bar, pane.clientHeight - note.offsetHeight - 4);
		return Math.min(Math.max(top, bar), lowest);
	}

	/** A press on the grip, or on the card where nothing else takes it, starts moving the card. */
	function grab(ev: PointerEvent) {
		if (!note || ev.button !== 0) return;
		const target = ev.target as Element;
		const grip = target.closest<HTMLElement>('.grip');
		// While a box is drawn, only the grip takes the pointer; the rest reaches the page.
		if (!grip && (edit.mode !== 'idle' || target.closest(FIELDS))) return;
		drag = { pointer: ev.pointerId, y: ev.clientY, from: note.offsetTop, moving: !!grip };
		(grip ?? note).setPointerCapture(ev.pointerId);
		if (grip) ev.preventDefault();
	}

	function follow(ev: PointerEvent) {
		if (!drag || ev.pointerId !== drag.pointer) return;
		const by = ev.clientY - drag.y;
		// A press that barely moves stays a click (on the label, it focuses the field).
		if (!drag.moving && Math.abs(by) < 4) return;
		drag.moving = true;
		movedTo = within(drag.from + by);
	}

	function letGo(ev: PointerEvent) {
		if (!drag || ev.pointerId !== drag.pointer) return;
		drag = null;
		const held = ev.target as Element;
		if (held.hasPointerCapture(ev.pointerId)) held.releasePointerCapture(ev.pointerId);
	}

	/** A double click on the grip or the card's empty parts puts it back in its own place. */
	function putBackCard(ev: MouseEvent) {
		const target = ev.target as Element;
		if (!target.closest('.grip') && target.closest(FIELDS)) return;
		movedTo = null;
	}

	/** The grip's arrow keys move the card a step, or with Shift a long one. */
	function onGripKey(ev: KeyboardEvent) {
		if (!note || (ev.key !== 'ArrowUp' && ev.key !== 'ArrowDown')) return;
		ev.preventDefault();
		ev.stopPropagation();
		const step = (ev.shiftKey ? 80 : 16) * (ev.key === 'ArrowUp' ? -1 : 1);
		movedTo = within((movedTo ?? note.offsetTop) + step);
	}

	/** Puts the card under the pane's bar, or at the foot of the scan when it would cover the
	 * region chosen there, or the box being moved or resized, with what to do. */
	function place() {
		const pane = note?.closest<HTMLElement>('.pane');
		const box =
			pane?.querySelector<HTMLElement>('[data-adjusted-box]') ??
			pane?.querySelector<HTMLElement>('.layer .region.chosen');
		if (movedTo !== null) {
			movedTo = within(movedTo);
			return;
		}
		if (!note || !pane || !box) {
			atFoot = false;
			return;
		}
		const top =
			pane.getBoundingClientRect().top +
			(pane.querySelector<HTMLElement>('.pane-bar')?.offsetHeight ?? 0) +
			12;
		const b = box.getBoundingClientRect();
		atFoot = noteCovers(b, { top, bottom: top + note.offsetHeight });
	}

	// Placed again when another region is chosen, a box is drawn or moved, and as the scan is
	// zoomed, moved or the window resized.
	$effect(() => {
		void regions.chosen;
		void region;
		void edit.mode;
		void edit.box;
		const pane = note?.closest<HTMLElement>('.pane');
		if (!note || !pane) return;
		const again = () => requestAnimationFrame(place);
		again();
		const watch = new ResizeObserver(again);
		watch.observe(pane);
		watch.observe(note);
		pane.addEventListener('pointerup', again);
		pane.addEventListener('wheel', again, { passive: true });
		pane.addEventListener('keyup', again);
		pane.addEventListener('scroll', again, true);
		return () => {
			watch.disconnect();
			pane.removeEventListener('pointerup', again);
			pane.removeEventListener('wheel', again);
			pane.removeEventListener('keyup', again);
			pane.removeEventListener('scroll', again, true);
		};
	});

	/** Lets the choice go; the focus, lost with the card, goes to the regions' tab stop. */
	async function close() {
		regions.chosen = null;
		await tick();
		const stop = document.querySelector<HTMLElement>('.layer .region[tabindex="0"]');
		if (stop) stop.focus();
		else focusDraw();
	}

	function onTextKey(ev: KeyboardEvent & { currentTarget: HTMLTextAreaElement }) {
		if (!region || !primary(ev) || ev.key !== 'Enter') return;
		// Kept here: Ctrl Enter elsewhere in the review marks the page reviewed.
		ev.preventDefault();
		ev.stopPropagation();
		void edit.saveText(region, ev.currentTarget.value);
	}
</script>

{#if regions.shown && (edit.mode !== 'idle' || region || fallback)}
	<!-- Dragged by its grip, which the keyboard reaches; the rest of the card is only a larger
	     place to grab. -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="region-note"
		class:through={edit.mode !== 'idle'}
		class:foot={atFoot && movedTo === null}
		class:moved={movedTo !== null}
		style:top={movedTo !== null ? `${movedTo}px` : undefined}
		bind:this={note}
		onpointerdown={grab}
		onpointermove={follow}
		onpointerup={letGo}
		onpointercancel={letGo}
		ondblclick={putBackCard}
	>
		<button
			type="button"
			class="grip"
			aria-label={e.moveCard}
			title={e.moveCardHint}
			onkeydown={onGripKey}
		>
			<Icon name="grip" size={14} />
		</button>
		{#if edit.mode === 'drawing'}
			<div class="actions row">
				<p class="status">{e.drawing}</p>
				<button type="button" class="act" onclick={() => edit.cancel()}>{e.cancel}</button>
			</div>
		{:else if edit.mode === 'adjusting'}
			<div class="actions row">
				<p class="status">{e.adjusting(isMac ? '⇧' : m.keys.shift)}</p>
				<button type="button" class="act strong" onclick={() => void edit.done()}>{e.done}</button>
				<button type="button" class="act" onclick={() => edit.cancel()}>{e.cancel}</button>
			</div>
		{:else if region}
			{@const r = region}
			{#if own}
				<p class="title">{e.yours}</p>
				{#key r.id}
					<label class="label" for="region-text">{e.text}</label>
					<textarea
						id="region-text"
						class="text own-direction"
						rows="2"
						value={r.text}
						aria-describedby="region-text-hint"
						onblur={(ev) => void edit.saveText(r, ev.currentTarget.value)}
						onkeydown={onTextKey}></textarea>
					<p id="region-text-hint" class="status">{e.textHint(shortcut('Enter'))}</p>
				{/key}
			{:else}
				<div class="said" role="status">
					<p class="words own-direction">{r.text}</p>
					<p class="status">{status}</p>
					{#if r.origin === 'estimated'}<p class="status">{m.review.regionEstimated}</p>{/if}
					{#if adjusted}<p class="status">{e.adjusted}</p>{/if}
				</div>
			{/if}
			<div class="actions">
				{#if adjustable}
					<button type="button" class="act" onclick={() => edit.adjustChosen()}>
						{e.moveResize}
					</button>
				{/if}
				{#if own}
					<button type="button" class="act" onclick={() => void edit.remove(r)}>{e.remove}</button>
				{:else if adjusted && regions.drawable}
					<button type="button" class="act" onclick={() => void edit.putBack(r)}>
						{e.putBack}
					</button>
				{/if}
				{#if regions.onread}
					{@const read = regions.onread}
					<button type="button" class="act" onclick={() => read(r)}>{e.read}</button>
				{/if}
			</div>
			<button
				type="button"
				class="close"
				aria-label={m.review.regionClose}
				title={m.review.regionClose}
				onclick={close}
			>
				<Icon name="close" size={14} />
			</button>
		{:else}
			<p class="status">{fallback}</p>
		{/if}
	</div>
{/if}

<style>
	/* A small card on the stage, in its own light, under the pane's bar. */
	.region-note {
		position: absolute;
		top: calc(var(--bar-height, 2.75rem) + var(--space-3));
		left: var(--space-4);
		right: var(--space-4);
		z-index: 3;
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		column-gap: var(--space-2);
		/* Clear of the zoom bar at the foot of the lightbox; in a pane too short for that (a small
		   window stacks the panes), as tall as its buttons need, over what follows. */
		max-height: max(calc(100% - var(--bar-height, 2.75rem) - 5.5rem), 10rem);
		overflow: auto;
		padding: var(--space-2) var(--space-2) var(--space-2) var(--space-3);
		border-radius: 9px;
		background: var(--stage-raised);
		box-shadow:
			0 0 0 1px var(--stage-line),
			0 8px 24px rgba(0, 0, 0, 0.35);
		color: var(--on-stage);
	}
	/* At the foot of the scan, above the zoom bar, when under the bar it would cover the region
	   chosen. */
	.region-note.foot {
		top: auto;
		bottom: calc(var(--space-4) + 3rem);
	}
	/* While a box is drawn or adjusted, a drag that starts over the card still reaches the page
	   under it; only its buttons take the pointer. */
	.region-note.through {
		pointer-events: none;
	}
	.region-note.through button {
		pointer-events: auto;
	}
	.region-note > :not(.close, .grip) {
		grid-column: 1;
	}
	/* Where the researcher dragged it: neither under the bar nor at the foot. */
	.region-note.moved {
		bottom: auto;
	}
	p {
		margin: 0;
	}
	/* A long block's words scroll on their own, so the buttons under them stay in view. */
	.said {
		display: grid;
		max-height: 4.75rem;
		overflow: auto;
	}
	/* The words as read: the text being checked, in the reading face. */
	.words {
		font-family: var(--font-text);
		font-size: var(--type-md);
		line-height: 1.5;
		overflow-wrap: anywhere;
	}
	.title {
		font-size: var(--type-sm);
		font-weight: 600;
	}
	.status {
		margin-top: var(--space-1);
		font-size: var(--type-xs);
		color: var(--on-stage-muted);
	}
	.region-note > .status:first-child {
		margin-top: 0;
	}
	.label {
		margin-top: var(--space-2);
		font-size: var(--type-xs);
		color: var(--on-stage);
	}
	/* What the researcher reads there, in the reading face, in the stage's own light. */
	.text {
		margin-top: var(--space-1);
		width: 100%;
		min-height: 2.75rem;
		resize: vertical;
		font-family: var(--font-text);
		font-size: var(--type-sm);
		line-height: 1.5;
		color: var(--on-stage);
		/* The page's caret is drawn for the theme's light; the stage keeps its dark in both. */
		caret-color: var(--on-stage);
		background: var(--stage);
		border: 1px solid var(--stage-line);
		border-radius: 6px;
		padding: var(--space-1) var(--space-2);
	}
	.text:focus-visible {
		outline-color: var(--on-stage);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
		margin-top: var(--space-2);
	}
	/* While a box is drawn or adjusted: what to do and the way out, close together, so the
	   card covers as little of the page as it can. */
	.actions.row {
		align-items: center;
		margin-top: 0;
	}
	.actions.row .status {
		flex: 1 1 16rem;
		margin: 0;
	}
	.act {
		min-height: max(var(--control-sm), 24px);
		padding: 0 0.625rem;
		border: 1px solid var(--stage-line);
		border-radius: 6px;
		background: transparent;
		color: var(--on-stage);
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.act:hover {
		background: rgba(255, 255, 255, 0.08);
	}
	/* The way out that keeps the box: drawn a little stronger. */
	.act.strong {
		background: rgba(255, 255, 255, 0.14);
		border-color: var(--on-stage-muted);
		font-weight: 600;
	}
	.act:focus-visible {
		outline-color: var(--on-stage);
	}
	/* The grip, beside the card's first line, or under the close button when there is one. */
	.grip,
	.close {
		grid-column: 2;
		grid-row: 1;
		display: grid;
		place-items: center;
		width: var(--control-sm);
		height: var(--control-sm);
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.grip:has(~ .close) {
		grid-row: 2;
	}
	.grip {
		color: var(--on-stage-muted);
		cursor: grab;
		touch-action: none;
	}
	.region-note:not(.through) {
		cursor: grab;
	}
	.region-note:not(.through) :is(button, textarea, .said) {
		cursor: auto;
	}
	.region-note:not(.through) button {
		cursor: pointer;
	}
	.region-note:not(.through) .grip {
		cursor: grab;
	}
	.grip:hover,
	.close:hover {
		background: rgba(255, 255, 255, 0.08);
	}
	.grip:focus-visible,
	.close:focus-visible {
		outline-color: var(--on-stage);
	}
	@media (forced-colors: active) {
		.act,
		.text {
			border-color: ButtonText;
		}
	}
</style>
