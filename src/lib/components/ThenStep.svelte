<script lang="ts">
	// What New job does after the text is read: nothing more, or an analysis of the text as the
	// researcher corrects it, by default once they have checked it. Behind it is a two-step
	// workflow, but nothing here asks the researcher to know that.
	import TermLink from './TermLink.svelte';
	import WorkflowDiagram from './WorkflowDiagram.svelte';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		presetId = $bindable(),
		check = $bindable(),
		presets,
		task,
		model
	}: {
		presetId: string | null;
		check: boolean;
		/** The analyses to choose from. */
		presets: Preset[];
		/** The task that reads the text first. */
		task: Task;
		/** The model that would analyse it. */
		model: string;
	} = $props();

	const m = messages();
	const t = m.newJob.then;
	const chosen = $derived(presets.find((p) => p.id === presetId));
</script>

<section class="step block" aria-labelledby="then-title">
	<h2 id="then-title">{t.title}<TermLink term="workflow" /></h2>
	<p class="hint">{t.hint}</p>
	<label class="field">
		<span>{t.analyse}</span>
		<select value={presetId ?? ''} onchange={(e) => (presetId = e.currentTarget.value || null)}>
			<option value="">{t.nothing}</option>
			{#each presets as p (p.id)}
				<option value={p.id}>{p.name}</option>
			{/each}
		</select>
	</label>
	{#if chosen}
		<label class="check">
			<input type="checkbox" bind:checked={check} />
			<span>{t.check}<span class="sub">{t.checkHint}</span></span>
		</label>
		<p class="hint">
			{check ? t.whatHappensChecked(chosen.name, model) : t.whatHappens(chosen.name, model)}
		</p>
		<WorkflowDiagram
			compact
			steps={[
				{ task, name: m.task[task], review: check },
				{ task: 'analysis', name: chosen.name, review: false }
			]}
		/>
	{/if}
</section>

<style>
	section {
		display: grid;
		gap: var(--space-2);
	}
	.check {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
		font-size: var(--type-sm);
	}
	.sub {
		display: block;
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
</style>
