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
	import {
		type Draft,
		FORMATS,
		newDraft,
		offeredFormats,
		readsKind,
		suggestName,
		toRequest
	} from '#lib/newJob.js';
	import { api, onDrop, pickFolder } from '#lib/tauri.js';
	import { app, go, hasKey, liveJob, refreshJobs } from '#lib/state.svelte.js';

	let { opening = {} }: { opening?: { task?: Task; paths?: string[] } } = $props();

	const m = messages();

	let task = $state<Task>('text_extraction');
	let files = $state<InspectedFile[]>([]);
	let inspecting = $state(false);
	let languageMode = $state<'auto' | 'explicit'>('auto');
	let languages = $state<string[]>([]);
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

	const catalog = $derived(app.catalog!);
	const sources = $derived(files.flatMap((f) => (f.source ? [f.source] : [])));

	async function loadTask(next: Task) {
		task = next;
		presets = await api.listPresets(task);
		drafts = [newDraft(catalog, task, presets[0]?.id ?? null, hasKey)];
		formats = [
			...(task === 'audio_transcription' ? app.settings!.audio_formats : app.settings!.text_formats)
		].filter((f) => FORMATS[task].includes(f));
		// A summary is written in one language.
		if (task === 'summary') languages = languages.slice(0, 1);
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

	async function start() {
		error = null;
		starting = true;
		try {
			const job = await api.createJob({
				name: name.trim() || sources[0]?.file_name || m.newJob.title,
				task,
				sources: $state.snapshot(sources),
				setups: drafts.map((d) => toRequest(catalog, task, d)),
				settings: {
					languages:
						languageMode === 'explicit' && languages.length > 0
							? { mode: 'explicit', languages: [...languages] }
							: { mode: 'auto' },
					sample_pages: sample ? samplePages : null
				},
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
	<section aria-labelledby="task-label">
		<h2 id="task-label" class="label">{m.newJob.taskLabel}</h2>
		<div class="tasks">
			{#each ['text_extraction', 'audio_transcription', 'summary'] as const as t (t)}
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
			{task === 'summary' ? m.newJob.summaryLanguageTitle : m.newJob.languageTitle}
		</h2>
		<div class="row">
			<label class="inline"
				><input type="radio" bind:group={languageMode} value="auto" />
				{task === 'summary' ? m.newJob.summaryLanguageAuto : m.newJob.languageAuto}</label
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
						if (v && task === 'summary') languages = [v];
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
			{task === 'summary' ? m.newJob.summaryLanguageHint : m.newJob.languageHint}
		</p>
	</section>

	<SetupList bind:drafts {task} {presets} />

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
		.tasks {
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
