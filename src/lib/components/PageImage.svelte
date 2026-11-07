<script lang="ts">
	// One page of a source, a single-page PDF rendered with pdf.js or an image, drawn on a canvas
	// at the pane's width times the zoom, turned by the reader's rotation. The page is opened once
	// per preview. A change of zoom is shown at once by stretching the drawing, so zooming follows
	// the wheel or the pinch frame by frame; the page is drawn again, sharp, once the zoom has
	// rested a moment. A new page, a turn or a resized pane draw it straight away.
	import { untrack } from 'svelte';
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

	/** The page's width on screen: the pane's at zoom 1. */
	const shown = $derived(Math.max(width - 24, 200) * zoom);
	/** Whether the canvas holds a drawing of the page now shown, which zooming may stretch. */
	let drawn = $state(false);
	/** The width the canvas was last drawn sharp at. */
	let drawnAt = 0;

	// Stretches the drawing to the zoom at once; read synchronously by the source pane, which
	// keeps the point under the pointer in place.
	$effect.pre(() => {
		if (canvas && drawn) canvas.style.width = `${shown}px`;
	});

	// A new page, a turn or a resized pane: drawn now.
	$effect(() => {
		if (!page || !canvas || width <= 0) return;
		void rotation;
		drawn = false;
		const target = untrack(() => shown);
		page
			.draw(canvas, target, rotation)
			.then((done) => {
				if (!done) return;
				drawnAt = target;
				drawn = true;
			})
			.catch(() => (failed = true));
	});

	// The zoom: drawn sharp again once it has rested.
	$effect(() => {
		const target = shown;
		const current = untrack(() => page);
		if (!current || !canvas || !drawn || Math.abs(target - drawnAt) < 1) return;
		const turned = untrack(() => rotation);
		const timer = setTimeout(() => {
			if (!canvas) return;
			current
				.draw(canvas, target, turned)
				.then((done) => {
					if (done) drawnAt = target;
				})
				.catch(() => (failed = true));
		}, 160);
		return () => clearTimeout(timer);
	});
</script>

<div class="page-image" bind:clientWidth={width}>
	{#if !preview}
		<p class="muted">{m.common.loading}</p>
	{:else if drawable && !failed}
		<canvas bind:this={canvas} aria-label={m.review.source}></canvas>
	{:else if preview.mime === 'error'}
		<!-- The page could not be read: `data` holds why, in words. -->
		<p class="muted">{preview.data}</p>
	{:else}
		<p class="muted">{m.review.noPreview}</p>
	{/if}
</div>

<style>
	/* Centred while it fits; zoomed wider than the pane, it starts at the left edge, so the pane
	   scrolls over all of it (plain centring would push its left side out of reach). */
	.page-image {
		display: flex;
		justify-content: safe center;
		align-items: flex-start;
		min-height: 100%;
		background: transparent;
	}
	/* The page lies on the lightbox: a crisp edge and a soft drop beneath. */
	/* Never shrunk to the pane: stretched by a zoom, it keeps the size the zoom gives it. */
	canvas {
		flex: none;
		max-width: none;
		border-radius: 2px;
		box-shadow: var(--shadow-page);
		background: white;
	}
	p {
		font-size: var(--type-sm);
	}
</style>
