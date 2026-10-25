<script lang="ts">
	// Whisper models: download, cancel and remove, as in Live Translation's settings. While a
	// download runs the list is polled for the bytes received.
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api } from '#lib/tauri.js';
	import { app, refreshOffline, toast } from '#lib/state.svelte.js';
	import ProgressBar from './ProgressBar.svelte';

	const m = messages();

	const local = $derived(
		(app.catalog?.models ?? []).filter(
			(mod) => app.catalog?.providers.find((p) => p.id === mod.provider)?.local
		)
	);
	const status = (id: string) => app.offline?.models.find((s) => s.id === id);
	const downloading = $derived(app.offline?.models.some((s) => s.downloading) ?? false);

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

<section class="card block" aria-labelledby="offline-title">
	<h2 id="offline-title">{m.settings.offlineTitle}</h2>
	<p class="muted small">{m.settings.offlineHint}</p>
	{#if app.offline && app.offline.missing_cpu.length > 0}
		<p class="warning small">{m.settings.offlineCpu(app.offline.missing_cpu.join(', '))}</p>
	{/if}
	{#each local as mod (mod.id)}
		{@const s = status(mod.id)}
		<div class="model">
			<div class="name">
				<strong>{mod.name}</strong>
				<span class="muted small">{m.settings.size(Number(s?.bytes ?? 0))}</span>
				{#if s?.downloading}
					<span class="chip"
						>{m.settings.modelDownloading(
							Math.floor((100 * Number(s.downloaded_bytes)) / Math.max(1, Number(s.bytes)))
						)}</span
					>
				{:else if s?.installed}
					<span class="chip ok">{s.in_use ? m.settings.modelInUse : m.settings.modelInstalled}</span
					>
				{:else}
					<span class="chip">{m.settings.modelMissing}</span>
				{/if}
			</div>
			<p class="muted small">{mod.description}</p>
			{#if s?.downloading}
				<ProgressBar done={Number(s.downloaded_bytes)} total={Number(s.bytes)} label={mod.name} />
			{/if}
			<div class="actions">
				{#if s?.downloading}
					<button class="btn sm" onclick={() => api.cancelModelDownload()}
						>{m.settings.cancelDownload}</button
					>
				{:else if s?.installed}
					<button class="btn sm ghost danger" disabled={s.in_use} onclick={() => remove(mod.id)}
						>{m.settings.removeModel}</button
					>
				{:else}
					<button
						class="btn sm primary"
						disabled={downloading || (app.offline?.missing_cpu.length ?? 0) > 0}
						onclick={() => download(mod.id)}>{m.settings.download}</button
					>
				{/if}
			</div>
		</div>
	{/each}
</section>

<style>
	.block {
		padding: var(--space-4) 0 0;
		display: grid;
		gap: var(--space-3);
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
