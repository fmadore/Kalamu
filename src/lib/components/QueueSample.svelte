<script lang="ts">
	// A random sample of the pages nothing flags, beside what to check: how many to draw, the
	// seed that decides the draw (shown, kept with the project, so the same pages come back and
	// can be cited), how many of those drawn are reviewed, and the pages themselves. Rust draws
	// them; drawing again, with a new seed, is the researcher's explicit choice.
	import QueueRow from './QueueRow.svelte';
	import type { QueueItem } from '#lib/bindings/QueueItem.js';
	import type { SampleCounts } from '#lib/bindings/SampleCounts.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { itemKey } from '#lib/queue.js';

	let {
		sample,
		items,
		onopen,
		onkeep,
		ondraw,
		onremove
	}: {
		sample: SampleCounts | null;
		items: QueueItem[];
		onopen: (item: QueueItem) => void;
		onkeep: (item: QueueItem, note: string | null) => Promise<void>;
		ondraw: (size: number) => Promise<void>;
		onremove: () => Promise<void>;
	} = $props();
	const m = messages();
	const s = m.queue.sample;
	const MAX = 1000;

	// svelte-ignore state_referenced_locally
	let size = $state(sample?.size ?? 20);
	let busy = $state(false);

	async function run(work: () => Promise<void>) {
		busy = true;
		try {
			await work();
		} finally {
			busy = false;
		}
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		void run(() => ondraw(Math.min(MAX, Math.max(1, Math.round(size) || 1))));
	}
</script>

<section class="sample" aria-labelledby="sample-title">
	<h3 id="sample-title">
		{s.title}
		{#if sample}<span class="num muted">{sample.drawn}</span>{/if}
	</h3>
	<p class="hint">{s.hint}</p>
	<form class="draw" onsubmit={submit}>
		<label>
			<span>{s.size}</span>
			<input type="number" min="1" max={MAX} step="1" bind:value={size} />
		</label>
		<button
			class="btn sm"
			type="submit"
			disabled={busy}
			title={sample ? s.drawAgainHint : undefined}>{sample ? s.drawAgain : s.draw}</button
		>
		{#if sample}
			<button class="btn ghost sm" type="button" disabled={busy} onclick={() => run(onremove)}
				>{s.remove}</button
			>
		{/if}
	</form>
	{#if sample}
		<p class="facts num">
			<span>{s.summary(sample.drawn, sample.candidates)}</span>
			<span title={s.seedHint}
				>{s.seed(sample.seed)}, {s.drawnOn(formatDateTime(sample.drawn_at))}</span
			>
			<span>{s.reviewed(sample.reviewed, sample.drawn)}</span>
		</p>
		{#if sample.drawn === 0}
			<p class="muted">{s.none}</p>
		{:else}
			<ul class="items">
				{#each items as item (itemKey(item))}
					<QueueRow {item} {onopen} {onkeep} />
				{/each}
			</ul>
		{/if}
	{/if}
</section>

<style>
	.sample {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-3);
	}
	h3 {
		margin: 0;
		font-size: var(--type-sm);
		font-weight: 600;
	}
	.hint {
		margin: 0;
		max-width: 48rem;
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.draw {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
	}
	.draw label {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-secondary);
	}
	.draw input {
		width: 6rem;
	}
	.facts {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-4);
		margin: 0;
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.items {
		display: grid;
		margin: 0;
		padding: 0;
		list-style: none;
		border-top: 1px solid var(--line);
	}
</style>
