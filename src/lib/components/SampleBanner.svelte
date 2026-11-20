<script lang="ts">
	// A sample's page, once pages are read (#215): what the sample read, then one button per
	// set-up to read every page with it, priced with the pages already read at no cost, and one
	// to try another model on the same pages.
	import SampleCompareDialog from './SampleCompareDialog.svelte';
	import type { Job } from '#lib/bindings/Job.js';
	import type { RestEstimate } from '#lib/bindings/RestEstimate.js';
	import { modelNameOf } from '#lib/catalogText.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { SampleNext } from '#lib/sampleNext.svelte.js';
	import { app } from '#lib/state.svelte.js';

	let { job }: { job: Job } = $props();
	const m = messages();
	const t = m.jobs.sample;

	// svelte-ignore state_referenced_locally
	const next = new SampleNext(job);
	next.load();
	let comparing = $state(false);

	const pages = $derived(job.settings.sample_pages ?? 0);
	/** Pages left to read with a set-up: the reading's, not a second step's. */
	const leftOf = (rest: RestEstimate) => rest.estimate.rows.find((r) => !r.merge)?.pages ?? 0;
	const nameOf = (provider: string, model: string) =>
		app.catalog ? modelNameOf(app.catalog, provider, model) : model;
	/** The set-ups with pages left, or whose estimate is not known yet. */
	const offered = $derived(
		job.setups.filter((s) => {
			const rest = next.rests[s.id];
			return !rest || leftOf(rest) > 0;
		})
	);
</script>

<section class="sample" aria-labelledby="sample-head">
	<p>
		<strong id="sample-head">{job.settings.sample_spread ? t.spread(pages) : t.first(pages)}</strong
		>
		{t.next}
	</p>
	{#if offered.length > 0}
		<ul class="rests">
			{#each offered as s (s.id)}
				{@const rest = next.rests[s.id]}
				<li>
					<button class="btn primary sm" disabled={next.busy} onclick={() => next.runRest(s)}
						>{t.runRest(nameOf(s.provider, s.model))}</button
					>
					{#if rest}
						<span class="price">
							{t.left(leftOf(rest))}, {rest.estimate.total_usd !== null
								? t.about(formatUsd(rest.estimate.total_usd))
								: m.newJob.estimateUnknown}.
							{#if rest.cached_pages > 0}{t.free(rest.cached_pages)}{/if}
						</span>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
	<div>
		<button class="btn sm" disabled={next.busy} onclick={() => (comparing = true)}
			>{m.newJob.compareAdd}</button
		>
	</div>
	{#if next.error}<p class="chip danger" role="alert">{next.error}</p>{/if}
</section>

{#if comparing}
	<SampleCompareDialog {job} onclose={() => (comparing = false)} />
{/if}

<style>
	.sample {
		display: grid;
		gap: var(--space-3);
		padding: var(--space-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		font-size: var(--type-sm);
	}
	p {
		margin: 0;
		max-width: 70ch;
		line-height: 1.5;
	}
	.rests {
		display: grid;
		gap: var(--space-2);
		margin: 0;
		padding: 0;
		list-style: none;
	}
	li {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
	}
	.price {
		color: var(--text-secondary);
	}
</style>
