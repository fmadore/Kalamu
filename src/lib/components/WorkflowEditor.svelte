<script lang="ts">
	// One workflow: its name and steps, each a saved pipeline with what it reads and whether to
	// stop for a review after it. Saving makes a new version; a saved workflow can be run.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import RunStart from './RunStart.svelte';
	import TermLink from './TermLink.svelte';
	import WorkflowDiagram from './WorkflowDiagram.svelte';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import type { StepInput } from '#lib/bindings/StepInput.js';
	import type { Task } from '#lib/bindings/Task.js';
	import type { Workflow } from '#lib/bindings/Workflow.js';
	import { describeError } from '#lib/errors.js';
	import { rulesFor } from '#lib/newJob.js';
	import { messages } from '#lib/i18n/index.js';
	import { app, go, hasKey, toast, type FromProject } from '#lib/state.svelte.js';
	import { api, ask, pickPipelineExport } from '#lib/tauri.js';
	import {
		blankWorkflow,
		type Example,
		exampleWorkflow,
		inputChoices,
		isRead,
		moveStep,
		removeStep,
		sameInput,
		stepFrom
	} from '#lib/workflows.js';

	let {
		workflowId,
		example,
		paths,
		project
	}: { workflowId: string; example?: Example; paths?: string[]; project?: FromProject } = $props();

	const m = messages();
	const w = m.workflows;
	let workflow = $state<Workflow>(blankWorkflow());
	/** The workflow as last saved, to tell whether the form holds changes. */
	let saved = $state<string>('');
	let pipelines = $state<Pipeline[]>([]);
	let adding = $state('');
	let error = $state<string | null>(null);
	let loaded = $state(false);

	const dirty = $derived(JSON.stringify(workflow) !== saved);
	const catalog = $derived(app.catalog!);
	const modelName = (provider: string, model: string) =>
		catalog.models.find((x) => x.provider === provider && x.id === model)?.name ?? model;

	onMount(async () => {
		try {
			const [all, list] = await Promise.all([api.listWorkflows(), api.listPipelines()]);
			pipelines = list;
			const found = all.find((x) => x.id === workflowId);
			if (!found && example) {
				// The researcher's usual formats for each task, as New job offers them.
				const formatsFor = (task: Task) => {
					const offered = rulesFor(app.rules, task).formats;
					const usual = (
						task === 'audio_transcription'
							? app.settings!.audio_formats
							: app.settings!.text_formats
					).filter((f) => offered.includes(f));
					return usual.length > 0 ? usual : offered.slice(0, 1);
				};
				workflow = exampleWorkflow(
					example,
					m.workflows.examples[example],
					catalog,
					app.rules,
					hasKey,
					formatsFor
				);
			}
			if (found) {
				workflow = found;
				saved = JSON.stringify(found);
			}
		} catch (e) {
			error = describeError(e, m);
		} finally {
			loaded = true;
		}
	});

	function add() {
		const pipeline = pipelines.find((p) => p.id === adding);
		if (!pipeline) return;
		workflow.steps = [...workflow.steps, stepFrom(pipeline, workflow.steps, app.rules)];
		if (!workflow.name.trim()) workflow.name = pipeline.name;
		adding = '';
	}

	function inputLabel(input: StepInput): string {
		if (input.kind === 'sources') return w.readsFiles;
		return w.readsStep(input.step + 1, workflow.steps[input.step]?.pipeline.name ?? '');
	}

	function setInput(index: number, value: string) {
		const choice = inputChoices(workflow.steps, index, app.rules).find(
			(c) => JSON.stringify(c) === value
		);
		if (choice) workflow.steps[index].input = choice;
	}

	async function save() {
		error = null;
		try {
			const result = await api.saveWorkflow($state.snapshot(workflow));
			workflow = result;
			saved = JSON.stringify(result);
			toast(w.savedAs(result.name, result.version));
			if (workflowId !== result.id)
				go({ name: 'workflows', workflowId: result.id, paths, project });
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function duplicate() {
		try {
			const copy = await api.duplicateWorkflow(workflow.id, w.copyName(workflow.name));
			go({ name: 'workflows', workflowId: copy.id });
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function exportFile() {
		const path = await pickPipelineExport(workflow.name);
		if (!path) return;
		try {
			await api.exportWorkflow(workflow.id, path);
			toast(w.exported(path));
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function remove() {
		if (!(await ask(w.deleteConfirm(workflow.name)))) return;
		try {
			await api.deleteWorkflow(workflow.id);
			go({ name: 'workflows' });
		} catch (e) {
			error = describeError(e, m);
		}
	}
</script>

<header class="masthead">
	<div class="title">
		<button class="btn ghost sm" onclick={() => go({ name: 'workflows', paths, project })}
			><Icon name="back" />{w.back}</button
		>
		<label class="field name">
			<span>{w.name}</span>
			<input type="text" bind:value={workflow.name} />
		</label>
		{#if workflow.version > 0}<p class="muted num">{w.version(workflow.version)}</p>{/if}
	</div>
	<div class="buttons">
		{#if workflow.id}
			<button class="btn ghost" onclick={duplicate}>{w.duplicate}</button>
			<button class="btn ghost" onclick={exportFile}><Icon name="download" />{w.export}</button>
			<button class="btn ghost danger" onclick={remove}>{w.delete}</button>
		{/if}
		<button class="btn primary" onclick={save} disabled={!dirty || workflow.steps.length === 0}
			><Icon name="check" />{w.save}</button
		>
	</div>
</header>

<div class="sheet">
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}

	<section aria-labelledby="workflow-steps">
		<h2 id="workflow-steps">{w.stepsTitle}<TermLink term="workflow" /></h2>
		{#if workflow.steps.length > 0}
			<WorkflowDiagram
				steps={workflow.steps.map((s, i) => ({
					task: s.pipeline.task,
					name: s.pipeline.name,
					review: s.review && isRead(workflow.steps, i)
				}))}
			/>
		{/if}
		{#if loaded && workflow.steps.length === 0}
			<p class="muted">{pipelines.length === 0 ? w.noPipelines : w.noSteps}</p>
		{/if}
		<ol class="steps">
			{#each workflow.steps as step, i (i)}
				{@const choices = inputChoices(workflow.steps, i, app.rules)}
				<li class="card step">
					<div class="head">
						<span class="what">
							<Icon name={step.pipeline.task} size={16} />
							<strong>{step.pipeline.name}</strong>
							<span class="muted">{m.task[step.pipeline.task]}</span>
						</span>
						<span class="moves">
							<button
								class="btn ghost sm icon"
								aria-label={w.moveUp(i + 1)}
								title={w.moveUp(i + 1)}
								disabled={i === 0}
								onclick={() => (workflow.steps = moveStep(workflow.steps, i, i - 1))}
								><Icon name="up" /></button
							>
							<button
								class="btn ghost sm icon"
								aria-label={w.moveDown(i + 1)}
								title={w.moveDown(i + 1)}
								disabled={i === workflow.steps.length - 1}
								onclick={() => (workflow.steps = moveStep(workflow.steps, i, i + 1))}
								><Icon name="down" /></button
							>
							<button
								class="btn ghost sm icon danger"
								aria-label={w.remove(i + 1)}
								title={w.remove(i + 1)}
								onclick={() => (workflow.steps = removeStep(workflow.steps, i))}
								><Icon name="close" /></button
							>
						</span>
					</div>
					<p class="models">
						{step.pipeline.setups.map((s) => modelName(s.provider, s.model)).join(' · ')}
					</p>
					{#if choices.length < 2}
						<p class="reads"><span class="label">{w.reads}:</span> {inputLabel(step.input)}</p>
					{:else}
						<label class="field">
							<span>{w.reads}</span>
							<select
								value={JSON.stringify(step.input)}
								onchange={(e) => setInput(i, e.currentTarget.value)}
							>
								{#each choices as choice (JSON.stringify(choice))}
									<option value={JSON.stringify(choice)} selected={sameInput(choice, step.input)}
										>{inputLabel(choice)}</option
									>
								{/each}
							</select>
						</label>
					{/if}
					{#if isRead(workflow.steps, i)}
						<label class="check">
							<input type="checkbox" bind:checked={workflow.steps[i].review} />
							<span>{w.review}<span class="hint">{w.reviewHint}</span></span>
						</label>
					{/if}
				</li>
			{/each}
		</ol>
		{#if pipelines.length > 0}
			<div class="add">
				<label class="field">
					<span>{w.addStep}</span>
					<select bind:value={adding}>
						<option value="" disabled>{w.addStepHint}</option>
						{#each pipelines as p (p.id)}
							<option value={p.id}>{p.name} · {m.task[p.task]}</option>
						{/each}
					</select>
				</label>
				<button class="btn" onclick={add} disabled={!adding}><Icon name="plus" />{w.addStep}</button
				>
			</div>
		{/if}
		{#if workflow.id}<p class="muted note">{w.savedNote}</p>{/if}
	</section>

	{#if workflow.id && workflow.steps.length > 0}
		{#if dirty}
			<p class="notice">{w.unsaved}</p>
		{:else}
			<RunStart {workflow} {paths} {project} />
		{/if}
	{/if}
</div>

<style>
	.title {
		display: grid;
		gap: var(--space-2);
		justify-items: start;
		min-width: min(28rem, 100%);
	}
	.name {
		width: 100%;
	}
	.buttons {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
		align-items: flex-start;
	}
	section {
		display: grid;
		gap: var(--space-3);
	}
	.steps {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-3);
	}
	.step {
		display: grid;
		gap: var(--space-2);
		max-width: 44rem;
	}
	.head {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.what {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex: 1;
		min-width: 0;
		flex-wrap: wrap;
	}
	.moves {
		display: flex;
		gap: var(--space-1);
	}
	.reads {
		margin: 0;
		font-size: var(--type-sm);
	}
	.models {
		margin: 0;
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.check {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
		font-size: var(--type-sm);
	}
	.hint {
		display: block;
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.add {
		display: flex;
		gap: var(--space-2);
		align-items: flex-end;
		flex-wrap: wrap;
		max-width: 44rem;
	}
	.add .field {
		flex: 1;
		min-width: 16rem;
	}
	.note {
		font-size: var(--type-sm);
		margin: 0;
	}
</style>
