<script lang="ts">
	import Icon from './Icon.svelte';
	import { onMount } from 'svelte';
	import FileList from './FileList.svelte';
	import LanguageStep from './LanguageStep.svelte';
	import LinkList from './LinkList.svelte';
	import OutputStep from './OutputStep.svelte';
	import PaceStep from './PaceStep.svelte';
	import PageOrder from './PageOrder.svelte';
	import PipelineList from './PipelineList.svelte';
	import ReadingOptions from './ReadingOptions.svelte';
	import SampleOffer from './SampleOffer.svelte';
	import SetAsideNotice from './SetAsideNotice.svelte';
	import SetupList from './SetupList.svelte';
	import StartPanel from './StartPanel.svelte';
	import TaskStep from './TaskStep.svelte';
	import ThenStep from './ThenStep.svelte';
	import type { Estimate } from '#lib/bindings/Estimate.js';
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import type { Pipeline } from '#lib/bindings/Pipeline.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { followEstimate } from '#lib/newJobEstimate.svelte.js';
	import { primary } from '#lib/platform.js';
	import { type Mode, nameOfFile, forgetForm, formToOpen, keepForm } from '#lib/newJob.js';
	import { JobChoices } from '#lib/newJobChoices.svelte.js';
	import { JobFiles } from '#lib/newJobFiles.svelte.js';
	import { dictionaryNames, modelInfo, readsInput, toRequest } from '#lib/newJobModels.js';
	import { longest, offersSample, SAMPLE_PAGES } from '#lib/newJobSample.js';
	import { startJob } from '#lib/newJobStart.svelte.js';
	import { FollowingAnalysis } from '#lib/newJobThen.svelte.js';
	import { api, listenWhileMounted, onDrop } from '#lib/tauri.js';
	import {
		go,
		hasKey,
		refreshOffline,
		shortcutsHeld,
		takeFiles,
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

	/** The files, the images as one document, and the job's name they suggest. */
	const files: JobFiles = new JobFiles(
		{
			mode: () => choices.mode,
			task: () => choices.task,
			rule: () => choices.rule,
			frozen: from.frozen,
			error: (message) => (error = message)
		},
		kept
	);
	/** The task, its set-ups and the settings they share. */
	const choices: JobChoices = new JobChoices(files, kept);
	/** The analysis that can follow the reading. */
	const following = new FollowingAnalysis(choices, kept);
	const task = $derived(choices.task);
	const rule = $derived(choices.rule);
	const catalog = $derived(choices.catalog);
	const sources = $derived(files.sources);

	let estimate = $state<Estimate | null>(null);
	const estimates = $derived(estimate?.rows ?? []);
	let error = $state<string | null>(null);
	let starting = $state(false);
	let pipelines = $state<Pipeline[]>([]);
	/** The name being typed for a new pipeline, while saving one. */
	let naming = $state<string | null>(null);
	/** Times a task was picked, so its diagram plays; opening New job shows it at rest. */
	let picked = $state(0);

	/** Moves the files set aside to a task that reads them. */
	async function switchTo(next: Mode) {
		const move = files.setAside?.move ?? [];
		files.setAside = null;
		picked++;
		await choices.loadTask(next);
		await files.add(move);
	}

	const ready = $derived(
		sources.length > 0 &&
			choices.languagesReady &&
			choices.drafts.length > 0 &&
			choices.drafts.every((d) => d.model && hasKey(d.provider)) &&
			(task !== 'contents' ||
				choices.drafts.every((d) => d.mergeModel && hasKey(d.mergeProvider))) &&
			choices.models.every((model) => readsInput(model, choices.input)) &&
			(choices.mode !== 'youtube' || choices.destination.kind === 'folder') &&
			choices.formats.length > 0
	);

	followEstimate(
		() => ({
			task,
			sources: $state.snapshot(sources),
			setups: choices.drafts.filter((d) => d.model).map((d) => toRequest(catalog, task, d)),
			sample_pages: choices.sample && choices.paged ? choices.samplePages : null
		}),
		(e) => (estimate = e)
	);

	/** Economy mode is offered when one of the chosen models goes through a batch service. */
	const economyOffered = $derived(
		choices.mode !== 'youtube' && estimates.some((r) => r.estimate.economy_usd !== null)
	);
	const chosenPace = $derived<Pace>(economyOffered ? choices.pace : 'fast');
	/** A model's cost at a pace; one without a batch service costs the same either way. */
	const rowCost = (row: EstimateRow, at: Pace) =>
		at === 'economy' ? (row.estimate.economy_usd ?? row.estimate.usd) : row.estimate.usd;
	/** The whole job at a pace, as the engine sums it; none while a price or a video's length
	 * is unknown. */
	const totalFor = (at: Pace) =>
		(at === 'economy' ? estimate?.total_economy_usd : estimate?.total_usd) ?? null;
	const totalEstimate = $derived(totalFor(chosenPace));

	/** Why Start is unavailable, when the form cannot say it where the user is looking. */
	const blocker = $derived(
		choices.mode === 'youtube' && sources.length > 0 && choices.destination.kind !== 'folder'
			? m.newJob.needFolder
			: rule.needs_language &&
				  sources.length > 0 &&
				  !(choices.languageMode === 'explicit' && choices.languages.length > 0)
				? m.error['job.no_language']
				: null
	);

	async function savePipeline() {
		const pipelineName = naming?.trim();
		if (!pipelineName) return;
		error = null;
		try {
			const saved = await api.savePipeline(choices.pipeline(pipelineName, chosenPace));
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
			await startJob({
				choices,
				files,
				following,
				pace: () => chosenPace,
				project: from.project,
				started
			});
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
			mode: choices.mode,
			files: $state.snapshot(files.list),
			frozen: from.frozen,
			project: from.project,
			languageMode: choices.languageMode,
			languages: $state.snapshot(choices.languages),
			lineBreaks: choices.lineBreaks,
			pace: choices.pace,
			drafts: $state.snapshot(choices.drafts),
			sample: choices.sample,
			samplePages: choices.samplePages,
			sampleSpread: choices.sampleSpread,
			together: choices.together,
			togetherPages: choices.togetherPages,
			formats: $state.snapshot(choices.formats),
			destination: $state.snapshot(choices.destination),
			name: files.name,
			suggested: files.suggested,
			fromFiles: files.fromFiles,
			thenPreset: following.presetId,
			thenCheck: following.check,
			grouping: files.grouping,
			pageDoc: $state.snapshot(files.pageDoc),
			docName: files.docName,
			byHand: files.byHand
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
				.then((p) => {
					choices.presets = p;
				})
				.catch(fail);
		} else {
			choices
				.loadTask(opening.task ?? choices.mode)
				.then(() => {
					// Text frozen from a job's corrected results, or an item's pages cut out of its
					// document: already inspected, nothing to do.
					if (opening.sources) {
						files.list = opening.sources.map((source) => ({
							path: source.path,
							source,
							warnings: [],
							error: null,
							pieces: null
						}));
						if (files.list[0]) files.suggest(nameOfFile(files.list[0], true));
					}
					return opening.paths ? files.add(opening.paths) : undefined;
				})
				.catch(fail);
		}
		api
			.listPipelines()
			.then((p) => (pipelines = p))
			.catch(() => undefined);
		following.load();
		const release = takeFiles(m.newJob.dropHere);
		const stop = listenWhileMounted(() => onDrop((paths) => void files.add(paths)));
		return () => {
			release();
			stop();
			files.photos.stop();
		};
	});
</script>

<svelte:window onkeydown={onKey} />

<header class="masthead">
	<div>
		<h1>{m.newJob.title}</h1>
		<p class="lede">{m.newJob.lede}</p>
	</div>
	{#if files.list.length > 0 || files.name}
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
		<PipelineList
			bind:pipelines
			onapply={(p) => choices.applyPipeline(p)}
			onerror={(e) => (error = e)}
		/>

		<TaskStep
			mode={choices.mode}
			files={files.list}
			{sources}
			presets={choices.presets}
			drafts={choices.drafts}
			records={choices.records}
			{catalog}
			{rule}
			namesOf={choices.namesOf}
			play={picked}
			onpick={(t) => {
				picked++;
				choices.loadTask(t).catch((e: unknown) => (error = describeError(e, m)));
			}}
		/>

		{#if choices.mode === 'youtube'}
			<LinkList
				bind:links={files.list}
				onadd={(added) => {
					const titled = added.find((l) => l.source?.title)?.source?.title;
					if (titled) files.suggest(titled);
				}}
				onerror={(e) => (error = e)}
			/>
		{:else}
			<FileList
				bind:files={files.list}
				{task}
				inspecting={files.inspecting}
				onadd={(paths) => files.add(paths)}
				photos={files.photos}
				onleaveout={(id) => files.leaveOut(id)}
			/>
			{#if files.oneOffered}
				<PageOrder
					bind:grouping={files.grouping}
					bind:doc={files.pageDoc}
					bind:name={files.docName}
					bind:byHand={files.byHand}
					sorting={files.sorting}
					onnatural={() => void files.orderPages(true)}
				/>
			{/if}
			<!-- A long document: a few pages first, a sample the results step also shows. -->
			{#if offersSample(task, choices.paged, sources) && (!choices.sample || choices.sampleSpread)}
				<SampleOffer
					pages={longest(sources)}
					n={choices.sample ? choices.samplePages : SAMPLE_PAGES}
					chosen={choices.sample}
					two={choices.drafts.length > 1}
					comparable={choices.drafts.length > 1 || choices.secondDraft() !== null}
					ontry={(two) => choices.trySample(two)}
					onevery={() => choices.readEveryPage()}
				/>
			{/if}
		{/if}

		{#if files.setAside}
			<SetAsideNotice
				aside={files.setAside}
				mode={choices.mode}
				onswitch={(reader) => switchTo(reader).catch((e: unknown) => (error = describeError(e, m)))}
				ondismiss={() => (files.setAside = null)}
			/>
		{/if}

		<SetupList
			bind:drafts={choices.drafts}
			{task}
			presets={choices.presets}
			input={choices.input}
			projectNames={from.project
				? () =>
						api.dictionaryView(from.project!.id, null).then((view) => dictionaryNames(view.rows))
				: undefined}
		/>

		<LanguageStep
			bind:mode={choices.languageMode}
			bind:languages={choices.languages}
			{task}
			models={choices.models}
			ignored={choices.languageIgnored}
		/>

		<ReadingOptions
			bind:lineBreaks={choices.lineBreaks}
			bind:together={choices.together}
			bind:togetherPages={choices.togetherPages}
			linesApply={choices.linesApply}
			pagesTogether={rule.pages_per_call}
			models={choices.models}
			noPrompt={choices.noPrompt}
			pace={chosenPace}
			{sources}
		/>

		{#if economyOffered}
			<PaceStep bind:pace={choices.pace} {estimates} {catalog} {totalFor} />
		{/if}

		<OutputStep
			bind:sample={choices.sample}
			bind:samplePages={choices.samplePages}
			bind:sampleSpread={choices.sampleSpread}
			bind:formats={choices.formats}
			bind:destination={choices.destination}
			bind:name={files.name}
			mode={choices.mode}
			{task}
			drafts={choices.drafts}
			paged={choices.paged}
			{sources}
			hasSources={sources.length > 0}
		/>

		{#if following.offered}
			<ThenStep
				bind:presetId={following.presetId}
				bind:check={following.check}
				presets={following.presets}
				{task}
				model={modelInfo(catalog, following.draft)?.name ?? following.draft.model}
			/>
		{/if}
	</div>

	<!-- Stays in view while the form scrolls: what it costs, and the one action. -->
	<StartPanel
		{estimates}
		cost={(row) => rowCost(row, chosenPace)}
		total={totalEstimate}
		video={choices.mode === 'youtube' && sources.length > 0}
		{error}
		{blocker}
		{ready}
		{starting}
		canSave={choices.drafts.length > 0}
		bind:naming
		onstart={start}
		onsave={() => void savePipeline()}
	/>
</div>

<style>
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
