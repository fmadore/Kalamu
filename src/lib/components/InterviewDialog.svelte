<script lang="ts">
	// "Analyse this interview": one question (what the research is about, which may be left
	// empty), the model that will read the transcript said in one line with what it would cost,
	// and one button. Another model and the language of the answers wait under "More choices",
	// closed. The transcript is frozen as the dialog opens, so the estimate prices the text the job
	// will read; Start then asks what New job asks before anything is sent.
	import Dialog from './Dialog.svelte';
	import LanguageStep from './LanguageStep.svelte';
	import ModelPicker from './ModelPicker.svelte';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { InterviewAnalysis } from '#lib/interviewStart.svelte.js';
	import { takesLanguage } from '#lib/languages.js';
	import { followEstimate } from '#lib/newJobEstimate.svelte.js';
	import { modelsFor, providerDefault } from '#lib/newJobModels.js';
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
		(estimate) => (a.cost = estimate?.total_usd ?? null)
	);

	async function start() {
		if (await a.start(project)) onclose();
	}
</script>

<Dialog labelledby="interview-title" size="medium" onescape={onclose}>
	<h2 id="interview-title">{t.title}</h2>
	<p class="lede">{t.lede}</p>

	<label class="field">
		<span>{t.question}</span>
		<textarea
			rows="3"
			bind:value={a.question}
			placeholder={t.questionPlaceholder}
			aria-describedby="interview-question-hint"></textarea>
		<small id="interview-question-hint">{t.questionHint}</small>
	</label>

	<p class="model">
		<span>{t.model(a.model?.name ?? a.draft.model, a.providerName)}</span>
		{#if a.cost !== null}<span class="cost">{t.cost(formatUsd(a.cost))}</span>{/if}
	</p>
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
			disabled={!a.sources || !a.keyed || !a.draft.model || a.busy}
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
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
