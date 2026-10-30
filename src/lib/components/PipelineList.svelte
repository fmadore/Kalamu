<script lang="ts">
	// Saved pipelines at the top of New job: apply one, export its record, delete it, or import
	// one shared by a colleague.
	import TermLink from './TermLink.svelte';
	import Icon from './Icon.svelte';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, ask, pickPipelineExport, pickPresetFile } from '#lib/tauri.js';
	import { toast } from '#lib/state.svelte.js';

	let {
		pipelines = $bindable(),
		onapply,
		onerror
	}: {
		pipelines: Pipeline[];
		onapply: (p: Pipeline) => void;
		onerror: (message: string) => void;
	} = $props();

	const m = $derived(messages());

	async function attempt(work: () => Promise<void>) {
		try {
			await work();
		} catch (e) {
			onerror(describeError(e, m));
		}
	}

	const exportPipeline = (p: Pipeline) =>
		attempt(async () => {
			const path = await pickPipelineExport(p.name);
			if (!path) return;
			await api.exportPipeline(p.id, path);
			const file = (path.split(/[\\/]/).pop() ?? path).replace(/\.json$/i, '');
			toast(m.newJob.pipelineExported(file));
		});

	const importPipeline = () =>
		attempt(async () => {
			const path = await pickPresetFile();
			if (!path) return;
			const imported = await api.importPipeline(path);
			pipelines = await api.listPipelines();
			toast(m.newJob.pipelineImported(imported.name));
		});

	const deletePipeline = (p: Pipeline) =>
		attempt(async () => {
			if (!(await ask(m.newJob.pipelineDeleteConfirm(p.name)))) return;
			await api.deletePipeline(p.id);
			pipelines = await api.listPipelines();
		});
</script>

<section class="pipelines" aria-labelledby="pipelines-label">
	<div class="head">
		<h2 id="pipelines-label" class="label">
			{m.newJob.pipelinesTitle}<TermLink term="pipeline" />
		</h2>
		<button class="btn sm ghost" onclick={importPipeline}>{m.newJob.pipelineImport}</button>
	</div>
	{#if pipelines.length > 0}
		<ul>
			{#each pipelines as p (p.id)}
				<li>
					<button class="pipeline" onclick={() => onapply(p)}>
						<strong>{p.name}</strong>
						<span class="muted">{m.task[p.task]}</span>
					</button>
					<button
						class="x"
						aria-label={m.newJob.pipelineExport(p.name)}
						title={m.newJob.pipelineExport(p.name)}
						onclick={() => exportPipeline(p)}><Icon name="download" size={14} /></button
					>
					<button
						class="x"
						aria-label={m.newJob.pipelineDelete(p.name)}
						title={m.newJob.pipelineDelete(p.name)}
						onclick={() => deletePipeline(p)}><Icon name="close" size={14} /></button
					>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.pipelines {
		display: grid;
		gap: var(--space-2);
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	li {
		display: flex;
		align-items: stretch;
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
	}
	li:hover {
		border-color: var(--line-hover);
	}
	.pipeline {
		display: grid;
		text-align: left;
		gap: 1px;
		padding: var(--space-2) var(--space-3);
		border: 0;
		background: none;
		cursor: pointer;
		color: var(--text-body);
		font-size: var(--type-sm);
	}
	.pipeline strong {
		color: var(--text-bright);
		font-weight: 500;
	}
	.pipeline .muted {
		font-size: var(--type-xs);
	}
	.x {
		display: grid;
		place-items: center;
		border: 0;
		border-left: 1px solid var(--line);
		background: none;
		cursor: pointer;
		padding: 0 var(--space-2);
		color: var(--text-muted);
	}
	.x:hover {
		color: var(--text-bright);
	}
</style>
