<script lang="ts">
	import Icon from './Icon.svelte';
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
	import ThenStep from './ThenStep.svelte';
	import type { Estimate } from '#lib/bindings/Estimate.js';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.js';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { OutputDestination } from '#lib/bindings/OutputDestination.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Refused } from '#lib/bindings/Refused.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, listNames, messages } from '#lib/i18n/index.js';
	import { primary } from '#lib/platform.js';
	import { modelsNotDetecting, takesLanguage } from '#lib/languages.js';
	import type { LineBreaks } from '#lib/bindings/LineBreaks.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import {
		acceptsPrompt,
		type Draft,
		draftsOf,
		followingAnalysis,
		formPipeline,
		inputOf,
		jobSettings as settingsOf,
		mergeInfo,
		type Mode,
		modelInfo,
		nameOfFile,
		newDraft,
		offeredFormats,
		pagesPerCallOf,
		providerDefault,
		dictionaryNames,
		forgetForm,
		formToOpen,
		keepForm,
		readersOf,
		readsKind,
		rulesFor,
		suggestName,
		suggestionFits,
		taskOf,
		togetherOf,
		toRequest,
		type Together
	} from '#lib/newJob.js';
	import { api, listenWhileMounted, mayStart, onDrop } from '#lib/tauri.js';
	import {
		app,
		go,
		hasKey,
		liveJob,
		missingLanguageData,
		refreshJobs,
		refreshOffline,
		shortcutsHeld,
		toast,
		type FromProject
	} from '#lib/state.svelte.js';

	let {
		opening = {}
	}: {
		opening?: { task?: Task; paths?: string[]; sources?: SourceFile[]; project?: FromProject };
	} = $props();

	const m = messages();
	/** The form left here earlier, given back when New job opens with nothing new to start. */
	// svelte-ignore state_referenced_locally
	const kept = formToOpen(opening);
	/** Where the form came from: text frozen from results, a project; kept with the form. */
	// svelte-ignore state_referenced_locally
	const from = {
		frozen: kept ? kept.frozen : opening.sources !== undefined,
		project: kept ? kept.project : opening.project
	};

	let mode = $state<Mode>(kept?.mode ?? 'text_extraction');
	const task = $derived(taskOf(mode));
	/** What every model must read: YouTube links for videos, else anything the task takes. */
	const input = $derived(inputOf(mode));
	/** The engine's rules for the task: which files, models and formats suit it. */
	const rule = $derived(rulesFor(app.rules, task));
	let files = $state<InspectedFile[]>(kept?.files ?? []);
	let inspecting = $state(false);
	/**
	 * What the last files added gave that the task does not read, the tasks that read it, and
	 * the files to move there; on YouTube, the recordings and videos dropped.
	 */
	let setAside = $state<{
		refused: Refused[];
		readers: Mode[];
		move: string[];
		youtube: boolean;
	} | null>(null);
	let languageMode = $state<'auto' | 'explicit'>(kept?.languageMode ?? 'auto');
	let languages = $state<string[]>(kept?.languages ?? []);
	let lineBreaks = $state<LineBreaks>(kept?.lineBreaks ?? 'running_text');
	let pace = $state<Pace>(kept?.pace ?? 'fast');
	let drafts = $state<Draft[]>(kept?.drafts ?? []);
	let presets = $state<Preset[]>([]);
	let sample = $state(kept?.sample ?? false);
	let samplePages = $state(kept?.samplePages ?? 3);
	/** How many pages of a PDF go in one call: one, several, or as many as fit. */
	let together = $state<Together>(kept?.together ?? 'one');
	let togetherPages = $state(kept?.togetherPages ?? 4);
	let formats = $state<ExportFormat[]>(kept?.formats ?? []);
	let destination = $state<OutputDestination>(kept?.destination ?? { kind: 'beside_sources' });
	let name = $state(kept?.name ?? '');
	/** The name Kalamu suggested from the files, as long as the field still holds it. */
	let suggested = $state(kept?.suggested ?? '');

	/** Fills the name from the files, unless the researcher typed one of their own. */
	function suggest(value: string) {
		if (name && name !== suggested) return;
		name = value;
		suggested = value;
	}

	// A suggested name follows the files: once the file or folder it came from is removed, the
	// first file left names the job, and no file leaves the field empty again.
	$effect(() => {
		if (!suggested || name !== suggested) return;
		if (suggestionFits(suggested, files, from.frozen)) return;
		const next = files[0] ? nameOfFile(files[0], from.frozen) : '';
		name = next;
		suggested = next;
	});
	let estimate = $state<Estimate | null>(null);
	const estimates = $derived(estimate?.rows ?? []);
	let error = $state<string | null>(null);
	let starting = $state(false);
	let pipelines = $state<Pipeline[]>([]);
	/** The name being typed for a new pipeline, while saving one. */
	let naming = $state<string | null>(null);
	/** An analysis to run on the text once it is read (and checked), or none. */
	let thenPreset = $state<string | null>(kept?.thenPreset ?? null);
	let thenCheck = $state(kept?.thenCheck ?? true);
	let analysisPresets = $state<Preset[]>([]);

	const catalog = $derived(app.catalog!);
	const analysisRule = $derived(rulesFor(app.rules, 'analysis'));
	/** Whether an analysis can follow: the task's results are a text. */
	const thenOffered = $derived(rule.yields_text);
	/** The model that analyses the text: the reading provider's, when it has one and a key. */
	const thenDraft = $derived.by(() => {
		const draft = newDraft(catalog, analysisRule, thenPreset, hasKey);
		const provider = drafts[0]?.provider;
		const same =
			provider && hasKey(provider) ? providerDefault(catalog, analysisRule, provider) : '';
		return same ? { ...draft, provider: provider!, model: same } : draft;
	});
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
	/** The chosen models that can also write a searchable PDF, for the diagram. */
	const searchableFrom = $derived(
		offeredFormats(catalog, rule, drafts).includes('searchable_pdf')
			? namesOf((model) => model?.layout === true)
			: []
	);
	/** Where samples and line breaks apply, as the engine says. */
	const paged = $derived(rule.samples);
	const linesApply = $derived(rule.line_breaks);
	const togetherOptions = (['one', 'several', 'whole'] as const).map((value) => ({
		value,
		label: m.newJob.together[value],
		hint: m.newJob.together[`${value}_hint`]
	}));
	/** What the job sends for it; nothing for one page at a time, as before. */
	const pagesPerCall = $derived(pagesPerCallOf(rule, together, togetherPages));
	const lineOptions = (['running_text', 'as_on_page'] as const).map((value) => ({
		value,
		label: m.newJob.lines[value],
		hint: m.newJob.lines[`${value}_hint`]
	}));

	/** Moves the files set aside to a task that reads them. */
	async function switchTo(next: Mode) {
		const move = setAside?.move ?? [];
		setAside = null;
		picked++;
		await loadTask(next);
		await addPaths(move);
	}

	/** Raised by each task picked, so the prompts of a task picked earlier and read later are
	 * not put under the one picked since. */
	let taskTicket = 0;

	async function loadTask(next: Mode) {
		if (next !== mode) setAside = null;
		mode = next;
		const ticket = ++taskTicket;
		const listed = await api.listPresets(task);
		if (ticket !== taskTicket) return;
		presets = listed;
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
		if (paths.length === 0) return;
		error = null;
		setAside = null;
		try {
			const expanded = await api.expandPaths(paths, task);
			// Videos are added by their link: recordings and videos dropped here are offered to
			// Transcribe audio, which reads them.
			if (mode === 'youtube') {
				const move = expanded.paths;
				setAside = {
					refused: [],
					readers: move.length > 0 ? ['audio_transcription'] : [],
					move,
					youtube: true
				};
				return;
			}
			if (expanded.refused.length > 0) {
				setAside = {
					refused: expanded.refused,
					readers: readersOf(app.rules, expanded.refused, mode),
					move: expanded.refused.filter((r) => r.kind).map((r) => r.path),
					youtube: false
				};
			}
			if (expanded.paths.length === 0) return;
			inspecting = true;
			const known = new Set(files.map((f) => f.path));
			const inspected = await api.inspectFiles(expanded.paths.filter((p) => !known.has(p)));
			files = [...files, ...inspected];
			suggest(suggestName(paths, expanded.paths));
		} catch (e) {
			error = describeError(e, m);
		} finally {
			inspecting = false;
		}
	}

	/** A model that cannot detect the language needs the languages chosen and downloaded. */
	const languagesReady = $derived(
		modelsNotDetecting(models).length === 0 ||
			(languageMode === 'explicit' &&
				languages.length > 0 &&
				missingLanguageData(languages).length === 0)
	);

	const ready = $derived(
		sources.length > 0 &&
			languagesReady &&
			drafts.length > 0 &&
			drafts.every((d) => d.model && hasKey(d.provider)) &&
			(task !== 'contents' || drafts.every((d) => d.mergeModel && hasKey(d.mergeProvider))) &&
			(input === undefined || models.every((model) => model?.inputs.includes(input))) &&
			(mode !== 'youtube' || destination.kind === 'folder') &&
			formats.length > 0
	);

	let estimateTimer: ReturnType<typeof setTimeout> | null = null;
	/** Raised by each change to what is estimated, so an older, slower answer is dropped. */
	let estimateTicket = 0;
	$effect(() => {
		const req = {
			task,
			sources: $state.snapshot(sources),
			setups: drafts.filter((d) => d.model).map((d) => toRequest(catalog, task, d)),
			sample_pages: sample && paged ? samplePages : null
		};
		if (estimateTimer) clearTimeout(estimateTimer);
		const ticket = ++estimateTicket;
		if (req.sources.length === 0 || req.setups.length === 0) {
			estimate = null;
			return;
		}
		estimateTimer = setTimeout(() => {
			api
				.estimateCost(req)
				.then((e) => {
					if (ticket === estimateTicket) estimate = e;
				})
				.catch(() => {
					if (ticket === estimateTicket) estimate = null;
				});
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
	/** The whole job at a pace, as the engine sums it; none while a price or a video's length
	 * is unknown. */
	const totalFor = (at: Pace) =>
		(at === 'economy' ? estimate?.total_economy_usd : estimate?.total_usd) ?? null;
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
		return settingsOf({
			languageMode,
			languages,
			languagesTaken: languageIgnored.length < models.length,
			sample,
			samplePages,
			paged,
			lineBreaks,
			linesTaken: linesApply && noPrompt.length < models.length,
			pace: chosenPace,
			pagesPerCall
		});
	}

	/** The form as a pipeline: everything but the files. Run now, it keeps only the formats the
	 * chosen models can write, as a job does. */
	function currentPipeline(pipelineName: string, toRun = false): Pipeline {
		return formPipeline(
			{
				name: pipelineName,
				task,
				catalog,
				rule,
				drafts,
				settings: jobSettings(),
				formats,
				destination: $state.snapshot(destination)
			},
			toRun
		);
	}

	/** The analysis that follows, as a pipeline: the researcher's usual formats for it. */
	function thenPipeline(preset: Preset): Pipeline {
		return followingAnalysis(
			preset,
			toRequest(catalog, 'analysis', thenDraft),
			analysisRule,
			app.settings!.text_formats,
			chosenPace,
			$state.snapshot(destination)
		);
	}

	/** Fills the form from a saved pipeline; the files stay as they are. */
	async function applyPipeline(p: Pipeline) {
		// A transcription pipeline applied to videos keeps the videos.
		await loadTask(mode === 'youtube' && p.task === 'audio_transcription' ? 'youtube' : p.task);
		drafts = draftsOf(p);
		languageMode = p.settings.languages.mode;
		languages = p.settings.languages.mode === 'explicit' ? [...p.settings.languages.languages] : [];
		lineBreaks = p.settings.line_breaks ?? 'running_text';
		pace = p.settings.pace ?? 'fast';
		const sent = togetherOf(p.settings.pages_per_call);
		together = sent.together;
		if (sent.pages !== undefined) togetherPages = sent.pages;
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
			const saved = await api.savePipeline(currentPipeline(pipelineName));
			pipelines = await api.listPipelines();
			naming = null;
			toast(m.newJob.pipelineSaved(saved.name));
		} catch (e) {
			error = describeError(e, m);
		}
	}

	/** Reads the text, then analyses it: a two-step workflow, unsaved, run on these files. */
	async function startWithAnalysis(preset: Preset) {
		const jobName = name.trim() || sources[0]?.file_name || m.newJob.title;
		const view = await api.startRun({
			workflow: {
				id: '',
				name: m.newJob.then.workflowName(m.task[task], preset.name),
				version: 0,
				steps: [
					{
						pipeline: currentPipeline(m.task[task], true),
						input: { kind: 'sources' },
						review: thenCheck
					},
					{ pipeline: thenPipeline(preset), input: { kind: 'step', step: 0 }, review: false }
				],
				updated_at: 0
			},
			name: jobName,
			sources: $state.snapshot(sources),
			project: from.project?.id ?? null,
			collection: from.project?.collectionId ?? null
		});
		const first = view.steps[0]?.job?.id;
		if (first) liveJob(first);
		await refreshJobs();
		started();
		go(first ? { name: 'jobs', jobId: first } : { name: 'workflows', runId: view.run.id });
	}

	async function start() {
		error = null;
		starting = true;
		try {
			const preset = thenOffered ? analysisPresets.find((p) => p.id === thenPreset) : undefined;
			// Asked once for each cloud provider the job, or its analysis after, sends files to.
			const setups = [
				...drafts.map((d) => toRequest(catalog, task, d)),
				...(preset ? thenPipeline(preset).setups : [])
			];
			const beside = destination.kind === 'beside_sources';
			if (!(await mayStart(setups, catalog, $state.snapshot(sources), beside))) return;
			if (preset) {
				await startWithAnalysis(preset);
				return;
			}
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
			// Started from a project: its files are the project's documents. The job runs even
			// if this fails, and can be added from the project later.
			const project = from.project;
			if (project) {
				await api
					.linkJob(project.id, job.id, project.collectionId ?? null)
					.catch(() => toast(m.projects.linkFailed(project.name)));
			}
			await api.startJob(job.id);
			await refreshJobs();
			started();
			go({ name: 'jobs', jobId: job.id });
		} catch (e) {
			error = describeError(e, m);
		} finally {
			starting = false;
		}
	}

	/** Set once the job has started: the form is not kept for next time. */
	let done = false;

	function started() {
		done = true;
		forgetForm();
	}

	/** Empties the form, files and all, for a job of another kind. */
	function startOver() {
		done = true;
		forgetForm();
		go({ name: 'new' });
	}

	// The form is kept as it changes, so leaving New job (for Settings, say) loses nothing.
	$effect(() => {
		const form = {
			mode,
			files: $state.snapshot(files),
			frozen: from.frozen,
			project: from.project,
			languageMode,
			languages: $state.snapshot(languages),
			lineBreaks,
			pace,
			drafts: $state.snapshot(drafts),
			sample,
			samplePages,
			together,
			togetherPages,
			formats: $state.snapshot(formats),
			destination: $state.snapshot(destination),
			name,
			suggested,
			thenPreset,
			thenCheck
		};
		if (!done) keepForm(form);
	});

	function onKey(e: KeyboardEvent) {
		if (primary(e) && e.key === 'Enter' && ready && !starting && !shortcutsHeld()) {
			e.preventDefault();
			void start();
		}
	}

	onMount(() => {
		// The OCR models' seconds a page and device change as jobs run: read them again.
		void refreshOffline();
		// Opened from a finished job's "Summarise" button: its results are the files.
		const fail = (e: unknown) => (error = describeError(e, m));
		// Coming back to the form left here: only the task's prompts are read again.
		if (kept) {
			api
				.listPresets(task)
				.then((p) => (presets = p))
				.catch(fail);
		} else {
			loadTask(opening.task ?? mode)
				.then(() => {
					// Text frozen from a job's corrected results, or an item's pages cut out of its
					// document: already inspected, nothing to do.
					if (opening.sources) {
						files = opening.sources.map((source) => ({
							path: source.path,
							source,
							warnings: [],
							error: null,
							pieces: null
						}));
						if (files[0]) suggest(nameOfFile(files[0], true));
					}
					return opening.paths ? addPaths(opening.paths) : undefined;
				})
				.catch(fail);
		}
		api
			.listPipelines()
			.then((p) => (pipelines = p))
			.catch(() => undefined);
		api
			.listPresets('analysis')
			.then((p) => (analysisPresets = p))
			.catch(() => undefined);
		return listenWhileMounted(() => onDrop((paths) => void addPaths(paths)));
	});
</script>

<svelte:window onkeydown={onKey} />

<header class="masthead">
	<div>
		<h1>{m.newJob.title}</h1>
		<p class="lede">{m.newJob.lede}</p>
	</div>
	{#if files.length > 0 || name}
		<button class="btn ghost" title={m.newJob.startOverHint} onclick={startOver}
			>{m.newJob.startOver}</button
		>
	{/if}
</header>
{#if from.project}
	<p class="notice from-project">
		<Icon name="project" size={14} />{m.projects.fromProject(from.project.name)}
	</p>
{/if}

<div class="layout">
	<div class="form steps">
		<PipelineList bind:pipelines onapply={applyPipeline} onerror={(e) => (error = e)} />

		<section class="step" aria-labelledby="task-label">
			<h2 id="task-label">{m.newJob.taskLabel}</h2>
			<TaskPicker
				{mode}
				onpick={(t) => {
					picked++;
					loadTask(t).catch((e: unknown) => (error = describeError(e, m)));
				}}
			/>
			<PipelineDiagram
				{mode}
				{files}
				fields={chosenFields}
				models={namesOf((model) => model !== undefined)}
				mergeModels={mergeNames}
				searchable={searchableFrom}
				play={picked}
			/>
		</section>

		{#if mode === 'youtube'}
			<LinkList
				bind:links={files}
				onadd={(added) => {
					const titled = added.find((l) => l.source?.title)?.source?.title;
					if (titled) suggest(titled);
				}}
				onerror={(e) => (error = e)}
			/>
		{:else}
			<FileList bind:files {task} {inspecting} onadd={addPaths} />
		{/if}

		{#if setAside}
			{@const aside = setAside}
			{@const taskName = m.common.quoted(m.task[mode])}
			{@const set = m.newJob.setAside}
			{@const refusedFiles = aside.refused.filter((r) => !r.folder)}
			{@const folders = aside.refused.filter((r) => r.folder)}
			<div class="notice set-aside" role="status">
				<div class="set-aside-text">
					{#if aside.youtube}
						<p>{set.youtube}</p>
					{/if}
					{#if refusedFiles.length > 0}
						<p>
							{set.files(
								listNames(refusedFiles.map((r) => r.file_name)),
								taskName,
								refusedFiles.length
							)}
						</p>
					{/if}
					{#if folders.length > 0}
						<p>{set.folders(listNames(folders.map((r) => r.file_name)), taskName)}</p>
					{/if}
					{#if refusedFiles.some((r) => r.kind === null)}
						<p class="set-aside-kinds">{set.kinds}</p>
					{/if}
				</div>
				<div class="set-aside-actions">
					{#each aside.readers as reader (reader)}
						<button
							class="btn sm"
							title={set.insteadHint}
							onclick={() => switchTo(reader).catch((e: unknown) => (error = describeError(e, m)))}
							><Icon name={taskOf(reader)} size={14} />{set.instead(m.task[reader])}</button
						>
					{/each}
					<button
						class="btn ghost sm icon"
						aria-label={set.dismiss}
						title={set.dismiss}
						onclick={() => (setAside = null)}><Icon name="close" size={14} /></button
					>
				</div>
			</div>
		{/if}

		<SetupList
			bind:drafts
			{task}
			{presets}
			{input}
			projectNames={from.project
				? () =>
						api.dictionaryView(from.project!.id, null).then((view) => dictionaryNames(view.rows))
				: undefined}
		/>

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

		{#if rule.pages_per_call}
			<section class="step block" aria-labelledby="together-title">
				<h2 id="together-title">{m.newJob.together.title}</h2>
				<ChoiceStrip bind:value={together} options={togetherOptions} />
				{#if together === 'several'}
					<label class="field pages">
						<span>{m.newJob.together.pages}</span>
						<input type="number" min="2" max="100" bind:value={togetherPages} />
					</label>
				{/if}
				{#if together !== 'one'}
					<p class="hint">{m.newJob.together.safety}</p>
					{#if chosenPace === 'economy'}
						<p class="hint">{m.newJob.together.economy}</p>
					{/if}
					{#if sources.some((s) => s.kind !== 'pdf' || s.split_spreads)}
						<p class="hint">{m.newJob.together.alone}</p>
					{/if}
					{#if noPrompt.length > 0}
						<p class="hint">{m.newJob.together.noPrompt(listNames(noPrompt))}</p>
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

		{#if thenOffered}
			<ThenStep
				bind:presetId={thenPreset}
				bind:check={thenCheck}
				presets={analysisPresets}
				{task}
				model={modelInfo(catalog, thenDraft)?.name ?? thenDraft.model}
			/>
		{/if}
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
	/* Files added that the task does not read, beside the files: what and where else. */
	.set-aside {
		justify-content: space-between;
		align-items: flex-start;
	}
	.set-aside-text {
		display: grid;
		gap: var(--space-1);
		flex: 1 1 18rem;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.set-aside-kinds {
		color: var(--text-secondary);
	}
	.set-aside-actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
	/* A page count, not a sentence: as wide as a few digits. */
	.field.pages {
		max-width: 10rem;
	}
	.from-project {
		margin: var(--space-3) var(--space-5) 0;
	}
	/* The cost column is painted the whole height of the form, its line and its ground, so it
	   reaches the foot of the page however long the form; the panel stays in view over it. */
	.layout {
		--panel: 21rem;
		display: grid;
		grid-template-columns: minmax(0, 1fr) var(--panel);
		min-height: calc(100% - 3.5rem);
		background: linear-gradient(
			to right,
			transparent calc(100% - var(--panel)),
			var(--line) calc(100% - var(--panel)),
			var(--line) calc(100% - var(--panel) + 1px),
			var(--surface-1) calc(100% - var(--panel) + 1px)
		);
	}
	.form {
		display: grid;
		/* As wide as the window leaves, whatever the steps hold. */
		grid-template-columns: minmax(0, 1fr);
		align-content: start;
		gap: var(--space-6);
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
			background: none;
		}
	}
	@media (max-width: 760px) {
		.form {
			padding: var(--space-4) var(--space-4) var(--space-6);
		}
	}
</style>
