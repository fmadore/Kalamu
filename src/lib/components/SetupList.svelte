<script lang="ts">
	// The model set-ups of a new job: one for a normal job, several to compare, each named by the
	// siglum it will carry in the review.
	import Icon from './Icon.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		acceptsPrompt,
		type Draft,
		labelsSpeakers,
		mergeDefault,
		mergeInfo,
		mergeModelsFor,
		mergeProviders,
		modelInfo,
		modelsFor,
		newDraft,
		providerDefault
	} from '#lib/newJob.js';
	import { app, go, hasKey, isInstalled } from '#lib/state.svelte.js';

	let {
		drafts = $bindable(),
		task,
		presets,
		input
	}: {
		drafts: Draft[];
		task: Task;
		presets: Preset[];
		/** What every model must read, such as YouTube links; any input of the task when absent. */
		input?: string;
	} = $props();

	const m = messages();
	const catalog = $derived(app.catalog!);
	const providers = $derived(
		catalog.providers.filter((p) => modelsFor(catalog, p.id, task, input).length > 0)
	);

	function addSetup() {
		const used = drafts.map((d) => `${d.provider}/${d.model}`);
		drafts = [...drafts, newDraft(catalog, task, presets[0]?.id ?? null, hasKey, used, input)];
	}
</script>

<section class="step block" aria-labelledby="setups-title">
	<h2 id="setups-title">{m.newJob.setupsTitle}</h2>
	{#if !app.keys.some((k) => k.saved)}
		<p class="warning">
			{m.newJob.noProvider}
			<button class="btn sm" onclick={() => go({ name: 'settings' })}>{m.nav.settings}</button>
		</p>
	{/if}
	<!-- One set-up lays its fields in a row; two or more stand side by side, one column per
	     witness, so the set-ups read across like the sigla key in the review. -->
	<div class="setups" class:compare={drafts.length > 1}>
		{#each drafts as d, i (i)}
			{@const info = modelInfo(catalog, d)}
			<div class="setup">
				{#if drafts.length > 1}
					<div class="witness">
						<i class="siglum">{siglum(i)}</i>
						<span class="witness-name mono">{d.label || info?.name || d.model}</span>
						<button class="btn ghost sm" onclick={() => (drafts = drafts.filter((_, j) => j !== i))}
							>{m.newJob.removeSetup}</button
						>
					</div>
				{/if}
				<label class="field">
					<span>{m.newJob.provider}</span>
					<select
						bind:value={d.provider}
						onchange={() => {
							d.model = providerDefault(catalog, d.provider, task, input);
						}}
					>
						{#each providers as p (p.id)}
							<option value={p.id} disabled={!hasKey(p.id)}>{p.name}</option>
						{/each}
					</select>
				</label>
				<label class="field">
					<span>{m.newJob.model}</span>
					<select bind:value={d.model}>
						{#each modelsFor(catalog, d.provider, task, input) as mod (mod.id)}
							<option value={mod.id}
								>{mod.name}{mod.status === 'preview'
									? ` (${m.common.preview})`
									: mod.status === 'deprecated'
										? ` (${m.common.retiring})`
										: ''}</option
							>
						{/each}
					</select>
				</label>
				<label class="field">
					<span
						>{task === 'analysis'
							? m.newJob.analysis
							: task === 'contents'
								? m.newJob.kindOfDocument
								: m.newJob.prompt}</span
					>
					{#if !acceptsPrompt(info)}
						<select disabled><option>{m.newJob.noPrompt}</option></select>
					{:else}
						<select bind:value={d.presetId}>
							{#each presets as p (p.id)}
								<option value={p.id}>{p.name}</option>
							{/each}
						</select>
					{/if}
				</label>
				{#if drafts.length > 1}
					<label class="field">
						<span>{m.newJob.label}</span>
						<input type="text" bind:value={d.label} placeholder={d.model} />
					</label>
				{/if}
				{#if task === 'audio_transcription'}
					<div class="row opts">
						<label class="inline"
							><input type="checkbox" bind:checked={d.diarize} disabled={!labelsSpeakers(info)} />
							{labelsSpeakers(info) ? m.newJob.diarize : m.newJob.noSpeakers}</label
						>
						<label class="inline"
							><input type="checkbox" bind:checked={d.timestamps} /> {m.newJob.timestamps}</label
						>
					</div>
				{/if}
				{#if info?.description}
					<p class="muted small desc">{info.description}</p>
				{/if}
				{#if task === 'contents'}
					<!-- The second step: any provider's model that reads text. -->
					<div class="merge" role="group" aria-labelledby="merge-{i}">
						<p class="label merge-title" id="merge-{i}">{m.newJob.mergeTitle}</p>
						<label class="field">
							<span>{m.newJob.provider}</span>
							<select
								bind:value={d.mergeProvider}
								onchange={() => {
									d.mergeModel = mergeDefault(catalog, d.mergeProvider);
								}}
							>
								{#each mergeProviders(catalog) as id (id)}
									<option value={id} disabled={!hasKey(id)}
										>{catalog.providers.find((p) => p.id === id)?.name ?? id}</option
									>
								{/each}
							</select>
						</label>
						<label class="field">
							<span>{m.newJob.model}</span>
							<select bind:value={d.mergeModel}>
								{#each mergeModelsFor(catalog, d.mergeProvider) as mod (mod.id)}
									<option value={mod.id}>{mod.name}</option>
								{/each}
							</select>
						</label>
						<p class="muted small desc">{m.newJob.mergeHint}</p>
						{#if d.mergeProvider && !hasKey(d.mergeProvider)}
							<p class="warning small">
								{m.newJob.noKey(
									catalog.providers.find((p) => p.id === d.mergeProvider)?.name ?? d.mergeProvider
								)}
							</p>
						{/if}
						{#if mergeInfo(catalog, d)?.costly}
							<p class="warning small">{m.newJob.costly}</p>
						{/if}
					</div>
				{/if}
				{#if task === 'analysis' || task === 'contents'}
					{@const fields = presets.find((p) => p.id === d.presetId)?.schema?.fields ?? []}
					{#if fields.length > 0}
						<p class="small desc finds">
							<span class="label">{task === 'contents' ? m.newJob.entryHas : m.newJob.finds}</span>
							{(task === 'contents'
								? [m.review.entryTitle, m.review.entryPages, ...fields.map((f) => f.label)]
								: fields.map((f) => f.label)
							).join(' · ')}
							<button class="link" onclick={() => go({ name: 'prompts' })}
								>{m.newJob.editFields}</button
							>
						</p>
					{/if}
				{/if}
				{#if info?.costly}
					<p class="warning small">{m.newJob.costly}</p>
				{/if}
				{#if d.model && !isInstalled(d.provider, d.model)}
					<p class="warning small">
						{m.newJob.notDownloaded}
						<button class="btn sm" onclick={() => go({ name: 'settings' })}
							>{m.newJob.openSettings}</button
						>
					</p>
				{/if}
				{#if d.provider && !hasKey(d.provider)}
					<p class="warning small">
						{m.newJob.noKey(catalog.providers.find((p) => p.id === d.provider)?.name ?? d.provider)}
					</p>
				{/if}
			</div>
		{/each}
		{#if drafts.length > 1}
			<button class="add-column" onclick={addSetup}
				><Icon name="plus" size={18} />{m.newJob.compareAdd}</button
			>
		{/if}
	</div>
	{#if drafts.length === 1}
		<div class="add">
			<button class="btn" onclick={addSetup}
				><Icon name="plus" size={16} />{m.newJob.compareAdd}</button
			>
			<p class="muted small">{m.newJob.setupsHint}</p>
		</div>
	{/if}
</section>

<style>
	.block {
		display: grid;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		gap: var(--space-3);
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
	.setup {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
		gap: var(--space-3);
		align-items: end;
		align-content: start;
		padding: var(--space-4);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		background: var(--surface-2);
	}
	/* In a comparison each column stacks its fields, so every set-up reads top to bottom. */
	.compare .setup {
		grid-template-columns: minmax(0, 1fr);
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
	.setup select,
	.setup input {
		width: 100%;
		min-width: 0;
	}
	.desc,
	.witness,
	.setup .warning {
		grid-column: 1 / -1;
	}
	.witness {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin: 0;
		font-size: var(--type-sm);
		color: var(--text-bright);
	}
	.witness-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
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
	.opts {
		min-height: var(--control-md);
	}
	.finds {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: baseline;
	}
	.link {
		border: 0;
		background: none;
		padding: 0;
		color: var(--text-bright);
		text-decoration: underline;
		text-underline-offset: 0.18em;
		cursor: pointer;
		font-size: var(--type-xs);
	}
	/* The second step, under the first: its own provider and model, side by side. */
	.merge {
		grid-column: 1 / -1;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
		gap: var(--space-2) var(--space-3);
		align-items: end;
		padding: var(--space-3) 0 0;
		border-top: 1px solid var(--line);
	}
	.merge-title,
	.merge .desc,
	.merge .warning {
		grid-column: 1 / -1;
		margin: 0;
	}
	.merge select {
		width: 100%;
		min-width: 0;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
</style>
