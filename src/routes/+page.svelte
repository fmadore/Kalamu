<script lang="ts">
	import { onMount } from 'svelte';
	import AnalysisReview from '#lib/components/AnalysisReview.svelte';
	import CloseDialog from '#lib/components/CloseDialog.svelte';
	import Icon from '#lib/components/Icon.svelte';
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
		refreshOffline,
		type View
	} from '#lib/state.svelte.js';

	const m = messages();
	let loadError = $state<string | null>(null);
	let startView: View | null = null;

	const nav: { name: 'new' | 'jobs' | 'prompts' | 'settings'; label: string }[] = [
		{ name: 'new', label: m.nav.newJob },
		{ name: 'jobs', label: m.nav.jobs },
		{ name: 'prompts', label: m.nav.prompts },
		{ name: 'settings', label: m.nav.settings }
	];

	// The rail can fold down to its icons; the choice is this computer's, kept between sessions.
	const RAIL_KEY = 'scriptorium.rail';
	let folded = $state(readFolded());

	function readFolded(): boolean {
		try {
			return localStorage.getItem(RAIL_KEY) === 'folded';
		} catch {
			return false;
		}
	}

	function toggleRail() {
		folded = !folded;
		try {
			localStorage.setItem(RAIL_KEY, folded ? 'folded' : 'open');
		} catch {
			// Storage refused: the rail still folds for this session.
		}
	}

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

	const running = $derived(app.jobs.filter((j) => j.status === 'running'));
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
				const mock = await import('#lib/dev/mock.js');
				mock.installMock();
				startView = mock.mockView();
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
				await Promise.all([refreshKeys(), refreshJobs(), refreshOffline()]);
				// Jobs cut short by the last close: show them first.
				if (startView) go(startView);
				else if (info.interrupted.length > 0) go({ name: 'jobs' });
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

<div class="shell" class:folded>
	<nav class="rail" aria-label="Main">
		<div class="wordmark" aria-label={m.app.name}>
			<span class="full">{m.app.name}</span><span class="initial" aria-hidden="true"
				>{m.app.name.charAt(0)}</span
			>
		</div>
		<ul>
			{#each nav as item (item.name)}
				<li>
					<button
						class="nav-item"
						class:active={app.view.name === item.name ||
							(item.name === 'jobs' && app.view.name === 'review')}
						aria-current={app.view.name === item.name ? 'page' : undefined}
						aria-label={folded ? item.label : undefined}
						title={folded ? item.label : undefined}
						onclick={() => go({ name: item.name } as View)}
					>
						<span class="nav-icon"><Icon name={item.name} size={18} /></span>
						<span class="nav-label">{item.label}</span>
						{#if item.name === 'jobs' && unfinished > 0}
							<span class="count" aria-label={m.shell.unfinished(unfinished)}>{unfinished}</span>
						{/if}
					</button>
				</li>
			{/each}
		</ul>
		{#if running.length > 0}
			<div class="running" role="status">
				{#each running.slice(0, 2) as job (job.id)}
					<button class="run" onclick={() => go({ name: 'jobs', jobId: job.id })}>
						<span class="run-name">{m.shell.running(job.name)}</span>
						<span class="run-bar" aria-hidden="true"
							><i style:width="{(100 * job.pieces_finished) / Math.max(job.pieces_total, 1)}%"
							></i></span
						>
						<span class="run-count num"
							>{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span
						>
					</button>
				{/each}
			</div>
		{/if}
		<button
			class="fold"
			aria-label={folded ? m.shell.expand : m.shell.collapse}
			title={folded ? m.shell.expand : m.shell.collapse}
			aria-expanded={!folded}
			onclick={toggleRail}
		>
			<Icon name={folded ? 'expand' : 'collapse'} size={18} />
			<span class="nav-label">{m.shell.collapse}</span>
		</button>
	</nav>

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
			{#if app.jobs.find((j) => j.id === jobId)?.task === 'analysis'}
				<AnalysisReview {jobId} {sourceId} />
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
		grid-template-columns: 14.5rem 1fr;
		height: 100vh;
	}
	.shell.folded {
		grid-template-columns: 3.75rem 1fr;
	}
	.rail {
		background: var(--shell);
		color: var(--on-shell);
		display: flex;
		flex-direction: column;
		border-right: 1px solid var(--shell-line);
		min-height: 0;
	}
	/* As tall as a masthead, so its rule continues the header's bottom edge. */
	.wordmark {
		display: flex;
		align-items: flex-end;
		white-space: nowrap;
		overflow: hidden;
		height: var(--masthead-height);
		flex: none;
		font-family: var(--font-text);
		font-size: 1.875rem;
		font-weight: 500;
		letter-spacing: 0.01em;
		line-height: 1;
		padding: 0 var(--space-5) var(--space-4);
		border-bottom: 1px solid var(--shell-line);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: var(--space-4) var(--space-3);
		display: grid;
		gap: 2px;
	}
	.nav-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-height: 2.5rem;
		padding: 0 var(--space-3);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--on-shell-muted);
		font-size: 1rem;
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.nav-item:hover {
		background: var(--shell-2);
		color: var(--on-shell);
	}
	.nav-item.active {
		background: var(--shell-2);
		color: var(--on-shell);
		font-weight: 600;
	}
	.nav-item:focus-visible,
	.fold:focus-visible {
		outline-color: var(--on-shell);
	}
	/* Words lead in the open rail; icons stand in for them only when it is folded. */
	.nav-icon,
	.initial {
		display: none;
	}
	.folded .nav-icon,
	.folded .initial {
		display: inline-flex;
	}
	.folded .nav-label,
	.folded .full,
	.folded .running {
		display: none;
	}
	.folded .wordmark {
		justify-content: center;
		padding-left: 0;
		padding-right: 0;
	}
	.folded ul {
		padding: var(--space-4) var(--space-2);
	}
	.folded .nav-item {
		justify-content: center;
		padding: 0;
		position: relative;
	}
	.folded .count {
		position: absolute;
		top: 2px;
		right: 2px;
		min-width: 1.125rem;
		line-height: 1.125rem;
	}
	.fold {
		margin-top: auto;
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-height: 2.5rem;
		margin-inline: var(--space-3);
		margin-bottom: var(--space-3);
		padding: 0 var(--space-3);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--on-shell-muted);
		font-size: var(--type-sm);
		cursor: pointer;
	}
	.fold:hover {
		background: var(--shell-2);
		color: var(--on-shell);
	}
	.running + .fold {
		margin-top: 0;
	}
	.folded .fold {
		justify-content: center;
		margin-inline: var(--space-2);
		padding: 0;
	}
	.count {
		margin-left: auto;
		min-width: 1.375rem;
		padding: 0 var(--space-1);
		border-radius: var(--radius-pill);
		background: var(--on-shell);
		color: var(--shell);
		font-size: var(--type-xs);
		font-weight: 700;
		text-align: center;
		line-height: 1.25rem;
	}
	.running {
		margin-top: auto;
		border-top: 1px solid var(--shell-line);
		padding: var(--space-3) var(--space-3) var(--space-4);
		display: grid;
		gap: var(--space-2);
	}
	.run {
		display: grid;
		gap: var(--space-1);
		padding: var(--space-2) var(--space-3);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--on-shell-muted);
		text-align: left;
		cursor: pointer;
		font-size: var(--type-sm);
	}
	.run:hover {
		background: var(--shell-2);
	}
	.run-name {
		color: var(--on-shell);
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.run-bar {
		height: 4px;
		border-radius: 2px;
		background: var(--shell-line);
		overflow: hidden;
	}
	.run-bar i {
		display: block;
		height: 100%;
		background: var(--on-shell);
		transition: width 400ms ease-out;
	}
	main {
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
		background: var(--shell);
		color: var(--on-shell);
		border-radius: var(--radius-control);
		box-shadow: var(--shadow-flyout);
		z-index: 40;
	}
	@media (max-width: 820px) {
		.shell {
			grid-template-columns: 1fr;
			grid-template-rows: auto 1fr;
		}
		.rail {
			flex-direction: row;
			align-items: center;
			flex-wrap: wrap;
			border-right: 0;
			border-bottom: 1px solid var(--shell-line);
		}
		.wordmark {
			height: auto;
			font-size: 1.5rem;
			padding: var(--space-3) var(--space-4);
			border-bottom: 0;
		}
		ul {
			display: flex;
			flex-wrap: wrap;
			padding: var(--space-2);
		}
		.nav-item {
			min-height: 2.25rem;
			font-size: var(--type-md);
		}
		.running,
		.fold,
		.nav-icon,
		.initial {
			display: none !important;
		}
		.shell.folded {
			grid-template-columns: 1fr;
		}
		.folded .nav-label,
		.folded .full {
			display: inline;
		}
	}
</style>
