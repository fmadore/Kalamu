<script lang="ts">
	// New job's first step: the task, and the diagram of its work drawn with the form's own files,
	// prompt and models.
	import PipelineDiagram from './PipelineDiagram.svelte';
	import TaskPicker from './TaskPicker.svelte';
	import type { Catalog } from '#lib/bindings/Catalog.js';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { TaskRules } from '#lib/bindings/TaskRules.js';
	import { messages } from '#lib/i18n/index.js';
	import { type Mode, offeredFormats } from '#lib/newJob.js';
	import { type Draft, mergeInfo } from '#lib/newJobModels.js';

	let {
		mode,
		files,
		sources,
		presets,
		drafts,
		catalog,
		rule,
		namesOf,
		play,
		onpick
	}: {
		mode: Mode;
		files: InspectedFile[];
		/** The files as the job reads them: images read as one document are one source. */
		sources: SourceFile[];
		presets: Preset[];
		drafts: Draft[];
		catalog: Catalog;
		rule: TaskRules;
		/** Distinct names of the chosen models matching a test. */
		namesOf: (test: (model: ModelInfo | undefined) => boolean) => string[];
		/** Times a task was picked, so its diagram plays; opening New job shows it at rest. */
		play: number;
		onpick: (mode: Mode) => void;
	} = $props();
	const m = messages();

	/** The files as the job reads them, for the diagram: the document in place of its images. */
	const readFiles = $derived<InspectedFile[]>(
		sources.map(
			(s) =>
				files.find((f) => f.source?.id === s.id) ?? {
					path: s.path,
					source: s,
					warnings: [],
					error: null,
					pieces: null
				}
		)
	);
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
</script>

<section class="step" aria-labelledby="task-label">
	<h2 id="task-label">{m.newJob.taskLabel}</h2>
	<TaskPicker {mode} {onpick} />
	<PipelineDiagram
		{mode}
		files={readFiles}
		fields={chosenFields}
		models={namesOf((model) => model !== undefined)}
		mergeModels={mergeNames}
		searchable={searchableFrom}
		{play}
	/>
</section>
