<script lang="ts">
	// The model set-ups of a new job: one for a normal job, several to compare.
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		acceptsPrompt,
		type Draft,
		labelsSpeakers,
		modelInfo,
		modelsFor,
		newDraft
	} from '#lib/newJob.js';
	import { app, go, hasKey, isInstalled } from '#lib/state.svelte.js';

	let {
		drafts = $bindable(),
		task,
		presets
	}: { drafts: Draft[]; task: Task; presets: Preset[] } = $props();

	const m = messages();
	const catalog = $derived(app.catalog!);
	const providers = $derived(
		catalog.providers.filter((p) =>
			catalog.models.some((mod) => mod.provider === p.id && mod.tasks.includes(task))
		)
	);

	function addSetup() {
		const used = drafts.map((d) => `${d.provider}/${d.model}`);
		drafts = [...drafts, newDraft(catalog, task, presets[0]?.id ?? null, hasKey, used)];
	}
</script>

<section class="card block" aria-labelledby="setups-title">
	<h2 id="setups-title">{m.newJob.setupsTitle}</h2>
	{#if !app.keys.some((k) => k.saved)}
		<p class="warning">
			{m.newJob.noProvider}
			<button class="btn sm" onclick={() => go({ name: 'settings' })}>{m.nav.settings}</button>
		</p>
	{/if}
	{#each drafts as d, i (i)}
		{@const info = modelInfo(catalog, d)}
		<div class="setup">
			<label class="field">
				<span>{m.newJob.provider}</span>
				<select
					bind:value={d.provider}
					onchange={() => {
						d.model = modelsFor(catalog, d.provider, task)[0]?.id ?? '';
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
					{#each modelsFor(catalog, d.provider, task) as mod (mod.id)}
						<option value={mod.id}
							>{mod.name}{mod.status === 'preview' ? ` (${m.common.preview})` : ''}</option
						>
					{/each}
				</select>
			</label>
			<label class="field">
				<span>{m.newJob.prompt}</span>
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
			{#if drafts.length > 1}
				<button class="btn ghost sm" onclick={() => (drafts = drafts.filter((_, j) => j !== i))}
					>{m.newJob.removeSetup}</button
				>
			{/if}
			{#if info?.description}
				<p class="muted small desc">{info.description}</p>
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
	<div>
		<button class="btn" onclick={addSetup}>+ {m.newJob.compareAdd}</button>
		<p class="muted small">{m.newJob.setupsHint}</p>
	</div>
</section>

<style>
	.block {
		padding: var(--space-4) var(--space-5);
		display: grid;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.small {
		font-size: var(--type-sm);
	}
	.setup {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
		gap: var(--space-3);
		align-items: end;
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
	}
	.desc,
	.setup .warning {
		grid-column: 1 / -1;
	}
	.warning {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--warn-border);
		background: var(--warn-wash);
		border-radius: var(--radius-control);
		font-size: var(--type-sm);
	}
	.opts {
		min-height: var(--control-md);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
	}
</style>
