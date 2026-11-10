<script lang="ts">
	// Under a table drawn a page at a time: how many of its rows are drawn, and the button that
	// draws more. Nothing shows while every row is drawn.
	import { messages } from '#lib/i18n/index.js';
	import type { Paging } from '#lib/paging.svelte.js';

	let {
		paging,
		total,
		table = null
	}: {
		paging: Paging;
		/** Every row the table would hold, filtered as it is. */
		total: number;
		/** The table, whose first new row takes the focus. */
		table?: HTMLElement | null;
	} = $props();
	const m = messages();
	const next = $derived(paging.next(total));
</script>

{#if next > 0}
	<div class="more-rows">
		<span class="num">{m.common.shownRows(paging.limit, total)}</span>
		<button class="btn ghost sm" onclick={() => paging.more(table)}
			>{m.common.showMore(next)}</button
		>
	</div>
{/if}

<style>
	.more-rows {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-3);
		color: var(--text-muted);
		font-size: var(--type-sm);
	}
</style>
