<script lang="ts">
	// The source pane's switch for the regions over the scan, in its bar, and once they are shown
	// *Draw a region* beside it: pressed, the next drag on the page draws a box; pressed again, a
	// box is placed in the middle of the view, to move from the keyboard. Escape leaves drawing
	// from anywhere. What the box does as it moves is said here to a screen reader, in a word.
	import { messages } from '#lib/i18n/index.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions }: { regions: Regions } = $props();
	const m = messages();
	const edit = $derived(regions.edit);

	/** The engine placed regions here, or may: the text review's page with them. */
	const engineHint = $derived(regions.engine && (regions.page === null || regions.any));

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && edit.mode === 'drawing') {
			e.preventDefault();
			edit.cancel();
		}
	}
</script>

<svelte:window {onkeydown} />

<span class="region-tools">
	<button
		type="button"
		class="regions-tool"
		data-region-toggle
		aria-pressed={regions.shown}
		title={engineHint ? m.review.regionsHint : m.review.regionEdit.ownHint}
		onclick={() => regions.toggle()}
		>{regions.engine ? m.review.regionsShow : m.review.regionEdit.showOwn}</button
	>
	{#if regions.shown && regions.drawable}
		<button
			type="button"
			class="regions-tool"
			data-region-draw
			aria-pressed={edit.mode === 'drawing'}
			title={m.review.regionEdit.drawHint}
			onclick={() => edit.startDrawing()}>{m.review.regionEdit.draw}</button
		>
	{/if}
	<span class="sr-only" aria-live="polite">{regions.shown ? edit.said : ''}</span>
</span>

<style>
	.region-tools {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-left: auto;
		min-width: 0;
	}
	/* A tool over the lightbox, in the stage's own light, as the pane's other tools. */
	.regions-tool {
		display: grid;
		place-items: center;
		height: var(--control-sm);
		padding: 0 0.625rem;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: var(--on-stage);
		font-size: var(--type-xs);
		white-space: nowrap;
		cursor: pointer;
	}
	.regions-tool:hover {
		background: rgba(255, 255, 255, 0.08);
	}
	/* On: pressed in, as a view that is on. */
	.regions-tool[aria-pressed='true'] {
		background: rgba(255, 255, 255, 0.12);
		box-shadow: inset 0 0 0 1px var(--stage-line);
	}
	.regions-tool:focus-visible {
		outline-color: var(--on-stage);
	}
	@media (forced-colors: active) {
		.regions-tool[aria-pressed='true'] {
			border: 2px solid ButtonText;
		}
	}
</style>
