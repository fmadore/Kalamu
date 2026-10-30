<script lang="ts">
	// The sidebar: the app's name, New job, the list of jobs grouped by what they need from the
	// researcher, and the tools at its foot (Prompts, Settings, support). It folds to a rail of
	// icons named by their tooltips, leaving the room to the work.
	import Icon from './Icon.svelte';
	import SupportLink from './SupportLink.svelte';
	import type { JobStatus } from '#lib/bindings/JobStatus.js';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import { messages } from '#lib/i18n/index.js';
	import { app, go } from '#lib/state.svelte.js';

	let {
		collapsed = $bindable()
	}: {
		/** Folded to the rail; the page keeps it, and its shortcut, on this computer. */
		collapsed: boolean;
	} = $props();

	const m = messages();

	// The list of jobs, grouped by what they need from the researcher.
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
</script>

<nav class="side" class:collapsed aria-label={m.shell.jobs}>
	<div class="brand">
		{#if !collapsed}
			<!-- The app icon's small drawing (src-tauri/icons/small.svg): a handwritten sheet
			     behind its transcription, one line under the highlighter. -->
			<svg class="mark" width="22" height="22" viewBox="0 0 22 22" aria-hidden="true">
				<rect width="22" height="22" rx="5" class="mark-ground" />
				<rect x="0.4" y="0.4" width="21.2" height="21.2" rx="4.6" class="mark-rim" />
				<g transform="rotate(-9 8.5 11)">
					<rect x="3.6" y="4.4" width="9.6" height="13" rx="1" class="mark-hand" />
					<rect x="5" y="7" width="5" height="1.3" rx="0.65" class="mark-ink" />
					<rect x="5" y="10" width="6" height="1.3" rx="0.65" class="mark-ink" />
				</g>
				<g transform="rotate(6 13.5 11.5)">
					<rect x="8.8" y="4.6" width="9.6" height="13" rx="1" class="mark-sheet" />
					<rect x="10.4" y="7" width="6.4" height="1.4" rx="0.7" class="mark-line" />
					<rect x="10" y="10" width="7.2" height="2.4" rx="0.8" class="mark-marker" />
					<rect x="10.4" y="14" width="5" height="1.4" rx="0.7" class="mark-line" />
				</g>
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
										<span class="num">{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span
										>
									{/if}
								</span>
								{#if group.key !== 'finished'}
									<span class="job-bar" aria-hidden="true"><i style:width="{share(job)}%"></i></span
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
		<SupportLink variant="tool" iconOnly={collapsed} />
		{#if offlineReady && !collapsed}
			<p class="offline"><span class="dot" aria-hidden="true"></span>{m.shell.offlineReady}</p>
		{/if}
	</div>
</nav>

<style>
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
	/* The app icon's own colours, the same in both themes: it is the icon, not interface. */
	.mark-ground {
		fill: #1e1d1b;
	}
	.mark-rim {
		fill: none;
		stroke: #4d4a43;
		stroke-width: 0.8;
	}
	.mark-hand {
		fill: #efe1c3;
	}
	.mark-ink {
		fill: #8a5a3b;
	}
	.mark-sheet {
		fill: #ffffff;
	}
	.mark-line {
		fill: #9a9ea6;
	}
	.mark-marker {
		fill: #ffd84d;
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
	/* The name is the one place the reading serif signs the interface, at display size. */
	.name {
		font-family: var(--font-text);
		font-size: 1.1875rem;
		font-weight: 600;
		font-variation-settings: 'opsz' 48;
		letter-spacing: -0.015em;
		line-height: 1;
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
	.side.collapsed {
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
	/* A narrow window: the list sits above the work, short enough to leave it room. */
	@media (max-width: 760px) {
		.side {
			max-height: 38vh;
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
		.side.collapsed {
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
