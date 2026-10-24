<script lang="ts">
	import { onMount } from 'svelte';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { InputWarning } from '#lib/bindings/InputWarning.js';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { SetupRequest } from '#lib/bindings/SetupRequest.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, languageName, messages } from '#lib/i18n/index.js';
	import { api, onDrop, pickFiles, pickFolder } from '#lib/tauri.js';
	import { app, go, hasKey, liveJob, refreshJobs } from '#lib/state.svelte.js';

	const m = messages();

	interface Draft {
		provider: string;
		model: string;
		presetId: string | null;
		label: string;
		diarize: boolean;
		timestamps: boolean;
	}

	const TEXT_FORMATS: ExportFormat[] = ['txt', 'markdown', 'docx', 'json', 'csv'];
	const AUDIO_FORMATS: ExportFormat[] = ['txt', 'markdown', 'docx', 'srt', 'vtt', 'json', 'csv'];

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
	let showTips = $state(false);

	const catalog = $derived(app.catalog!);
	const providers = $derived(
		catalog.providers.filter((p) =>
			catalog.models.some((mod) => mod.provider === p.id && mod.tasks.includes(task))
		)
	);
	const sources = $derived(files.flatMap((f) => (f.source ? [f.source] : [])));
	const formatChoices = $derived(task === 'text_extraction' ? TEXT_FORMATS : AUDIO_FORMATS);

	function modelsFor(provider: string): ModelInfo[] {
		return catalog.models.filter((mod) => mod.provider === provider && mod.tasks.includes(task));
	}

	function modelInfo(d: Draft): ModelInfo | undefined {
		return catalog.models.find((mod) => mod.provider === d.provider && mod.id === d.model);
	}

	function defaultDraft(): Draft {
		const usable = catalog.models.filter((mod) => mod.tasks.includes(task) && hasKey(mod.provider));
		const pick =
			usable.find((mod) => mod.recommended_for.includes(task)) ??
			usable[0] ??
			catalog.models.find((mod) => mod.tasks.includes(task));
		return {
			provider: pick?.provider ?? '',
			model: pick?.id ?? '',
			presetId: presets[0]?.id ?? null,
			label: '',
			diarize: true,
			timestamps: true
		};
	}

	async function loadTask(next: Task) {
		task = next;
		presets = await api.listPresets(task);
		drafts = [defaultDraft()];
		formats = [
			...(task === 'text_extraction' ? app.settings!.text_formats : app.settings!.audio_formats)
		];
		files = files.filter((f) =>
			task === 'text_extraction'
				? f.source?.kind === 'pdf' || f.source?.kind === 'image'
				: f.source?.kind === 'audio' || f.source?.kind === 'video'
		);
	}

	async function addPaths(paths: string[]) {
		if (paths.length === 0) return;
		error = null;
		inspecting = true;
		try {
			const expanded = await api.expandPaths(paths, task);
			const known = new Set(files.map((f) => f.path));
			const fresh = expanded.filter((p) => !known.has(p));
			const inspected = await api.inspectFiles(fresh);
			files = [...files, ...inspected];
			if (!name && files.length > 0) {
				const first = files[0].path.split(/[\\/]/);
				name =
					paths.length === 1 && expanded.length > 1
						? (paths[0].split(/[\\/]/).pop() ?? '')
						: (first.pop() ?? '');
			}
		} catch (e) {
			error = describeError(e, m);
		} finally {
			inspecting = false;
		}
	}

	function setSplit(index: number, split: boolean) {
		const source = files[index].source;
		if (source) source.split_spreads = split;
	}

	function warningText(w: InputWarning): string {
		const pages = (list: number[]) =>
			list.length > 6 ? `${list.slice(0, 6).join(', ')}…` : list.join(', ');
		switch (w.kind) {
			case 'double_page_spread':
				return m.warning.double_page_spread(pages(w.pages));
			case 'low_resolution':
				return m.warning.low_resolution(w.dpi, pages(w.pages));
			case 'has_text_layer':
				return m.warning.has_text_layer(w.pages_with_text);
			case 'unsupported_image':
				return m.warning.unsupported_image(w.detail);
			case 'large_file':
				return m.warning.large_file;
		}
	}

	function request(d: Draft): SetupRequest {
		const accepts = modelInfo(d)?.accepts_prompt ?? true;
		return {
			provider: d.provider,
			model: d.model,
			preset_id: accepts ? d.presetId : null,
			label: d.label.trim() || null,
			options: {
				temperature: null,
				diarize: task === 'audio_transcription' && d.diarize,
				timestamps: task === 'audio_transcription' && d.timestamps,
				speakers: null
			}
		};
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
			setups: drafts.filter((d) => d.model).map(request),
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
				setups: drafts.map(request),
				settings: {
					languages:
						languageMode === 'explicit' && languages.length > 0
							? { mode: 'explicit', languages: [...languages] }
							: { mode: 'auto' },
					sample_pages: sample ? samplePages : null
				},
				output: {
					formats: [...formats],
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
		void loadTask(task);
		let unlisten: (() => void) | undefined;
		void onDrop((paths) => void addPaths(paths)).then((u) => (unlisten = u));
		return () => unlisten?.();
	});
</script>

<div class="page">
	<header>
		<h1>{m.newJob.title}</h1>
	</header>

	<section aria-labelledby="task-label">
		<h2 id="task-label" class="label">{m.newJob.taskLabel}</h2>
		<div class="tasks">
			{#each ['text_extraction', 'audio_transcription'] as const as t (t)}
				<button
					class="task card"
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

	<section class="card block" aria-labelledby="files-title">
		<div class="row between">
			<h2 id="files-title">{m.newJob.filesTitle}</h2>
			<div class="row">
				<button class="btn" onclick={async () => addPaths(await pickFiles(task))}
					>{m.newJob.addFiles}</button
				>
				<button
					class="btn"
					onclick={async () => {
						const folder = await pickFolder();
						if (folder) await addPaths([folder]);
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
								<span>{warningText(w)}</span>
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
							</div>
						{/each}
					</li>
				{/each}
			</ul>
		{/if}
		<button
			class="btn ghost sm tips-toggle"
			aria-expanded={showTips}
			onclick={() => (showTips = !showTips)}
		>
			{m.tips.title}
		</button>
		{#if showTips}
			<ul class="tips muted">
				<li>{task === 'text_extraction' ? m.tips.scanning : m.tips.recording}</li>
				<li>{m.tips.results}</li>
			</ul>
		{/if}
	</section>

	<section class="card block" aria-labelledby="lang-title">
		<h2 id="lang-title">{m.newJob.languageTitle}</h2>
		<div class="row">
			<label class="inline"
				><input type="radio" bind:group={languageMode} value="auto" />
				{m.newJob.languageAuto}</label
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
							onclick={() => (languages = languages.filter((c) => c !== code))}>×</button
						>
					</span>
				{/each}
				<select
					aria-label={m.newJob.addLanguage}
					onchange={(e) => {
						const v = e.currentTarget.value;
						if (v && !languages.includes(v)) languages = [...languages, v];
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
		<p class="muted small">{m.newJob.languageHint}</p>
	</section>

	<section class="card block" aria-labelledby="setups-title">
		<h2 id="setups-title">{m.newJob.setupsTitle}</h2>
		{#if !app.keys.some((k) => k.saved)}
			<p class="warning">
				{m.newJob.noProvider}
				<button class="btn sm" onclick={() => go({ name: 'settings' })}>{m.nav.settings}</button>
			</p>
		{/if}
		{#each drafts as d, i (i)}
			{@const info = modelInfo(d)}
			<div class="setup">
				<label class="field">
					<span>{m.newJob.provider}</span>
					<select
						bind:value={d.provider}
						onchange={() => {
							d.model = modelsFor(d.provider)[0]?.id ?? '';
						}}
					>
						{#each providers as p (p.id)}
							<option value={p.id} disabled={!hasKey(p.id)}>{p.name}</option>
						{/each}
					</select>
				</label>
				<label class="field">
					<span>{m.newJob.model}</span>
					<select bind:value={d.model}>
						{#each modelsFor(d.provider) as mod (mod.id)}
							<option value={mod.id}
								>{mod.name}{mod.status === 'preview' ? ` (${m.common.preview})` : ''}</option
							>
						{/each}
					</select>
				</label>
				<label class="field">
					<span>{m.newJob.prompt}</span>
					{#if info && !info.accepts_prompt}
						<select disabled><option>{m.newJob.noPrompt}</option></select>
					{:else}
						<select bind:value={d.presetId}>
							{#each presets as p (p.id)}
								<option value={p.id}>{p.name}</option>
							{/each}
						</select>
					{/if}
				</label>
				{#if drafts.length > 1}
					<label class="field">
						<span>{m.newJob.label}</span>
						<input type="text" bind:value={d.label} placeholder={d.model} />
					</label>
				{/if}
				{#if task === 'audio_transcription'}
					<div class="row opts">
						<label class="inline"
							><input type="checkbox" bind:checked={d.diarize} /> {m.newJob.diarize}</label
						>
						<label class="inline"
							><input type="checkbox" bind:checked={d.timestamps} /> {m.newJob.timestamps}</label
						>
					</div>
				{/if}
				{#if drafts.length > 1}
					<button class="btn ghost sm" onclick={() => (drafts = drafts.filter((_, j) => j !== i))}
						>{m.newJob.removeSetup}</button
					>
				{/if}
				{#if info?.description}
					<p class="muted small desc">{info.description}</p>
				{/if}
				{#if d.provider && !hasKey(d.provider)}
					<p class="warning small">
						{m.newJob.noKey(catalog.providers.find((p) => p.id === d.provider)?.name ?? d.provider)}
					</p>
				{/if}
			</div>
		{/each}
		<div>
			<button class="btn" onclick={() => (drafts = [...drafts, { ...defaultDraft(), model: '' }])}>
				+ {m.newJob.compareAdd}
			</button>
			<p class="muted small">{m.newJob.setupsHint}</p>
		</div>
	</section>

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
				{#each formatChoices as f (f)}
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
				<span class="mono small">{destination.path}</span>
				<button class="btn sm" onclick={chooseDestination}>{m.newJob.chooseFolder}</button>
			{/if}
		</div>
		<label class="field name">
			<span>{m.newJob.name}</span>
			<input type="text" bind:value={name} />
		</label>
	</section>

	<footer class="card block start">
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
	</footer>
</div>

<style>
	.page {
		display: grid;
		gap: var(--space-4);
		max-width: 64rem;
	}
	.block {
		padding: var(--space-4) var(--space-5);
		display: grid;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.row.between {
		justify-content: space-between;
	}
	.row.wrap {
		flex-wrap: wrap;
	}
	.small {
		font-size: var(--type-sm);
	}
	.tasks {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-3);
		margin-top: var(--space-2);
	}
	.task {
		text-align: left;
		padding: var(--space-4);
		display: grid;
		gap: var(--space-1);
		cursor: pointer;
	}
	.task.selected {
		border-color: var(--accent-border);
		background: var(--accent-wash);
		box-shadow: inset 0 0 0 1px var(--accent-border);
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
		border-left: 3px solid var(--warn-border);
		background: var(--warn-wash);
		border-radius: var(--radius-control);
		color: var(--text-body);
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
	.setup {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
		gap: var(--space-3);
		align-items: end;
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
	}
	.desc,
	.setup .warning {
		grid-column: 1 / -1;
	}
	.opts {
		min-height: var(--control-md);
	}
	.x {
		border: 0;
		background: none;
		cursor: pointer;
		padding: 0 2px;
		color: inherit;
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
	.tips-toggle {
		justify-self: start;
	}
	.tips {
		margin: 0;
		display: grid;
		gap: var(--space-1);
		font-size: var(--type-sm);
	}
	.start {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-4);
		position: sticky;
		bottom: 0;
		background: var(--surface-1);
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
</style>
