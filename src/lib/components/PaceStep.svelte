<script lang="ts">
	// New job's pace, offered when some of the chosen models' work goes through a batch service:
	// fast, or economy at its lower price, each with what the job costs at it, and why a model
	// gets no discount in it.
	import ChoiceStrip from './ChoiceStrip.svelte';
	import TermLink from './TermLink.svelte';
	import type { Catalog } from '#lib/bindings/Catalog.js';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { notBatchedOf } from '#lib/newJobEstimate.svelte.js';

	let {
		pace = $bindable(),
		estimates,
		catalog,
		totalFor
	}: {
		pace: Pace;
		/** The estimate for each model of the job. */
		estimates: EstimateRow[];
		catalog: Catalog;
		/** The whole job at a pace, as the engine sums it; null while it cannot be known. */
		totalFor: (at: Pace) => number | null;
	} = $props();
	const m = messages();

	/** Why some models get no discount in economy mode, as the engine says. */
	const notBatched = $derived(notBatchedOf(estimates, catalog, m));

	/** The paces, each with what the job costs at it. */
	const paceOptions = $derived(
		(['fast', 'economy'] as const).map((value) => {
			const total = totalFor(value);
			return {
				value,
				label: m.newJob.pace[value],
				hint: m.newJob.pace[`${value}_hint`],
				price: total !== null ? formatUsd(total) : null
			};
		})
	);
</script>

<section class="step block" aria-labelledby="pace-title">
	<h2 id="pace-title">{m.newJob.paceTitle}<TermLink term="economy" /></h2>
	<ChoiceStrip bind:value={pace} options={paceOptions} />
	{#if pace === 'economy'}
		{#each notBatched as sentence (sentence)}
			<p class="hint">{sentence}</p>
		{/each}
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
		max-width: 60ch;
	}
</style>
