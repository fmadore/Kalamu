<script lang="ts">
	// Offline engines' downloads: Whisper's models, as in Live Translation's settings, and the
	// language data of an engine built into the app (Tesseract), one file per language. While a
	// download runs the lists are polled for the bytes received.
	import { modelDescription } from '#lib/catalogText.js';
	import type { OfflineModel } from '#lib/bindings/OfflineModel.js';
	import { describeError } from '#lib/errors.js';
	import { listNames, messages } from '#lib/i18n/index.js';
	import { languageLabel } from '#lib/languages.js';
	import { api } from '#lib/tauri.js';
	import { app, refreshOffline, toast } from '#lib/state.svelte.js';
	import { locale } from '#lib/i18n/index.js';
	import ProgressBar from './ProgressBar.svelte';

	const m = messages();

	const local = $derived(
		(app.catalog?.models ?? []).filter(
			(mod) => app.catalog?.providers.find((p) => p.id === mod.provider)?.local
		)
	);
	const status = (id: string) => app.offline?.models.find((s) => s.id === id);
	/** Models downloaded on their own (Whisper). */
	const downloadable = $derived(local.filter((mod) => status(mod.id)));
	/** Engines built into the app, which need language data only (Tesseract). */
	const builtIn = $derived(local.filter((mod) => !status(mod.id)));
	const downloading = $derived(
		[...(app.offline?.models ?? []), ...(app.offline?.language_data ?? [])].some(
			(s) => s.downloading
		)
	);
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
				toast(describeError(e, m));
			}
		} finally {
			await refreshOffline();
		}
	}

	async function remove(id: string) {
		try {
			await api.removeModel(id);
		} catch (e) {
			toast(describeError(e, m));
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
	<div class="model">
		<div class="name">
			<strong>{name}</strong>
			<span class="muted small">{m.settings.size(Number(s?.bytes ?? 0))}</span>
			{#if s?.downloading}
				<span class="chip"
					>{m.settings.modelDownloading(
						Math.floor((100 * Number(s.downloaded_bytes)) / Math.max(1, Number(s.bytes)))
					)}</span
				>
			{:else if s?.installed}
				<span class="chip ok">{s.in_use ? m.settings.modelInUse : m.settings.modelInstalled}</span>
			{:else}
				<span class="chip">{m.settings.modelMissing}</span>
			{/if}
		</div>
		{#if description}
			<p class="muted small">{description}</p>
		{/if}
		{#if s?.downloading}
			<ProgressBar done={Number(s.downloaded_bytes)} total={Number(s.bytes)} label={name} />
		{/if}
		<div class="actions">
			{#if s?.downloading}
				<button class="btn sm" onclick={() => api.cancelModelDownload()}
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
					onclick={() => download(id)}>{m.settings.download}</button
				>
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
		{#each languageData as { s, name } (s.id)}
			{@render row(s, s.id, name, '', false)}
		{/each}
	{/if}
</section>

<style>
	.block {
		padding: var(--space-4) 0 0;
		display: grid;
		gap: var(--space-3);
	}
	h3 {
		font-size: var(--type-md);
		margin-top: var(--space-3);
	}
	.small {
		font-size: var(--type-sm);
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
