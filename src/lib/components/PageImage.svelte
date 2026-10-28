<script lang="ts">
	// One page of a source: a single-page PDF rendered with pdf.js, or an image as it is.
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import { messages } from '#lib/i18n/index.js';
	import { renderPdfPage } from '#lib/pdf.js';

	let { preview, zoom = 1 }: { preview: PagePreview | null; zoom?: number } = $props();
	const m = messages();

	let canvas = $state<HTMLCanvasElement | null>(null);
	let failed = $state(false);
	let width = $state(0);

	$effect(() => {
		failed = false;
		if (preview?.mime === 'application/pdf' && canvas && width > 0) {
			// Fit the pane's width at zoom 1.
			renderPdfPage(preview.data, canvas, Math.max(width - 24, 200) * zoom).catch(
				() => (failed = true)
			);
		}
	});
</script>

<div class="page-image" bind:clientWidth={width}>
	{#if !preview}
		<p class="muted">{m.common.loading}</p>
	{:else if preview.mime === 'application/pdf'}
		<canvas bind:this={canvas} aria-label={m.review.source}></canvas>
		{#if failed}<p class="muted">{m.review.noPreview}</p>{/if}
	{:else if preview.mime.startsWith('image/') && !preview.mime.includes('hei')}
		<img
			src="data:{preview.mime};base64,{preview.data}"
			alt={m.review.source}
			style:width="{Math.round(zoom * 100)}%"
		/>
	{:else}
		<p class="muted">{m.review.noPreview}</p>
	{/if}
</div>

<style>
	.page-image {
		display: flex;
		justify-content: center;
		align-items: flex-start;
		min-height: 100%;
		background: transparent;
	}
	/* The page lies on the lightbox: a crisp edge and a soft drop beneath. */
	canvas,
	img {
		max-width: none;
		border-radius: 2px;
		box-shadow: var(--shadow-page);
		background: white;
	}
	p {
		font-size: var(--type-sm);
	}
</style>
