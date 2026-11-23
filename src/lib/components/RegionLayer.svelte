<script lang="ts">
	// The regions drawn over the scan, inside the page's box so they follow its zoom and moves;
	// turned with the reader's quarter turns. The engine's blocks are drawn at the pane's width,
	// lines and then words as the reader zooms in; the researcher's own after them, at every zoom.
	// A region the engine placed is outlined; one Kalamu estimated within a block placed whole is
	// dashed, as approximate; one the researcher drew or gave a box is drawn in the hand's blue
	// with a heavier line, and named as theirs. The regions drawn are one tab stop, in reading
	// order then the researcher's: the arrow keys go from one to the next and choose it, Home and
	// End to the first and last, Escape lets the choice go. Each is named by its text. A page of
	// thousands of words draws only those around the part of the page in view, and the one
	// holding the tab stop, so zooming and moving stay smooth. Drawing and adjusting a box are
	// `RegionDraw`'s, over the regions. A right click on a region, or the menu key or Shift+F10
	// on it, chooses it and opens its menu: what its card offers, and copying its text.
	import { tick } from 'svelte';
	import ActionMenu, { type Action } from './ActionMenu.svelte';
	import RegionDraw from './RegionDraw.svelte';
	import type { ShownRegion } from '#lib/bindings/ShownRegion.js';
	import { messages } from '#lib/i18n/index.js';
	import type { Rotation } from '#lib/pdf.js';
	import { levelFor, overlaps, turnBox, type Box } from '#lib/regions.js';
	import { copyText, isMenuKey, openFromClick } from '#lib/menu.js';
	import { focusDraw } from '#lib/regionEdit.svelte.js';
	import type { List, Regions } from '#lib/regions.svelte.js';

	let { regions, zoom, rotation }: { regions: Regions; zoom: number; rotation: Rotation } =
		$props();
	const m = messages();

	let layer = $state<HTMLElement | null>(null);
	const page = $derived(regions.page);
	const level = $derived(page ? levelFor(zoom, page) : 'blocks');
	/** The regions drawn: the engine's at the granularity that suits the zoom, then the
	 * researcher's own. */
	const items = $derived.by(() => {
		if (!page) return [];
		const at = (list: List) => (r: ShownRegion, index: number) => ({ list, index, region: r });
		return [...page[level].map(at(level)), ...page.drawn.map(at('drawn'))];
	});
	const boxes = $derived(items.map((it) => turnBox(it.region, rotation)));
	const edit = $derived(regions.edit);
	/** The region whose box is being adjusted, drawn by `RegionDraw` instead. */
	const adjusting = $derived(
		edit.mode === 'adjusting' && edit.target?.kind !== 'new' ? regions.chosen : null
	);
	/** The place in `items` of the region chosen, where it is drawn, else -1. */
	const chosenHere = $derived(
		regions.chosen
			? items.findIndex(
					(it) => it.list === regions.chosen?.level && it.index === regions.chosen.index
				)
			: -1
	);
	/** A region chosen at another granularity, still shown where it is. */
	const chosenElsewhere = $derived(
		regions.region && chosenHere < 0 && !adjusting ? turnBox(regions.region, rotation) : null
	);
	/** The region that holds the layer's tab stop. */
	let stop = $state(0);
	const tabStop = $derived(chosenHere >= 0 ? chosenHere : Math.min(stop, items.length - 1));

	/** From how many regions only those near the view are drawn. */
	const DRAW_ALL_BELOW = 600;
	/** The part of the page in view, in fractions of the page as drawn; with a quarter of the
	 * pane around it (`near`) for what to draw; null until measured. */
	let view = $state<Box | null>(null);
	let near = $state<Box | null>(null);
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
			const at = (mx: number, my: number): Box => ({
				x: (d.left - mx - s.left) / s.width,
				y: (d.top - my - s.top) / s.height,
				width: (d.width + 2 * mx) / s.width,
				height: (d.height + 2 * my) / s.height
			});
			view = at(0, 0);
			near = at(d.width / 4, d.height / 4);
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
	// A box placed from the keyboard goes in the middle of what is in view.
	$effect(() => {
		edit.seen = () => ({ view, rotation });
		return () => {
			edit.seen = null;
		};
	});
	/** The regions drawn, by their place in `items`. */
	const drawn = $derived.by(() => {
		const all = boxes.map((_, i) => i);
		const skip = (i: number) =>
			adjusting !== null && items[i].list === adjusting.level && items[i].index === adjusting.index;
		if (boxes.length < DRAW_ALL_BELOW) return all.filter((i) => !skip(i));
		const around = near;
		return all.filter(
			(i) =>
				!skip(i) &&
				(i === tabStop ||
					i === chosenHere ||
					items[i].list === 'drawn' ||
					(around !== null && overlaps(boxes[i], around)))
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
			regionEl(chosenHere)?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
		});
	});

	// A region saved, put back or brought back takes the focus once it is drawn again; one not
	// drawn at this zoom leaves it on *Draw a region*, so it never falls to nothing.
	$effect(() => {
		// Not while adjusting: the regions are inert under the box until it is let go.
		if (!regions.focusing || edit.mode === 'adjusting') return;
		const at = chosenHere;
		void tick().then(() => {
			regions.focusing = false;
			const el = at >= 0 ? regionEl(at) : null;
			if (el) el.focus();
			else focusDraw();
		});
	});

	function choose(i: number, focus: boolean) {
		const it = items[i];
		if (!it) return;
		stop = i;
		regions.choose(it.list, it.index);
		// Drawn once it holds the tab stop, wherever it is.
		if (focus) void tick().then(() => regionEl(i)?.focus());
	}

	/** The menu of the region chosen, and where it opens. */
	let menu = $state<{ x: number; y: number; actions: Action[] } | null>(null);

	/** What can be done with the region chosen: as its card offers, and copying its text. */
	function actions(): Action[] {
		const r = regions.region;
		if (!r) return [];
		const e = m.review.regionEdit;
		const out: Action[] = [];
		if (r.text) out.push({ label: e.copyText, icon: 'copy', run: () => void copyText(r.text) });
		if (regions.adjustable)
			out.push({ label: e.moveResize, icon: 'grip', run: () => edit.adjustChosen() });
		if (regions.own) out.push({ label: e.remove, icon: 'close', run: () => void edit.remove(r) });
		else if (regions.adjusted && regions.drawable)
			out.push({ label: e.putBack, icon: 'replay', run: () => void edit.putBack(r) });
		const read = regions.onread;
		if (read) out.push({ label: e.read, icon: 'search', run: () => read(r) });
		return out;
	}

	/** Chooses region `i` and opens its menu at `at`, or under it from the keyboard. */
	function openMenu(i: number, at: { x: number; y: number } | null) {
		choose(i, false);
		// Focused now, so the menu gives the focus back to it.
		const el = regionEl(i);
		el?.focus();
		const box = el?.getBoundingClientRect();
		const list = actions();
		if (list.length === 0) return;
		menu = { x: at?.x ?? box?.left ?? 0, y: at?.y ?? box?.bottom ?? 0, actions: list };
	}

	function onkeydown(e: KeyboardEvent, i: number) {
		if (isMenuKey(e)) {
			e.preventDefault();
			e.stopPropagation();
			openMenu(i, null);
			return;
		}
		if (e.altKey || e.ctrlKey || e.metaKey) return;
		const last = items.length - 1;
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

	/** A region's name: its text, and whose box it is when the researcher's. */
	function nameOf(list: List, r: ShownRegion): string {
		if (list === 'drawn') return m.review.regionEdit.ownName(r.text);
		return r.origin === 'manual' ? m.review.regionEdit.adjustedName(r.text) : r.text;
	}

	const pct = (v: number) => `${v * 100}%`;
</script>

{#if page && items.length > 0}
	<div
		class="layer"
		bind:this={layer}
		role="group"
		aria-label={m.review.regionsLabel}
		inert={edit.mode === 'adjusting'}
	>
		{#each drawn as i (`${items[i].list}:${items[i].index}`)}
			{@const it = items[i]}
			{@const b = boxes[i]}
			<button
				data-i={i}
				type="button"
				class="region"
				class:estimated={it.region.origin === 'estimated'}
				class:own={it.region.origin === 'manual'}
				class:chosen={i === chosenHere}
				style:left={pct(b.x)}
				style:top={pct(b.y)}
				style:width={pct(b.width)}
				style:height={pct(b.height)}
				tabindex={i === tabStop ? 0 : -1}
				aria-label={nameOf(it.list, it.region)}
				aria-pressed={i === chosenHere}
				onclick={() => choose(i, true)}
				oncontextmenu={(e) => {
					if (menu) e.preventDefault();
					else openFromClick(e, (at) => openMenu(i, at));
				}}
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
{:else}
	<!-- Measured all the same, for a box placed in the middle of the view. -->
	<div class="layer" bind:this={layer} aria-hidden="true"></div>
{/if}
<RegionDraw {regions} {zoom} {rotation} />
{#if menu}
	<ActionMenu
		x={menu.x}
		y={menu.y}
		label={m.review.regionEdit.menu}
		actions={menu.actions}
		onclose={() => (menu = null)}
	/>
{/if}

<style>
	/* Over the page's own box, which the zoom stretches. */
	.layer {
		position: absolute;
		inset: 0;
	}
	.layer[aria-hidden='true'] {
		pointer-events: none;
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
	/* The researcher's own box: the hand's blue, as their corrections, and a heavier line, so it
	   is told apart without the colour. */
	.region.own {
		border: 2px solid var(--hand-on-page);
		background: color-mix(in srgb, var(--hand-on-page) 6%, transparent);
	}
	.region:hover {
		background: color-mix(in srgb, var(--on-marker) 8%, transparent);
	}
	.region.chosen {
		border-color: var(--on-marker);
		background: color-mix(in srgb, var(--marker) 45%, transparent);
	}
	.region.own.chosen {
		border-color: var(--hand-on-page);
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
		.region.own {
			border: 3px double Highlight;
		}
	}
</style>
