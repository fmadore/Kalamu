<script lang="ts">
	import { onMount } from 'svelte';
	import AnalysisReview from '#lib/components/AnalysisReview.svelte';
	import ContentsReview from '#lib/components/ContentsReview.svelte';
	import CloseDialog from '#lib/components/CloseDialog.svelte';
	import JobsView from '#lib/components/JobsView.svelte';
	import NewJob from '#lib/components/NewJob.svelte';
	import PromptsView from '#lib/components/PromptsView.svelte';
	import ReviewView from '#lib/components/ReviewView.svelte';
	import SettingsView from '#lib/components/SettingsView.svelte';
	import Sidebar from '#lib/components/Sidebar.svelte';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, on } from '#lib/tauri.js';
	import {
		app,
		applyProgress,
		go,
		refreshJobs,
		refreshKeys,
		refreshOffline,
		type View
	} from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';

	const m = messages();
	let loadError = $state<string | null>(null);
	let startView: View | null = null;

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

	// The job list folds to a rail of icons, leaving the room to the work; remembered on this
	// computer, and unfolded when storage is unavailable.
	const SIDEBAR_KEY = 'kalamu.sidebar.collapsed';
	let collapsed = $state(recall(SIDEBAR_KEY, (v) => v === 1, false));
	$effect(() => keep(SIDEBAR_KEY, collapsed ? 1 : 0));

	function onKey(e: KeyboardEvent) {
		if (!e.ctrlKey || e.altKey || e.shiftKey) return;
		const key = e.key.toLowerCase();
		if (key === 'n') {
			e.preventDefault();
			go({ name: 'new' });
		} else if (key === 'b') {
			e.preventDefault();
			collapsed = !collapsed;
		}
	}

	onMount(() => {
		const unlisten: (() => void)[] = [];
		void (async () => {
			// Sample data for looking at the interface in a browser; development builds only.
			if (
				import.meta.env.DEV &&
				location.search.includes('mock') &&
				!('__TAURI_INTERNALS__' in window)
			) {
				const mock = await import('#lib/dev/mock.js');
				mock.installMock();
				startView = mock.mockView();
			}
			try {
				const [info, catalog, rules, settings] = await Promise.all([
					api.appInfo(),
					api.getCatalog(),
					api.taskRules(),
					api.getSettings()
				]);
				app.info = info;
				app.catalog = catalog;
				app.rules = rules;
				app.settings = settings;
				await Promise.all([refreshKeys(), refreshJobs(), refreshOffline()]);
				// Jobs cut short by the last close: show them first.
				if (startView) go(startView);
				else if (info.interrupted.length > 0) go({ name: 'jobs', jobId: info.interrupted[0] });
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

<svelte:window onkeydown={onKey} />

<div class="shell" class:collapsed>
	<Sidebar bind:collapsed />

	<main>
		{#if loadError}
			<div class="sheet"><p class="chip danger">{loadError}</p></div>
		{:else if !app.catalog || !app.settings}
			<div class="sheet"><p class="muted">{m.common.loading}</p></div>
		{:else if app.view.name === 'new'}
			<NewJob opening={app.view} />
		{:else if app.view.name === 'jobs'}
			<JobsView jobId={app.view.jobId} />
		{:else if app.view.name === 'review'}
			{@const { jobId, sourceId } = app.view}
			{@const task = app.jobs.find((j) => j.id === jobId)?.task}
			{#if task === 'analysis'}
				<AnalysisReview {jobId} {sourceId} />
			{:else if task === 'contents'}
				<ContentsReview {jobId} {sourceId} />
			{:else}
				<ReviewView {jobId} {sourceId} />
			{/if}
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
	<div class="toast" role="status">{app.toast}</div>
{/if}

<style>
	.shell {
		display: grid;
		grid-template-columns: 16rem minmax(0, 1fr);
		height: 100vh;
	}
	.shell.collapsed {
		grid-template-columns: 3.25rem minmax(0, 1fr);
	}
	/* Positioned, so what is placed absolutely inside a screen (the links' screen-reader text)
	   scrolls with it instead of stretching the window below the shell. */
	main {
		position: relative;
		overflow: auto;
		min-width: 0;
		background: var(--surface-0);
	}
	.toast {
		position: fixed;
		bottom: var(--space-5);
		right: var(--space-5);
		max-width: 28rem;
		padding: var(--space-3) var(--space-4);
		background: var(--ink);
		color: var(--on-ink);
		border-radius: var(--radius-control);
		box-shadow: var(--shadow-flyout);
		font-size: var(--type-sm);
		z-index: 40;
	}
	/* A narrow window: the list sits above the work, short enough to leave it room. */
	@media (max-width: 760px) {
		.shell {
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: auto minmax(0, 1fr);
		}
		.shell.collapsed {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
