<script lang="ts">
	// Offline engines' downloads: Whisper's models, as in Live Translation's settings, the
	// language data of an engine built into the app (Tesseract), one file per language, the
	// OCR vision models, each with how long a page took on this computer, and the language
	// models for analyses. Of the hundred-odd
	// languages only those downloaded are listed; the others are found by name in a search box.
	// While a download runs the lists are polled for the bytes received; on opening they are read
	// again, since the seconds a page took change as jobs run.
	import { onMount } from 'svelte';
	import { modelDescription } from '#lib/catalogText.js';
	import type { OfflineModel } from '#lib/bindings/OfflineModel.js';
	import { describeError } from '#lib/errors.js';
	import { listNames, messages } from '#lib/i18n/index.js';
	import { languageLabel } from '#lib/languages.js';
	import { api } from '#lib/tauri.js';
	import { app, onGraphics, refreshOffline, secondsPerPage, toast } from '#lib/state.svelte.js';
	import { locale } from '#lib/i18n/index.js';
	import LanguageDataPicker from './LanguageDataPicker.svelte';
	import ProgressBar from './ProgressBar.svelte';

	const m = messages();

	onMount(() => void refreshOffline());

	const local = $derived(
		(app.catalog?.models ?? []).filter(
			(mod) => app.catalog?.providers.find((p) => p.id === mod.provider)?.local
		)
	);
	const status = (id: string) => app.offline?.models.find((s) => s.id === id);
	const ocrStatus = (id: string) => app.offline?.ocr_models.find((s) => s.id === id);
	const llmStatus = (id: string) => app.offline?.llm_models.find((s) => s.id === id);
	/** Models downloaded on their own (Whisper). */
	const downloadable = $derived(local.filter((mod) => status(mod.id)));
	/** OCR vision models, each with its image encoder. */
	const ocr = $derived(local.filter((mod) => ocrStatus(mod.id)));
	/** Language models for analyses, each with its encoder. */
	const llm = $derived(local.filter((mod) => llmStatus(mod.id)));
	/** Engines built into the app, which need language data only (Tesseract). */
	const builtIn = $derived(
		local.filter((mod) => !status(mod.id) && !ocrStatus(mod.id) && !llmStatus(mod.id))
	);
	const downloading = $derived(
		[
			...(app.offline?.models ?? []),
			...(app.offline?.language_data ?? []),
			...(app.offline?.ocr_models ?? []),
			...(app.offline?.llm_models ?? [])
		].some((s) => s.downloading)
	);
	const noOcrEngine = $derived(app.offline !== null && !app.offline.ocr_engine);
	const noLlmEngine = $derived(app.offline !== null && !app.offline.llm_engine);
	/** How long a page took with an OCR model on this computer, in words. */
	function ocrSpeed(id: string): string {
		const seconds = secondsPerPage(id);
		return seconds ? m.settings.ocrSpeed(seconds, onGraphics(id)) : m.settings.ocrNotTimed;
	}
	/** The graphics processor found for the OCR models on Windows, offered as a choice. */
	const graphics = $derived(app.offline?.ocr_graphics_processor ?? null);

	/** Saved, then the status read again: the speeds shown follow the device. */
	async function useGraphics() {
		if (!app.settings) return;
		try {
			await api.saveSettings($state.snapshot(app.settings));
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		} finally {
			await refreshOffline();
		}
	}
	const missingCpu = $derived((app.offline?.missing_cpu.length ?? 0) > 0);

	/** Language data by the name of what it reads, in the interface language's order. */
	const languageData = $derived.by(() => {
		const catalog = app.catalog;
		if (!catalog) return [];
		const collator = new Intl.Collator(locale(), { sensitivity: 'base' });
		return (app.offline?.language_data ?? [])
			.map((s) => {
				const names = s.languages.map((code) => languageLabel(catalog, code));
				const name = names.length === 1 ? names[0] : m.settings.latinScript(listNames(names));
				return { s, name, script: names.length > 1 };
			})
			.sort((a, b) => Number(a.script) - Number(b.script) || collator.compare(a.name, b.name));
	});
	/** Downloaded or downloading: listed as rows. */
	const languagesKept = $derived(languageData.filter(({ s }) => s.installed || s.downloading));
	/** The others, found by name. */
	const languagesOffered = $derived(
		languageData
			.filter(({ s }) => !s.installed && !s.downloading)
			.map(({ s, name }) => ({ id: s.id, name, bytes: Number(s.bytes) }))
	);

	let poll: ReturnType<typeof setInterval> | null = null;
	$effect(() => {
		if (downloading && !poll) {
			poll = setInterval(() => void refreshOffline(), 500);
		} else if (!downloading && poll) {
			clearInterval(poll);
			poll = null;
		}
		return () => {
			if (poll) clearInterval(poll);
			poll = null;
		};
	});

	async function download(id: string) {
		const started = api.downloadModel(id);
		await refreshOffline();
		try {
			await started;
		} catch (e) {
			if (!(typeof e === 'object' && e && 'id' in e && e.id === 'offline.cancelled')) {
				toast(describeError(e, m), { kind: 'error' });
			}
		} finally {
			await refreshOffline();
		}
	}

	async function remove(id: string) {
		try {
			await api.removeModel(id);
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		} finally {
			await refreshOffline();
		}
	}
</script>

{#snippet row(
	s: OfflineModel | undefined,
	id: string,
	name: string,
	description: string,
	blocked: boolean
)}
	{@const received = Number(s?.downloaded_bytes ?? 0)}
	{@const pct = Math.floor((100 * received) / Math.max(1, Number(s?.bytes ?? 0)))}
	<!-- A download stopped part-way: what it kept is continued by the next one. -->
	{@const stopped = !!s && !s.downloading && !s.installed && received > 0}
	<div class="model">
		<div class="name">
			<strong>{name}</strong>
			<span class="muted small">{m.settings.size(Number(s?.bytes ?? 0))}</span>
			{#if s?.downloading}
				<span class="chip">{m.settings.modelDownloading(pct)}</span>
			{:else if s?.installed}
				<span class="chip ok">{s.in_use ? m.settings.modelInUse : m.settings.modelInstalled}</span>
			{:else if stopped}
				<span class="chip">{m.settings.modelStopped(pct)}</span>
			{:else}
				<span class="chip">{m.settings.modelMissing}</span>
			{/if}
		</div>
		{#if description}
			<p class="muted small">{description}</p>
		{/if}
		{#if s?.downloading || stopped}
			<ProgressBar done={received} total={Number(s?.bytes ?? 0)} label={name} />
		{/if}
		{#if stopped}
			<p class="muted small">{m.settings.modelKept(m.settings.size(received))}</p>
		{/if}
		<div class="actions">
			{#if s?.downloading}
				<button
					class="btn sm"
					onclick={() =>
						api
							.cancelModelDownload()
							.catch((e: unknown) => toast(describeError(e, m), { kind: 'error' }))}
					>{m.settings.cancelDownload}</button
				>
			{:else if s?.installed}
				<button class="btn sm ghost danger" disabled={s.in_use} onclick={() => remove(id)}
					>{m.settings.removeModel}</button
				>
			{:else}
				<button
					class="btn sm primary"
					disabled={downloading || blocked}
					onclick={() => download(id)}
					>{stopped ? m.settings.continueDownload : m.settings.download}</button
				>
				{#if stopped}
					<button class="btn sm ghost danger" disabled={downloading} onclick={() => remove(id)}
						>{m.settings.discardDownload}</button
					>
				{/if}
			{/if}
		</div>
	</div>
{/snippet}

<section class="card block" aria-labelledby="offline-title">
	<h2 id="offline-title">{m.settings.offlineTitle}</h2>
	{#if downloadable.length > 0}
		<h3>{m.settings.whisperTitle}</h3>
		<p class="muted small">{m.settings.offlineHint}</p>
		{#if missingCpu}
			<p class="warning small">
				{m.settings.offlineCpu(app.offline?.missing_cpu.join(', ') ?? '')}
			</p>
		{/if}
		{#each downloadable as mod (mod.id)}
			{@render row(status(mod.id), mod.id, mod.name, modelDescription(mod), missingCpu)}
		{/each}
	{/if}
	{#if builtIn.length > 0 && languageData.length > 0}
		<h3>{m.settings.tesseractTitle}</h3>
		<p class="muted small">{m.settings.tesseractHint(app.offline?.tesseract_version ?? '')}</p>
		{#if languagesKept.length === 0}
			<p class="muted small">{m.settings.noLanguageData}</p>
		{/if}
		{#each languagesKept as { s, name } (s.id)}
			{@render row(s, s.id, name, '', false)}
		{/each}
		{#if languagesOffered.length > 0}
			<div class="add-language">
				<h4>{m.settings.addLanguageData}</h4>
				<LanguageDataPicker
					items={languagesOffered}
					disabled={downloading}
					onpick={(id) => void download(id)}
				/>
			</div>
		{/if}
	{/if}
	{#if ocr.length > 0}
		<h3>{m.settings.ocrTitle}</h3>
		<p class="muted small">{m.settings.ocrHint}</p>
		{#if noOcrEngine}
			<p class="warning small">{m.settings.ocrNoEngine}</p>
		{:else if missingCpu}
			<p class="warning small">
				{m.settings.ocrCpu(app.offline?.missing_cpu.join(', ') ?? '')}
			</p>
		{/if}
		{#if graphics && app.settings && !noOcrEngine && !missingCpu}
			<div class="graphics">
				<label class="inline small">
					<input
						type="checkbox"
						bind:checked={app.settings.ocr_graphics_processor}
						onchange={useGraphics}
					/>
					{m.settings.ocrGraphics(graphics)}
				</label>
				<p class="muted small">{m.settings.ocrGraphicsHint}</p>
			</div>
		{/if}
		{#each ocr as mod (mod.id)}
			{@const s = ocrStatus(mod.id)}
			{@render row(
				s,
				mod.id,
				mod.name,
				`${modelDescription(mod)}${s?.installed ? ` ${ocrSpeed(mod.id)}` : ''}`,
				missingCpu || noOcrEngine
			)}
		{/each}
	{/if}
	{#if llm.length > 0}
		<h3>{m.settings.llmTitle}</h3>
		<p class="muted small">{m.settings.llmHint}</p>
		{#if noLlmEngine}
			<p class="warning small">{m.settings.llmNoEngine}</p>
		{:else if missingCpu}
			<p class="warning small">
				{m.settings.llmCpu(app.offline?.missing_cpu.join(', ') ?? '')}
			</p>
		{/if}
		{#each llm as mod (mod.id)}
			{@render row(
				llmStatus(mod.id),
				mod.id,
				mod.name,
				modelDescription(mod),
				missingCpu || noLlmEngine
			)}
		{/each}
	{/if}
</section>

<style>
	.block {
		padding: var(--space-4) 0 var(--space-5);
		display: grid;
		gap: var(--space-3);
	}
	/* Each engine after the first starts below a stronger line and well clear of the one before,
	   so a part such as Tesseract's language download reads as the engine's above it. */
	h3 {
		font-size: var(--type-md);
		margin-top: var(--space-5);
		padding-top: var(--space-5);
		border-top: 2px solid var(--line-strong);
	}
	h3:first-of-type {
		margin-top: var(--space-3);
		padding-top: 0;
		border-top: 0;
	}
	.small {
		font-size: var(--type-sm);
	}
	.graphics {
		display: grid;
		gap: var(--space-1);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
	/* With Tesseract's languages above it, not set apart like the next engine. */
	.add-language {
		display: grid;
		gap: var(--space-2);
	}
	h4 {
		font-size: var(--type-sm);
		font-weight: 500;
	}
	.model {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3) 0;
		border-top: 1px solid var(--line);
	}
	.name {
		display: flex;
		gap: var(--space-2);
		align-items: baseline;
		flex-wrap: wrap;
	}
	.actions {
		display: flex;
		gap: var(--space-2);
	}
	.ok {
		color: var(--ok);
	}
	.warning {
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--warn-border);
		background: var(--warn-wash);
		border-radius: var(--radius-control);
	}
</style>
