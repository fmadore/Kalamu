<script lang="ts">
	// New job's results step: a sample of the first pages to try first (or of pages spread
	// through each document, from New job's suggestion), the formats to write,
	// where to write them (a folder must be chosen for videos, which have none of their own), and
	// the job's name.
	import TermLink from './TermLink.svelte';
	import Icon from './Icon.svelte';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';
	import { type Mode, keepsTimes, offeredFormats, rulesFor } from '#lib/newJob.js';
	import type { Draft } from '#lib/newJobModels.js';
	import { pickFolder } from '#lib/dialogs.js';
	import { api } from '#lib/tauri.js';
	import { app } from '#lib/state.svelte.js';

	let {
		sample = $bindable(),
		samplePages = $bindable(),
		sampleSpread = $bindable(),
		formats = $bindable(),
		destination = $bindable(),
		name = $bindable(),
		mode,
		task,
		drafts,
		paged,
		sources,
		hasSources
	}: {
		/** Whether to run on the first pages only. */
		sample: boolean;
		samplePages: number;
		/** The sample's pages are spread through each document, as New job's suggestion asks. */
		sampleSpread: boolean;
		formats: ExportFormat[];
		destination: OutputDestination;
		name: string;
		mode: Mode;
		task: Task;
		drafts: Draft[];
		/** The task writes text out page by page, so a sample applies. */
		paged: boolean;
		/** The sources as the job reads them: a translation writes subtitles of a transcript
		 * sent on with its times only. */
		sources: SourceFile[];
		/** Files or links have been added. */
		hasSources: boolean;
	} = $props();
	const m = messages();
	const catalog = $derived(app.catalog!);
	const rule = $derived(rulesFor(app.rules, task));
	const offered = $derived(offeredFormats(catalog, rule, drafts, sources));
	/** A translation of text without times: its subtitles are not offered, and the hint says
	 * why. */
	const untimed = $derived(task === 'translation' && !keepsTimes(rule, sources));

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
			<!-- Ticked here, a sample is of the first pages; untick it, and its pages are spread no more. -->
			<input type="checkbox" bind:checked={sample} onchange={() => (sampleSpread = false)} />
			{m.newJob.sampleTitle}{m.common.colon}
			<input
				type="number"
				min="1"
				max="50"
				bind:value={samplePages}
				class="num"
				disabled={!sample}
			/>
			<span class="muted">
				{sampleSpread ? m.newJob.trySample.spread(samplePages) : m.newJob.sample(samplePages)}
			</span>
		</label>
	{/if}
	<fieldset>
		<legend class="label">{m.newJob.formats}<TermLink term="formats" /></legend>
		<div class="row wrap">
			{#each offered as f (f)}
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
		{#if untimed}
			<p class="muted">{m.newJob.subtitlesNeedTimes}</p>
		{/if}
	</fieldset>
	{#if mode === 'youtube'}
		<!-- A video has no folder of its own, so a folder must be chosen. -->
		<div class="row wrap">
			<span class="label">{m.newJob.saveIn}</span>
			{#if destination.kind === 'folder'}
				<span class="path mono">{destination.path}</span>
			{/if}
			<button class="btn sm" onclick={chooseDestination}>
				<Icon name="folder" size={14} />{m.newJob.chooseFolder}
			</button>
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
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
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
