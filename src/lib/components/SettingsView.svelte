<script lang="ts">
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, openUrl } from '#lib/tauri.js';
	import { app, hasKey, refreshKeys, toast } from '#lib/state.svelte.js';
	import OfflineModels from './OfflineModels.svelte';

	const m = messages();

	let drafts = $state<Record<string, string>>({});
	let status = $state<Record<string, { ok: boolean; text: string }>>({});
	let busy = $state<string | null>(null);

	async function withProvider(id: string, action: () => Promise<string>) {
		busy = id;
		try {
			status = { ...status, [id]: { ok: true, text: await action() } };
		} catch (e) {
			status = { ...status, [id]: { ok: false, text: describeError(e, m) } };
		} finally {
			busy = null;
			await refreshKeys();
		}
	}

	async function saveSettings() {
		if (!app.settings) return;
		try {
			await api.saveSettings($state.snapshot(app.settings));
		} catch (e) {
			toast(describeError(e, m));
		}
	}

	async function refreshCatalog() {
		try {
			app.catalog = await api.refreshCatalog();
			toast(m.settings.catalogUpdated(app.catalog.updated));
		} catch (e) {
			toast(describeError(e, m));
		}
	}
</script>

<div class="page">
	<h1>{m.settings.title}</h1>

	<section class="card block" aria-labelledby="keys-title">
		<h2 id="keys-title">{m.settings.keysTitle}</h2>
		<p class="muted small">{m.settings.keysHint}</p>
		{#each (app.catalog?.providers ?? []).filter((p) => !p.local) as p (p.id)}
			<div class="key">
				<div class="key-name">
					<strong>{p.name}</strong>
					<span class="chip" class:ok={hasKey(p.id)}
						>{hasKey(p.id) ? m.settings.keySaved : m.settings.keyMissing}</span
					>
				</div>
				<input
					type="password"
					autocomplete="off"
					spellcheck="false"
					placeholder={m.settings.keyPlaceholder}
					aria-label="{p.name} {m.settings.keyPlaceholder}"
					bind:value={drafts[p.id]}
				/>
				<div class="key-actions">
					<button
						class="btn sm primary"
						disabled={!drafts[p.id]?.trim() || busy === p.id}
						onclick={() =>
							withProvider(p.id, async () => {
								await api.setApiKey(p.id, drafts[p.id]);
								drafts[p.id] = '';
								const models = await api.testApiKey(p.id);
								return m.settings.testOk(models.length);
							})}>{m.settings.save}</button
					>
					<button
						class="btn sm"
						disabled={!hasKey(p.id) || busy === p.id}
						onclick={() =>
							withProvider(p.id, async () =>
								m.settings.testOk((await api.testApiKey(p.id)).length)
							)}>{m.settings.test}</button
					>
					<button
						class="btn sm ghost danger"
						disabled={!hasKey(p.id) || busy === p.id}
						onclick={() =>
							withProvider(p.id, async () => {
								await api.clearApiKey(p.id);
								return m.settings.keyMissing;
							})}>{m.settings.remove}</button
					>
					<button class="btn sm ghost" onclick={() => openUrl(p.key_url)}
						>{m.settings.getKey}</button
					>
				</div>
				{#if status[p.id]}
					<p class="small" class:ok={status[p.id].ok} class:err={!status[p.id].ok}>
						{status[p.id].text}
					</p>
				{/if}
			</div>
		{/each}
	</section>

	<OfflineModels />

	{#if app.settings}
		<section class="card block" aria-labelledby="jobs-settings">
			<h2 id="jobs-settings">{m.settings.jobsTitle}</h2>
			<label class="field narrow">
				<span>{m.settings.concurrency}</span>
				<input
					type="number"
					min="1"
					max="8"
					bind:value={app.settings.concurrency}
					onchange={saveSettings}
				/>
			</label>
			<p class="muted small">{m.settings.concurrencyHint}</p>
		</section>

		<section class="card block" aria-labelledby="catalog-settings">
			<h2 id="catalog-settings">{m.settings.catalogTitle}</h2>
			<p>{m.settings.catalogUpdated(app.catalog?.updated ?? '—')}</p>
			<div class="row">
				<button class="btn sm" onclick={refreshCatalog}>{m.settings.catalogRefresh}</button>
				<label class="inline small">
					<input
						type="checkbox"
						bind:checked={app.settings.refresh_catalog}
						onchange={saveSettings}
					/>
					{m.settings.catalogAuto}
				</label>
			</div>
			{#if app.catalog?.notes}<p class="muted small">{app.catalog.notes}</p>{/if}
		</section>

		<section class="card block" aria-labelledby="look-settings">
			<h2 id="look-settings">{m.settings.appearanceTitle}</h2>
			<label class="field narrow">
				<span>{m.settings.theme}</span>
				<select bind:value={app.settings.theme} onchange={saveSettings}>
					<option value="system">{m.settings.themeSystem}</option>
					<option value="light">{m.settings.themeLight}</option>
					<option value="dark">{m.settings.themeDark}</option>
				</select>
			</label>
		</section>
	{/if}

	<section class="card block" aria-labelledby="about">
		<h2 id="about">{m.settings.aboutTitle}</h2>
		<p>{m.settings.version(app.info?.version ?? '')}</p>
		<p class="small">{m.settings.dataFolder}: <span class="mono">{app.info?.data_dir}</span></p>
		<p class="muted small">{m.settings.privacy}</p>
		<p class="muted small">{m.settings.credits}</p>
	</section>
</div>

<style>
	.page {
		display: grid;
		gap: var(--space-4);
		max-width: 52rem;
	}
	.block {
		padding: var(--space-4) var(--space-5);
		display: grid;
		gap: var(--space-3);
	}
	.small {
		font-size: var(--type-sm);
	}
	.key {
		display: grid;
		grid-template-columns: 12rem 1fr;
		gap: var(--space-2) var(--space-3);
		align-items: center;
		padding: var(--space-3) 0;
		border-top: 1px solid var(--line);
	}
	.key-name {
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.key-actions {
		grid-column: 2;
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.key p {
		grid-column: 2;
	}
	.ok {
		color: var(--ok);
	}
	.err {
		color: var(--danger);
	}
	.narrow {
		max-width: 18rem;
	}
	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
</style>
