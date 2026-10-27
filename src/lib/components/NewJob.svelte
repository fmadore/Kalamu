<script lang="ts">
	import { onMount } from 'svelte';
	import FileList from './FileList.svelte';
	import Icon from './Icon.svelte';
	import LanguagePicker from './LanguagePicker.svelte';
	import LinkList from './LinkList.svelte';
	import PipelineList from './PipelineList.svelte';
	import SetupList from './SetupList.svelte';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, listNames, messages } from '#lib/i18n/index.js';
	import { languageLabel, modelsWithout, offeredLanguages, takesLanguage } from '#lib/languages.js';
	import type { LineBreaks } from '#lib/bindings/LineBreaks.js';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import {
		acceptsPrompt,
		type Draft,
		FORMATS,
		inputOf,
		type Mode,
		MODES,
		modelInfo,
		newDraft,
		offeredFormats,
		readsKind,
		suggestName,
		taskOf,
		toRequest
	} from '#lib/newJob.js';
	import { api, onDrop, pickFolder } from '#lib/tauri.js';
	import { app, go, hasKey, liveJob, refreshJobs, toast } from '#lib/state.svelte.js';

	let { opening = {} }: { opening?: { task?: Task; paths?: string[] } } = $props();

	const m = messages();

	let mode = $state<Mode>('text_extraction');
	const task = $derived(taskOf(mode));
	/** What every model must read: YouTube links for videos, else anything the task takes. */
	const input = $derived(inputOf(mode));
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
	const models = $derived(drafts.map((d) => modelInfo(catalog, d)));
	/** Distinct names of the chosen models matching a test, for the notes under a choice. */
	const namesOf = (test: (model: ModelInfo | undefined) => boolean) => [
		...new Set(models.filter(test).map((model) => model?.name ?? ''))
	];
	const languageIgnored = $derived(namesOf((model) => !takesLanguage(model)));
	const narrowedBy = $derived(
		namesOf((model) => takesLanguage(model) && Array.isArray(model?.languages))
	);
	const unsupportedChosen = $derived(
		languages
			.map((code) => [code, modelsWithout(code, models)] as const)
			.filter(([, names]) => names.length > 0)
	);
	const noPrompt = $derived(namesOf((model) => !acceptsPrompt(model)));
	const sources = $derived(files.flatMap((f) => (f.source ? [f.source] : [])));

	async function loadTask(next: Mode) {
		mode = next;
		presets = await api.listPresets(task);
		drafts = [newDraft(catalog, task, presets[0]?.id ?? null, hasKey, [], input)];
		formats = [
			...(task === 'audio_transcription' ? app.settings!.audio_formats : app.settings!.text_formats)
		].filter((f) => FORMATS[task].includes(f));
		// An analysis is written in one language.
		if (task === 'analysis') languages = languages.slice(0, 1);
		// A video has no folder of its own: offer the folder last chosen.
		const last = app.settings!.last_folder;
		if (mode === 'youtube' && destination.kind !== 'folder' && last) {
			destination = { kind: 'folder', path: last };
		}
		files = files.filter((f) => readsKind(mode, f.source?.kind));
	}

	async function addPaths(paths: string[]) {
		// Videos are added by their link; dropped files belong to the other tasks.
		if (paths.length === 0 || mode === 'youtube') return;
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
			(input === undefined || models.every((model) => model?.inputs.includes(input))) &&
			(mode !== 'youtube' || destination.kind === 'folder') &&
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

	// A video's length, and so its cost, is known only once Gemini has read it.
	const totalEstimate = $derived(
		mode !== 'youtube' && estimates.every((r) => r.estimate.usd !== null)
			? estimates.reduce((sum, r) => sum + (r.estimate.usd ?? 0), 0)
			: null
	);

	async function chooseDestination() {
		const folder = await pickFolder();
		if (!folder) return;
		destination = { kind: 'folder', path: folder };
		// Remembered, so the next job that needs a folder starts with this one.
		app.settings!.last_folder = folder;
		api.saveSettings($state.snapshot(app.settings!)).catch(() => undefined);
	}

	/** Why Start is unavailable, when the form cannot say it where the user is looking. */
	const blocker = $derived(
		mode === 'youtube' && sources.length > 0 && destination.kind !== 'folder'
			? m.newJob.needFolder
			: null
	);

	/** The settings shared by every set-up, as a job and a pipeline store them. */
	function jobSettings() {
		return {
			languages:
				languageMode === 'explicit' &&
				languages.length > 0 &&
				languageIgnored.length < models.length
					? { mode: 'explicit' as const, languages: [...languages] }
					: { mode: 'auto' as const },
			sample_pages: sample ? samplePages : null,
			line_breaks: task === 'text_extraction' && noPrompt.length < models.length ? lineBreaks : null
		};
	}

	/** Fills the form from a saved pipeline; the files stay as they are. */
	async function applyPipeline(p: Pipeline) {
		// A transcription pipeline applied to videos keeps the videos.
		await loadTask(mode === 'youtube' && p.task === 'audio_transcription' ? 'youtube' : p.task);
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

	function onKey(e: KeyboardEvent) {
		if (e.ctrlKey && e.key === 'Enter' && ready && !starting) {
			e.preventDefault();
			void start();
		}
	}

	onMount(() => {
		// Opened from a finished job's "Summarise" button: its results are the files.
		void loadTask(opening.task ?? mode).then(() =>
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

<svelte:window onkeydown={onKey} />

<header class="masthead">
	<div>
		<h1>{m.newJob.title}</h1>
		<p class="lede">{m.newJob.lede}</p>
	</div>
</header>

<div class="layout">
	<div class="form steps">
		<PipelineList bind:pipelines onapply={applyPipeline} onerror={(e) => (error = e)} />

		<section class="step" aria-labelledby="task-label">
			<h2 id="task-label">{m.newJob.taskLabel}</h2>
			<div class="tasks">
				{#each MODES as t (t)}
					<button
						class="task"
						class:selected={mode === t}
						aria-pressed={mode === t}
						onclick={() => loadTask(t)}
					>
						<span class="task-name"><span class="radio" aria-hidden="true"></span>{m.task[t]}</span>
						<span class="task-hint">{m.task[`${t}Hint`]}</span>
					</button>
				{/each}
			</div>
		</section>

		{#if mode === 'youtube'}
			<LinkList
				bind:links={files}
				onadd={(added) => {
					if (!name) name = added.find((l) => l.source?.title)?.source?.title ?? '';
				}}
				onerror={(e) => (error = e)}
			/>
		{:else}
			<FileList bind:files {task} {inspecting} onadd={addPaths} />
		{/if}

		<SetupList bind:drafts {task} {presets} {input} />

		<section class="step block" aria-labelledby="lang-title">
			<h2 id="lang-title">
				{task === 'analysis' ? m.newJob.analysisLanguageTitle : m.newJob.languageTitle}
			</h2>
			{#if languageIgnored.length === models.length}
				<p class="hint">{m.newJob.languageIgnoredAll(listNames(languageIgnored))}</p>
			{:else}
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
					<LanguagePicker
						bind:selected={languages}
						offered={offeredLanguages(catalog, models)}
						single={task === 'analysis'}
						unsupported={(code) => modelsWithout(code, models)}
					/>
					{#each unsupportedChosen as [code, names] (code)}
						<p class="hint warn-text">
							{m.newJob.languageUnsupported(languageLabel(catalog, code), listNames(names))}
						</p>
					{/each}
					{#if narrowedBy.length > 0}
						<p class="hint">{m.newJob.languagesNarrowed(listNames(narrowedBy))}</p>
					{/if}
				{/if}
				<p class="hint">
					{task === 'analysis' ? m.newJob.analysisLanguageHint : m.newJob.languageHint}
				</p>
				{#if languageIgnored.length > 0}
					<p class="hint">{m.newJob.languageIgnoredSome(listNames(languageIgnored))}</p>
				{/if}
			{/if}
		</section>

		{#if task === 'text_extraction'}
			<section class="step block" aria-labelledby="lines-title">
				<h2 id="lines-title">{m.newJob.linesTitle}</h2>
				{#if noPrompt.length === models.length}
					<p class="hint">{m.newJob.linesNoPromptAll(listNames(noPrompt))}</p>
				{:else}
					<div class="choices">
						{#each ['running_text', 'as_on_page'] as const as mode (mode)}
							<label class="choice" class:selected={lineBreaks === mode}>
								<input type="radio" bind:group={lineBreaks} value={mode} />
								<span>
									<strong>{m.newJob.lines[mode]}</strong>
									<span class="hint">{m.newJob.lines[`${mode}_hint`]}</span>
								</span>
							</label>
						{/each}
					</div>
					{#if noPrompt.length > 0}
						<p class="hint">{m.newJob.linesNoPromptSome(listNames(noPrompt))}</p>
					{/if}
				{/if}
			</section>
		{/if}

		<section class="step block" aria-labelledby="out-title">
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
			{#if mode === 'youtube'}
				<!-- A video has no folder of its own, so a folder must be chosen. -->
				<div class="row wrap">
					<span class="label">{m.newJob.saveIn}</span>
					{#if destination.kind === 'folder'}
						<span class="path mono">{destination.path}</span>
					{/if}
					<button class="btn sm" onclick={chooseDestination}
						><Icon name="folder" size={14} />{m.newJob.chooseFolder}</button
					>
				</div>
				{#if destination.kind !== 'folder' && sources.length > 0}
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
						<input
							type="radio"
							checked={destination.kind === 'folder'}
							onchange={chooseDestination}
						/>
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
	</div>

	<!-- Stays in view while the form scrolls: what it costs, and the one action. -->
	<aside class="summary" aria-labelledby="estimate-label">
		<div class="estimate">
			<span id="estimate-label" class="label">{m.newJob.estimate}</span>
			<strong class="total mono"
				>{estimates.length > 0 && totalEstimate !== null ? formatUsd(totalEstimate) : '—'}</strong
			>
			{#if mode === 'youtube' && sources.length > 0}
				<p class="hint">{m.newJob.estimateVideo}</p>
			{:else if estimates.length > 0}
				<ul class="rows">
					{#each estimates as row (row.provider + row.model)}
						<li>
							<span class="mono">{row.model}</span>
							<span class="mono"
								>{row.estimate.usd !== null
									? formatUsd(row.estimate.usd)
									: m.newJob.estimateUnknown}</span
							>
						</li>
					{/each}
				</ul>
				<p class="hint">{m.newJob.estimateNote}</p>
			{/if}
		</div>

		{#if error}
			<p class="chip danger wrap-chip" role="alert">{error}</p>
		{/if}

		<div class="go">
			{#if blocker && !starting}
				<p class="blocker">{blocker}</p>
			{/if}
			<button class="btn primary lg" disabled={!ready || starting} onclick={start}>
				{starting ? m.newJob.starting : m.newJob.start}<span class="kbd"
					>{m.newJob.startShortcut}</span
				>
			</button>
			{#if naming === null}
				<button class="btn" disabled={drafts.length === 0} onclick={() => (naming = '')}
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
					<div class="naming-actions">
						<button class="btn" type="submit" disabled={!naming.trim()}>{m.common.save}</button>
						<button class="btn ghost" type="button" onclick={() => (naming = null)}
							>{m.common.cancel}</button
						>
					</div>
				</form>
			{/if}
		</div>
	</aside>
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 21rem;
		min-height: calc(100% - 3.5rem);
	}
	.form {
		display: grid;
		align-content: start;
		gap: var(--space-6);
		max-width: 52rem;
		padding: var(--space-5) var(--space-6) var(--space-7);
	}
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
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 60ch;
	}
	.warn-text {
		color: var(--warn-text);
	}
	/* The tasks as one joined choice, two by two: written sources above, spoken ones below. */
	.tasks {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		margin-top: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
	}
	.task {
		display: grid;
		align-content: start;
		gap: var(--space-1);
		padding: var(--space-3) var(--space-4) var(--space-4);
		border: 0;
		background: transparent;
		color: var(--text-body);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.task:nth-child(even) {
		border-left: 1px solid var(--line);
	}
	.task:nth-child(n + 3) {
		border-top: 1px solid var(--line);
	}
	.task:nth-child(1) {
		border-radius: var(--radius-card) 0 0 0;
	}
	.task:nth-child(2) {
		border-radius: 0 var(--radius-card) 0 0;
	}
	.task:nth-child(3) {
		border-radius: 0 0 0 var(--radius-card);
	}
	.task:nth-child(4) {
		border-radius: 0 0 var(--radius-card) 0;
	}
	.task:hover {
		background: var(--surface-1);
	}
	.task.selected {
		background: var(--surface-2);
		box-shadow: inset 0 0 0 1.5px var(--ink);
	}
	.task-name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-md);
		font-weight: 500;
		color: var(--text-bright);
	}
	.selected .task-name {
		font-weight: 600;
	}
	.radio {
		width: 0.875rem;
		height: 0.875rem;
		border-radius: 50%;
		border: 1.5px solid var(--line-hover);
		flex: none;
	}
	.selected .radio {
		border: 4px solid var(--ink);
	}
	.task-hint {
		padding-left: 1.375rem;
		font-size: var(--type-xs);
		line-height: 1.45;
		color: var(--text-muted);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
	.path {
		overflow-wrap: anywhere;
		font-size: var(--type-xs);
	}
	.choices {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
	}
	.choice {
		display: flex;
		gap: var(--space-3);
		align-items: start;
		padding: var(--space-3) var(--space-4);
		border-left: 1px solid var(--line);
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.choice:first-child {
		border-left: 0;
		border-radius: var(--radius-card) 0 0 var(--radius-card);
	}
	.choice:last-child {
		border-radius: 0 var(--radius-card) var(--radius-card) 0;
	}
	.choice:hover {
		background: var(--surface-1);
	}
	.choice.selected {
		background: var(--surface-2);
		box-shadow: inset 0 0 0 1.5px var(--ink);
	}
	.choice input {
		margin-top: 0.25em;
	}
	.choice > span {
		display: grid;
		gap: var(--space-1);
	}
	.choice strong {
		font-size: var(--type-sm);
		font-weight: 500;
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
	.summary {
		position: sticky;
		top: 0;
		align-self: start;
		display: grid;
		gap: var(--space-5);
		min-height: calc(100vh - 3.5rem);
		align-content: start;
		padding: var(--space-5);
		border-left: 1px solid var(--line);
		background: var(--surface-1);
	}
	.estimate {
		display: grid;
		gap: var(--space-2);
	}
	.total {
		font-size: 1.875rem;
		font-weight: 500;
		letter-spacing: -0.02em;
		color: var(--text-bright);
	}
	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
		border-top: 1px solid var(--line);
	}
	.rows li {
		display: flex;
		justify-content: space-between;
		gap: var(--space-3);
		padding: var(--space-2) 0;
		border-bottom: 1px solid var(--line);
		font-size: var(--type-xs);
	}
	.rows li > :first-child {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.wrap-chip {
		white-space: normal;
	}
	.go {
		display: grid;
		gap: var(--space-2);
	}
	/* What still stops Start, said where Start is, in the footer too. */
	.blocker {
		margin: 0;
		font-size: var(--type-xs);
		color: var(--warn-text);
	}
	.naming {
		display: grid;
		gap: var(--space-2);
	}
	.naming-actions {
		display: flex;
		gap: var(--space-2);
	}
	@media (max-width: 1100px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}
		.summary {
			position: sticky;
			bottom: 0;
			top: auto;
			min-height: 0;
			grid-template-columns: minmax(0, 1fr) auto;
			align-items: center;
			padding: var(--space-3) var(--space-5);
			border-left: 0;
			border-top: 1px solid var(--line);
			box-shadow: 0 -10px 24px -18px rgba(17, 18, 20, 0.35);
		}
		.summary .rows,
		.summary .hint {
			display: none;
		}
		.total {
			font-size: var(--type-xl);
		}
	}
	@media (max-width: 760px) {
		.tasks,
		.choices {
			grid-template-columns: 1fr;
		}
		.task:nth-child(n),
		.choice {
			border-left: 0;
			border-top: 1px solid var(--line);
			border-radius: 0;
		}
		.task:first-child {
			border-top: 0;
			border-radius: var(--radius-card) var(--radius-card) 0 0;
		}
		.task:last-child {
			border-radius: 0 0 var(--radius-card) var(--radius-card);
		}
		.form {
			padding: var(--space-4) var(--space-4) var(--space-6);
		}
	}
</style>
