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
	import type { JobStatus } from '#lib/bindings/JobStatus.js';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
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

	// The sidebar is the list of jobs, grouped by what they need from the researcher.
	const UNFINISHED: JobStatus[] = ['interrupted', 'paused', 'pending'];
	const groups = $derived(
		[
			{
				key: 'running',
				label: m.shell.running,
				jobs: app.jobs.filter((j) => j.status === 'running')
			},
			{
				key: 'unfinished',
				label: m.shell.unfinished,
				jobs: app.jobs.filter((j) => UNFINISHED.includes(j.status))
			},
			{
				key: 'finished',
				label: m.shell.finished,
				jobs: app.jobs.filter((j) => j.status !== 'running' && !UNFINISHED.includes(j.status))
			}
		].filter((g) => g.jobs.length > 0)
	);

	/** The job the main area shows, whether as its detail or in review. */
	const openJob = $derived(
		app.view.name === 'review'
			? app.view.jobId
			: app.view.name === 'jobs'
				? (app.view.jobId ?? app.jobs[0]?.id)
				: undefined
	);
	const offlineReady = $derived(app.offline?.models.some((s) => s.installed) ?? false);

	const share = (j: JobSummary) => (100 * j.pieces_finished) / Math.max(j.pieces_total, 1);

	// The job list folds to a rail of icons, leaving the room to the work; remembered on this
	// computer, and unfolded when storage is unavailable.
	const SIDEBAR_KEY = 'scriptorium.sidebar.collapsed';
	function storedCollapsed(): boolean {
		try {
			return localStorage.getItem(SIDEBAR_KEY) === '1';
		} catch {
			return false;
		}
	}
	let collapsed = $state(storedCollapsed());
	$effect(() => {
		const v = collapsed ? '1' : '0';
		try {
			localStorage.setItem(SIDEBAR_KEY, v);
		} catch {
			// Folding is only a convenience.
		}
	});

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
	<nav class="side" aria-label={m.shell.jobs}>
		<div class="brand">
			{#if !collapsed}
				<svg class="mark" width="22" height="22" viewBox="0 0 22 22" aria-hidden="true">
					<rect width="22" height="22" rx="5" class="mark-ground" />
					<rect x="5" y="5.2" width="12" height="2" rx="1" class="mark-line" />
					<rect x="4" y="9" width="11" height="4" rx="1" class="mark-marker" />
					<rect x="5" y="14.8" width="9" height="2" rx="1" class="mark-line" />
				</svg>
				<span class="name">{m.app.name}</span>
			{/if}
			<button
				class="fold"
				aria-label={collapsed ? m.shell.expand : m.shell.collapse}
				title="{collapsed ? m.shell.expand : m.shell.collapse} ({m.shell.sidebarShortcut})"
				aria-expanded={!collapsed}
				aria-controls="job-list"
				onclick={() => (collapsed = !collapsed)}
				><Icon name={collapsed ? 'sidebar-open' : 'sidebar-close'} size={16} /></button
			>
		</div>

		<button
			class="new"
			class:current={app.view.name === 'new'}
			aria-current={app.view.name === 'new' ? 'page' : undefined}
			aria-label={collapsed ? m.nav.newJob : undefined}
			title={collapsed ? `${m.nav.newJob} (${m.shell.newJobShortcut})` : undefined}
			onclick={() => go({ name: 'new' })}
		>
			<Icon name="plus" size={16} />
			{#if !collapsed}
				<span class="grow">{m.nav.newJob}</span>
				<span class="kbd">{m.shell.newJobShortcut}</span>
			{/if}
		</button>

		<div class="jobs" id="job-list" hidden={collapsed}>
			{#each groups as group (group.key)}
				<section class="group" aria-labelledby="group-{group.key}">
					<h2 class="group-head" id="group-{group.key}">
						<span>{group.label}</span><span class="num">{group.jobs.length}</span>
					</h2>
					<ul>
						{#each group.jobs as job (job.id)}
							<li>
								<button
									class="job"
									class:current={openJob === job.id}
									aria-current={openJob === job.id ? 'page' : undefined}
									onclick={() => go({ name: 'jobs', jobId: job.id })}
								>
									<span class="job-name">
										<Icon name={job.task} size={14} />
										<span class="ellipsis">{job.name}</span>
									</span>
									<span
										class="job-meta"
										class:warn={UNFINISHED.includes(job.status)}
										class:danger={job.status === 'completed_with_errors'}
									>
										{#if group.key === 'finished'}
											{job.status === 'completed' ? m.task[job.task] : m.status[job.status]}
										{:else}
											{m.status[job.status]} ·
											<span class="num"
												>{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span
											>
										{/if}
									</span>
									{#if group.key !== 'finished'}
										<span class="job-bar" aria-hidden="true"
											><i style:width="{share(job)}%"></i></span
										>
									{/if}
								</button>
							</li>
						{/each}
					</ul>
				</section>
			{:else}
				<p class="no-jobs">{m.shell.noJobs}</p>
			{/each}
		</div>

		<div class="tools">
			<button
				class="tool"
				class:current={app.view.name === 'prompts'}
				aria-current={app.view.name === 'prompts' ? 'page' : undefined}
				aria-label={collapsed ? m.nav.prompts : undefined}
				title={collapsed ? m.nav.prompts : undefined}
				onclick={() => go({ name: 'prompts' })}
				><Icon name="prompts" size={16} />{#if !collapsed}{m.nav.prompts}{/if}</button
			>
			<button
				class="tool"
				class:current={app.view.name === 'settings'}
				aria-current={app.view.name === 'settings' ? 'page' : undefined}
				aria-label={collapsed ? m.nav.settings : undefined}
				title={collapsed ? m.nav.settings : undefined}
				onclick={() => go({ name: 'settings' })}
				><Icon name="settings" size={16} />{#if !collapsed}{m.nav.settings}{/if}</button
			>
			{#if offlineReady && !collapsed}
				<p class="offline"><span class="dot" aria-hidden="true"></span>{m.shell.offlineReady}</p>
			{/if}
		</div>
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
		grid-template-columns: 16rem minmax(0, 1fr);
		height: 100vh;
	}
	.shell.collapsed {
		grid-template-columns: 3.25rem minmax(0, 1fr);
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		min-height: 0;
		padding: var(--space-3) var(--space-3) var(--space-3);
		background: var(--chrome);
		border-right: 1px solid var(--line);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		min-height: 2rem;
		padding: 0 var(--space-2);
	}
	.mark-ground {
		fill: var(--ink);
	}
	.mark-line {
		fill: var(--on-ink);
	}
	.mark-marker {
		fill: var(--marker);
	}
	/* Folds the list: quiet until pointed at, like the other tools of the frame. */
	.fold {
		display: grid;
		place-items: center;
		width: var(--control-sm);
		height: var(--control-sm);
		margin-left: auto;
		padding: 0;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			background-color 120ms ease-out,
			color 120ms ease-out;
	}
	.fold:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.name {
		font-size: 1rem;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-bright);
	}
	.new {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-height: var(--control-md);
		padding: 0 var(--space-3);
		border: 0;
		border-radius: var(--radius-control);
		background: var(--ink);
		color: var(--on-ink);
		font-size: var(--type-sm);
		font-weight: 500;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.new:hover {
		background: var(--ink-hover);
	}
	.new .kbd {
		color: var(--on-ink-muted);
	}
	.new.current {
		box-shadow:
			0 0 0 2px var(--chrome),
			0 0 0 3.5px var(--ink);
	}
	.grow {
		flex: 1;
		text-align: left;
	}
	.jobs {
		flex: 1;
		min-height: 0;
		overflow: auto;
		display: grid;
		align-content: start;
		gap: var(--space-4);
		margin: 0 calc(-1 * var(--space-1));
		padding: 0 var(--space-1);
	}
	.jobs[hidden] {
		display: none;
	}
	.group-head {
		display: flex;
		justify-content: space-between;
		padding: 0 var(--space-2) var(--space-1);
		font-size: var(--type-xs);
		font-weight: 500;
		letter-spacing: 0;
		color: var(--text-muted);
	}
	.group-head .num {
		font-family: var(--font-mono);
		font-size: 0.6875rem;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 2px;
	}
	.job {
		width: 100%;
		display: grid;
		gap: 0.3rem;
		padding: var(--space-2) 0.625rem;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-bright);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.job:hover {
		background: var(--wash);
	}
	.job.current {
		background: var(--surface-0);
		box-shadow: var(--shadow-raised);
	}
	.job-name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
		font-size: var(--type-sm);
		font-weight: 500;
	}
	.job-name :global(.icon) {
		color: var(--text-secondary);
	}
	.ellipsis {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.job-meta {
		padding-left: 1.375rem;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.job-meta.warn {
		color: var(--warn-text);
	}
	.job-meta.danger {
		color: var(--danger-text);
	}
	.job-bar {
		display: block;
		height: 3px;
		margin-left: 1.375rem;
		border-radius: 2px;
		background: var(--line);
		overflow: hidden;
	}
	.job-bar i {
		display: block;
		height: 100%;
		background: var(--ink);
		transition: width 400ms ease-out;
	}
	.no-jobs {
		padding: 0 var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-muted);
	}
	.tools {
		display: grid;
		gap: 2px;
		margin-top: auto;
		padding-top: var(--space-3);
		border-top: 1px solid var(--line);
	}
	/* The rail: the same controls as icons, named by their tooltips. */
	.collapsed .side {
		padding-inline: var(--space-2);
		align-items: center;
	}
	.collapsed .brand {
		padding: 0;
	}
	.collapsed .fold {
		margin: 0;
	}
	.collapsed .new,
	.collapsed .tool {
		width: 2.25rem;
		padding: 0;
		justify-content: center;
	}
	.collapsed .tools {
		align-self: stretch;
		justify-items: center;
	}
	.tool {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		min-height: var(--control-md);
		padding: 0 0.625rem;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-body);
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.tool :global(.icon) {
		color: var(--text-secondary);
	}
	.tool:hover {
		background: var(--wash);
	}
	.tool.current {
		background: var(--surface-0);
		box-shadow: var(--shadow-raised);
		color: var(--text-bright);
		font-weight: 500;
	}
	.offline {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) 0.625rem 0;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.dot {
		width: 6px;
		height: 6px;
		border-radius: 3px;
		background: var(--ok);
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
		.side {
			max-height: 38vh;
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
		.shell.collapsed {
			grid-template-columns: minmax(0, 1fr);
		}
		.collapsed .side {
			flex-direction: row;
			padding-block: var(--space-2);
		}
		.collapsed .tools {
			display: flex;
			margin: 0 0 0 auto;
			padding: 0;
			border: 0;
		}
	}
</style>
