<script lang="ts" module>
	// Thumbnails drawn once per page and kept for the session, so paging through a long scan
	// does not draw them again. Drawn two at a time: a 400-page strip asks for them as it
	// scrolls into view, not all at once.
	const drawn = new Map<string, string>();
	const waiting: (() => Promise<void>)[] = [];
	let running = 0;

	function enqueue(task: () => Promise<void>) {
		waiting.push(task);
		pump();
	}

	function pump() {
		while (running < 2 && waiting.length > 0) {
			const task = waiting.shift();
			if (!task) break;
			running++;
			// Each task catches its own failure, so the queue only has to move on.
			void task().finally(() => {
				running--;
				pump();
			});
		}
	}
</script>

<script lang="ts">
	// One page of a source, small, in the review's strip of pages.
	import { renderPdfPage } from '#lib/pdf.js';
	import { api } from '#lib/tauri.js';

	let { jobId, sourceId, index }: { jobId: string; sourceId: string; index: number } = $props();

	const WIDTH = 40;
	const key = $derived(`${jobId}/${sourceId}/${index}`);
	let src = $state<string | null>(null);
	let box = $state<HTMLElement | null>(null);

	async function draw(key: string) {
		const preview = await api.piecePreview(jobId, sourceId, index);
		let url: string | null = null;
		if (preview.mime === 'application/pdf') {
			const canvas = document.createElement('canvas');
			await renderPdfPage(preview.data, canvas, WIDTH * 2);
			url = canvas.toDataURL('image/jpeg', 0.8);
		} else if (preview.mime.startsWith('image/') && !preview.mime.includes('hei')) {
			url = `data:${preview.mime};base64,${preview.data}`;
		}
		if (url) drawn.set(key, url);
		src = url;
	}

	$effect(() => {
		const k = key;
		src = drawn.get(k) ?? null;
		if (src || !box) return;
		const observer = new IntersectionObserver((entries) => {
			if (!entries.some((e) => e.isIntersecting)) return;
			observer.disconnect();
			enqueue(() => draw(k).catch(() => {}));
		});
		observer.observe(box);
		return () => observer.disconnect();
	});
</script>

<span class="thumb" bind:this={box} aria-hidden="true">
	{#if src}<img {src} alt="" />{/if}
</span>

<style>
	.thumb {
		display: block;
		width: 2.5rem;
		aspect-ratio: 3 / 4;
		overflow: hidden;
		border-radius: 2px;
		background: var(--paper);
		box-shadow: 0 0 0 1px var(--line);
	}
	img {
		display: block;
		width: 100%;
		height: 100%;
		object-fit: cover;
		object-position: top;
	}
</style>
