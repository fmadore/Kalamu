<script lang="ts">
	import { onMount } from 'svelte';
	import TermLink from './TermLink.svelte';
	import ChoiceStrip from './ChoiceStrip.svelte';
	import FileList from './FileList.svelte';
	import LanguageStep from './LanguageStep.svelte';
	import LinkList from './LinkList.svelte';
	import OutputStep from './OutputStep.svelte';
	import PipelineDiagram from './PipelineDiagram.svelte';
	import PipelineList from './PipelineList.svelte';
	import SetupList from './SetupList.svelte';
	import StartPanel from './StartPanel.svelte';
	import TaskPicker from './TaskPicker.svelte';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, listNames, messages } from '#lib/i18n/index.js';
	import { takesLanguage } from '#lib/languages.js';
	import type { LineBreaks } from '#lib/bindings/LineBreaks.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import {
		acceptsPrompt,
		type Draft,
		inputOf,
		mergeInfo,
		type Mode,
		modelInfo,
		newDraft,
		offeredFormats,
		readsKind,
		rulesFor,
		suggestName,
		taskOf,
		toRequest
	} from '#lib/newJob.js';
	import { api, onDrop } from '#lib/tauri.js';
	import { app, go, hasKey, liveJob, refreshJobs, toast } from '#lib/state.svelte.js';

	let { opening = {} }: { opening?: { task?: Task; paths?: string[] } } = $props();

	const m = messages();

	let mode = $state<Mode>('text_extraction');
	const task = $derived(taskOf(mode));
	/** What every model must read: YouTube links for videos, else anything the task takes. */
	const input = $derived(inputOf(mode));
	/** The engine's rules for the task: which files, models and formats suit it. */
	const rule = $derived(rulesFor(app.rules, task));
	let files = $state<InspectedFile[]>([]);
	let inspecting = $state(false);
	let languageMode = $state<'auto' | 'explicit'>('auto');
	let languages = $state<string[]>([]);
	let lineBreaks = $state<LineBreaks>('running_text');
	let pace = $state<Pace>('fast');
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
	const noPrompt = $derived(namesOf((model) => !acceptsPrompt(model)));
	const sources = $derived(files.flatMap((f) => (f.source ? [f.source] : [])));
	/** Times a task was picked, so its diagram plays; opening New job shows it at rest. */
	let picked = $state(0);
	const chosenFields = $derived(
		presets.find((p) => p.id === drafts[0]?.presetId)?.schema?.fields.map((f) => f.label) ?? []
	);
	/** The models that combine a table of contents' pages, for the diagram. */
	const mergeNames = $derived([
		...new Set(drafts.map((d) => mergeInfo(catalog, d)?.name ?? d.mergeModel).filter(Boolean))
	]);
	/** Tasks that write text out page by page, where lines and samples apply. */
	const paged = $derived(
		task === 'text_extraction' || task === 'text_correction' || task === 'contents'
	);
	const linesApply = $derived(task === 'text_extraction' || task === 'text_correction');
	const lineOptions = (['running_text', 'as_on_page'] as const).map((value) => ({
		value,
		label: m.newJob.lines[value],
		hint: m.newJob.lines[`${value}_hint`]
	}));

	async function loadTask(next: Mode) {
		mode = next;
		presets = await api.listPresets(task);
		drafts = [newDraft(catalog, rule, presets[0]?.id ?? null, hasKey, [], input)];
		// A table of contents reads best as a document and a spreadsheet.
		formats =
			task === 'contents'
				? ['markdown', 'csv']
				: [
						...(task === 'audio_transcription'
							? app.settings!.audio_formats
							: app.settings!.text_formats)
					].filter((f) => rule.formats.includes(f));
		// An analysis or a table of contents is written in one language.
		if (rule.answers_in_language) languages = languages.slice(0, 1);
		// A video has no folder of its own: offer the folder last chosen.
		const last = app.settings!.last_folder;
		if (mode === 'youtube' && destination.kind !== 'folder' && last) {
			destination = { kind: 'folder', path: last };
		}
		files = files.filter((f) => readsKind(app.rules, mode, f.source?.kind));
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
			(task !== 'contents' || drafts.every((d) => d.mergeModel && hasKey(d.mergeProvider))) &&
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
			sample_pages: sample && paged ? samplePages : null
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

	/** Economy mode is offered when one of the chosen models goes through a batch service. */
	const economyOffered = $derived(
		mode !== 'youtube' && estimates.some((r) => r.estimate.economy_usd !== null)
	);
	const chosenPace = $derived<Pace>(economyOffered ? pace : 'fast');
	/** A model's cost at a pace; one without a batch service costs the same either way. */
	const rowCost = (row: EstimateRow, at: Pace) =>
		at === 'economy' ? (row.estimate.economy_usd ?? row.estimate.usd) : row.estimate.usd;
	// A video's length, and so its cost, is known only once Gemini has read it.
	const totalFor = (at: Pace) =>
		mode !== 'youtube' && estimates.every((r) => r.estimate.usd !== null)
			? estimates.reduce((sum, r) => sum + (rowCost(r, at) ?? 0), 0)
			: null;
	const totalEstimate = $derived(totalFor(chosenPace));
	const notBatched = $derived(
		estimates
			.filter((r) => r.estimate.economy_usd === null)
			.map(
				(r) =>
					catalog.models.find((x) => x.provider === r.provider && x.id === r.model)?.name ?? r.model
			)
	);

	/** The paces, each with what the job costs at it. */
	const paceOptions = $derived(
		(['fast', 'economy'] as const).map((value) => {
			const total = totalFor(value);
			return {
				value,
				label: m.newJob.pace[value],
				hint: m.newJob.pace[`${value}_hint`],
				price: total !== null ? formatUsd(total) : null
			};
		})
	);

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
			sample_pages: sample && paged ? samplePages : null,
			line_breaks: linesApply && noPrompt.length < models.length ? lineBreaks : null,
			pace: chosenPace
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
			timestamps: s.options.timestamps,
			mergeProvider: s.merge?.provider ?? s.provider,
			mergeModel: s.merge?.model ?? s.model
		}));
		languageMode = p.settings.languages.mode;
		languages = p.settings.languages.mode === 'explicit' ? [...p.settings.languages.languages] : [];
		lineBreaks = p.settings.line_breaks ?? 'running_text';
		pace = p.settings.pace ?? 'fast';
		sample = p.settings.sample_pages !== null;
		samplePages = p.settings.sample_pages ?? samplePages;
		formats = p.formats.filter((f) => rulesFor(app.rules, p.task).formats.includes(f));
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
						options: r.options,
						merge: r.merge
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
					formats: formats.filter((f) => offeredFormats(catalog, rule, drafts).includes(f)),
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
			<TaskPicker
				{mode}
				onpick={(t) => {
					picked++;
					void loadTask(t);
				}}
			/>
			<PipelineDiagram
				{mode}
				{files}
				fields={chosenFields}
				models={namesOf((model) => model !== undefined)}
				mergeModels={mergeNames}
				play={picked}
			/>
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

		<LanguageStep
			bind:mode={languageMode}
			bind:languages
			{task}
			{models}
			ignored={languageIgnored}
		/>

		{#if linesApply}
			<section class="step block" aria-labelledby="lines-title">
				<h2 id="lines-title">{m.newJob.linesTitle}</h2>
				{#if noPrompt.length === models.length}
					<p class="hint">{m.newJob.linesNoPromptAll(listNames(noPrompt))}</p>
				{:else}
					<ChoiceStrip bind:value={lineBreaks} options={lineOptions} />
					{#if noPrompt.length > 0}
						<p class="hint">{m.newJob.linesNoPromptSome(listNames(noPrompt))}</p>
					{/if}
				{/if}
			</section>
		{/if}

		{#if economyOffered}
			<section class="step block" aria-labelledby="pace-title">
				<h2 id="pace-title">{m.newJob.paceTitle}<TermLink term="economy" /></h2>
				<ChoiceStrip bind:value={pace} options={paceOptions} />
				{#if pace === 'economy' && notBatched.length > 0}
					<p class="hint">{m.newJob.paceNotBatched(listNames(notBatched))}</p>
				{/if}
			</section>
		{/if}

		<OutputStep
			bind:sample
			bind:samplePages
			bind:formats
			bind:destination
			bind:name
			{mode}
			{task}
			{drafts}
			{paged}
			hasSources={sources.length > 0}
		/>
	</div>

	<!-- Stays in view while the form scrolls: what it costs, and the one action. -->
	<StartPanel
		{estimates}
		cost={(row) => rowCost(row, chosenPace)}
		total={totalEstimate}
		video={mode === 'youtube' && sources.length > 0}
		{error}
		{blocker}
		{ready}
		{starting}
		canSave={drafts.length > 0}
		bind:naming
		onstart={start}
		onsave={() => void savePipeline()}
	/>
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
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 60ch;
	}
	@media (max-width: 1100px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	@media (max-width: 760px) {
		.form {
			padding: var(--space-4) var(--space-4) var(--space-6);
		}
	}
</style>
