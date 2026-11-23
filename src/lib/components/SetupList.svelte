<script lang="ts">
	// The model set-ups of a new job: one for a normal job, several to compare, each named by the
	// siglum it will carry in the review.
	import TermLink from './TermLink.svelte';
	import Icon from './Icon.svelte';
	import ModelsByToggle from './ModelsByToggle.svelte';
	import SetupCard from './SetupCard.svelte';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';
	import { type Draft, modelsFor, type Needs, newDraft } from '#lib/newJobModels.js';
	import { rulesFor } from '#lib/newJob.js';
	import { app, go, hasKey } from '#lib/state.svelte.js';
	import { keepModelsBy, recallModelsBy } from '#lib/tiers.js';

	let {
		drafts = $bindable(),
		task,
		presets,
		input,
		projectNames
	}: {
		drafts: Draft[];
		task: Task;
		presets: Preset[];
		/** What every model must read, such as YouTube links; any input of the task when absent. */
		input?: Needs;
		/** The names of the project the job is started from, for a model that spells them. */
		projectNames?: () => Promise<string[]>;
	} = $props();

	const m = messages();
	const catalog = $derived(app.catalog!);
	/** The engine's rules for the task: which models do it. */
	const rule = $derived(rulesFor(app.rules, task));
	const providers = $derived(
		catalog.providers
			.filter((p) => modelsFor(catalog, rule, p.id, input).length > 0)
			.map((p) => p.id)
	);
	/** By provider or by tier, as the researcher last listed them. */
	let modelsBy = $state(recallModelsBy());
	$effect(() => keepModelsBy(modelsBy));

	function addSetup() {
		const used = drafts.map((d) => `${d.provider}/${d.model}`);
		// A comparison asks the same thing of another model: the new set-up starts on the prompt
		// (or analysis) the others use, not on the task's default.
		const shared = drafts.find((d) => presets.some((p) => p.id === d.presetId))?.presetId;
		drafts = [
			...drafts,
			newDraft(catalog, rule, shared ?? presets[0]?.id ?? null, hasKey, used, input)
		];
	}
</script>

<section class="step block" aria-labelledby="setups-title">
	<div class="head">
		<h2 id="setups-title">{m.newJob.setupsTitle}<TermLink term="setup" /></h2>
		<span class="by">
			<ModelsByToggle bind:by={modelsBy} /><TermLink term="tier" />
		</span>
	</div>
	{#if !app.keys.some((k) => k.saved)}
		<p class="warning">
			{m.newJob.noProvider}
			<button class="btn sm" onclick={() => go({ name: 'settings', tab: 'keys' })}>
				{m.nav.settings}
			</button>
		</p>
	{/if}
	<!-- One set-up lays its fields in a row; two or more stand side by side, one column per
	     witness, so the set-ups read across like the sigla key in the review. -->
	<div class="setups" class:compare={drafts.length > 1}>
		{#each drafts as _, i (i)}
			<SetupCard
				bind:draft={drafts[i]}
				index={i}
				compare={drafts.length > 1}
				{task}
				{presets}
				{input}
				{projectNames}
				{rule}
				{providers}
				by={modelsBy}
				onremove={() => (drafts = drafts.filter((_, j) => j !== i))}
			/>
		{/each}
		{#if drafts.length > 1}
			<button class="add-column" onclick={addSetup}>
				<Icon name="plus" size={18} />{m.newJob.compareAdd}
			</button>
		{/if}
	</div>
	{#if drafts.length === 1}
		<div class="add">
			<button class="btn" onclick={addSetup}>
				<Icon name="plus" size={16} />{m.newJob.compareAdd}
			</button>
			<p class="muted small">{m.newJob.setupsHint}</p>
		</div>
	{/if}
</section>

<style>
	.block {
		display: grid;
		gap: var(--space-3);
	}
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
	}
	.head h2 {
		flex: 1 1 auto;
	}
	.by {
		display: inline-flex;
		align-items: center;
	}
	.small {
		font-size: var(--type-xs);
	}
	.setups {
		display: grid;
		gap: var(--space-3);
	}
	.setups.compare {
		grid-template-columns: repeat(auto-fit, minmax(17rem, 1fr));
		align-items: stretch;
	}
	.add-column {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		min-height: 10rem;
		padding: var(--space-4);
		border: 1px dashed var(--line-strong);
		border-radius: var(--radius-card);
		background: transparent;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		font-weight: 500;
		cursor: pointer;
		transition:
			border-color 120ms ease-out,
			color 120ms ease-out;
	}
	.add-column:hover {
		border-color: var(--line-hover);
		color: var(--text-bright);
	}
	.add {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
	}
	.add .btn {
		border-style: dashed;
		border-color: var(--line-strong);
	}
	.warning {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-control);
		background: var(--warn-wash);
		color: var(--warn-text);
		font-size: var(--type-xs);
	}
</style>
