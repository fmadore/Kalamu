<script lang="ts">
	// "Analyse this interview": one question (what the research is about, which may be left
	// empty), the model that will read the transcript said in one line with what it would cost,
	// and one button. Another model and the language of the answers wait under "More choices",
	// closed. The transcript is frozen as the dialog opens, so the estimate prices the text the job
	// will read, and says in its place when the text is too long for the model chosen (Start then
	// waits for another); a transcript too long to be read at once is said to be read in parts,
	// whose cost the estimate holds. Start then asks what New job asks before anything is sent.
	// What is typed can be kept for the next interviews, as a prompt of the researcher's own: once
	// one is kept, a choice above the box offers it, and the box only for a new description.
	import Dialog from './Dialog.svelte';
	import LanguageStep from './LanguageStep.svelte';
	import ModelPicker from './ModelPicker.svelte';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { InterviewAnalysis, tooLongOf } from '#lib/interviewStart.svelte.js';
	import { takesLanguage } from '#lib/languages.js';
	import { followEstimate } from '#lib/newJobEstimate.svelte.js';
	import { modelsFor, providerDefault } from '#lib/newJobModels.js';
	import { NEW_RESEARCH } from '#lib/research.js';
	import { go, type FromProject } from '#lib/state.svelte.js';

	let {
		jobId,
		setupId,
		sourceId,
		project,
		onclose
	}: {
		jobId: string;
		setupId: string;
		sourceId: string | null;
		project?: FromProject;
		onclose: () => void;
	} = $props();
	const m = messages();
	const t = m.interview;

	const a = new InterviewAnalysis();
	// svelte-ignore state_referenced_locally
	void a.prepare(jobId, setupId, sourceId);

	const providers = $derived(
		a.catalog.providers.map((p) => p.id).filter((id) => modelsFor(a.catalog, a.rule, id).length > 0)
	);
	const ignored = $derived(takesLanguage(a.model) ? [] : [a.model?.name ?? '']);

	followEstimate(
		() => ({ task: 'analysis', sources: a.sources ?? [], setups: [a.request], sample_pages: null }),
		(estimate) => {
			a.cost = estimate?.total_usd ?? null;
			a.tooLong = tooLongOf(estimate, a.catalog, m);
			a.parts = estimate?.rows.find((r) => r.parts)?.parts ?? null;
		}
	);

	async function start() {
		if (await a.start(project)) onclose();
	}

	/** Prompts, on the kept description, where it is changed or deleted. */
	function openPrompts(presetId: string) {
		onclose();
		go({ name: 'prompts', presetId });
	}

	// The choice of description takes the focus when it appears, the prompts read after the
	// dialog opened on the box.
	let choiceBox = $state<HTMLSelectElement | null>(null);
	$effect(() => {
		choiceBox?.focus();
	});
</script>

<Dialog labelledby="interview-title" size="medium" onescape={onclose}>
	<h2 id="interview-title">{t.title}</h2>
	<p class="lede">{t.lede}</p>

	{#if a.kept.length > 0}
		<label class="field">
			<span>{t.research}</span>
			<select bind:this={choiceBox} bind:value={a.choice}>
				{#each a.kept as p (p.id)}
					<option value={p.id}>{p.name}</option>
				{/each}
				<option value={NEW_RESEARCH}>{t.researchNew}</option>
			</select>
		</label>
	{/if}

	{#if a.chosen}
		<div class="kept">
			<p class="described">{a.chosen.question}</p>
			<p class="small muted">
				{t.keptHint}
				<button class="btn sm ghost" onclick={() => openPrompts(a.chosen!.id)}
					>{t.openPrompts}</button
				>
			</p>
		</div>
	{:else}
		<label class="field">
			<span>{t.question}</span>
			<textarea
				rows="3"
				bind:value={a.question}
				placeholder={t.questionPlaceholder}
				aria-describedby="interview-question-hint"></textarea>
			<small id="interview-question-hint">{t.questionHint}</small>
		</label>
		{#if a.question.trim()}
			<div class="keep">
				<label class="inline">
					<input
						type="checkbox"
						checked={a.keep}
						onchange={(e) => a.setKeep(e.currentTarget.checked, project?.name)}
					/>
					{t.keep}
				</label>
				{#if a.keep}
					<label class="field name">
						<span>{t.keptName}</span>
						<input type="text" bind:value={a.keptName} />
					</label>
				{/if}
			</div>
		{/if}
	{/if}

	<p class="model">
		<span>{t.model(a.model?.name ?? a.draft.model, a.providerName)}</span>
		{#if a.cost !== null && !a.tooLong}<span class="cost">{t.cost(formatUsd(a.cost))}</span>{/if}
	</p>
	{#if a.parts && !a.tooLong}<p class="small parts">{t.inParts(a.parts)}</p>{/if}
	{#if a.tooLong}<p class="chip danger" role="alert">{a.tooLong}</p>{/if}
	{#if !a.keyed}
		<p class="notice" role="status">
			{m.newJob.noKey(a.providerName)}
			<button class="btn sm" onclick={() => go({ name: 'settings', tab: 'keys' })}
				>{m.newJob.openSettings}</button
			>
		</p>
	{/if}

	<details class="more">
		<summary>{t.more}</summary>
		<div class="choices">
			<ModelPicker
				catalog={a.catalog}
				by="tier"
				{providers}
				models={(provider) => modelsFor(a.catalog, a.rule, provider)}
				defaultFor={(provider) => providerDefault(a.catalog, a.rule, provider)}
				bind:provider={a.draft.provider}
				bind:model={a.draft.model}
			/>
			<LanguageStep
				bind:mode={a.languageMode}
				bind:languages={a.languages}
				task="analysis"
				models={[a.model]}
				{ignored}
			/>
		</div>
	</details>

	{#if a.error}<p class="chip danger" role="alert">{a.error}</p>{/if}
	{#if !a.sources && !a.error}<p class="muted small" role="status">{t.preparing}</p>{/if}

	<div class="actions">
		<button class="btn" onclick={onclose}>{m.common.cancel}</button>
		<button
			class="btn primary"
			disabled={!a.sources || !a.keyed || !a.draft.model || a.busy || a.tooLong !== null}
			onclick={start}>{t.start}</button
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
	select {
		max-width: 24rem;
	}
	.kept {
		display: grid;
		gap: var(--space-2);
	}
	.described {
		padding: var(--space-2) var(--space-3);
		border-left: 3px solid var(--line-strong);
		background: var(--wash);
		border-radius: var(--radius-control);
		font-size: var(--type-md);
		line-height: 1.5;
		white-space: pre-wrap;
		max-width: 60ch;
	}
	.kept .small {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.keep {
		display: grid;
		gap: var(--space-2);
		margin-top: calc(-1 * var(--space-2));
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		width: fit-content;
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.name input {
		max-width: 24rem;
	}
	textarea {
		width: 100%;
		font-size: var(--type-md);
		line-height: 1.5;
		resize: vertical;
	}
	.model {
		display: flex;
		flex-wrap: wrap;
		gap: 0 var(--space-3);
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	/* New job's language step, here one choice among others: no step number, a field's label. */
	.choices :global(.step) {
		padding-left: 0;
	}
	.choices :global(.step::before) {
		content: none;
	}
	.choices :global(.step > h2) {
		font-size: var(--type-sm);
		font-weight: 500;
		color: var(--text-secondary);
	}
	.cost {
		color: var(--text-secondary);
	}
	.notice {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--warn-text);
	}
	.more summary {
		width: fit-content;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		cursor: pointer;
	}
	.choices {
		display: grid;
		gap: var(--space-4);
		margin-top: var(--space-3);
	}
	.small {
		font-size: var(--type-xs);
	}
	.parts {
		color: var(--text-secondary);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
