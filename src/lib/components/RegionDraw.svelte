<script lang="ts">
	// Over the regions on the page, while the researcher draws or adjusts one. Drawing: the next
	// drag on the page draws a box, the page not moving under it; let go, the box is kept, unless
	// it was a click or a slip of the hand, which keep nothing; either way drawing ends. Adjusting:
	// the box has grips at its corners and sides and is dragged whole; from the keyboard (the box
	// has the focus) the arrows move it, Shift and the arrows resize it, Enter keeps it and Escape
	// puts it back. The grips are at least 24 pixels square whatever the zoom. Mouse, pen and
	// touch alike (pointer events); a wheel zoom meanwhile keeps the box where it was on the page.
	import { tick } from 'svelte';
	import { messages } from '#lib/i18n/index.js';
	import type { Rotation } from '#lib/pdf.js';
	import { isMac } from '#lib/platform.js';
	import { boxBetween, isSlip, pointIn, type Grip, type Point } from '#lib/regionBox.js';
	import { turnBox, unturnBox, type Box } from '#lib/regions.js';
	import type { Regions } from '#lib/regions.svelte.js';

	let { regions, zoom, rotation }: { regions: Regions; zoom: number; rotation: Rotation } =
		$props();
	const m = messages();
	const edit = $derived(regions.edit);

	let surface = $state<HTMLElement | null>(null);
	let frame = $state<HTMLElement | null>(null);
	let frameWidth = $state(0);
	let frameHeight = $state(0);

	/** The page's own box on screen, which the zoom stretches. */
	const sheet = (el: Element | null) => el?.closest('.page-sheet')?.getBoundingClientRect() ?? null;
	/** Where the pointer is on the page, in fractions of it as drawn, not kept on the page. */
	const at = (rect: DOMRect, e: PointerEvent): Point => ({
		x: (e.clientX - rect.left) / rect.width,
		y: (e.clientY - rect.top) / rect.height
	});
	const mainButton = (e: PointerEvent) => e.pointerType !== 'mouse' || e.button === 0;

	/* Drawing: where the drag started on the page, where it is now, and where on screen it
	   started, to tell a click from a drag. */
	let drawing = $state<{ start: Point; now: Point; id: number; x: number; y: number } | null>(null);

	function startDraw(e: PointerEvent) {
		const rect = sheet(surface);
		if (!surface || !rect || !mainButton(e)) return;
		// The page is not moved by this drag: the pane never sees it.
		e.preventDefault();
		e.stopPropagation();
		const p = pointIn(rect, e.clientX, e.clientY);
		drawing = { start: p, now: p, id: e.pointerId, x: e.clientX, y: e.clientY };
		surface.setPointerCapture(e.pointerId);
	}

	function moveDraw(e: PointerEvent) {
		const rect = sheet(surface);
		if (!drawing || e.pointerId !== drawing.id || !rect) return;
		e.stopPropagation();
		drawing.now = pointIn(rect, e.clientX, e.clientY);
	}

	function endDraw(e: PointerEvent) {
		const rect = sheet(surface);
		const d = drawing;
		if (!d || e.pointerId !== d.id) return;
		e.stopPropagation();
		drawing = null;
		if (surface?.hasPointerCapture(e.pointerId)) surface.releasePointerCapture(e.pointerId);
		if (!rect) return edit.reset();
		const box = boxBetween(d.start, pointIn(rect, e.clientX, e.clientY));
		// A click, or a box too small to be meant: nothing kept, and drawing ends.
		if (Math.hypot(e.clientX - d.x, e.clientY - d.y) < 4 || isSlip(box, rect)) return edit.reset();
		void edit.drawn(unturnBox(box, rotation));
	}

	function cancelDraw(e: PointerEvent) {
		if (!drawing || e.pointerId !== drawing.id) return;
		drawing = null;
		edit.reset();
	}

	/* Adjusting: the grip held, the box as it was when it was taken, and where on the page the
	   pointer took it. */
	let held: { grip: Grip; from: Box; start: Point; id: number } | null = null;

	function take(e: PointerEvent, grip: Grip) {
		const rect = sheet(frame);
		const box = edit.box;
		if (!rect || !box || !mainButton(e)) return;
		e.preventDefault();
		e.stopPropagation();
		held = { grip, from: box, start: at(rect, e), id: e.pointerId };
		frame?.setPointerCapture(e.pointerId);
		frame?.focus({ preventScroll: true });
	}

	function moveHeld(e: PointerEvent) {
		const rect = sheet(frame);
		if (!held || e.pointerId !== held.id || !rect) return;
		e.stopPropagation();
		const now = at(rect, e);
		edit.drag(held.from, held.grip, now.x - held.start.x, now.y - held.start.y, rotation);
	}

	function letGo(e: PointerEvent) {
		if (!held || e.pointerId !== held.id) return;
		e.stopPropagation();
		held = null;
		if (frame?.hasPointerCapture(e.pointerId)) frame.releasePointerCapture(e.pointerId);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.altKey || e.ctrlKey || e.metaKey) return;
		if (e.key === 'Enter') {
			e.preventDefault();
			e.stopPropagation();
			void edit.done();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			edit.cancel();
		} else if (edit.nudge(e.key, e.shiftKey, zoom, rotation)) {
			// The page's own arrow keys move it; here they move the box.
			e.preventDefault();
			e.stopPropagation();
		}
	}

	// The box takes the focus as adjusting begins, and is brought into view.
	$effect(() => {
		const el = frame;
		if (el) void tick().then(() => el.focus());
	});

	const pct = (v: number) => `${v * 100}%`;
	/** A grip on a side is left out where the box is too short on screen for three grips. */
	const GRIPS: Grip[] = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];
	const shown = (g: Grip) =>
		g.length === 2 ||
		((g === 'n' || g === 's') && frameWidth >= 72) ||
		((g === 'e' || g === 'w') && frameHeight >= 72);
	const preview = $derived(drawing ? boxBetween(drawing.start, drawing.now) : null);
	const hint = $derived(m.review.regionEdit.adjusting(isMac ? '⇧' : m.keys.shift));
</script>

{#if edit.mode === 'drawing'}
	<div
		class="surface"
		bind:this={surface}
		aria-hidden="true"
		onpointerdown={startDraw}
		onpointermove={moveDraw}
		onpointerup={endDraw}
		onpointercancel={cancelDraw}
		ondblclick={(e) => e.stopPropagation()}
	>
		{#if preview}
			<span
				class="preview"
				style:left={pct(preview.x)}
				style:top={pct(preview.y)}
				style:width={pct(preview.width)}
				style:height={pct(preview.height)}
			></span>
		{/if}
	</div>
{:else if edit.mode === 'adjusting' && edit.box}
	{@const b = turnBox(edit.box, rotation)}
	<!-- An application: its arrow keys move the box, not a screen reader's reading cursor. -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
	<div
		class="frame"
		bind:this={frame}
		bind:clientWidth={frameWidth}
		bind:clientHeight={frameHeight}
		role="application"
		tabindex="0"
		aria-label={m.review.regionEdit.box}
		aria-describedby="region-adjust-hint"
		style:left={pct(b.x)}
		style:top={pct(b.y)}
		style:width={pct(b.width)}
		style:height={pct(b.height)}
		{onkeydown}
		onpointerdown={(e) => take(e, 'move')}
		onpointermove={moveHeld}
		onpointerup={letGo}
		onpointercancel={letGo}
		ondblclick={(e) => e.stopPropagation()}
	>
		{#each GRIPS.filter(shown) as g (g)}
			<span class="grip {g}" aria-hidden="true" onpointerdown={(e) => take(e, g)}></span>
		{/each}
	</div>
	<span id="region-adjust-hint" class="sr-only">{hint}</span>
{/if}

<style>
	/* Over the whole page while drawing: the next drag is the box's. */
	.surface {
		position: absolute;
		inset: 0;
		z-index: 2;
		cursor: crosshair;
		touch-action: none;
	}
	.preview {
		position: absolute;
		border: 2px dashed var(--hand-on-page);
		background: color-mix(in srgb, var(--hand-on-page) 10%, transparent);
		pointer-events: none;
	}
	/* The box being adjusted, in the hand's blue, dragged whole from inside it. */
	.frame {
		position: absolute;
		z-index: 2;
		box-sizing: border-box;
		border: 2px solid var(--hand-on-page);
		background: color-mix(in srgb, var(--hand-on-page) 10%, transparent);
		cursor: move;
		touch-action: none;
	}
	/* A box smaller than a fingertip on screen is still taken whole from around it. */
	.frame::before {
		content: '';
		position: absolute;
		left: 50%;
		top: 50%;
		width: max(100%, 24px);
		height: max(100%, 24px);
		transform: translate(-50%, -50%);
	}
	.frame:focus-visible {
		outline: 2px solid var(--on-marker);
		outline-offset: 3px;
	}
	/* A grip: 24 pixels to take, a small square to see, centred on its corner or side. */
	.grip {
		position: absolute;
		width: 24px;
		height: 24px;
		margin: -12px 0 0 -12px;
		touch-action: none;
	}
	.grip::after {
		content: '';
		position: absolute;
		inset: 7px;
		border: 2px solid var(--hand-on-page);
		border-radius: 2px;
		background: white;
	}
	.nw {
		left: 0;
		top: 0;
		cursor: nwse-resize;
	}
	.n {
		left: 50%;
		top: 0;
		cursor: ns-resize;
	}
	.ne {
		left: 100%;
		top: 0;
		cursor: nesw-resize;
	}
	.e {
		left: 100%;
		top: 50%;
		cursor: ew-resize;
	}
	.se {
		left: 100%;
		top: 100%;
		cursor: nwse-resize;
	}
	.s {
		left: 50%;
		top: 100%;
		cursor: ns-resize;
	}
	.sw {
		left: 0;
		top: 100%;
		cursor: nesw-resize;
	}
	.w {
		left: 0;
		top: 50%;
		cursor: ew-resize;
	}
	@media (forced-colors: active) {
		.preview,
		.frame {
			border-color: Highlight;
		}
		.grip::after {
			border-color: Highlight;
			background: Canvas;
		}
	}
</style>
