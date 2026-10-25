<script lang="ts">
	import { onMount } from 'svelte';
	import FileList from './FileList.svelte';
	import Icon from './Icon.svelte';
	import SetupList from './SetupList.svelte';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, languageName, messages } from '#lib/i18n/index.js';
	import type { LineBreaks } from '#lib/bindings/LineBreaks.js';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import {
		acceptsPrompt,
		type Draft,
		FORMATS,
		modelInfo,
		newDraft,
		offeredFormats,
		readsKind,
		suggestName,
		toRequest
	} from '#lib/newJob.js';
	import { api, ask, onDrop, pickFolder } from '#lib/tauri.js';
	import { app, go, hasKey, liveJob, refreshJobs, toast } from '#lib/state.svelte.js';

	let { opening = {} }: { opening?: { task?: Task; paths?: string[] } } = $props();

	const m = messages();

	let task = $state<Task>('text_extraction');
	let files = $state<InspectedFile[]>([]);
	let inspecting = $state(false);
	let languageMode = $state<'auto' | 'explicit'>('auto');
	let languages = $state<string[]>([]);
	let lineBreaks = $state<LineBreaks>('running_text');
	let drafts = $state<Draft[]>([]);
	let presets = $state<Preset[]>([]);
	let sample = $state(false);
	let samplePages = $state(3);
	let formats = $state<ExportFormat[]>([]);
	let destination = $state<OutputDestination>({ kind: 'beside_sources' });
	let name = $state('');
	let estimates = $state<EstimateRow[]>([]);
	let error = $state<string | null>(null);
	let starting = $state(false);
	let pipelines = $state<Pipeline[]>([]);
	/** The name being typed for a new pipeline, while saving one. */
	let naming = $state<string | null>(null);

	const catalog = $derived(app.catalog!);
	const sources = $derived(files.flatMap((f) => (f.source ? [f.source] : [])));

	async function loadTask(next: Task) {
		task = next;
		presets = await api.listPresets(task);
		drafts = [newDraft(catalog, task, presets[0]?.id ?? null, hasKey)];
		formats = [
			...(task === 'audio_transcription' ? app.settings!.audio_formats : app.settings!.text_formats)
		].filter((f) => FORMATS[task].includes(f));
		// An analysis is written in one language.
		if (task === 'analysis') languages = languages.slice(0, 1);
		files = files.filter((f) => readsKind(task, f.source?.kind));
	}

	async function addPaths(paths: string[]) {
		if (paths.length === 0) return;
		error = null;
		inspecting = true;
		try {
			const expanded = await api.expandPaths(paths, task);
			const known = new Set(files.map((f) => f.path));
			const inspected = await api.inspectFiles(expanded.filter((p) => !known.has(p)));
			files = [...files, ...inspected];
			if (!name) name = suggestName(paths, expanded);
		} catch (e) {
			error = describeError(e, m);
		} finally {
			inspecting = false;
		}
	}

	const ready = $derived(
		sources.length > 0 &&
			drafts.length > 0 &&
			drafts.every((d) => d.model && hasKey(d.provider)) &&
			formats.length > 0
	);

	let estimateTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const req = {
			task,
			sources: $state.snapshot(sources),
			setups: drafts.filter((d) => d.model).map((d) => toRequest(catalog, task, d)),
			sample_pages: sample ? samplePages : null
		};
		if (estimateTimer) clearTimeout(estimateTimer);
		if (req.sources.length === 0 || req.setups.length === 0) {
			estimates = [];
			return;
		}
		estimateTimer = setTimeout(() => {
			api
				.estimateCost(req)
				.then((rows) => (estimates = rows))
				.catch(() => (estimates = []));
		}, 250);
	});

	const totalEstimate = $derived(
		estimates.every((r) => r.estimate.usd !== null)
			? estimates.reduce((sum, r) => sum + (r.estimate.usd ?? 0), 0)
			: null
	);

	async function chooseDestination() {
		const folder = await pickFolder();
		if (folder) destination = { kind: 'folder', path: folder };
	}

	/** The settings shared by every set-up, as a job and a pipeline store them. */
	function jobSettings() {
		return {
			languages:
				languageMode === 'explicit' && languages.length > 0
					? { mode: 'explicit' as const, languages: [...languages] }
					: { mode: 'auto' as const },
			sample_pages: sample ? samplePages : null,
			line_breaks: task === 'text_extraction' ? lineBreaks : null
		};
	}

	/** Fills the form from a saved pipeline; the files stay as they are. */
	async function applyPipeline(p: Pipeline) {
		await loadTask(p.task);
		drafts = p.setups.map((s) => ({
			provider: s.provider,
			model: s.model,
			presetId: s.preset_id,
			label: s.label ?? '',
			diarize: s.options.diarize,
			timestamps: s.options.timestamps
		}));
		languageMode = p.settings.languages.mode;
		languages = p.settings.languages.mode === 'explicit' ? [...p.settings.languages.languages] : [];
		lineBreaks = p.settings.line_breaks ?? 'running_text';
		sample = p.settings.sample_pages !== null;
		samplePages = p.settings.sample_pages ?? samplePages;
		formats = p.formats.filter((f) => FORMATS[p.task].includes(f));
		destination = p.destination;
		toast(m.newJob.pipelineApplied(p.name));
	}

	async function savePipeline() {
		const pipelineName = naming?.trim();
		if (!pipelineName) return;
		error = null;
		try {
			const saved = await api.savePipeline({
				id: '',
				name: pipelineName,
				task,
				setups: drafts.map((d) => {
					const r = toRequest(catalog, task, d);
					return {
						provider: r.provider,
						model: r.model,
						preset_id: r.preset_id,
						label: r.label,
						options: r.options
					};
				}),
				settings: jobSettings(),
				formats: [...formats],
				destination: $state.snapshot(destination),
				updated_at: 0
			});
			pipelines = await api.listPipelines();
			naming = null;
			toast(m.newJob.pipelineSaved(saved.name));
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function deletePipeline(p: Pipeline) {
		if (!(await ask(m.newJob.pipelineDeleteConfirm(p.name)))) return;
		try {
			await api.deletePipeline(p.id);
			pipelines = await api.listPipelines();
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function start() {
		error = null;
		starting = true;
		try {
			const job = await api.createJob({
				name: name.trim() || sources[0]?.file_name || m.newJob.title,
				task,
				sources: $state.snapshot(sources),
				setups: drafts.map((d) => toRequest(catalog, task, d)),
				settings: jobSettings(),
				output: {
					formats: formats.filter((f) => offeredFormats(catalog, task, drafts).includes(f)),
					destination: $state.snapshot(destination),
					options: $state.snapshot(app.settings!.export_options)
				}
			});
			liveJob(job.id);
			await api.startJob(job.id);
			await refreshJobs();
			files = [];
			name = '';
			go({ name: 'jobs', jobId: job.id });
		} catch (e) {
			error = describeError(e, m);
		} finally {
			starting = false;
		}
	}

	onMount(() => {
		// Opened from a finished job's "Summarise" button: its results are the files.
		void loadTask(opening.task ?? task).then(() =>
			opening.paths ? addPaths(opening.paths) : undefined
		);
		api
			.listPipelines()
			.then((p) => (pipelines = p))
			.catch(() => undefined);
		let unlisten: (() => void) | undefined;
		void onDrop((paths) => void addPaths(paths)).then((u) => (unlisten = u));
		return () => unlisten?.();
	});
</script>

<header class="masthead">
	<div>
		<h1>{m.newJob.title}</h1>
		<p class="lede">{m.newJob.lede}</p>
	</div>
</header>

<div class="sheet">
	{#if pipelines.length > 0}
		<section class="pipelines" aria-labelledby="pipelines-label">
			<h2 id="pipelines-label" class="label">{m.newJob.pipelinesTitle}</h2>
			<ul>
				{#each pipelines as p (p.id)}
					<li>
						<button class="pipeline" onclick={() => applyPipeline(p)}>
							<strong>{p.name}</strong>
							<span class="muted">{m.task[p.task]}</span>
						</button>
						<button
							class="x"
							aria-label={m.newJob.pipelineDelete(p.name)}
							title={m.newJob.pipelineDelete(p.name)}
							onclick={() => deletePipeline(p)}><Icon name="close" size={14} /></button
						>
					</li>
				{/each}
			</ul>
		</section>
	{/if}

	<section aria-labelledby="task-label">
		<h2 id="task-label" class="label">{m.newJob.taskLabel}</h2>
		<div class="tasks">
			{#each ['text_extraction', 'audio_transcription', 'analysis'] as const as t (t)}
				<button
					class="task"
					class:selected={task === t}
					aria-pressed={task === t}
					onclick={() => loadTask(t)}
				>
					<strong>{m.task[t]}</strong>
					<span class="muted">{m.task[`${t}Hint`]}</span>
				</button>
			{/each}
		</div>
	</section>

	<FileList bind:files {task} {inspecting} onadd={addPaths} />

	<section class="card block" aria-labelledby="lang-title">
		<h2 id="lang-title">
			{task === 'analysis' ? m.newJob.analysisLanguageTitle : m.newJob.languageTitle}
		</h2>
		<div class="row">
			<label class="inline"
				><input type="radio" bind:group={languageMode} value="auto" />
				{task === 'analysis' ? m.newJob.analysisLanguageAuto : m.newJob.languageAuto}</label
			>
			<label class="inline"
				><input type="radio" bind:group={languageMode} value="explicit" />
				{m.newJob.languagePick}</label
			>
		</div>
		{#if languageMode === 'explicit'}
			<div class="row wrap">
				{#each languages as code (code)}
					<span class="chip">
						{languageName(code)}
						<button
							class="x"
							aria-label="{m.newJob.remove} {languageName(code)}"
							onclick={() => (languages = languages.filter((c) => c !== code))}
							><Icon name="close" size={12} /></button
						>
					</span>
				{/each}
				<select
					aria-label={m.newJob.addLanguage}
					onchange={(e) => {
						const v = e.currentTarget.value;
						if (v && task === 'analysis') languages = [v];
						else if (v && !languages.includes(v)) languages = [...languages, v];
						e.currentTarget.value = '';
					}}
				>
					<option value="">{m.newJob.addLanguage}</option>
					{#each catalog.languages.filter((l) => !languages.includes(l.code)) as l (l.code)}
						<option value={l.code}>{languageName(l.code, l.name)}</option>
					{/each}
				</select>
			</div>
		{/if}
		<p class="muted small">
			{task === 'analysis' ? m.newJob.analysisLanguageHint : m.newJob.languageHint}
		</p>
	</section>

	<SetupList bind:drafts {task} {presets} />

	{#if task === 'text_extraction'}
		<section class="card block" aria-labelledby="lines-title">
			<h2 id="lines-title">{m.newJob.linesTitle}</h2>
			<div class="choices">
				{#each ['running_text', 'as_on_page'] as const as mode (mode)}
					<label class="choice" class:selected={lineBreaks === mode}>
						<input type="radio" bind:group={lineBreaks} value={mode} />
						<span>
							<strong>{m.newJob.lines[mode]}</strong>
							<span class="muted small">{m.newJob.lines[`${mode}_hint`]}</span>
						</span>
					</label>
				{/each}
			</div>
			{#if drafts.some((d) => !acceptsPrompt(modelInfo(catalog, d)))}
				<p class="muted small">{m.newJob.linesNoPrompt}</p>
			{/if}
		</section>
	{/if}

	<section class="card block" aria-labelledby="out-title">
		<h2 id="out-title">{m.newJob.outputTitle}</h2>
		{#if task === 'text_extraction'}
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
			<legend class="label">{m.newJob.formats}</legend>
			<div class="row wrap">
				{#each offeredFormats(catalog, task, drafts) as f (f)}
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
				<span class="small path">{destination.path}</span>
				<button class="btn sm" onclick={chooseDestination}>{m.newJob.chooseFolder}</button>
			{/if}
		</div>
		<label class="field name">
			<span>{m.newJob.name}</span>
			<input type="text" bind:value={name} />
		</label>
	</section>
</div>

<!-- Stays at the bottom of the window while the form scrolls under it. -->
<footer class="start">
	<div class="start-inner">
		<div>
			<span class="label">{m.newJob.estimate}</span>
			{#if estimates.length > 0}
				<div class="estimate">
					{#if totalEstimate !== null}
						<strong>{formatUsd(totalEstimate)}</strong>
					{/if}
					{#each estimates as row (row.provider + row.model)}
						<span class="muted small">
							{row.model}: {row.estimate.usd !== null
								? formatUsd(row.estimate.usd)
								: m.newJob.estimateUnknown}
						</span>
					{/each}
				</div>
				<p class="muted small">{m.newJob.estimateNote}</p>
			{:else}
				<p class="muted">—</p>
			{/if}
		</div>
		<div class="start-actions">
			{#if error}
				<p class="chip danger">{error}</p>
			{/if}
			{#if naming === null}
				<button class="btn ghost" disabled={drafts.length === 0} onclick={() => (naming = '')}
					>{m.newJob.pipelineSave}</button
				>
			{:else}
				<form
					class="naming"
					onsubmit={(e) => {
						e.preventDefault();
						void savePipeline();
					}}
				>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						bind:value={naming}
						placeholder={m.newJob.pipelineName}
						aria-label={m.newJob.pipelineName}
						autofocus
					/>
					<button class="btn" type="submit" disabled={!naming.trim()}>{m.common.save}</button>
					<button class="btn ghost" type="button" onclick={() => (naming = null)}
						>{m.common.cancel}</button
					>
				</form>
			{/if}
			<button class="btn primary big" disabled={!ready || starting} onclick={start}>
				{starting ? m.newJob.starting : m.newJob.start}
			</button>
		</div>
	</div>
</footer>

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
	.row.wrap {
		flex-wrap: wrap;
	}
	.small {
		font-size: var(--type-sm);
	}
	.pipelines ul {
		list-style: none;
		margin: var(--space-2) 0 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.pipelines li {
		display: flex;
		align-items: stretch;
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
	}
	.pipelines li:hover {
		border-color: var(--accent);
	}
	.pipeline {
		display: grid;
		text-align: left;
		gap: 2px;
		padding: var(--space-2) var(--space-3);
		border: 0;
		background: none;
		cursor: pointer;
		color: var(--text-body);
	}
	.pipeline strong {
		color: var(--text-bright);
	}
	.pipeline .muted {
		font-size: var(--type-xs);
	}
	.pipelines .x {
		padding: 0 var(--space-2);
		border-left: 1px solid var(--line);
		color: var(--text-muted);
	}
	.naming {
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.naming input {
		width: 14rem;
	}
	.tasks {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-3);
		margin-top: var(--space-2);
	}
	.task {
		text-align: left;
		padding: var(--space-4);
		display: grid;
		align-content: start;
		gap: var(--space-1);
		cursor: pointer;
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-card);
		background: var(--surface-2);
		color: var(--text-body);
		transition:
			border-color 120ms ease-out,
			background-color 120ms ease-out;
	}
	.task strong {
		font-family: var(--font-text);
		font-size: 1.25rem;
		font-weight: 500;
		color: var(--text-bright);
	}
	.task:hover {
		border-color: var(--line-hover);
	}
	.task.selected {
		border-color: var(--accent);
		background: var(--accent-wash);
		box-shadow: inset 0 0 0 1px var(--accent);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
	}
	.x {
		display: grid;
		place-items: center;
		border: 0;
		background: none;
		cursor: pointer;
		padding: 2px;
		color: inherit;
	}
	.path {
		overflow-wrap: anywhere;
	}
	.choices {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--space-3);
	}
	.choice {
		display: flex;
		gap: var(--space-3);
		align-items: start;
		padding: var(--space-3) var(--space-4);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-card);
		cursor: pointer;
		transition: border-color 120ms ease-out;
	}
	.choice:hover {
		border-color: var(--line-hover);
	}
	.choice.selected {
		border-color: var(--accent);
		background: var(--accent-wash);
	}
	.choice input {
		margin-top: 0.3em;
	}
	.choice > span {
		display: grid;
		gap: var(--space-1);
	}
	.choice strong {
		color: var(--text-bright);
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
	/* The footer spans the window, set off from the sections by a lifted edge rather than their
	   black rule, so content passing beneath it reads as passing under, not as cut. */
	.start {
		position: sticky;
		bottom: 0;
		z-index: 1;
		background: var(--surface-0);
		border-top: 1px solid var(--line-strong);
		box-shadow: 0 -10px 24px -18px rgba(15, 20, 35, 0.45);
	}
	.start-inner {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-4);
		max-width: 64rem;
		padding: var(--space-3) var(--space-6) var(--space-4);
	}
	.estimate {
		display: flex;
		gap: var(--space-3);
		align-items: baseline;
		flex-wrap: wrap;
	}
	.start-actions {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.big {
		min-height: var(--control-lg);
		padding: 0 var(--space-6);
	}
	@media (max-width: 760px) {
		.tasks,
		.choices {
			grid-template-columns: 1fr;
		}
		.start-inner {
			flex-wrap: wrap;
		}
	}
	@media (max-width: 820px) {
		.start-inner {
			padding: var(--space-3) var(--space-4);
		}
	}
</style>
