<script lang="ts">
	// "Which of these speak of my question?": one box (the question), how many documents will be
	// read and which are left out, the model said in one line with what it would cost, and one
	// button. Another model and the language of the answers wait under "More choices", closed.
	// The documents' texts are frozen as the dialog opens, so the estimate prices the texts the
	// job will read; a document the model chosen cannot read even in parts is named with the
	// others left out, and the job reads the rest. Start then asks what New job asks before
	// anything is sent.
	import Dialog from './Dialog.svelte';
	import LanguageStep from './LanguageStep.svelte';
	import ModelPicker from './ModelPicker.svelte';
	import { formatUsd, listNames, messages } from '#lib/i18n/index.js';
	import { takesLanguage } from '#lib/languages.js';
	import { followEstimate } from '#lib/newJobEstimate.svelte.js';
	import { modelsFor, providerDefault } from '#lib/newJobModels.js';
	import { tooLongOf } from '#lib/screening.js';
	import { Screening, type Again } from '#lib/screeningStart.svelte.js';
	import { go, type FromProject } from '#lib/state.svelte.js';

	let {
		project,
		documents,
		again,
		onclose
	}: {
		project: FromProject;
		/** The documents chosen, by id. */
		documents: string[];
		/** A question kept with the project, asked again as it was. */
		again?: Again;
		onclose: () => void;
	} = $props();
	const m = messages();
	const t = m.screening;

	// svelte-ignore state_referenced_locally
	const s = new Screening(again);
	// svelte-ignore state_referenced_locally
	void s.prepare(project.id, documents);

	const providers = $derived(
		s.catalog.providers.map((p) => p.id).filter((id) => modelsFor(s.catalog, s.rule, id).length > 0)
	);
	const ignored = $derived(takesLanguage(s.model) ? [] : [s.model?.name ?? '']);

	followEstimate(
		() => ({ task: 'analysis', sources: s.sources ?? [], setups: [s.request], sample_pages: null }),
		(estimate) => {
			s.cost = estimate?.total_usd ?? null;
			s.tooLong = tooLongOf(estimate, s.catalog, m);
		}
	);

	async function start() {
		if (await s.start(project)) onclose();
	}
</script>

<Dialog labelledby="screening-title" size="medium" onescape={onclose}>
	<h2 id="screening-title">{t.title}</h2>
	<p class="lede">{t.lede}</p>

	<label class="field">
		<span>{t.question}</span>
		<textarea
			rows="3"
			bind:value={s.question}
			placeholder={t.questionPlaceholder}
			aria-describedby="screening-question-hint"></textarea>
		<small id="screening-question-hint">{t.questionHint}</small>
	</label>

	{#if s.sources}
		{#if s.readable.length > 0}
			<p class="count" role="status">{t.documents(s.readable.length)}</p>
		{:else if s.sources.length === 0}
			<p class="notice" role="status">{t.nothing}</p>
		{/if}
		{#if s.noText.length > 0}
			<p class="left small muted">{t.noText(listNames(s.noText), s.noText.length)}</p>
		{/if}
		{#if s.undecided.length > 0}
			<p class="left small muted">{t.undecided(listNames(s.undecided))}</p>
		{/if}
		{#if s.tooLong}
			<p class="left small muted">{s.tooLong.said}</p>
		{/if}
	{/if}

	<p class="model">
		<span>{t.model(s.model?.name ?? s.draft.model, s.providerName)}</span>
		{#if s.cost !== null && s.readable.length > 0}<span class="cost"
				>{t.cost(formatUsd(s.cost))}</span
			>{/if}
	</p>
	{#if !s.keyed}
		<p class="notice" role="status">
			{m.newJob.noKey(s.providerName)}
			<button class="btn sm" onclick={() => go({ name: 'settings', tab: 'keys' })}
				>{m.newJob.openSettings}</button
			>
		</p>
	{/if}

	<details class="more">
		<summary>{t.more}</summary>
		<div class="choices">
			<ModelPicker
				catalog={s.catalog}
				by="tier"
				{providers}
				models={(provider) => modelsFor(s.catalog, s.rule, provider)}
				defaultFor={(provider) => providerDefault(s.catalog, s.rule, provider)}
				bind:provider={s.draft.provider}
				bind:model={s.draft.model}
			/>
			<LanguageStep
				bind:mode={s.languageMode}
				bind:languages={s.languages}
				task="analysis"
				models={[s.model]}
				{ignored}
			/>
		</div>
	</details>

	{#if s.error}<p class="chip danger" role="alert">{s.error}</p>{/if}
	{#if !s.sources && !s.error}<p class="muted small" role="status">{t.preparing}</p>{/if}

	<div class="actions">
		<button class="btn" onclick={onclose}>{m.common.cancel}</button>
		<button class="btn primary" disabled={!s.ready} onclick={start}>{t.start}</button>
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
	.count {
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	.left {
		max-width: 60ch;
		line-height: 1.5;
	}
	.model {
		display: flex;
		flex-wrap: wrap;
		gap: 0 var(--space-3);
		font-size: var(--type-sm);
		color: var(--text-body);
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
