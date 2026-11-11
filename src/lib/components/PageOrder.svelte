<script lang="ts">
	// Several images in New job: each a document of its own, or the pages of one document, named
	// and put in order. The order comes from the engine (the numbers in the names read as
	// numbers); the researcher moves pages with buttons, or by dragging them. Numbers that skip
	// or repeat are pointed out beside their page, never fixed.
	import ChoiceStrip from './ChoiceStrip.svelte';
	import Icon from './Icon.svelte';
	import type { ImageDocument } from '#lib/bindings/ImageDocument.js';
	import { messages } from '#lib/i18n/index.js';
	import { type Grouping, movePage, noteTexts } from '#lib/pageOrder.js';

	let {
		grouping = $bindable(),
		doc = $bindable(),
		name = $bindable(),
		sorting,
		onnatural
	}: {
		grouping: Grouping;
		/** The document the engine made of the images, once it has. */
		doc: ImageDocument | null;
		name: string;
		/** The engine is putting the pages in order. */
		sorting: boolean;
		/** Puts the pages back in the order of their names. */
		onnatural: () => void;
	} = $props();

	const m = messages();
	const g = m.newJob.grouping;
	const options = (['separate', 'one'] as const).map((value) => ({
		value,
		label: g[value],
		hint: g[`${value}_hint`]
	}));
	const pages = $derived(doc?.source.pages ?? []);
	/** The page being dragged, by its place. */
	let dragged = $state<number | null>(null);
	/** The place a dragged page would go. */
	let over = $state<number | null>(null);

	function move(from: number, to: number) {
		if (!doc) return;
		doc.source.pages = movePage(pages, from, to);
	}

	/** Moves a page by a button and keeps the focus on the same button of the page moved, so the
	 * keyboard can go on moving it. */
	function step(from: number, by: -1 | 1, button: HTMLButtonElement) {
		const to = from + by;
		move(from, to);
		const which = button.dataset.move;
		requestAnimationFrame(() => {
			const list = button.closest('ol');
			const target = list?.querySelectorAll<HTMLButtonElement>(`button[data-move="${which}"]`)[to];
			(target && !target.disabled ? target : list?.querySelectorAll('button')[to * 2])?.focus();
		});
	}
</script>

<section class="step block" aria-labelledby="grouping-title">
	<h2 id="grouping-title">{g.title}</h2>
	<ChoiceStrip bind:value={grouping} {options} />
	{#if grouping === 'one'}
		<label class="field">
			<span>{g.name}</span>
			<input type="text" bind:value={name} spellcheck="false" />
			<span class="hint">{g.nameHint}</span>
		</label>
		<div class="order-head">
			<h3 id="order-title">{g.order}</h3>
			<button class="btn sm ghost" onclick={onnatural} disabled={!doc}
				><Icon name="replay" size={14} />{g.naturalOrder}</button
			>
		</div>
		<p class="hint">{g.orderHint}</p>
		{#if sorting && !doc}
			<p class="hint">{g.sorting}</p>
		{/if}
		{#if doc}
			<ol class="pages" aria-labelledby="order-title">
				{#each pages as page, i (page.id)}
					{@const notes = noteTexts(
						doc.notes.find((n) => n.page_id === page.id),
						m
					)}
					<li
						draggable="true"
						class:dragging={dragged === i}
						class:over={over === i && dragged !== i}
						class:noted={notes.length > 0}
						ondragstart={(e) => {
							dragged = i;
							e.dataTransfer?.setData('text/plain', String(i));
							if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
						}}
						ondragover={(e) => {
							if (dragged === null) return;
							e.preventDefault();
							over = i;
						}}
						ondrop={(e) => {
							e.preventDefault();
							if (dragged !== null) move(dragged, i);
							dragged = null;
							over = null;
						}}
						ondragend={() => {
							dragged = null;
							over = null;
						}}
					>
						<span class="grip" aria-hidden="true"><Icon name="grip" size={14} /></span>
						<span class="number mono">{g.position(i + 1)}</span>
						<span class="name mono">{page.file_name}</span>
						<span class="moves">
							<button
								class="btn sm icon ghost"
								data-move="up"
								aria-label={g.moveUp(page.file_name)}
								title={g.moveUp(page.file_name)}
								disabled={i === 0}
								onclick={(e) => step(i, -1, e.currentTarget)}><Icon name="up" size={14} /></button
							>
							<button
								class="btn sm icon ghost"
								data-move="down"
								aria-label={g.moveDown(page.file_name)}
								title={g.moveDown(page.file_name)}
								disabled={i === pages.length - 1}
								onclick={(e) => step(i, 1, e.currentTarget)}><Icon name="down" size={14} /></button
							>
						</span>
						{#each notes as note (note)}
							<span class="note">{note}</span>
						{/each}
					</li>
				{/each}
			</ol>
			<p class="hint">{g.formats}</p>
		{/if}
	{/if}
</section>

<style>
	.block {
		display: grid;
		gap: var(--space-3);
	}
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 75ch;
	}
	.field {
		display: grid;
		gap: var(--space-1);
		max-width: 40rem;
	}
	.order-head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}
	.order-head h3 {
		font-size: var(--type-sm);
	}
	.pages {
		list-style: none;
		padding: 0;
		margin: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		max-height: 24rem;
		overflow-y: auto;
	}
	.pages li {
		display: grid;
		grid-template-columns: auto auto minmax(0, 1fr) auto;
		gap: var(--space-1) var(--space-3);
		align-items: center;
		padding: var(--space-1) var(--space-2) var(--space-1) var(--space-3);
		border-top: 1px solid var(--line-soft);
		cursor: grab;
	}
	.pages li:first-child {
		border-top: 0;
	}
	.pages li.dragging {
		opacity: 0.5;
	}
	.pages li.over {
		box-shadow: inset 0 2px 0 var(--accent, var(--text-bright));
	}
	.grip {
		color: var(--text-muted);
		display: inline-flex;
	}
	.number {
		font-size: var(--type-xs);
		color: var(--text-secondary);
		min-width: 4.5em;
	}
	.name {
		overflow-wrap: anywhere;
		color: var(--text-bright);
	}
	.moves {
		display: inline-flex;
		gap: var(--space-1);
	}
	.note {
		grid-column: 3 / -1;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-control);
		background: var(--warn-wash);
		color: var(--warn-text);
		font-size: var(--type-xs);
	}
	@media (prefers-reduced-motion: no-preference) {
		.pages li {
			transition: background-color 120ms ease-out;
		}
	}
</style>
