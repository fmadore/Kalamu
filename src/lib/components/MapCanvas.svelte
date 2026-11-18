<script lang="ts">
	// The map's drawing: one mark per document, its shape and tone its group's, zoomed with the
	// wheel or the buttons and moved by dragging. One point is reachable with Tab at a time;
	// the arrow keys move to the nearest point in their direction, Enter (or a double click)
	// opens the document, plus and minus zoom. Pointing at a point or reaching it shows its title;
	// a click selects it, and the list beside the map shows the documents nearest to it.
	import Icon from './Icon.svelte';
	import MapShape from './MapShape.svelte';
	import type { MapPoint } from '#lib/bindings/MapPoint.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		WHOLE_MAP,
		nextPoint,
		panned,
		placed,
		zoomed,
		type Direction,
		type MapGroup,
		type MapWindow
	} from '#lib/meaning.js';

	let {
		points,
		groupOf,
		groupName,
		selected = $bindable(),
		onopen
	}: {
		points: MapPoint[];
		groupOf: Map<string, MapGroup>;
		/** A group's name, as the legend writes it. */
		groupName: (group: MapGroup | undefined) => string;
		selected: string | null;
		onopen: (id: string) => void;
	} = $props();
	const m = messages();

	let view = $state<MapWindow>({ ...WHOLE_MAP });
	let svg = $state<SVGSVGElement | null>(null);
	let hovered = $state<string | null>(null);
	let focused = $state<string | null>(null);
	const r = $derived(view.size * 0.016);
	/** The point reached with Tab: the one selected, else the first. */
	const tabStop = $derived(selected ?? points[0]?.document_id ?? null);
	const shown = $derived(
		points.find((p) => p.document_id === (hovered ?? focused ?? null)) ?? null
	);

	/** A position on the screen in the map's units. */
	function toMap(clientX: number, clientY: number): { x: number; y: number } {
		const box = svg?.getBoundingClientRect();
		if (!box || box.width === 0) return { x: 0.5, y: 0.5 };
		return {
			x: view.x + ((clientX - box.left) / box.width) * view.size,
			y: view.y + ((clientY - box.top) / box.height) * view.size
		};
	}

	function zoomBy(factor: number, at?: { x: number; y: number }) {
		const c = at ?? { x: view.x + view.size / 2, y: view.y + view.size / 2 };
		view = zoomed(view, factor, c.x, c.y);
	}

	function wheel(e: WheelEvent) {
		e.preventDefault();
		zoomBy(e.deltaY < 0 ? 1.25 : 0.8, toMap(e.clientX, e.clientY));
	}

	let drag: { x: number; y: number; moved: boolean } | null = null;
	function down(e: PointerEvent) {
		if (e.button !== 0) return;
		drag = { x: e.clientX, y: e.clientY, moved: false };
	}
	function move(e: PointerEvent) {
		if (!drag || !svg) return;
		const box = svg.getBoundingClientRect();
		const dx = e.clientX - drag.x;
		const dy = e.clientY - drag.y;
		if (!drag.moved && Math.hypot(dx, dy) < 4) return;
		if (!drag.moved) svg.setPointerCapture(e.pointerId);
		drag = { x: e.clientX, y: e.clientY, moved: true };
		view = panned(view, (-dx / box.width) * view.size, (-dy / box.height) * view.size);
	}
	function up(e: PointerEvent) {
		if (drag?.moved && svg?.hasPointerCapture(e.pointerId)) svg.releasePointerCapture(e.pointerId);
		drag = null;
	}

	/** Moves the focus to a point, keeping it in sight. */
	function reach(p: MapPoint) {
		selected = p.document_id;
		const at = placed(p);
		const margin = view.size * 0.08;
		if (
			at.x < view.x + margin ||
			at.x > view.x + view.size - margin ||
			at.y < view.y + margin ||
			at.y > view.y + view.size - margin
		) {
			view = { ...view, x: at.x - view.size / 2, y: at.y - view.size / 2 };
		}
		queueMicrotask(() =>
			svg?.querySelector<SVGGElement>(`[data-id="${CSS.escape(p.document_id)}"]`)?.focus()
		);
	}

	const KEYS: Record<string, Direction> = {
		ArrowLeft: 'left',
		ArrowRight: 'right',
		ArrowUp: 'up',
		ArrowDown: 'down'
	};

	function key(e: KeyboardEvent, p: MapPoint) {
		const direction = KEYS[e.key];
		if (direction) {
			e.preventDefault();
			const next = nextPoint(points, p, direction);
			if (next) reach(next);
		} else if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onopen(p.document_id);
		} else if (e.key === '+' || e.key === '=') {
			e.preventDefault();
			zoomBy(1.25, placed(p));
		} else if (e.key === '-') {
			e.preventDefault();
			zoomBy(0.8, placed(p));
		}
	}

	/** Where a title is written beside its point, kept inside the drawing. */
	const label = $derived.by(() => {
		if (!shown) return null;
		const at = placed(shown);
		const right = at.x < view.x + view.size * 0.6;
		return {
			x: at.x + (right ? 1.8 : -1.8) * r,
			y: at.y - 1.5 * r,
			anchor: right ? 'start' : 'end'
		};
	});
</script>

<div class="canvas">
	<div class="tools" role="group" aria-label={m.meaning.map.title}>
		<button
			class="btn icon sm"
			aria-label={m.meaning.map.zoomIn}
			title={m.meaning.map.zoomIn}
			onclick={() => zoomBy(1.5)}
		>
			<Icon name="zoom-in" size={15} />
		</button>
		<button
			class="btn icon sm"
			aria-label={m.meaning.map.zoomOut}
			title={m.meaning.map.zoomOut}
			onclick={() => zoomBy(1 / 1.5)}
		>
			<Icon name="zoom-out" size={15} />
		</button>
		<button class="btn sm" onclick={() => (view = { ...WHOLE_MAP })}>{m.meaning.map.whole}</button>
	</div>
	<svg
		bind:this={svg}
		viewBox="{view.x} {view.y} {view.size} {view.size}"
		role="group"
		aria-label={m.meaning.map.canvas}
		onwheel={wheel}
		onpointerdown={down}
		onpointermove={move}
		onpointerup={up}
		onpointercancel={up}
	>
		{#each points as p (p.document_id)}
			{@const at = placed(p)}
			{@const group = groupOf.get(p.document_id)}
			<g
				role="button"
				tabindex={p.document_id === tabStop ? 0 : -1}
				data-id={p.document_id}
				aria-label={m.meaning.map.point(p.title, groupName(group))}
				aria-pressed={p.document_id === selected}
				class="point"
				class:selected={p.document_id === selected}
				onclick={() => reach(p)}
				ondblclick={() => onopen(p.document_id)}
				onkeydown={(e) => key(e, p)}
				onfocus={() => {
					focused = p.document_id;
					selected = p.document_id;
				}}
				onblur={() => (focused = null)}
				onpointerenter={() => (hovered = p.document_id)}
				onpointerleave={() => (hovered = null)}
			>
				<circle class="halo" cx={at.x} cy={at.y} r={r * 2.2} />
				<MapShape shape={group?.shape ?? 'hollow'} tone={group?.tone ?? -1} x={at.x} y={at.y} {r} />
			</g>
		{/each}
		{#if shown && label}
			<text
				class="title"
				x={label.x}
				y={label.y}
				text-anchor={label.anchor}
				font-size={r * 2.6}
				stroke-width={r * 0.6}
				aria-hidden="true">{shown.title}</text
			>
		{/if}
	</svg>
</div>

<style>
	.canvas {
		display: grid;
		gap: var(--space-2);
		min-width: 0;
	}
	.tools {
		display: flex;
		gap: var(--space-1);
		align-items: center;
	}
	svg {
		width: 100%;
		max-height: 70vh;
		aspect-ratio: 1;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		background: var(--surface-2);
		touch-action: none;
		cursor: grab;
	}
	.point {
		cursor: pointer;
		outline: none;
	}
	.halo {
		fill: transparent;
	}
	.point:hover .halo,
	.point:focus-visible .halo {
		fill: var(--found);
	}
	/* The selected point ringed in ink, the one the keyboard is on ringed and washed, both
	   strokes the same width on screen whatever the zoom. */
	.point.selected .halo {
		fill: none;
		stroke: var(--ink);
		stroke-width: 1.5px;
		vector-effect: non-scaling-stroke;
	}
	.point:focus-visible .halo {
		stroke: var(--ink);
		stroke-width: 2px;
		vector-effect: non-scaling-stroke;
	}
	.title {
		fill: var(--text-bright);
		stroke: var(--surface-2);
		paint-order: stroke;
		font-family: var(--font-ui);
		pointer-events: none;
	}
</style>
