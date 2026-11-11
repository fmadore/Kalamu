<script lang="ts">
	// The source in a review, beside the text: a page of a scan on its lightbox, dark in both
	// themes so the page is the brightest thing on screen, with zoom (also Ctrl and the wheel or
	// a pinch, Ctrl + and Ctrl −, Ctrl 0 to fit; ⌘ on a Mac) and a quarter turn either way for a
	// page scanned sideways. Zooming keeps the point under the pointer where it is (the middle of
	// the pane, from the buttons and keys); the page is moved by dragging it, or with the arrow
	// keys once the page has the focus, and a double click zooms in on a place or back to the
	// pane's width. A hint says so until the reader has zoomed once. Or a text file set as text, with words looked for marked; or a player for a
	// recording or a video. Where no strip of pages runs below, buttons over the scan turn its
	// pages.
	import { flushSync, type Snippet } from 'svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
	import { findTerm } from '#lib/analysis.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import { formatPercent, messages } from '#lib/i18n/index.js';
	import { panStep, scrollableByKeys } from '#lib/keys.js';
	import { modifier, primary, shortcut, zoomsWithWheel } from '#lib/platform.js';
	import { shortcutsHeld } from '#lib/state.svelte.js';
	import { keep, keepFor, recall, recallFor } from '#lib/stored.js';
	import {
		between,
		clampZoom,
		placeUnder,
		scrollToPlace,
		wheelFactor,
		type Place
	} from '#lib/zoom.js';
	import { type Rotation, turn } from '#lib/pdf.js';

	let {
		kind,
		sourceId = null,
		piece = null,
		paging = null,
		onstep,
		preview = null,
		text = null,
		find = '',
		found = false,
		player,
		switchable = null
	}: {
		kind: 'scan' | 'text' | 'player';
		/** The source, whose turn is remembered on this computer. */
		sourceId?: string | null;
		/** The piece shown, after "Source". */
		piece?: string | null;
		paging?: { at: number; count: number } | null;
		onstep?: (delta: number) => void;
		preview?: PagePreview | null;
		text?: string | null;
		/** Words to find in a text file. */
		find?: string;
		/** Whether they are a search's, marked grey, rather than a name pointed at, in yellow. */
		found?: boolean;
		player?: Snippet;
		/** A PDF whose text was read for a correction or a translation: the scan or that text,
		 * switched from the bar. */
		switchable?: { onswitch: () => void } | null;
	} = $props();
	const m = messages();

	let zoom = $state(1);
	/** The reader's quarter turns, for every page of the source, remembered for the sources
	 * turned last so a page scanned sideways stays turned. */
	const ROTATION_KEY = 'kalamu.review.rotation';
	const ROTATIONS: unknown[] = [90, 180, 270];
	// svelte-ignore state_referenced_locally
	let rotation = $state<Rotation>(
		(sourceId &&
			recallFor(ROTATION_KEY, sourceId, (v) => (ROTATIONS.includes(v) ? (v as Rotation) : null))) ||
			0
	);
	function turnBy(by: 1 | -1) {
		rotation = turn(rotation, by);
		if (sourceId) keepFor(ROTATION_KEY, sourceId, rotation === 0 ? null : rotation, 200);
	}
	let desk = $state<HTMLElement | null>(null);

	const still = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

	/** A point in the window the zoom keeps in place; the middle of the pane when none is given. */
	type Anchor = { x: number; y: number };
	function middle(): Anchor | null {
		const box = desk?.getBoundingClientRect();
		return box ? { x: box.left + box.width / 2, y: box.top + box.height / 2 } : null;
	}

	/** The place on the page a gesture zooms about, kept while it goes on at the same point: a
	 * page narrower than the pane stays centred and cannot scroll, so the place drifts from the
	 * pointer for a moment, and is brought back under it once the page is large enough. */
	let aim: { at: Anchor; place: Place; until: number } | null = null;

	/** Sets the zoom, the point of the page under `at` staying under it. The page is stretched
	 * at once (`PageImage`), so the scroll is set in the same frame and nothing jumps. */
	function zoomTo(next: number, at: Anchor | null = middle()) {
		next = clampZoom(next);
		const page = desk?.querySelector('canvas');
		if (!desk || !page || !at || next === zoom) {
			zoom = next;
			return;
		}
		const now = performance.now();
		const same =
			aim && now < aim.until && Math.abs(aim.at.x - at.x) < 3 && Math.abs(aim.at.y - at.y) < 3;
		const place = same && aim ? aim.place : placeUnder(page.getBoundingClientRect(), at);
		aim = { at, place, until: now + 400 };
		flushSync(() => (zoom = next));
		const by = scrollToPlace(page.getBoundingClientRect(), place, at);
		desk.scrollLeft += by.x;
		desk.scrollTop += by.y;
	}

	/** Glides to a zoom over a few frames, for the buttons, the keys and a double click. */
	let glide = 0;
	function glideTo(target: number, at: Anchor | null = middle()) {
		cancelAnimationFrame(glide);
		target = clampZoom(target);
		if (still()) return zoomTo(target, at);
		const from = zoom;
		const start = performance.now();
		const frame = (now: number) => {
			const t = Math.min(1, (now - start) / 180);
			zoomTo(between(from, target, t), at);
			if (t < 1) glide = requestAnimationFrame(frame);
		};
		glide = requestAnimationFrame(frame);
	}

	const zoomStep = (factor: number) => glideTo(zoom * factor);

	// Whether the reader has zoomed with the wheel, a pinch or a double click: until then a hint
	// says how, while the pointer is over the page.
	const HINT_KEY = 'kalamu.review.zoomLearned';
	let learned = $state(recall(HINT_KEY, (v) => v === 1, false));
	let hovering = $state(false);
	function learn() {
		if (learned) return;
		learned = true;
		keep(HINT_KEY, 1);
	}
	const hint = $derived(m.review.zoomHint(modifier()));

	// Ctrl and the wheel zoom the page, not the window; so does a trackpad pinch, which reaches
	// the page as the wheel with Ctrl on every system (⌘ and the wheel too, on a Mac). The
	// wheel's steps are gathered over a frame and applied once, in proportion to how far it
	// turned, so a pinch is as smooth as a mouse's notches. Not passive, to keep the window's own
	// zoom and scrolling out of it.
	let wheel: { factor: number; at: Anchor } | null = null;
	$effect(() => {
		if (!desk || kind !== 'scan') return;
		const onwheel = (e: WheelEvent) => {
			if (!zoomsWithWheel(e)) return;
			e.preventDefault();
			cancelAnimationFrame(glide);
			const factor = wheelFactor(e.deltaY, e.deltaMode);
			const at = { x: e.clientX, y: e.clientY };
			learn();
			if (wheel) {
				wheel = { factor: wheel.factor * factor, at };
				return;
			}
			wheel = { factor, at };
			requestAnimationFrame(() => {
				if (wheel) zoomTo(zoom * wheel.factor, wheel.at);
				wheel = null;
			});
		};
		desk.addEventListener('wheel', onwheel, { passive: false });
		return () => desk?.removeEventListener('wheel', onwheel);
	});

	/* Moving the page by dragging it, with the main button or the wheel's; let go while moving,
	   it glides on a little and slows to a stop. */
	let drag: { x: number; y: number; id: number } | null = null;
	let grabbing = $state(false);
	let velocity = { x: 0, y: 0, at: 0 };
	let coast = 0;

	function canMove(): boolean {
		return !!desk && (desk.scrollWidth > desk.clientWidth || desk.scrollHeight > desk.clientHeight);
	}

	function grab(e: PointerEvent) {
		if ((e.button !== 0 && e.button !== 1) || e.pointerType === 'touch' || !canMove()) return;
		if (!(e.target as Element).closest('.page-image')) return;
		e.preventDefault();
		cancelAnimationFrame(coast);
		drag = { x: e.clientX, y: e.clientY, id: e.pointerId };
		velocity = { x: 0, y: 0, at: performance.now() };
		grabbing = true;
		desk?.setPointerCapture(e.pointerId);
	}

	function move(e: PointerEvent) {
		if (!drag || !desk || e.pointerId !== drag.id) return;
		const dx = e.clientX - drag.x;
		const dy = e.clientY - drag.y;
		desk.scrollLeft -= dx;
		desk.scrollTop -= dy;
		const now = performance.now();
		const dt = Math.max(now - velocity.at, 1);
		// Smoothed, so the last jitter of the hand does not decide the glide.
		velocity = {
			x: 0.7 * (dx / dt) + 0.3 * velocity.x,
			y: 0.7 * (dy / dt) + 0.3 * velocity.y,
			at: now
		};
		drag = { ...drag, x: e.clientX, y: e.clientY };
	}

	function release(e: PointerEvent) {
		if (!drag || e.pointerId !== drag.id) return;
		drag = null;
		grabbing = false;
		desk?.releasePointerCapture(e.pointerId);
		// No glide once the hand has stopped, nor under reduced motion.
		if (still() || performance.now() - velocity.at > 60) return;
		let { x, y } = velocity;
		let last = performance.now();
		const frame = (now: number) => {
			const dt = now - last;
			last = now;
			if (!desk || Math.hypot(x, y) < 0.02) return;
			desk.scrollLeft -= x * dt;
			desk.scrollTop -= y * dt;
			const slow = Math.pow(0.995, dt);
			x *= slow;
			y *= slow;
			coast = requestAnimationFrame(frame);
		};
		coast = requestAnimationFrame(frame);
	}

	/** The arrow keys move the page under the focus, a short step or, with Shift, most of the
	 * pane; Alt and the arrows are left to turn the page. */
	function pan(e: KeyboardEvent) {
		if (e.altKey || e.ctrlKey || e.metaKey || !desk) return;
		const by = panStep(e.key, e.shiftKey, { width: desk.clientWidth, height: desk.clientHeight });
		if (!by || !canMove()) return;
		e.preventDefault();
		cancelAnimationFrame(coast);
		desk.scrollLeft += by.x;
		desk.scrollTop += by.y;
	}

	/** A double click zooms in on the place, or back to the pane's width when zoomed in. */
	function ondblclick(e: MouseEvent) {
		// Measured, not read from the target: while the page is held, clicks reach the desk.
		const page = desk?.querySelector('canvas')?.getBoundingClientRect();
		const onPage =
			!!page &&
			e.clientX >= page.left &&
			e.clientX <= page.right &&
			e.clientY >= page.top &&
			e.clientY <= page.bottom;
		if (!onPage) return;
		learn();
		glideTo(zoom > 1.05 ? 1 : 2.5, { x: e.clientX, y: e.clientY });
	}

	function onkeydown(e: KeyboardEvent) {
		if (kind !== 'scan' || !primary(e) || e.altKey || shortcutsHeld()) return;
		const target = e.target;
		if (target instanceof Element && target.closest('input, textarea, select, [contenteditable]'))
			return;
		if (e.key === '+' || e.key === '=') zoomStep(1.25);
		else if (e.key === '-') zoomStep(1 / 1.25);
		else if (e.key === '0') glideTo(1);
		else return;
		e.preventDefault();
	}
</script>

<svelte:window {onkeydown} />

<section class="pane source" class:stage={kind === 'scan'} aria-label={m.review.source}>
	<div class="pane-bar">
		<span class="title mono">{m.review.source}{piece ? ` · ${piece}` : ''}</span>
		{#if switchable}
			<button
				class="tool wide switch"
				aria-pressed={kind === 'text'}
				title={kind === 'text' ? m.review.showPageHint : m.review.showOriginalHint}
				onclick={switchable.onswitch}
				>{kind === 'text' ? m.review.showPage : m.review.showOriginal}</button
			>
		{/if}
		{#if paging && paging.count > 1 && onstep}
			<span class="paging">
				<button
					class="tool"
					aria-label={m.review.previous}
					title={m.review.previous}
					disabled={paging.at <= 0}
					onclick={() => onstep(-1)}><Icon name="previous" /></button
				>
				<span class="count mono">{paging.at + 1} / {paging.count}</span>
				<button
					class="tool"
					aria-label={m.review.next}
					title={m.review.next}
					disabled={paging.at >= paging.count - 1}
					onclick={() => onstep(1)}><Icon name="chevron" /></button
				>
			</span>
		{/if}
	</div>
	{#if kind === 'scan'}
		<!-- A tab stop of its own, so the keyboard can move the page as the pointer drags it. -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
		<div
			class="desk scan"
			class:movable={zoom > 1}
			class:grabbing
			bind:this={desk}
			role="group"
			tabindex="0"
			aria-label={m.review.pageImage}
			onkeydown={pan}
			onpointerdown={grab}
			onpointermove={move}
			onpointerup={release}
			onpointercancel={release}
			onpointerenter={() => (hovering = true)}
			onpointerleave={() => (hovering = false)}
			{ondblclick}
		>
			<PageImage {preview} {zoom} {rotation} />
		</div>
		{#if !learned && hovering && preview}
			<p class="zoom-hint" role="note">{hint}</p>
		{/if}
		<div class="zoom">
			<button
				class="tool"
				aria-label={m.review.zoomOut}
				title="{m.review.zoomOut} ({shortcut(m.review.zoomOutShortcut)})"
				onclick={() => zoomStep(1 / 1.25)}><Icon name="zoom-out" /></button
			>
			<span class="zoom-value mono" title={hint}>{formatPercent(Math.round(zoom * 100) / 100)}</span
			>
			<button
				class="tool"
				aria-label={m.review.zoomIn}
				title="{m.review.zoomIn} ({shortcut(m.review.zoomInShortcut)})"
				onclick={() => zoomStep(1.25)}><Icon name="zoom-in" /></button
			>
			<span class="zoom-rule" aria-hidden="true"></span>
			<button
				class="tool wide"
				title="{m.review.fitWidth} ({shortcut(m.review.zoomFitShortcut)})"
				onclick={() => glideTo(1)}>{m.review.fitWidth}</button
			>
			<span class="zoom-rule" aria-hidden="true"></span>
			<button
				class="tool"
				aria-label={m.review.turnLeft}
				title={m.review.turnLeft}
				onclick={() => turnBy(-1)}><Icon name="turn-left" /></button
			>
			<button
				class="tool"
				aria-label={m.review.turnRight}
				title={m.review.turnRight}
				onclick={() => turnBy(1)}><Icon name="turn-right" /></button
			>
		</div>
	{:else if kind === 'text'}
		<div class="desk document" lang="" {@attach scrollableByKeys(m.review.source)}>
			{#if text === null}
				<p class="muted">{m.common.loading}</p>
			{:else}
				{#each findTerm(text, find) as part, k (k)}{#if part.hit}<mark class:found>{part.text}</mark
						>{:else}{part.text}{/if}{/each}
			{/if}
		</div>
	{:else}
		<div class="desk">{@render player?.()}</div>
	{/if}
</section>

<style>
	.source {
		border-right: 1px solid var(--line);
		background: var(--surface-1);
	}
	.title {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.stage {
		background: var(--stage);
		border-right-color: var(--stage-line);
	}
	.stage .pane-bar {
		border-bottom-color: var(--stage-line);
	}
	.stage .title {
		color: var(--on-stage-muted);
	}
	.desk {
		overflow: auto;
		min-height: 0;
		padding: var(--space-4);
	}
	.stage .desk {
		padding: var(--space-5) var(--space-5) 4.5rem;
	}
	/* Once the page is larger than the pane, it is moved by dragging it. */
	.scan.movable {
		cursor: grab;
	}
	.scan.grabbing {
		cursor: grabbing;
		user-select: none;
	}
	/* Over the lightbox the focus is drawn in the stage's own light, inside the pane's edge. */
	.scan:focus-visible {
		outline-color: var(--on-stage);
		outline-offset: calc(-1 * var(--focus-width) - 2px);
	}
	/* How to zoom and move, over the lightbox above the zoom bar, until the reader has zoomed. */
	.zoom-hint {
		position: absolute;
		left: 50%;
		bottom: calc(var(--space-4) + var(--control-sm) + var(--space-4));
		transform: translateX(-50%);
		width: max-content;
		max-width: calc(100% - 2 * var(--space-4));
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius-pill);
		background: var(--stage-raised);
		box-shadow: 0 0 0 1px var(--stage-line);
		color: var(--on-stage);
		font-size: var(--type-xs);
		text-align: center;
		pointer-events: none;
		animation: hint-in 200ms ease-out;
	}
	@keyframes hint-in {
		from {
			opacity: 0;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.zoom-hint {
			animation: none;
		}
	}
	.stage .desk :global(.muted) {
		color: var(--on-stage-muted);
	}
	/* A text file, set in the reading face. */
	.document {
		padding: var(--space-5) var(--space-6);
		background: var(--paper);
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.65;
		color: var(--text-body);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	mark {
		background: var(--marker);
		color: var(--on-marker);
		border-radius: 2px;
		box-shadow: 0 0 0 2px var(--marker);
	}
	/* Yellow is where models disagree or a name pointed at; a search's words are grey. */
	mark.found {
		background: var(--found);
		color: inherit;
		box-shadow: 0 0 0 2px var(--found);
	}
	/* The buttons over the lightbox: turning its pages, zooming. */
	.paging {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 2px;
		color: var(--on-stage);
	}
	.count {
		font-size: var(--type-xs);
		color: var(--on-stage-muted);
	}
	.zoom {
		position: absolute;
		left: 50%;
		bottom: var(--space-4);
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 2px;
		padding: var(--space-1);
		border-radius: 9px;
		background: var(--stage-raised);
		box-shadow:
			0 0 0 1px var(--stage-line),
			0 8px 24px rgba(0, 0, 0, 0.35);
		color: var(--on-stage);
	}
	.tool {
		display: grid;
		place-items: center;
		min-width: var(--control-sm);
		height: var(--control-sm);
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.tool.wide {
		padding: 0 0.625rem;
		white-space: nowrap;
	}
	.tool:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.08);
	}
	/* Switching between the page and its text, at the bar's end, before any paging. */
	.switch {
		margin-left: auto;
		color: var(--on-stage);
	}
	.switch + .paging {
		margin-left: 0;
	}
	.source:not(.stage) .switch {
		color: var(--text-secondary);
	}
	.source:not(.stage) .switch:hover {
		background: var(--surface-2);
	}
	.tool:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.tool:focus-visible {
		outline-color: var(--on-stage);
	}
	.zoom-value {
		min-width: 3rem;
		text-align: center;
		font-size: var(--type-xs);
	}
	.zoom-rule {
		width: 1px;
		height: 1.125rem;
		margin: 0 var(--space-1);
		background: var(--stage-line);
	}
	/* A narrow window stacks the panes: the source on top. */
	@media (max-width: 1000px) {
		.source {
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
	}
</style>
