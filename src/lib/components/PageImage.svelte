<script lang="ts">
	// One page of a source, a single-page PDF rendered with pdf.js or an image, drawn on a canvas
	// at the pane's width times the zoom, turned by the reader's rotation. The page is opened once
	// per preview and drawn again when the pane, the zoom or the rotation change.
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import { messages } from '#lib/i18n/index.js';
	import { type Rotation, SourcePage } from '#lib/pdf.js';

	let {
		preview,
		zoom = 1,
		rotation = 0
	}: { preview: PagePreview | null; zoom?: number; rotation?: Rotation } = $props();
	const m = messages();

	let canvas = $state<HTMLCanvasElement | null>(null);
	let page = $state<SourcePage | null>(null);
	let failed = $state(false);
	let width = $state(0);

	/** Whether the preview is something a canvas can show: a PDF page or a browser image. */
	const drawable = $derived(
		!!preview &&
			(preview.mime === 'application/pdf' ||
				(preview.mime.startsWith('image/') && !preview.mime.includes('hei')))
	);

	// Each preview is opened once; the one before is closed.
	$effect(() => {
		const current = preview;
		failed = false;
		page = null;
		if (!current || !drawable) return;
		let opened: SourcePage | null = null;
		let gone = false;
		SourcePage.open(current.mime, current.data)
			.then((p) => {
				if (gone) void p.destroy();
				else page = opened = p;
			})
			.catch(() => {
				if (!gone) failed = true;
			});
		return () => {
			gone = true;
			void opened?.destroy();
		};
	});

	// Fit the pane's width at zoom 1.
	$effect(() => {
		if (!page || !canvas || width <= 0) return;
		page.draw(canvas, Math.max(width - 24, 200) * zoom, rotation).catch(() => (failed = true));
	});
</script>

<div class="page-image" bind:clientWidth={width}>
	{#if !preview}
		<p class="muted">{m.common.loading}</p>
	{:else if drawable && !failed}
		<canvas bind:this={canvas} aria-label={m.review.source}></canvas>
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
	canvas {
		max-width: none;
		border-radius: 2px;
		box-shadow: var(--shadow-page);
		background: white;
	}
	p {
		font-size: var(--type-sm);
	}
</style>
