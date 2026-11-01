<script lang="ts">
	// The saved workflows, and every run with where it stands, newest first. Files chosen in a
	// project wait here until a workflow is picked to run on them.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import type { RunView } from '#lib/bindings/RunView.js';
	import type { Workflow } from '#lib/bindings/Workflow.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { app, go, toast, type FromProject } from '#lib/state.svelte.js';
	import { api, pickPresetFile } from '#lib/tauri.js';
	import { tone } from '#lib/workflows.js';

	let { paths, project }: { paths?: string[]; project?: FromProject } = $props();

	const m = messages();
	const w = m.workflows;
	let workflows = $state<Workflow[]>([]);
	let runs = $state<RunView[]>([]);
	let loaded = $state(false);
	let error = $state<string | null>(null);

	async function load() {
		try {
			[workflows, runs] = await Promise.all([api.listWorkflows(), api.listRuns()]);
		} catch (e) {
			error = describeError(e, m);
		} finally {
			loaded = true;
		}
	}

	async function importOne() {
		const path = await pickPresetFile();
		if (!path) return;
		try {
			const workflow = await api.importWorkflow(path);
			toast(w.imported(workflow.name));
			await load();
		} catch (e) {
			error = describeError(e, m);
		}
	}

	const open = (id: string) => go({ name: 'workflows', workflowId: id, paths, project });

	onMount(() => void load());
	// A run moving on (a step started, finished or approved) is read again.
	$effect(() => {
		void app.runs;
		if (loaded) void api.listRuns().then((r) => (runs = r));
	});
</script>

<header class="masthead">
	<div>
		<h1>{w.title}</h1>
		<p class="lede">{w.lede}</p>
	</div>
	<div class="buttons">
		<button class="btn" onclick={importOne}><Icon name="download" />{w.import}</button>
		<button class="btn primary" onclick={() => open('new')}
			><Icon name="plus" />{w.newWorkflow}</button
		>
	</div>
</header>

<div class="sheet">
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	{#if project && paths?.length}
		<p class="chip">{w.files(paths.length)} · {w.fromProject(project.name)}</p>
	{/if}

	<section aria-labelledby="saved-workflows">
		<h2 id="saved-workflows">{w.saved}</h2>
		{#if loaded && workflows.length === 0}
			<p class="muted">{w.empty}</p>
		{/if}
		{#if workflows.length > 0}
			<ul class="list">
				{#each workflows as workflow (workflow.id)}
					<li>
						<button class="row" onclick={() => open(workflow.id)}>
							<span class="name">
								<Icon name="workflow" size={16} />
								<span class="ellipsis">{workflow.name}</span>
							</span>
							<span class="steps">
								{workflow.steps.map((s) => s.pipeline.name).join(' → ')}
							</span>
							<span class="meta num">
								{w.steps(workflow.steps.length)} · {w.version(workflow.version)}
							</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section aria-labelledby="workflow-runs">
		<h2 id="workflow-runs">{w.runs}</h2>
		{#if loaded && runs.length === 0}
			<p class="muted">{w.noRuns}</p>
		{/if}
		{#if runs.length > 0}
			<ul class="list">
				{#each runs as { run, status } (run.id)}
					<li>
						<button class="row" onclick={() => go({ name: 'workflows', runId: run.id })}>
							<span class="name">
								<span class="ellipsis">{run.name}</span>
								<span class="chip {tone(status)}">{w.runStatus[status]}</span>
							</span>
							<span class="meta num">
								{w.runOf(run.workflow_name, run.version)} · {w.files(run.sources.length)} ·
								{w.started(formatDateTime(run.created_at))}
							</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.buttons {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	section {
		display: grid;
		gap: var(--space-2);
	}
	.list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		border-top: 1px solid var(--line);
	}
	.row {
		display: grid;
		gap: var(--space-1);
		width: 100%;
		padding: var(--space-3) var(--space-2);
		border: 0;
		border-bottom: 1px solid var(--line-soft);
		background: none;
		text-align: left;
		cursor: pointer;
	}
	.row:hover {
		background: var(--wash);
	}
	.name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-bright);
		font-weight: 600;
		min-width: 0;
	}
	.ellipsis {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.steps {
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.meta {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
</style>
