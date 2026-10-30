<script lang="ts">
	// New job's results step: a sample of the first pages to try first, the formats to write,
	// where to write them (a folder must be chosen for videos, which have none of their own), and
	// the job's name.
	import TermLink from './TermLink.svelte';
	import Icon from './Icon.svelte';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';
	import { type Draft, type Mode, offeredFormats, rulesFor } from '#lib/newJob.js';
	import { api, pickFolder } from '#lib/tauri.js';
	import { app } from '#lib/state.svelte.js';

	let {
		sample = $bindable(),
		samplePages = $bindable(),
		formats = $bindable(),
		destination = $bindable(),
		name = $bindable(),
		mode,
		task,
		drafts,
		paged,
		hasSources
	}: {
		/** Whether to run on the first pages only. */
		sample: boolean;
		samplePages: number;
		formats: ExportFormat[];
		destination: OutputDestination;
		name: string;
		mode: Mode;
		task: Task;
		drafts: Draft[];
		/** The task writes text out page by page, so a sample applies. */
		paged: boolean;
		/** Files or links have been added. */
		hasSources: boolean;
	} = $props();
	const m = messages();
	const catalog = $derived(app.catalog!);

	async function chooseDestination() {
		const folder = await pickFolder();
		if (!folder) return;
		destination = { kind: 'folder', path: folder };
		// Remembered, so the next job that needs a folder starts with this one.
		app.settings!.last_folder = folder;
		api.saveSettings($state.snapshot(app.settings!)).catch(() => undefined);
	}
</script>

<section class="step block" aria-labelledby="out-title">
	<h2 id="out-title">{m.newJob.outputTitle}</h2>
	{#if paged}
		<label class="inline">
			<input type="checkbox" bind:checked={sample} />
			{m.newJob.sampleTitle}:
			<input
				type="number"
				min="1"
				max="50"
				bind:value={samplePages}
				class="num"
				disabled={!sample}
			/>
			<span class="muted">{m.newJob.sample(samplePages)}</span>
		</label>
	{/if}
	<fieldset>
		<legend class="label">{m.newJob.formats}<TermLink term="formats" /></legend>
		<div class="row wrap">
			{#each offeredFormats(catalog, rulesFor(app.rules, task), drafts) as f (f)}
				<label class="inline">
					<input
						type="checkbox"
						checked={formats.includes(f)}
						onchange={(e) =>
							(formats = e.currentTarget.checked
								? [...formats, f]
								: formats.filter((x) => x !== f))}
					/>
					{m.format[f]}
				</label>
			{/each}
		</div>
	</fieldset>
	{#if mode === 'youtube'}
		<!-- A video has no folder of its own, so a folder must be chosen. -->
		<div class="row wrap">
			<span class="label">{m.newJob.saveIn}</span>
			{#if destination.kind === 'folder'}
				<span class="path mono">{destination.path}</span>
			{/if}
			<button class="btn sm" onclick={chooseDestination}
				><Icon name="folder" size={14} />{m.newJob.chooseFolder}</button
			>
		</div>
		{#if destination.kind !== 'folder' && hasSources}
			<p class="notice">{m.newJob.linkFolder}</p>
		{/if}
	{:else}
		<div class="row wrap">
			<label class="inline">
				<input
					type="radio"
					checked={destination.kind === 'beside_sources'}
					onchange={() => (destination = { kind: 'beside_sources' })}
				/>
				{m.newJob.besideSources}
			</label>
			<label class="inline">
				<input type="radio" checked={destination.kind === 'folder'} onchange={chooseDestination} />
				{m.newJob.inFolder}
			</label>
			{#if destination.kind === 'folder'}
				<span class="path mono">{destination.path}</span>
				<button class="btn sm" onclick={chooseDestination}>{m.newJob.chooseFolder}</button>
			{/if}
		</div>
	{/if}
	<label class="field name">
		<span>{m.newJob.name}</span>
		<input type="text" bind:value={name} />
	</label>
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
	.row.wrap {
		flex-wrap: wrap;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
	.path {
		overflow-wrap: anywhere;
		font-size: var(--type-xs);
	}
	fieldset {
		border: 0;
		padding: 0;
		margin: 0;
		display: grid;
		gap: var(--space-2);
	}
	.num {
		width: 4.5rem;
	}
	.name {
		max-width: 28rem;
	}
</style>
