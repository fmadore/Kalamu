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

	function setEvenOut(index: number, on: boolean) {
		const source = files[index].source;
		if (source) source.even_out_volume = on;
	}
</script>

<section class="card block" aria-labelledby="files-title">
	<div class="row between">
		<h2 id="files-title">{m.newJob.filesTitle}</h2>
		<div class="row">
			<button class="btn" onclick={async () => onadd(await pickFiles(task))}
				>{m.newJob.addFiles}</button
			>
			<button
				class="btn"
				onclick={async () => {
					const folder = await pickFolder();
					if (folder) await onadd([folder]);
				}}>{m.newJob.addFolder}</button
			>
		</div>
	</div>
	<p class="muted small">{m.newJob.dropHint}</p>
	{#if inspecting}
		<p class="muted">{m.newJob.inspecting}</p>
	{/if}
	{#if files.length === 0 && !inspecting}
		<p class="empty muted">{m.newJob.noFiles}</p>
	{:else}
		<ul class="files">
			{#each files as file, i (file.path)}
				<li>
					<div class="row between">
						<div class="file-name">
							<strong>{file.path.split(/[\\/]/).pop()}</strong>
							{#if file.source?.page_count}
								<span class="chip">{m.newJob.pages(file.source.page_count)}</span>
							{/if}
							{#if file.source?.duration_ms}
								<span class="chip">{duration(Number(file.source.duration_ms), m)}</span>
							{/if}
							{#if file.error}
								<span class="chip danger">{describeError(file.error, m)}</span>
							{/if}
						</div>
						<button class="btn ghost sm" onclick={() => (files = files.filter((_, j) => j !== i))}
							>{m.newJob.remove}</button
						>
					</div>
					{#each file.warnings as w (w.kind)}
						<div class="warning">
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
		<ul class="muted">
			<li>{task === 'text_extraction' ? m.tips.scanning : m.tips.recording}</li>
			<li>{m.tips.results}</li>
		</ul>
	</details>
</section>

<style>
	.block {
		padding: var(--space-4) 0 0;
		display: grid;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.between {
		justify-content: space-between;
	}
	.small {
		font-size: var(--type-sm);
	}
	.files {
		list-style: none;
		padding: 0;
		margin: 0;
		display: grid;
		gap: var(--space-2);
	}
	.files li {
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
		display: grid;
		gap: var(--space-2);
	}
	.file-name {
		display: flex;
		gap: var(--space-2);
		align-items: center;
		flex-wrap: wrap;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.warning {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		justify-content: space-between;
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--warn-border);
		background: var(--warn-wash);
		border-radius: var(--radius-control);
		font-size: var(--type-sm);
	}
	.empty {
		padding: var(--space-4);
		text-align: center;
		border: 1px dashed var(--line-strong);
		border-radius: var(--radius-card);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
	}
	.tips summary {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		min-height: var(--control-sm);
		list-style: none;
		color: var(--accent-soft);
		font-size: var(--type-sm);
		font-weight: 600;
		cursor: pointer;
	}
	.tips summary::-webkit-details-marker {
		display: none;
	}
	.tips summary:hover {
		text-decoration: underline;
		text-underline-offset: 0.18em;
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
		font-size: var(--type-sm);
		max-width: 48rem;
	}
</style>
