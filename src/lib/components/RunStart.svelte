<script lang="ts">
	// Running a saved workflow: the files, what each step sends and to whom, and what the steps
	// that read the files will cost. A step reading another's text costs what that text holds,
	// known only once it is read.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { Workflow } from '#lib/bindings/Workflow.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { app, go, refreshJobs, type FromProject } from '#lib/state.svelte.js';
	import { api, pickDocuments } from '#lib/tauri.js';

	let {
		workflow,
		paths,
		sources: given,
		project
	}: {
		workflow: Workflow;
		paths?: string[];
		/** Text frozen from earlier jobs: already read, nothing to inspect. */
		sources?: SourceFile[];
		project?: FromProject;
	} = $props();

	const m = messages();
	const w = m.workflows;
	let files = $state<InspectedFile[]>([]);
	let name = $state('');
	let costs = $state<(number | null | undefined)[]>([]);
	let error = $state<string | null>(null);
	let starting = $state(false);

	const sources = $derived(files.flatMap((f) => (f.source ? [f.source] : [])));
	const catalog = $derived(app.catalog!);
	const providerName = (id: string) => catalog.providers.find((p) => p.id === id)?.name ?? id;
	const local = (id: string) => catalog.providers.find((p) => p.id === id)?.local === true;
	const offline = (i: number) => workflow.steps[i].pipeline.setups.every((s) => local(s.provider));
	/** A step on this PC whose text a later step sends to a cloud provider. */
	const leaves = $derived(
		workflow.steps.some((s, i) => s.input.kind === 'step' && offline(s.input.step) && !offline(i))
	);

	async function add(chosen: string[]) {
		if (chosen.length === 0) return;
		error = null;
		try {
			const known = new Set(files.map((f) => f.path));
			files = [...files, ...(await api.inspectFiles(chosen.filter((p) => !known.has(p))))];
		} catch (e) {
			error = describeError(e, m);
		}
	}

	onMount(() => {
		name = workflow.name;
		if (given?.length) {
			files = given.map((source) => ({
				path: source.path,
				source,
				warnings: [],
				error: null,
				pieces: null
			}));
			name = given[0].file_name.replace(/\.txt$/, '');
		}
		void add(paths ?? []);
	});

	// What each step that reads the files will cost; the others wait for the text.
	$effect(() => {
		const current = $state.snapshot(sources);
		if (current.length === 0) {
			costs = [];
			return;
		}
		void Promise.all(
			workflow.steps.map(async (step) => {
				if (step.input.kind !== 'sources') return undefined;
				const p = step.pipeline;
				try {
					const estimate = await api.estimateCost({
						task: p.task,
						sources: current,
						setups: p.setups.map((s) => ({ ...s })),
						sample_pages: p.settings.sample_pages
					});
					return p.settings.pace === 'economy' ? estimate.total_economy_usd : estimate.total_usd;
				} catch {
					return null;
				}
			})
		).then((c) => (costs = c));
	});

	function cost(i: number): string {
		const input = workflow.steps[i].input;
		if (input.kind === 'step') return w.costLater(input.step + 1);
		const c = costs[i];
		return c === null || c === undefined ? w.costUnknown : w.costKnown(formatUsd(c));
	}

	function sends(i: number): string {
		const step = workflow.steps[i];
		if (offline(i)) return w.offline;
		const to = [...new Set(step.pipeline.setups.map((s) => providerName(s.provider)))].join(', ');
		return step.input.kind === 'step' ? w.sendsText(step.input.step + 1, to) : w.sendsFiles(to);
	}

	async function start() {
		error = null;
		starting = true;
		try {
			const view = await api.startRun({
				workflow: $state.snapshot(workflow),
				name,
				sources: $state.snapshot(sources),
				project: project?.id ?? null,
				collection: project?.collectionId ?? null
			});
			await refreshJobs();
			go({ name: 'workflows', runId: view.run.id });
		} catch (e) {
			error = describeError(e, m);
		} finally {
			starting = false;
		}
	}
</script>

<section class="card run" aria-labelledby="run-workflow">
	<h2 id="run-workflow">{w.runTitle}</h2>
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	{#if project}<p class="muted">{w.fromProject(project.name)}</p>{/if}

	<div class="files">
		<button class="btn" onclick={async () => add(await pickDocuments())}
			><Icon name="add-file" />{w.chooseFiles}</button
		>
		{#if files.length > 0}
			<span class="num">{w.files(sources.length)}</span>
		{/if}
	</div>
	{#if files.some((f) => f.error)}
		<ul class="problems">
			{#each files.filter((f) => f.error) as f (f.path)}
				<li class="chip danger">{f.path}: {f.error ? describeError(f.error, m) : ''}</li>
			{/each}
		</ul>
	{/if}

	{#if sources.length > 0}
		<h3>{w.sends}</h3>
		<ol class="plan">
			{#each workflow.steps as step, i (i)}
				<li>
					<span class="name">{w.stepNumber(i + 1)}: {step.pipeline.name}</span>
					<span class="muted">{sends(i)} · <span class="num">{cost(i)}</span></span>
				</li>
			{/each}
		</ol>
		{#if leaves}<p class="notice">{w.cloudAfterOffline}</p>{/if}

		<div class="start">
			<label class="field">
				<span>{w.runName}</span>
				<input type="text" bind:value={name} />
			</label>
			<button class="btn primary" onclick={start} disabled={starting}
				><Icon name="play" />{w.start}</button
			>
		</div>
	{/if}
</section>

<style>
	.run {
		display: grid;
		gap: var(--space-3);
		max-width: 44rem;
	}
	.files {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.problems {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-1);
	}
	h3 {
		font-size: var(--type-sm);
	}
	.plan {
		margin: 0;
		padding-left: 0;
		list-style: none;
		display: grid;
		gap: var(--space-2);
	}
	.plan li {
		display: grid;
		gap: 2px;
		font-size: var(--type-sm);
	}
	.name {
		color: var(--text-bright);
	}
	.start {
		display: flex;
		align-items: flex-end;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.start .field {
		flex: 1;
		min-width: 16rem;
	}
</style>
