<script lang="ts">
	// Over the top of the lightbox while the text regions are shown: the chosen region's own text,
	// as the model read it, and whether its place is marked in the text, or why not; or, on a page
	// the model placed no text on, that the page as a whole is what to check against.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions }: { regions: Regions } = $props();
	const m = messages();

	const region = $derived(regions.region);
	const none = $derived(regions.page !== null && !regions.any);
	const status = $derived(
		!region
			? ''
			: region.span
				? m.review.regionMarked
				: regions.page?.corrected
					? m.review.regionNotPlacedCorrected
					: m.review.regionNotPlaced
	);
</script>

{#if regions.shown && (region || none)}
	<div class="region-note" role="status">
		{#if region}
			<p class="words" dir="auto">{region.text}</p>
			<p class="status">{status}</p>
			{#if region.origin === 'estimated'}<p class="status">{m.review.regionEstimated}</p>{/if}
			<button
				type="button"
				class="close"
				aria-label={m.review.regionClose}
				title={m.review.regionClose}
				onclick={() => (regions.chosen = null)}><Icon name="close" size={14} /></button
			>
		{:else}
			<p class="status">{m.review.regionsFallback}</p>
		{/if}
	</div>
{/if}

<style>
	/* A small card on the stage, in its own light, under the pane's bar. */
	.region-note {
		position: absolute;
		top: calc(2.75rem + var(--space-3));
		left: var(--space-4);
		right: var(--space-4);
		z-index: 2;
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		column-gap: var(--space-2);
		max-height: 30%;
		overflow: auto;
		padding: var(--space-2) var(--space-2) var(--space-2) var(--space-3);
		border-radius: 9px;
		background: var(--stage-raised);
		box-shadow:
			0 0 0 1px var(--stage-line),
			0 8px 24px rgba(0, 0, 0, 0.35);
		color: var(--on-stage);
	}
	p {
		grid-column: 1;
		margin: 0;
	}
	/* The words as read: the text being checked, in the reading face. */
	.words {
		font-family: var(--font-text);
		font-size: var(--type-md);
		line-height: 1.5;
		overflow-wrap: anywhere;
	}
	.status {
		margin-top: var(--space-1);
		font-size: var(--type-xs);
		color: var(--on-stage-muted);
	}
	.region-note > .status:only-child {
		margin-top: 0;
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
</style>
