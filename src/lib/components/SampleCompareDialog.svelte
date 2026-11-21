<script lang="ts">
	// "Compare with another model" from a sample's page (#215): the model that will read the same
	// pages beside the sample's, first one with a key the sample does not use, what it costs, and
	// Start. The sample's set-ups come from the cache, so only the new model is paid for; the
	// comparison is a job of several set-ups like any other, reviewed side by side.
	import Dialog from './Dialog.svelte';
	import ModelPicker from './ModelPicker.svelte';
	import type { Job } from '#lib/bindings/Job.js';
	import { modelNameOf } from '#lib/catalogText.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { rulesFor } from '#lib/newJob.js';
	import { followEstimate } from '#lib/newJobEstimate.svelte.js';
	import { modelsFor, providerDefault, toRequest } from '#lib/newJobModels.js';
	import { compareDraft, compareSample, sampleInput } from '#lib/sampleNext.svelte.js';
	import { app, hasKey } from '#lib/state.svelte.js';
	import { recallModelsBy } from '#lib/tiers.js';

	let { job, onclose }: { job: Job; onclose: () => void } = $props();
	const m = messages();
	const t = m.jobs.sample;

	const catalog = $derived(app.catalog!);
	const rule = $derived(rulesFor(app.rules, job.task));
	const input = $derived(sampleInput(job));
	// svelte-ignore state_referenced_locally
	let draft = $state(compareDraft(job));
	let cost = $state<number | null>(null);
	let busy = $state(false);
	let error = $state<string | null>(null);
	const by = recallModelsBy();

	const providers = $derived(
		catalog.providers
			.filter((p) => modelsFor(catalog, rule, p.id, input).length > 0)
			.map((p) => p.id)
	);
	const first = $derived(job.setups[0]);
	const usedAlready = $derived(
		job.setups.some((s) => s.provider === draft.provider && s.model === draft.model)
	);

	followEstimate(
		() => ({
			task: job.task,
			sources: job.sources,
			setups: draft.model ? [toRequest(catalog, job.task, draft)] : [],
			sample_pages: job.settings.sample_pages
		}),
		(estimate) => (cost = estimate?.total_usd ?? null)
	);

	async function start() {
		busy = true;
		error = null;
		try {
			if (await compareSample(job, $state.snapshot(draft))) onclose();
		} catch (e) {
			error = describeError(e, m);
		} finally {
			busy = false;
		}
	}
</script>

<Dialog labelledby="sample-compare-title" size="medium" onescape={onclose}>
	<h2 id="sample-compare-title">{m.newJob.compareAdd}</h2>
	{#if first}
		<p class="lede">{t.compareLede(modelNameOf(catalog, first.provider, first.model))}</p>
	{/if}
	<div class="picker">
		<ModelPicker
			{catalog}
			{by}
			{providers}
			models={(provider) => modelsFor(catalog, rule, provider, input)}
			defaultFor={(provider) => providerDefault(catalog, rule, provider, input)}
			bind:provider={draft.provider}
			bind:model={draft.model}
		/>
	</div>
	{#if cost !== null && !usedAlready}<p class="cost">{t.compareCost(formatUsd(cost))}</p>{/if}
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	<div class="actions">
		<button class="btn" onclick={onclose}>{m.common.cancel}</button>
		<button
			class="btn primary"
			disabled={busy || !draft.model || !hasKey(draft.provider) || usedAlready}
			onclick={start}>{t.compareStart}</button
		>
	</div>
</Dialog>

<style>
	h2 {
		margin: 0;
		font-size: var(--type-lg);
	}
	p {
		margin: 0;
	}
	.lede {
		color: var(--text-secondary);
		font-size: var(--type-sm);
		line-height: 1.5;
		max-width: 60ch;
	}
	.picker {
		display: grid;
		gap: var(--space-3);
		max-width: 28rem;
	}
	.cost {
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
