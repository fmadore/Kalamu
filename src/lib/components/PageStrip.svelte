<script lang="ts" module>
	export type StripItem = {
		/** The piece's index in the source, for its thumbnail and for choosing it. */
		index: number;
		/** Its full name, for screen readers ("Page 3"). */
		label: string;
		/** Its short name in the strip ("3", "3l"). */
		number: string;
		done: boolean;
	};
</script>

<script lang="ts">
	// The strip of a source's pieces under a review: a scan's pages small with their numbers
	// below, a recording's or a text's pieces by number; the ring marks the one open, a tick the
	// ones reviewed. With a tally, how many are reviewed.
	import Icon from './Icon.svelte';
	import PageThumb from './PageThumb.svelte';
	import { messages } from '#lib/i18n/index.js';

	let {
		jobId,
		sourceId,
		items,
		current,
		paged,
		tally = false,
		onselect
	}: {
		jobId: string;
		sourceId: string;
		items: StripItem[];
		current: number;
		/** A scan or images: each piece drawn as its page. */
		paged: boolean;
		tally?: boolean;
		onselect: (index: number) => void;
	} = $props();
	const m = messages();

	let strip = $state<HTMLElement | null>(null);
	const reviewed = $derived(items.filter((i) => i.done).length);

	// The open piece stays in view when it is changed from elsewhere (the arrows, an entry).
	$effect(() => {
		const index = current;
		strip
			?.querySelector<HTMLElement>(`[data-index="${index}"]`)
			?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
	});
</script>

<footer class="pages" aria-label={m.review.pages}>
	<ol class="strip" bind:this={strip}>
		{#each items as item (item.index)}
			<li>
				<button
					class="piece"
					class:paged
					class:current={item.index === current}
					class:done={item.done}
					data-index={item.index}
					aria-current={item.index === current ? 'page' : undefined}
					aria-label={item.done ? m.review.pieceReviewed(item.label) : item.label}
					onclick={() => onselect(item.index)}
				>
					{#if paged}<PageThumb {jobId} {sourceId} index={item.index} />{/if}
					<span class="number"
						>{#if item.done}<Icon name="check" size={12} />{/if}<span class="mono"
							>{item.number}</span
						></span
					>
				</button>
			</li>
		{/each}
	</ol>
	{#if tally}
		<div class="tally">
			<span>{m.review.reviewedOf(reviewed, items.length)}</span>
			<span class="tally-bar" aria-hidden="true"
				><i style:width="{(100 * reviewed) / Math.max(items.length, 1)}%"></i></span
			>
		</div>
	{/if}
</footer>

<style>
	.pages {
		display: flex;
		align-items: center;
		gap: var(--space-5);
		padding: var(--space-2) var(--space-4);
		border-top: 1px solid var(--line);
		background: var(--surface-1);
	}
	.strip {
		flex: 1;
		min-width: 0;
		list-style: none;
		margin: 0;
		padding: 2px;
		display: flex;
		gap: var(--space-1);
		overflow-x: auto;
	}
	.piece {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 3px;
		min-width: 2.25rem;
		height: 2rem;
		padding: 0 var(--space-2);
		border: 1px solid var(--line);
		border-radius: 6px;
		background: var(--surface-2);
		color: var(--text-secondary);
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.piece:hover {
		border-color: var(--line-hover);
	}
	.piece.done {
		color: var(--ok-text);
	}
	.piece.current {
		border-color: var(--text-bright);
		box-shadow: inset 0 0 0 1px var(--text-bright);
		color: var(--text-bright);
	}
	.number {
		display: inline-flex;
		align-items: center;
		gap: 3px;
	}
	/* A scan's pages: the page small, its number below; the ring marks the one open. */
	.piece.paged {
		flex-direction: column;
		gap: 3px;
		height: auto;
		min-width: 0;
		padding: 4px 6px 2px;
		border-color: transparent;
		background: transparent;
	}
	.piece.paged:hover {
		border-color: var(--line-hover);
	}
	.piece.paged.current {
		border-color: var(--text-bright);
		box-shadow: inset 0 0 0 1px var(--text-bright);
		background: var(--surface-2);
	}
	.tally {
		flex: none;
		display: grid;
		gap: var(--space-1);
		min-width: 12rem;
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-bright);
	}
	.tally-bar {
		display: block;
		height: 4px;
		border-radius: 2px;
		background: var(--line);
		overflow: hidden;
	}
	.tally-bar i {
		display: block;
		height: 100%;
		background: var(--ok);
	}
	@media (max-width: 820px) {
		.tally {
			display: none;
		}
	}
</style>
