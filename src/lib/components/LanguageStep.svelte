<script lang="ts">
	// New job's language step: detected, or chosen from the languages the models read (one, for
	// an analysis, a table of contents or a translation, which answer in it), with notes on the
	// models that ignore or do not read a language. A model that cannot detect the language
	// (Tesseract) needs it chosen, and its language data downloaded; so does a translation, the
	// language it translates into.
	import LanguagePicker from './LanguagePicker.svelte';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { listNames, messages } from '#lib/i18n/index.js';
	import {
		languageLabel,
		modelsNeedingTwoLetters,
		modelsNotDetecting,
		modelsTakingOne,
		modelsWithout,
		modelsWithoutTimes,
		offeredLanguages,
		takesLanguage
	} from '#lib/languages.js';
	import { rulesFor } from '#lib/newJob.js';
	import { app, go, missingLanguageData } from '#lib/state.svelte.js';

	let {
		mode = $bindable(),
		languages = $bindable(),
		task,
		models,
		ignored
	}: {
		mode: 'auto' | 'explicit';
		/** The languages chosen, as codes. */
		languages: string[];
		task: Task;
		/** The chosen models, one per set-up (undefined for one not in the catalogue). */
		models: (ModelInfo | undefined)[];
		/** Names of the chosen models that take no language. */
		ignored: string[];
	} = $props();

	const m = messages();
	const catalog = $derived(app.catalog!);
	const narrowedBy = $derived([
		...new Set(
			models
				.filter((model) => takesLanguage(model) && Array.isArray(model?.languages))
				.map((model) => model?.name ?? '')
		)
	]);
	const unsupportedChosen = $derived(
		languages
			.map((code) => [code, modelsWithout(code, models)] as const)
			.filter(([, names]) => names.length > 0)
	);
	/** What the chosen models' APIs do with the languages chosen, for transcription. */
	const takingOne = $derived(modelsTakingOne(languages.length, models));
	const withoutTimes = $derived(modelsWithoutTimes(models));
	const twoLetters = $derived(
		languages
			.map((code) => [code, modelsNeedingTwoLetters(code, models)] as const)
			.filter(([, names]) => names.length > 0)
	);
	/** The language chosen is the answers', not the source's. */
	const answers = $derived(rulesFor(app.rules, task).answers_in_language);
	/** Models that cannot detect the language: it must be chosen, and its data downloaded. */
	const notDetecting = $derived(modelsNotDetecting(models));
	const missingData = $derived(notDetecting.length > 0 ? missingLanguageData(languages) : []);
	/** The language must be chosen: a translation's, or one a model cannot detect. */
	const needed = $derived(rulesFor(app.rules, task).needs_language);
	$effect(() => {
		if ((needed || notDetecting.length > 0) && mode === 'auto') mode = 'explicit';
	});
</script>

<section class="step block" aria-labelledby="lang-title">
	<h2 id="lang-title">
		{needed
			? m.newJob.translationLanguageTitle
			: answers
				? m.newJob.analysisLanguageTitle
				: m.newJob.languageTitle}
	</h2>
	{#if ignored.length === models.length}
		<p class="hint">{m.newJob.languageIgnoredAll(listNames(ignored))}</p>
	{:else}
		{#if !needed}
			<div class="row">
				<label class="inline"
					><input type="radio" bind:group={mode} value="auto" disabled={notDetecting.length > 0} />
					{answers ? m.newJob.analysisLanguageAuto : m.newJob.languageAuto}</label
				>
				<label class="inline"
					><input type="radio" bind:group={mode} value="explicit" />
					{m.newJob.languagePick}</label
				>
			</div>
		{/if}
		{#if notDetecting.length > 0}
			<p class="hint">{m.newJob.languageNotDetected(listNames(notDetecting))}</p>
		{/if}
		{#if mode === 'explicit'}
			<LanguagePicker
				bind:selected={languages}
				offered={offeredLanguages(catalog, models)}
				single={answers}
				unsupported={(code) => modelsWithout(code, models)}
			/>
			{#if missingData.length > 0}
				<p class="hint warn-text">
					{m.newJob.languageDataMissing(
						listNames(missingData.map((code) => languageLabel(catalog, code)))
					)}
					<button class="btn sm" onclick={() => go({ name: 'settings', tab: 'offline' })}
						>{m.newJob.openSettings}</button
					>
				</p>
			{/if}
			{#each unsupportedChosen as [code, names] (code)}
				<p class="hint warn-text">
					{m.newJob.languageUnsupported(languageLabel(catalog, code), listNames(names))}
				</p>
			{/each}
			{#if takingOne.length > 0}
				<p class="hint">{m.newJob.languageOneOnly(listNames(takingOne))}</p>
			{/if}
			{#if withoutTimes.length > 0 && languages.length === 1}
				<p class="hint">{m.newJob.languageNotWithTimes(listNames(withoutTimes))}</p>
			{/if}
			{#each twoLetters as [code, names] (code)}
				<p class="hint">
					{m.newJob.languageNeedsTwoLetters(languageLabel(catalog, code), listNames(names))}
				</p>
			{/each}
			{#if narrowedBy.length > 0}
				<p class="hint">{m.newJob.languagesNarrowed(listNames(narrowedBy))}</p>
			{/if}
		{/if}
		<p class="hint">
			{task === 'analysis'
				? m.newJob.analysisLanguageHint
				: task === 'contents'
					? m.newJob.contentsLanguageHint
					: task === 'translation'
						? m.newJob.translationLanguageHint
						: m.newJob.languageHint}
		</p>
		{#if ignored.length > 0}
			<p class="hint">{m.newJob.languageIgnoredSome(listNames(ignored))}</p>
		{/if}
	{/if}
</section>

<style>
	.block {
		display: grid;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
	}
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 60ch;
	}
	.warn-text {
		color: var(--warn-text);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
</style>
