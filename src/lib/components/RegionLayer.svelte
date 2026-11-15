<script lang="ts">
	// The text regions drawn over the scan, inside the page's box so they follow its zoom and
	// moves; turned with the reader's quarter turns. Blocks are drawn at the pane's width, lines
	// and then words as the reader zooms in. A region the model placed is outlined; one Kalamu
	// estimated within a block the model placed whole is dashed, as approximate. The regions drawn
	// are one tab stop, in reading order: the arrow keys go from one to the next and choose it,
	// Home and End to the first and last, Escape lets the choice go. Each is named by its text.
	// A page of thousands of words draws only those around the part of the page in view, and the
	// one holding the tab stop, so zooming and moving stay smooth.
	import { tick } from 'svelte';
	import { messages } from '#lib/i18n/index.js';
	import type { Rotation } from '#lib/pdf.js';
	import { levelFor, overlaps, turnBox, type Box } from '#lib/regions.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions, zoom, rotation }: { regions: Regions; zoom: number; rotation: Rotation } =
		$props();
	const m = messages();

	let layer = $state<HTMLElement | null>(null);
	const page = $derived(regions.page);
	const level = $derived(page ? levelFor(zoom, page) : 'blocks');
	const list = $derived(page ? page[level] : []);
	const boxes = $derived(list.map((r) => turnBox(r, rotation)));
	/** The region chosen at the granularity drawn, else -1. */
	const chosenHere = $derived(
		regions.chosen && regions.chosen.level === level ? regions.chosen.index : -1
	);
	/** A region chosen at another granularity, still shown where it is. */
	const chosenElsewhere = $derived(
		regions.region && chosenHere < 0 ? turnBox(regions.region, rotation) : null
	);
	/** The region that holds the layer's tab stop. */
	let stop = $state(0);
	const tabStop = $derived(chosenHere >= 0 ? chosenHere : Math.min(stop, list.length - 1));

	/** From how many regions only those near the view are drawn. */
	const DRAW_ALL_BELOW = 600;
	/** The part of the page in view, with a quarter of the pane around it, in fractions of the page as
	 * drawn; null until measured. */
	let view = $state<Box | null>(null);
	$effect(() => {
		const el = layer;
		const desk = el?.closest<HTMLElement>('.desk');
		if (!el || !desk) return;
		let frame = 0;
		const measure = () => {
			frame = 0;
			const s = el.getBoundingClientRect();
			const d = desk.getBoundingClientRect();
			if (s.width <= 0 || s.height <= 0) return;
			const [mx, my] = [d.width / 4, d.height / 4];
			view = {
				x: (d.left - mx - s.left) / s.width,
				y: (d.top - my - s.top) / s.height,
				width: (d.width + 2 * mx) / s.width,
				height: (d.height + 2 * my) / s.height
			};
		};
		const later = () => {
			if (!frame) frame = requestAnimationFrame(measure);
		};
		measure();
		desk.addEventListener('scroll', later, { passive: true });
		const sizes = new ResizeObserver(later);
		sizes.observe(el);
		sizes.observe(desk);
		return () => {
			cancelAnimationFrame(frame);
			desk.removeEventListener('scroll', later);
			sizes.disconnect();
		};
	});
	/** The regions drawn, by their place in `list`. */
	const drawn = $derived.by(() => {
		const all = boxes.map((_, i) => i);
		if (boxes.length < DRAW_ALL_BELOW) return all;
		const near = view;
		return all.filter(
			(i) => i === tabStop || i === chosenHere || (near !== null && overlaps(boxes[i], near))
		);
	});
	const regionEl = (i: number) => layer?.querySelector<HTMLElement>(`[data-i="${i}"]`);

	$effect(() => {
		regions.level = level;
	});

	// A region chosen from the text is brought into view on the page.
	$effect(() => {
		const chosen = regions.chosen;
		if (!chosen || chosen.from !== 'text' || chosen.level !== level) return;
		void tick().then(() => {
			regionEl(chosen.index)?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
		});
	});

	function choose(i: number, focus: boolean) {
		stop = i;
		regions.choose(level, i);
		// Drawn once it holds the tab stop, wherever it is.
		if (focus) void tick().then(() => regionEl(i)?.focus());
	}

	function onkeydown(e: KeyboardEvent, i: number) {
		if (e.altKey || e.ctrlKey || e.metaKey) return;
		const last = list.length - 1;
		const to =
			e.key === 'ArrowRight' || e.key === 'ArrowDown'
				? Math.min(i + 1, last)
				: e.key === 'ArrowLeft' || e.key === 'ArrowUp'
					? Math.max(i - 1, 0)
					: e.key === 'Home'
						? 0
						: e.key === 'End'
							? last
							: null;
		if (to !== null) {
			// The page's own arrow keys move it; here they go from region to region.
			e.preventDefault();
			e.stopPropagation();
			choose(to, true);
		} else if (e.key === 'Escape' && regions.chosen) {
			e.stopPropagation();
			regions.chosen = null;
		}
	}

	const pct = (v: number) => `${v * 100}%`;
</script>

{#if page && list.length > 0}
	<div class="layer" bind:this={layer} role="group" aria-label={m.review.regionsLabel}>
		{#each drawn as i (i)}
			{@const region = list[i]}
			{@const b = boxes[i]}
			<button
				data-i={i}
				type="button"
				class="region"
				class:estimated={region.estimated}
				class:chosen={i === chosenHere}
				style:left={pct(b.x)}
				style:top={pct(b.y)}
				style:width={pct(b.width)}
				style:height={pct(b.height)}
				tabindex={i === tabStop ? 0 : -1}
				aria-label={region.text}
				aria-pressed={i === chosenHere}
				onclick={() => choose(i, true)}
				onkeydown={(e) => onkeydown(e, i)}
			></button>
		{/each}
		{#if chosenElsewhere}
			{@const b = chosenElsewhere}
			<span
				class="region chosen still"
				aria-hidden="true"
				style:left={pct(b.x)}
				style:top={pct(b.y)}
				style:width={pct(b.width)}
				style:height={pct(b.height)}
			></span>
		{/if}
	</div>
{/if}

<style>
	/* Over the page's own box, which the zoom stretches. */
	.layer {
		position: absolute;
		inset: 0;
	}
	/* The page stays white under the regions in both themes: the lines are drawn in ink, which
	   does not change with the theme, and the region chosen takes the highlighter, as the place
	   it opens in the text does. */
	.region {
		position: absolute;
		display: block;
		min-width: 0;
		min-height: 0;
		padding: 0;
		margin: 0;
		border: 1px solid color-mix(in srgb, var(--on-marker) 45%, transparent);
		border-radius: 1px;
		background: transparent;
		cursor: pointer;
	}
	/* Estimated within a block placed whole: dashed, so it reads as approximate. */
	.region.estimated {
		border-style: dashed;
	}
	.region:hover {
		background: color-mix(in srgb, var(--on-marker) 8%, transparent);
	}
	.region.chosen {
		border-color: var(--on-marker);
		background: color-mix(in srgb, var(--marker) 45%, transparent);
	}
	.region.still {
		pointer-events: none;
	}
	.region:focus-visible {
		outline: 2px solid var(--on-marker);
		outline-offset: 1px;
		z-index: 1;
	}
	@media (forced-colors: active) {
		.region {
			border-color: CanvasText;
		}
		.region.chosen {
			border-width: 2px;
		}
	}
</style>
