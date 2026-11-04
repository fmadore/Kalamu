<script lang="ts">
	// The saved workflows, and every run with where it stands, newest first. Files chosen in a
	// project wait here until a workflow is picked to run on them.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import TermLink from './TermLink.svelte';
	import WorkflowDiagram from './WorkflowDiagram.svelte';
	import type { RunView } from '#lib/bindings/RunView.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { Workflow } from '#lib/bindings/Workflow.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { app, go, hasKey, toast, type FromProject } from '#lib/state.svelte.js';
	import { api, pickPresetFile } from '#lib/tauri.js';
	import { EXAMPLES, type Example, exampleWorkflow, startsOn, tone } from '#lib/workflows.js';

	let {
		paths,
		sources,
		project
	}: { paths?: string[]; sources?: SourceFile[]; project?: FromProject } = $props();

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

	const open = (id: string) => go({ name: 'workflows', workflowId: id, paths, sources, project });
	/** Text already read, sent here from a job or a document: what a workflow starts on. */
	const kinds = $derived(sources?.length ? [...new Set(sources.map((s) => s.kind))] : null);
	const fits = (workflow: Workflow) => !kinds || startsOn(workflow, kinds, app.rules);
	const examples = $derived(
		(Object.keys(EXAMPLES) as Example[]).filter((kind) =>
			fits(exampleWorkflow(kind, w.examples[kind], app.catalog!, app.rules, hasKey, () => []))
		)
	);

	onMount(() => void load());
	// A run moving on (a step started, finished or approved) is read again.
	$effect(() => {
		void app.runs;
		if (loaded) void api.listRuns().then((r) => (runs = r));
	});
</script>

<header class="masthead">
	<div>
		<h1>{w.title}<TermLink term="workflow" /></h1>
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
	{#if sources?.length}
		<p class="notice">
			{w.onText(sources.map((s) => s.file_name).join(', '))}
			{#if project}{w.fromProject(project.name)}{/if}
		</p>
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
						<button class="row" onclick={() => open(workflow.id)} disabled={!fits(workflow)}>
							<span class="name">
								<Icon name="workflow" size={16} />
								<span class="ellipsis">{workflow.name}</span>
							</span>
							<span class="steps">
								{workflow.steps.map((s) => s.pipeline.name).join(' → ')}
							</span>
							<span class="meta num">
								{w.steps(workflow.steps.length)} · {w.version(workflow.version)}
								{#if !fits(workflow)}· {w.notOnText}{/if}
							</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section aria-labelledby="workflow-examples">
		<h2 id="workflow-examples">{w.examplesTitle}</h2>
		<p class="muted hint">{w.examplesHint}</p>
		<ul class="examples">
			{#each examples as kind (kind)}
				{@const words = w.examples[kind]}
				<li class="card example">
					<h3>{words.name}</h3>
					<p class="about">{words.about}</p>
					<WorkflowDiagram
						compact
						steps={EXAMPLES[kind].map((s, i) => ({
							task: s.task,
							name: words.steps[i],
							review: i === 0
						}))}
					/>
					<button
						class="btn sm"
						onclick={() =>
							go({ name: 'workflows', workflowId: 'new', example: kind, paths, sources, project })}
						>{w.useExample}</button
					>
				</li>
			{/each}
		</ul>
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
	.row:hover:not(:disabled) {
		background: var(--wash);
	}
	.row:disabled {
		cursor: default;
		opacity: 0.55;
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
	.hint {
		margin: 0;
		font-size: var(--type-sm);
	}
	/* As many examples a row as fit, each at least wide enough for its diagram's steps. */
	.examples {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 30rem), 1fr));
		gap: var(--space-6) var(--space-5);
	}
	/* Room under the card's rule before its title. */
	.example {
		display: grid;
		grid-template-rows: auto 1fr auto auto;
		gap: var(--space-2);
		padding-top: var(--space-4);
		align-content: start;
		justify-items: start;
		min-width: 0;
	}
	.example :global(.workflow-diagram) {
		justify-self: stretch;
	}
	.example h3 {
		font-size: var(--type-md);
	}
	.about {
		margin: 0;
		font-size: var(--type-sm);
		color: var(--text-secondary);
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
