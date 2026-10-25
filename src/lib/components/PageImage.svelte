<script lang="ts">
	// One page of a source: a single-page PDF rendered with pdf.js, or an image as it is.
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import { messages } from '#lib/i18n/index.js';

	let { preview, zoom = 1 }: { preview: PagePreview | null; zoom?: number } = $props();
	const m = messages();

	let canvas = $state<HTMLCanvasElement | null>(null);
	let failed = $state(false);
	let width = $state(0);

	function bytes(b64: string): Uint8Array {
		const raw = atob(b64);
		const out = new Uint8Array(raw.length);
		for (let i = 0; i < raw.length; i++) out[i] = raw.charCodeAt(i);
		return out;
	}

	async function renderPdf(data: string, target: HTMLCanvasElement, zoom: number, fit: number) {
		const pdfjs = await import('pdfjs-dist');
		const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url');
		pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
		const task = pdfjs.getDocument({ data: bytes(data) });
		const doc = await task.promise;
		const page = await doc.getPage(1);
		// Fit the pane's width at zoom 1. Sharp on high-density screens: render at the device
		// pixel ratio, display at CSS size.
		const ratio = window.devicePixelRatio || 1;
		const natural = page.getViewport({ scale: 1 });
		const scale = (Math.max(fit - 24, 200) / natural.width) * zoom;
		const viewport = page.getViewport({ scale: scale * ratio });
		target.width = viewport.width;
		target.height = viewport.height;
		target.style.width = `${viewport.width / ratio}px`;
		await page.render({ canvas: target, viewport }).promise;
		await task.destroy();
	}

	$effect(() => {
		failed = false;
		if (preview?.mime === 'application/pdf' && canvas && width > 0) {
			renderPdf(preview.data, canvas, zoom, width).catch(() => (failed = true));
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
