<script lang="ts">
	// The readings of one place where the texts differ, over the text under its words (over them
	// when the pane has no room below): the words in the text, what each witness reads instead,
	// and a box for the researcher's own reading. Taking one stores it as a correction. Dragged
	// by its head, or moved with the arrow keys from its grip, it moves aside to show the
	// sentence it covered.
	import Icon from './Icon.svelte';
	import type { Entry } from '#lib/apparatus.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		id,
		entry,
		place,
		heading,
		inUse,
		witnessName,
		pane,
		onuse,
		onclose,
		onenter,
		onleave,
		onpin
	}: {
		/** The id the words that open it point to. */
		id: string;
		entry: Entry;
		/** Which place it is, of how many. */
		place: { n: number; total: number };
		heading: string;
		/** Whose words are in the text: the researcher's corrections or the leading set-up. */
		inUse: string;
		witnessName: (siglum: string) => string;
		/** The scrolling pane of the text, which holds the words and the readings. */
		pane: HTMLElement;
		onuse: (text: string) => void;
		onclose: () => void;
		onenter: () => void;
		onleave: () => void;
		/** Called when the readings take the focus, which keeps them open. */
		onpin: () => void;
	} = $props();
	const m = messages();

	let own = $state('');
	let box = $state<HTMLElement | null>(null);
	let at = $state<{ top: number; left: number } | null>(null);
	/** Set once the researcher moves the readings aside to see the sentence under them: they
	 * then stay where they were put. */
	let moved = $state(false);
	let drag = $state<{ x: number; y: number; top: number; left: number } | null>(null);

	/** Puts the readings at a place in the pane, kept inside it. */
	function moveTo(top: number, left: number) {
		if (!box) return;
		const { offsetWidth: w, offsetHeight: h } = box;
		at = {
			top: Math.max(0, Math.min(top, pane.scrollHeight - h)),
			left: Math.max(0, Math.min(left, pane.clientWidth - w))
		};
		moved = true;
	}

	function startDrag(e: PointerEvent) {
		if (e.button !== 0 || !at || (e.target as HTMLElement).closest('button:not(.grip)')) return;
		e.preventDefault();
		onpin();
		drag = { x: e.clientX, y: e.clientY, top: at.top, left: at.left };
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onDrag(e: PointerEvent) {
		if (drag) moveTo(drag.top + e.clientY - drag.y, drag.left + e.clientX - drag.x);
	}

	function endDrag() {
		drag = null;
	}

	/** The arrow keys move the readings from their handle, further with Shift. */
	function nudge(e: KeyboardEvent) {
		const step = e.shiftKey ? 64 : 16;
		const by = {
			ArrowUp: [-step, 0],
			ArrowDown: [step, 0],
			ArrowLeft: [0, -step],
			ArrowRight: [0, step]
		}[e.key];
		if (!by || !at) return;
		e.preventDefault();
		moveTo(at.top + by[0], at.left + by[1]);
	}

	function placeBox() {
		const words = pane.querySelector<HTMLElement>(`[data-entry="${entry.id}"]`);
		if (!box || !words || moved) return;
		const area = pane.getBoundingClientRect();
		const last = Array.from(words.getClientRects()).at(-1) ?? words.getBoundingClientRect();
		const { offsetWidth: w, offsetHeight: h } = box;
		const roomBelow = area.bottom - last.bottom > h + 16;
		const roomAbove = last.top - area.top > h + 16;
		at = {
			top:
				pane.scrollTop +
				(roomBelow || !roomAbove ? last.bottom - area.top + 8 : last.top - area.top - h - 8),
			left: Math.max(8, Math.min(last.left - area.left - 12, pane.clientWidth - w - 8))
		};
	}

	$effect(() => {
		if (!box) return;
		placeBox();
		const watch = new ResizeObserver(() => placeBox());
		watch.observe(pane);
		watch.observe(box);
		return () => watch.disconnect();
	});

	function useOwn() {
		if (own.trim()) onuse(own.trim());
	}
</script>

<div
	bind:this={box}
	{id}
	class="readings"
	role="dialog"
	tabindex="-1"
	aria-label={m.review.pickReading}
	style:top="{at?.top ?? 0}px"
	style:left="{at?.left ?? 0}px"
	style:visibility={at ? null : 'hidden'}
	onpointerenter={onenter}
	onpointerleave={onleave}
	onfocusin={onpin}
>
	<!-- The head is the handle: dragged, it moves the readings aside to show the sentence
	     under them; its grip does the same from the keyboard. -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="head"
		class:dragging={drag !== null}
		onpointerdown={startDrag}
		onpointermove={onDrag}
		onpointerup={endDrag}
		onpointercancel={endDrag}
	>
		<button
			class="grip"
			aria-label={m.review.moveReadings}
			title={m.review.moveReadings}
			onkeydown={nudge}><Icon name="grip" size={14} /></button
		>
		<span>{heading}</span>
		<span class="mono">{m.review.differenceOf(place.n, place.total)}</span>
		<button class="btn sm icon ghost" aria-label={m.common.close} onclick={onclose}
			><Icon name="close" size={14} /></button
		>
	</div>
	<ul class="options">
		<li class="option">
			<span class="option-text in-text">{entry.lemma || m.review.omitted}</span>
			<span class="option-who mono">{inUse}</span>
			<span class="option-end chip">{m.review.inUse}</span>
		</li>
		{#each entry.readings as r (r.siglum)}
			<li>
				<button class="option pick" onclick={() => onuse(r.text)}>
					<span class="option-text">{r.text || m.review.omitted}</span>
					<span class="option-who mono">{witnessName(r.siglum)}</span>
					<span class="option-end use">{m.review.useReading}</span>
				</button>
			</li>
		{/each}
		<li class="own">
			<input
				type="text"
				bind:value={own}
				aria-label={m.review.ownReading}
				placeholder={m.review.ownReadingPlaceholder}
				onkeydown={(e) => {
					if (e.key === 'Enter' && own.trim()) {
						e.preventDefault();
						useOwn();
					}
				}}
			/>
			<button class="btn sm primary" disabled={!own.trim()} onclick={useOwn}
				>{m.review.apply}</button
			>
		</li>
	</ul>
	<p class="note">{m.review.readingKept}</p>
</div>

<style>
	.readings {
		position: absolute;
		z-index: 8;
		width: min(25rem, calc(100% - 16px));
		padding: var(--space-3);
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-flyout);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		line-height: 1.45;
		color: var(--text-bright);
		display: grid;
		gap: var(--space-3);
		animation: rise 120ms ease-out;
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(-3px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.readings {
			animation: none;
		}
	}
	.head {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin: calc(-1 * var(--space-1)) 0;
		font-size: var(--type-xs);
		color: var(--text-muted);
		cursor: grab;
		user-select: none;
		touch-action: none;
	}
	.head.dragging {
		cursor: grabbing;
	}
	.head > .grip + span {
		flex: 1;
	}
	.grip {
		display: grid;
		place-items: center;
		width: 1.5rem;
		height: 1.75rem;
		margin-left: calc(-1 * var(--space-2));
		padding: 0;
		border: 0;
		border-radius: var(--radius-tag);
		background: none;
		color: var(--text-muted);
		cursor: inherit;
	}
	.grip:hover,
	.grip:focus-visible,
	.dragging .grip {
		color: var(--text-bright);
		background: var(--wash);
	}
	.options {
		list-style: none;
		margin: 0;
		padding: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		overflow: hidden;
	}
	.options > li + li {
		border-top: 1px solid var(--line-soft);
	}
	/* A reading: its words over who reads them, and what choosing it does. */
	.option {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		column-gap: var(--space-3);
		width: 100%;
		padding: var(--space-2) var(--space-3);
	}
	.option-text {
		font-family: var(--font-text);
		font-size: var(--type-lg);
		overflow-wrap: anywhere;
	}
	.option-text.in-text {
		font-weight: 600;
	}
	.option-who {
		grid-column: 1;
		font-size: var(--type-xs);
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.option-end {
		grid-column: 2;
		grid-row: 1 / span 2;
	}
	.pick {
		border: 0;
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.pick:hover,
	.pick:focus-visible {
		background: var(--wash);
	}
	.pick:focus-visible {
		outline-offset: -2px;
	}
	.use {
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-secondary);
	}
	.pick:hover .use,
	.pick:focus-visible .use {
		color: var(--text-bright);
		text-decoration: underline;
		text-underline-offset: 0.2em;
	}
	.own {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
	}
	.own input {
		min-height: var(--control-sm);
		font-family: var(--font-text);
		font-size: var(--type-md);
	}
	.note {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
</style>
