<script lang="ts">
	// New job's pace, offered when one of the chosen models goes through a batch service: fast,
	// or economy at its lower price, each with what the job costs at it.
	import ChoiceStrip from './ChoiceStrip.svelte';
	import TermLink from './TermLink.svelte';
	import type { Catalog } from '#lib/bindings/Catalog.js';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import { formatUsd, listNames, messages } from '#lib/i18n/index.js';

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

	const notBatched = $derived(
		estimates
			.filter((r) => r.estimate.economy_usd === null)
			.map(
				(r) =>
					catalog.models.find((x) => x.provider === r.provider && x.id === r.model)?.name ?? r.model
			)
	);

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
	{#if pace === 'economy' && notBatched.length > 0}
		<p class="hint">{m.newJob.paceNotBatched(listNames(notBatched))}</p>
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
