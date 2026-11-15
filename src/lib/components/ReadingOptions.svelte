<script lang="ts">
	// New job's reading options, where the engine says they apply: whether the text keeps the
	// page's line breaks, and how many pages of a PDF go in one call.
	import ChoiceStrip from './ChoiceStrip.svelte';
	import type { LineBreaks } from '#lib/bindings/LineBreaks.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import { listNames, messages } from '#lib/i18n/index.js';
	import type { Together } from '#lib/newJob.js';

	let {
		lineBreaks = $bindable(),
		together = $bindable(),
		togetherPages = $bindable(),
		linesApply,
		pagesTogether,
		models,
		noPrompt,
		pace,
		sources
	}: {
		lineBreaks: LineBreaks;
		/** How many pages of a PDF go in one call: one, several, or as many as fit. */
		together: Together;
		togetherPages: number;
		/** The task keeps or joins line breaks, as the engine says. */
		linesApply: boolean;
		/** The task can send several pages in one call, as the engine says. */
		pagesTogether: boolean;
		/** The chosen models. */
		models: (ModelInfo | undefined)[];
		/** Distinct names of the chosen models that take no prompt. */
		noPrompt: string[];
		/** The pace chosen. */
		pace: Pace;
		/** The sources as the job reads them. */
		sources: SourceFile[];
	} = $props();
	const m = messages();

	const togetherOptions = (['one', 'several', 'whole'] as const).map((value) => ({
		value,
		label: m.newJob.together[value],
		hint: m.newJob.together[`${value}_hint`]
	}));
	const lineOptions = (['running_text', 'as_on_page'] as const).map((value) => ({
		value,
		label: m.newJob.lines[value],
		hint: m.newJob.lines[`${value}_hint`]
	}));
</script>

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

{#if pagesTogether}
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
			{#if pace === 'economy'}
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

<style>
	/* A page count, not a sentence: as wide as a few digits. */
	.field.pages {
		max-width: 10rem;
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
</style>
