<script lang="ts">
	// The files of a new job, with each file's input checks and the fixes they offer.
	import Icon from './Icon.svelte';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { duration, warningText } from '#lib/newJob.js';
	import { pickFiles, pickFolder } from '#lib/tauri.js';

	let {
		files = $bindable(),
		task,
		inspecting,
		onadd
	}: {
		files: InspectedFile[];
		task: Task;
		inspecting: boolean;
		onadd: (paths: string[]) => void | Promise<void>;
	} = $props();

	const m = messages();

	function setSplit(index: number, split: boolean) {
		const source = files[index].source;
		if (source) source.split_spreads = split;
	}

	/**
	 * An analysis reads each document whole, and a correction reads a PDF's text layer, so
	 * splitting double pages applies to neither; the text layer is what a correction wants.
	 */
	function shownWarnings(file: (typeof files)[number]) {
		if (task === 'analysis') return file.warnings.filter((w) => w.kind !== 'double_page_spread');
		if (task === 'text_correction') {
			return file.warnings.filter(
				(w) => w.kind !== 'double_page_spread' && w.kind !== 'has_text_layer'
			);
		}
		return file.warnings;
	}

	function setEvenOut(index: number, on: boolean) {
		const source = files[index].source;
		if (source) source.even_out_volume = on;
	}
</script>

<section class="step block" aria-labelledby="files-title">
	<div class="head">
		<h2 id="files-title">{m.newJob.filesTitle}</h2>
		<span class="hint">{m.newJob.dropHint}</span>
		<div class="row">
			<button class="btn sm" onclick={async () => onadd(await pickFiles(task))}
				><Icon name="plus" size={14} />{m.newJob.addFiles}</button
			>
			<button
				class="btn sm"
				onclick={async () => {
					const folder = await pickFolder();
					if (folder) await onadd([folder]);
				}}><Icon name="folder" size={14} />{m.newJob.addFolder}</button
			>
		</div>
	</div>
	{#if inspecting}
		<p class="hint">{m.newJob.inspecting}</p>
	{/if}
	{#if files.length === 0 && !inspecting}
		<p class="empty">{m.newJob.noFiles}</p>
	{:else if files.length > 0}
		<ul class="files">
			{#each files as file, i (file.path)}
				{@const fileName = file.path.split(/[\\/]/).pop()}
				{@const warnings = shownWarnings(file)}
				<li class:warned={warnings.length > 0} class:failed={!!file.error}>
					<div class="file">
						<span class="file-name mono">{fileName}</span>
						<span class="meta mono">
							{#if file.source?.page_count}{m.newJob.pages(file.source.page_count)}{/if}
							{#if file.source?.duration_ms}{duration(Number(file.source.duration_ms), m)}{/if}
						</span>
						<button
							class="btn sm icon ghost"
							aria-label="{m.newJob.remove} {fileName}"
							title={m.newJob.remove}
							onclick={() => (files = files.filter((_, j) => j !== i))}
							><Icon name="close" size={14} /></button
						>
					</div>
					{#if file.error}
						<p class="problem danger">{describeError(file.error, m)}</p>
					{/if}
					{#each warnings as w (w.kind)}
						<div class="problem">
							<span>{warningText(w, m)}</span>
							{#if w.kind === 'double_page_spread' && file.source}
								<label class="inline">
									<input
										type="checkbox"
										checked={file.source.split_spreads}
										onchange={(e) => setSplit(i, e.currentTarget.checked)}
									/>
									{m.newJob.splitSpreads}
								</label>
							{/if}
							{#if w.kind === 'quiet_recording' && file.source}
								<label class="inline">
									<input
										type="checkbox"
										checked={file.source.even_out_volume}
										onchange={(e) => setEvenOut(i, e.currentTarget.checked)}
									/>
									{m.newJob.evenOut}
								</label>
							{/if}
						</div>
					{/each}
				</li>
			{/each}
		</ul>
	{/if}
	<details class="tips">
		<summary><Icon name="chevron" size={14} />{m.tips.title}</summary>
		<ul>
			{#if task === 'analysis'}
				<li>{m.tips.analysisUnit}</li>
				<li>{m.tips.analysisInput}</li>
				<li>{m.tips.analysisFields}</li>
			{:else if task === 'contents'}
				<li>{m.tips.contentsUnit}</li>
				<li>{m.tips.scanning}</li>
				<li>{m.tips.contentsCheck}</li>
			{:else if task === 'text_correction'}
				<li>{m.tips.correctionInput}</li>
				<li>{m.tips.correctionCheck}</li>
				<li>{m.tips.results}</li>
			{:else}
				<li>{task === 'text_extraction' ? m.tips.scanning : m.tips.recording}</li>
				<li>{m.tips.results}</li>
			{/if}
		</ul>
	</details>
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
	.row {
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The files as a table: name, size, remove; a check's fix right under its file. */
	.files {
		list-style: none;
		padding: 0;
		margin: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		overflow: hidden;
	}
	.files li {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-2) var(--space-2) var(--space-4);
		border-top: 1px solid var(--line-soft);
	}
	.files li:first-child {
		border-top: 0;
	}
	.file {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		gap: var(--space-4);
		align-items: center;
	}
	.file-name {
		overflow-wrap: anywhere;
		color: var(--text-bright);
	}
	.file-name::before {
		content: '';
		display: inline-block;
		width: 6px;
		height: 6px;
		margin-right: var(--space-3);
		border-radius: 3px;
		background: var(--ok);
		vertical-align: middle;
	}
	.warned .file-name::before {
		background: var(--warn);
	}
	.failed .file-name::before {
		background: var(--danger);
	}
	.meta {
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.problem {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-4);
		align-items: center;
		justify-content: space-between;
		margin: 0 var(--space-2) var(--space-1) 1.125rem;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-control);
		background: var(--warn-wash);
		color: var(--warn-text);
		font-size: var(--type-xs);
	}
	.problem.danger {
		background: var(--danger-wash);
		color: var(--danger-text);
	}
	.empty {
		padding: var(--space-5);
		text-align: center;
		border: 1px dashed var(--line-strong);
		border-radius: var(--radius-card);
		color: var(--text-muted);
		font-size: var(--type-sm);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-weight: 500;
	}
	.tips summary {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		min-height: var(--control-sm);
		list-style: none;
		color: var(--text-secondary);
		font-size: var(--type-xs);
		font-weight: 500;
		cursor: pointer;
	}
	.tips summary::-webkit-details-marker {
		display: none;
	}
	.tips summary:hover {
		color: var(--text-bright);
	}
	.tips summary :global(.icon) {
		transition: transform 120ms ease-out;
	}
	.tips[open] summary :global(.icon) {
		transform: rotate(90deg);
	}
	.tips ul {
		margin: var(--space-1) 0 0;
		padding-left: 1.25rem;
		display: grid;
		gap: var(--space-1);
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 48rem;
	}
</style>
