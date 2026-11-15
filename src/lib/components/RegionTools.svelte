<script lang="ts">
	// The source pane's switch for the text regions over the scan, in its bar. Where the model
	// placed no text on the page it stays reachable but does nothing, and says why.
	import { messages } from '#lib/i18n/index.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions }: { regions: Regions } = $props();
	const m = messages();

	/** Known to have none: loaded, and empty. */
	const none = $derived(regions.page !== null && !regions.any);
</script>

<button
	type="button"
	class="regions-tool"
	aria-pressed={regions.shown && !none}
	aria-disabled={none}
	aria-describedby={none ? 'regions-none' : undefined}
	title={none ? m.review.regionsNone : m.review.regionsHint}
	onclick={() => {
		if (!none) regions.toggle();
	}}>{m.review.regionsShow}</button
>
{#if none}<span id="regions-none" class="sr-only">{m.review.regionsNone}</span>{/if}

<style>
	/* A tool over the lightbox, in the stage's own light, as the pane's other tools. */
	.regions-tool {
		display: grid;
		place-items: center;
		height: var(--control-sm);
		padding: 0 0.625rem;
		margin-left: auto;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: var(--on-stage);
		font-size: var(--type-xs);
		white-space: nowrap;
		cursor: pointer;
	}
	.regions-tool:hover:not([aria-disabled='true']) {
		background: rgba(255, 255, 255, 0.08);
	}
	/* On: pressed in, as a view that is on. */
	.regions-tool[aria-pressed='true'] {
		background: rgba(255, 255, 255, 0.12);
		box-shadow: inset 0 0 0 1px var(--stage-line);
	}
	.regions-tool[aria-disabled='true'] {
		opacity: 0.45;
		cursor: default;
	}
	.regions-tool:focus-visible {
		outline-color: var(--on-stage);
	}
</style>
