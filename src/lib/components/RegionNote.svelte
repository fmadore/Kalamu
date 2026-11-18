<script lang="ts">
	// Over the top of the lightbox while the regions are shown. For an engine's region chosen: its
	// own text, as the model read it, and whether its place is marked in the text, or why not;
	// *Move or resize*, and once the researcher gave it a box, *Put back the model's box*. For one
	// of the researcher's own: what is written there, which they may type (saved when the field
	// is left, or with Ctrl Enter; ⌘ on a Mac), *Move or resize* and *Remove*. While a box is
	// drawn or adjusted, what to do and how to leave. On a page with no region, what to do
	// instead. *Read this region again…* is offered where the review gives a way to.
	import { tick } from 'svelte';
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { isMac, primary, shortcut } from '#lib/platform.js';
	import { focusDraw } from '#lib/regionEdit.svelte.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions }: { regions: Regions } = $props();
	const m = messages();
	const e = $derived(m.review.regionEdit);
	const edit = $derived(regions.edit);

	const region = $derived(regions.region);
	const own = $derived(regions.chosen?.level === 'drawn');
	const adjusted = $derived(!own && region?.origin === 'manual');
	/** An engine's region can be given a box where the page's reading is known. */
	const adjustable = $derived(regions.drawable && (own || !!regions.page?.result));
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
	<div class="region-note" class:through={edit.mode !== 'idle'}>
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
						class="text"
						dir="auto"
						rows="2"
						value={r.text}
						aria-describedby="region-text-hint"
						onblur={(ev) => void edit.saveText(r, ev.currentTarget.value)}
						onkeydown={onTextKey}></textarea>
					<p id="region-text-hint" class="status">{e.textHint(shortcut('Enter'))}</p>
				{/key}
			{:else}
				<div class="said" role="status">
					<p class="words" dir="auto">{r.text}</p>
					<p class="status">{status}</p>
					{#if r.origin === 'estimated'}<p class="status">{m.review.regionEstimated}</p>{/if}
					{#if adjusted}<p class="status">{e.adjusted}</p>{/if}
				</div>
			{/if}
			<div class="actions">
				{#if adjustable}
					<button type="button" class="act" onclick={() => edit.adjustChosen()}
						>{e.moveResize}</button
					>
				{/if}
				{#if own}
					<button type="button" class="act" onclick={() => void edit.remove(r)}>{e.remove}</button>
				{:else if adjusted && regions.drawable}
					<button type="button" class="act" onclick={() => void edit.putBack(r)}>{e.putBack}</button
					>
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
				onclick={close}><Icon name="close" size={14} /></button
			>
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
	/* While a box is drawn or adjusted, a drag that starts over the card still reaches the page
	   under it; only its buttons take the pointer. */
	.region-note.through {
		pointer-events: none;
	}
	.region-note.through button {
		pointer-events: auto;
	}
	.region-note > :not(.close) {
		grid-column: 1;
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
	.close:hover {
		background: rgba(255, 255, 255, 0.08);
	}
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
