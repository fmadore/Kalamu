<script lang="ts">
	// One workflow: its name and steps, each a saved pipeline or a prompt on the model New job
	// would offer, with what it reads and whether to stop for a review after it. A step of one
	// set-up can take another prompt. Saving makes a new version; a saved workflow can be run.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import RunStart from './RunStart.svelte';
	import TermLink from './TermLink.svelte';
	import WorkflowDiagram from './WorkflowDiagram.svelte';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { StepInput } from '#lib/bindings/StepInput.js';
	import type { Task } from '#lib/bindings/Task.js';
	import type { Workflow } from '#lib/bindings/Workflow.js';
	import { describeError } from '#lib/errors.js';
	import { acceptsPrompt, MODES, rulesFor, taskOf } from '#lib/newJob.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		app,
		go,
		guardDraft,
		returnTo,
		hasKey,
		toast,
		type FromProject
	} from '#lib/state.svelte.js';
	import { api, ask, pickJsonSave } from '#lib/tauri.js';
	import {
		blankWorkflow,
		type Example,
		exampleWorkflow,
		inputChoices,
		isRead,
		moveStep,
		presetPipeline,
		removeStep,
		sameInput,
		stepFrom,
		withPreset
	} from '#lib/workflows.js';

	let {
		workflowId,
		example,
		paths,
		sources,
		project
	}: {
		workflowId: string;
		example?: Example;
		paths?: string[];
		sources?: SourceFile[];
		project?: FromProject;
	} = $props();

	const m = messages();
	const w = m.workflows;
	let workflow = $state<Workflow>(blankWorkflow());
	/** The workflow as last saved, to tell whether the form holds changes. */
	let saved = $state<string>('');
	/** The workflow as it was opened or last saved, to tell whether leaving would lose
	 * anything: a new one counts as changed only once something in it was. */
	let openedAs = '';
	let pipelines = $state<Pipeline[]>([]);
	/** The prompts each task offers, as New job lists them. */
	let presets = $state<Preset[]>([]);
	/** The step chosen to add: `pipeline:<id>` or `preset:<id>`. */
	let adding = $state('');
	let error = $state<string | null>(null);
	let loaded = $state(false);

	const dirty = $derived(JSON.stringify(workflow) !== saved);
	const catalog = $derived(app.catalog!);
	const modelName = (provider: string, model: string) =>
		catalog.models.find((x) => x.provider === provider && x.id === model)?.name ?? model;

	/** The tasks, in New job's order. */
	const tasks: Task[] = [...new Set(MODES.map(taskOf))];
	const presetsOf = (task: Task) => presets.filter((p) => p.task === task);

	/** The researcher's usual formats for each task, as New job offers them. */
	function formatsFor(task: Task) {
		const offered = rulesFor(app.rules, task).formats;
		const usual = (
			task === 'audio_transcription' ? app.settings!.audio_formats : app.settings!.text_formats
		).filter((f) => offered.includes(f));
		return usual.length > 0 ? usual : offered.slice(0, 1);
	}

	onMount(async () => {
		try {
			const [all, list, offered] = await Promise.all([
				api.listWorkflows(),
				api.listPipelines(),
				Promise.all(tasks.map((t) => api.listPresets(t)))
			]);
			pipelines = list;
			presets = offered.flat();
			const found = all.find((x) => x.id === workflowId);
			if (!found && example) {
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
			openedAs = JSON.stringify(workflow);
			loaded = true;
		}
	});

	onMount(() =>
		guardDraft({
			dirty: () => loaded && JSON.stringify(workflow) !== openedAs,
			discard: () => (openedAs = JSON.stringify(workflow))
		})
	);

	function add() {
		const at = adding.indexOf(':');
		const [kind, id] = [adding.slice(0, at), adding.slice(at + 1)];
		const preset = kind === 'preset' ? presets.find((p) => p.id === id) : undefined;
		const pipeline = preset
			? presetPipeline(preset.task, preset.id, preset.name, catalog, app.rules, hasKey, formatsFor)
			: pipelines.find((p) => kind === 'pipeline' && p.id === id);
		if (!pipeline) return;
		workflow.steps = [...workflow.steps, stepFrom(pipeline, workflow.steps, app.rules)];
		if (!workflow.name.trim()) workflow.name = pipeline.name;
		adding = '';
	}

	/** The prompt of a step of one set-up on a model that takes one, to change; else none. */
	function promptOf(i: number): string | null {
		const setups = workflow.steps[i].pipeline.setups;
		const model = catalog.models.find(
			(x) => x.provider === setups[0]?.provider && x.id === setups[0]?.model
		);
		if (setups.length !== 1 || !acceptsPrompt(model)) return null;
		return setups[0].preset_id ?? '';
	}

	function setPrompt(i: number, id: string) {
		const step = workflow.steps[i];
		const preset = presets.find((p) => p.id === id);
		if (!preset) return;
		const previous = presets.find((p) => p.id === step.pipeline.setups[0]?.preset_id);
		workflow.steps[i] = withPreset(step, preset, previous);
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
			openedAs = saved;
			toast(w.savedAs(result.name, result.version));
			if (workflowId !== result.id)
				go({ name: 'workflows', workflowId: result.id, paths, sources, project });
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
		const path = await pickJsonSave(workflow.name);
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
			// Deleted on purpose: nothing left to ask about.
			openedAs = JSON.stringify(workflow);
			go({ name: 'workflows' });
		} catch (e) {
			error = describeError(e, m);
		}
	}
</script>

<header class="masthead">
	<div class="title">
		<button
			class="btn ghost sm"
			onclick={() => returnTo({ name: 'workflows', paths, sources, project })}
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
		<button
			class="btn primary"
			onclick={save}
			disabled={!dirty || workflow.steps.length === 0}
			title={workflow.steps.length === 0 ? w.saveNeedsStep : undefined}
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
			<p class="muted">{w.noSteps}</p>
		{/if}
		<ol class="steps">
			{#each workflow.steps as step, i (i)}
				{@const choices = inputChoices(workflow.steps, i, app.rules)}
				{@const prompt = promptOf(i)}
				{@const offered = presetsOf(step.pipeline.task)}
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
					{#if prompt !== null && offered.length > 0}
						<label class="field">
							<span>{w.prompt}</span>
							<select value={prompt} onchange={(e) => setPrompt(i, e.currentTarget.value)}>
								{#if !offered.some((p) => p.id === prompt)}
									<option value={prompt} disabled>{w.promptMissing}</option>
								{/if}
								{#each offered as p (p.id)}
									<option value={p.id}>{p.name}</option>
								{/each}
							</select>
						</label>
					{/if}
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
		{#if pipelines.length > 0 || presets.length > 0}
			<div class="add">
				<label class="field">
					<span>{w.addStep}</span>
					<select bind:value={adding}>
						<option value="" disabled>{w.addStepHint}</option>
						{#if pipelines.length > 0}
							<optgroup label={w.fromPipelines}>
								{#each pipelines as p (p.id)}
									<option value="pipeline:{p.id}">{p.name} · {m.task[p.task]}</option>
								{/each}
							</optgroup>
						{/if}
						{#each tasks as t (t)}
							{#if presetsOf(t).length > 0}
								<optgroup label={w.fromPrompts(m.task[t])}>
									{#each presetsOf(t) as p (p.id)}
										<option value="preset:{p.id}">{p.name}</option>
									{/each}
								</optgroup>
							{/if}
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
			<RunStart {workflow} {paths} {sources} {project} />
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
