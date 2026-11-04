<script lang="ts">
	// Settings in four tabs: provider keys, offline engines, how jobs run and look, and about.
	// The tab is part of the view, so a link from elsewhere ("Download in Settings") opens the
	// section it is about.
	import { AUTHOR_URL, CREDIT_URLS } from '#lib/credits.js';
	import { describeError } from '#lib/errors.js';
	import { formatDay, localeNames, messages, placeholders } from '#lib/i18n/index.js';
	import { osNames } from '#lib/platform.js';
	import { api } from '#lib/tauri.js';
	import {
		app,
		applyLanguage,
		go,
		hasKey,
		openLink,
		refreshKeys,
		toast,
		type SettingsTab
	} from '#lib/state.svelte.js';
	import ExternalLink from './ExternalLink.svelte';
	import OfflineModels from './OfflineModels.svelte';
	import ReportLink from './ReportLink.svelte';
	import SupportLink from './SupportLink.svelte';

	const m = messages();

	const TABS: SettingsTab[] = ['keys', 'offline', 'general', 'about'];
	const tab = $derived<SettingsTab>((app.view.name === 'settings' && app.view.tab) || 'keys');

	function choose(next: SettingsTab) {
		go({ name: 'settings', tab: next });
	}

	/** Arrow keys, Home and End move between tabs, as for any tab list. */
	function onTabKey(event: KeyboardEvent) {
		const i = TABS.indexOf(tab);
		const next =
			event.key === 'ArrowRight'
				? TABS[(i + 1) % TABS.length]
				: event.key === 'ArrowLeft'
					? TABS[(i + TABS.length - 1) % TABS.length]
					: event.key === 'Home'
						? TABS[0]
						: event.key === 'End'
							? TABS[TABS.length - 1]
							: null;
		if (!next) return;
		event.preventDefault();
		choose(next);
		document.getElementById(`settings-tab-${next}`)?.focus();
	}

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

	/** Saved first, then applied: this screen is drawn again in the new language. */
	async function changeLanguage() {
		await saveSettings();
		applyLanguage();
	}

	async function refreshCatalog() {
		try {
			app.catalog = await api.refreshCatalog();
			// A model added or retired changes which models suit each task.
			app.rules = await api.taskRules();
			toast(m.settings.catalogUpdated(formatDay(app.catalog.updated)));
		} catch (e) {
			toast(describeError(e, m));
		}
	}
</script>

<header class="masthead">
	<div>
		<h1>{m.settings.title}</h1>
		<p class="lede">{m.settings.lede}</p>
	</div>
</header>

<div class="sheet settings">
	<div
		class="tabs"
		role="tablist"
		aria-label={m.settings.tabsLabel}
		tabindex="-1"
		onkeydown={onTabKey}
	>
		{#each TABS as t (t)}
			<button
				id="settings-tab-{t}"
				class="tab"
				role="tab"
				aria-selected={tab === t}
				aria-controls="settings-panel"
				tabindex={tab === t ? 0 : -1}
				onclick={() => choose(t)}>{m.settings.tabs[t]}</button
			>
		{/each}
	</div>

	<div id="settings-panel" role="tabpanel" aria-labelledby="settings-tab-{tab}" class="panel">
		{#if tab === 'keys'}
			<section class="card block" aria-labelledby="keys-title">
				<h2 id="keys-title">{m.settings.keysTitle}</h2>
				<p class="muted small">{m.settings.keysHint(osNames().keychain)}</p>
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
							<button class="btn sm ghost" onclick={() => openLink(p.key_url)}
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
		{:else if tab === 'offline'}
			<OfflineModels />
		{:else if tab === 'general' && app.settings}
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
				<p>
					{m.settings.catalogUpdated(
						app.catalog ? formatDay(app.catalog.updated) : m.common.unknown.toLowerCase()
					)}
				</p>
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
				<label class="field narrow">
					<span>{m.settings.language}</span>
					<select bind:value={app.settings.language} onchange={changeLanguage}>
						<option value="system">{m.settings.languageSystem}</option>
						{#each localeNames() as { locale, name } (locale)}
							<option value={locale} lang={locale}>{name}</option>
						{/each}
					</select>
				</label>
				<p class="muted small">{m.settings.languageHint}</p>
			</section>
		{:else if tab === 'about'}
			<section class="card block" aria-labelledby="about">
				<h2 id="about">{m.settings.aboutTitle}</h2>
				<p>{m.settings.version(app.info?.version ?? '')}</p>
				<p class="small">
					{m.settings.dataFolder}{m.common.colon} <span class="path">{app.info?.data_dir}</span>
				</p>
				<p class="muted small">{m.settings.privacy}</p>
				<p class="muted small">
					{#each placeholders(m.settings.credits) as part, i (i)}{#if 'name' in part}<ExternalLink
								href={CREDIT_URLS[part.name] ?? ''}
								>{m.settings.creditNames[part.name as keyof typeof m.settings.creditNames] ??
									part.name}</ExternalLink
							>{:else}{part.text}{/if}{/each}
				</p>
				<p class="muted small">{m.report.note}</p>
				<div><ReportLink /></div>
			</section>

			<!-- Last, so it is there for anyone who looks and in the way of nobody. -->
			<section class="card block" aria-labelledby="support">
				<h2 id="support">{m.support.title}</h2>
				<p class="muted small">
					{#each placeholders(m.support.note) as part, i (i)}{#if 'name' in part}<ExternalLink
								href={AUTHOR_URL}>Frédérick Madore</ExternalLink
							>{:else}{part.text}{/if}{/each}
				</p>
				<SupportLink />
			</section>
		{/if}
	</div>
</div>

<style>
	/* Wrapping rather than scrolling in a narrow window. */
	.tabs {
		display: flex;
		flex-wrap: wrap;
		gap: 0 var(--space-1);
		border-bottom: 1px solid var(--line);
	}
	.tab {
		padding: var(--space-2) var(--space-3);
		border: 0;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		background: none;
		color: var(--text-muted);
		font: inherit;
		font-size: var(--type-sm);
		white-space: nowrap;
		cursor: pointer;
	}
	.tab:hover {
		color: var(--text-bright);
	}
	.tab[aria-selected='true'] {
		color: var(--text-bright);
		border-bottom-color: var(--ink);
		font-weight: 600;
	}
	.tab:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: -2px;
	}
	.panel {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
	}
	.block {
		padding: var(--space-4) 0 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-3);
	}
	.small {
		font-size: var(--type-sm);
	}
	.path {
		overflow-wrap: anywhere;
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
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
	}
	/* A long button label wraps in a narrow window. */
	.row .btn {
		white-space: normal;
		text-align: left;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
	@media (max-width: 700px) {
		.key {
			grid-template-columns: 1fr;
		}
		.key-actions,
		.key p {
			grid-column: 1;
		}
	}
</style>
