<script lang="ts">
	import { onMount } from 'svelte';
	import CloseDialog from '#lib/components/CloseDialog.svelte';
	import JobsView from '#lib/components/JobsView.svelte';
	import NewJob from '#lib/components/NewJob.svelte';
	import PromptsView from '#lib/components/PromptsView.svelte';
	import ReviewView from '#lib/components/ReviewView.svelte';
	import SettingsView from '#lib/components/SettingsView.svelte';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, on } from '#lib/tauri.js';
	import {
		app,
		applyProgress,
		go,
		refreshJobs,
		refreshKeys,
		type View
	} from '#lib/state.svelte.js';

	const m = messages();
	let loadError = $state<string | null>(null);

	const nav: { name: View['name']; label: string }[] = [
		{ name: 'new', label: m.nav.newJob },
		{ name: 'jobs', label: m.nav.jobs },
		{ name: 'prompts', label: m.nav.prompts },
		{ name: 'settings', label: m.nav.settings }
	];

	function applyTheme(theme: string) {
		if (theme === 'light' || theme === 'dark') {
			document.documentElement.setAttribute('data-theme', theme);
		} else {
			document.documentElement.removeAttribute('data-theme');
		}
	}

	$effect(() => {
		if (app.settings) applyTheme(app.settings.theme);
	});

	const unfinished = $derived(
		app.jobs.filter((j) => ['interrupted', 'paused', 'pending'].includes(j.status)).length
	);

	onMount(() => {
		const unlisten: (() => void)[] = [];
		void (async () => {
			// Sample data for looking at the interface in a browser; development builds only.
			if (
				import.meta.env.DEV &&
				location.search.includes('mock') &&
				!('__TAURI_INTERNALS__' in window)
			) {
				(await import('#lib/dev/mock.js')).installMock();
			}
			try {
				const [info, catalog, settings] = await Promise.all([
					api.appInfo(),
					api.getCatalog(),
					api.getSettings()
				]);
				app.info = info;
				app.catalog = catalog;
				app.settings = settings;
				await Promise.all([refreshKeys(), refreshJobs()]);
				// Jobs cut short by the last close: show them first.
				if (info.interrupted.length > 0) go({ name: 'jobs' });
				else if (!app.keys.some((k) => k.saved)) go({ name: 'settings' });
			} catch (e) {
				loadError = describeError(e, m);
			}
			unlisten.push(await on.progress(applyProgress));
			unlisten.push(await on.closeRequested(() => (app.closeRequested = true)));
		})();
		return () => unlisten.forEach((u) => u());
	});
</script>

<div class="shell">
	<nav class="side" aria-label="Main">
		<div class="brand">
			<img src="/icon.png" alt="" width="28" height="28" />
			<div>
				<strong>{m.app.name}</strong>
				<span class="muted">{m.app.tagline}</span>
			</div>
		</div>
		<ul>
			{#each nav as item (item.name)}
				<li>
					<button
						class="nav-item"
						class:active={app.view.name === item.name ||
							(item.name === 'jobs' && app.view.name === 'review')}
						aria-current={app.view.name === item.name ? 'page' : undefined}
						onclick={() => go({ name: item.name } as View)}
					>
						{item.label}
						{#if item.name === 'jobs' && unfinished > 0}
							<span class="chip warn">{unfinished}</span>
						{/if}
					</button>
				</li>
			{/each}
		</ul>
	</nav>

	<main>
		{#if loadError}
			<p class="chip danger">{loadError}</p>
		{:else if !app.catalog || !app.settings}
			<p class="muted">{m.common.loading}</p>
		{:else if app.view.name === 'new'}
			<NewJob />
		{:else if app.view.name === 'jobs'}
			<JobsView jobId={app.view.jobId} />
		{:else if app.view.name === 'review'}
			<ReviewView jobId={app.view.jobId} sourceId={app.view.sourceId} />
		{:else if app.view.name === 'prompts'}
			<PromptsView />
		{:else if app.view.name === 'settings'}
			<SettingsView />
		{/if}
	</main>
</div>

{#if app.closeRequested}
	<CloseDialog />
{/if}

{#if app.toast}
	<div class="toast card" role="status">{app.toast}</div>
{/if}

<style>
	.shell {
		display: grid;
		grid-template-columns: 14rem 1fr;
		height: 100vh;
	}
	.side {
		background: var(--surface-1);
		border-right: 1px solid var(--line);
		padding: var(--space-4) var(--space-3);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.brand {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		padding: 0 var(--space-2);
	}
	.brand div {
		display: grid;
		line-height: 1.2;
	}
	.brand span {
		font-size: var(--type-xs);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-1);
	}
	.nav-item {
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
		text-align: left;
		padding: var(--space-2) var(--space-3);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		cursor: pointer;
		color: var(--text-secondary);
		font-weight: 500;
	}
	.nav-item:hover {
		background: var(--accent-wash);
	}
	.nav-item.active {
		background: var(--accent-chip);
		color: var(--text-bright);
	}
	main {
		overflow: auto;
		padding: var(--space-5) var(--space-6);
	}
	.toast {
		position: fixed;
		bottom: var(--space-5);
		right: var(--space-5);
		padding: var(--space-3) var(--space-4);
		background: var(--surface-2);
		box-shadow: var(--shadow-flyout);
		z-index: 40;
	}
</style>
