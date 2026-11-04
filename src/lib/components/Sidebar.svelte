<script lang="ts">
	// The sidebar: the app's name, New job, Projects with the projects listed under it, the list of
	// jobs grouped by what they need from the researcher, and the tools at its foot (Prompts,
	// Settings, Glossary, support). Finished jobs are the most recent few, then a link to the Jobs
	// page, which lists them all; what runs, stopped early or waits for the researcher is always
	// listed in full, and so is the job open in the main area. A finished job dragged onto a listed project is added to it,
	// as its menu's Add to a project does. It folds to a rail of icons named by their tooltips,
	// leaving the room to the work.
	import Icon from './Icon.svelte';
	import JobMenu from './JobMenu.svelte';
	import SupportLink from './SupportLink.svelte';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import { messages } from '#lib/i18n/index.js';
	import { shortcut } from '#lib/platform.js';
	import { describeError } from '#lib/errors.js';
	import { UNFINISHED, finishedShown, groupOf, type GroupKey } from '#lib/jobList.js';
	import { app, go, refreshJobs, refreshProjects, toast } from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';
	import { api } from '#lib/tauri.js';

	let {
		collapsed = $bindable()
	}: {
		/** Folded to the rail; the page keeps it, and its shortcut, on this computer. */
		collapsed: boolean;
	} = $props();

	const m = messages();

	/** The job whose menu is open, and where. */
	let menu = $state<{ job: JobSummary; x: number; y: number } | null>(null);
	/** The job being renamed in the list, and the name typed so far. */
	let renaming = $state<{ id: string; name: string } | null>(null);

	function openMenu(e: MouseEvent, job: JobSummary) {
		e.preventDefault();
		// From the keyboard's menu key there is no pointer: open under the job.
		const at = (e.currentTarget as HTMLElement).getBoundingClientRect();
		const keyboard = e.button !== 2 && e.clientX === 0 && e.clientY === 0;
		menu = { job, x: keyboard ? at.left + 12 : e.clientX, y: keyboard ? at.bottom : e.clientY };
	}

	async function rename() {
		const r = renaming;
		renaming = null;
		const name = r?.name.trim();
		const job = app.jobs.find((j) => j.id === r?.id);
		if (!r || !name || !job || name === job.name) return;
		try {
			await api.renameJob(r.id, name);
			await refreshJobs();
		} catch (e) {
			toast(describeError(e, m));
		}
	}

	function renameKeys(e: KeyboardEvent) {
		if (e.key === 'Enter') void rename();
		else if (e.key === 'Escape') renaming = null;
	}

	/** Puts the caret in the name box as it opens, the name selected. */
	function chosen(input: HTMLInputElement) {
		input.focus();
		input.select();
	}

	/** The job the main area shows, whether as its detail or in review. */
	const openJob = $derived(
		app.view.name === 'review'
			? app.view.jobId
			: app.view.name === 'jobs'
				? (app.view.jobId ?? app.jobs[0]?.id)
				: undefined
	);

	// The list of jobs, grouped by what they need from the researcher. Workflow steps that wait
	// for the researcher come first: nothing moves until they act.
	const labels: Record<GroupKey, string> = {
		waiting: m.shell.toCheck,
		running: m.shell.running,
		unfinished: m.shell.unfinished,
		finished: m.shell.finished
	};
	const groups = $derived.by(() => {
		const by: Record<GroupKey, JobSummary[]> = {
			waiting: [],
			running: [],
			unfinished: [],
			finished: []
		};
		for (const job of app.jobs) by[groupOf(job, app.waiting)].push(job);
		return (Object.keys(by) as GroupKey[])
			.filter((key) => by[key].length > 0)
			.map((key) => ({
				key,
				label: labels[key],
				count: by[key].length,
				jobs: key === 'finished' ? finishedShown(by[key], openJob) : by[key]
			}));
	});
	const allJobsOpen = $derived(app.view.name === 'allJobs');
	const offlineReady = $derived(app.offline?.models.some((s) => s.installed) ?? false);

	const share = (j: JobSummary) => (100 * j.pieces_finished) / Math.max(j.pieces_total, 1);

	// The projects under Projects, unfolded unless the researcher folded them on this computer;
	// read again whenever the screen changes, so one created, renamed or deleted shows.
	const PROJECTS_KEY = 'kalamu.sidebar.projects';
	let showProjects = $state(recall(PROJECTS_KEY, (v) => v === 1, true));
	$effect(() => keep(PROJECTS_KEY, showProjects ? 1 : 0));
	$effect(() => {
		void app.view;
		void refreshProjects();
	});
	const listed = $derived(app.projects.filter((p) => !p.project.archived));
	const openProject = $derived(app.view.name === 'projects' ? app.view.projectId : undefined);

	/* Dragging a finished job onto a project. Done with the pointer rather than the browser's own
	   drag and drop, which the window gives over to files dropped from File Explorer or Finder. */
	let press: { job: JobSummary; x: number; y: number } | null = null;
	let dragging = $state<{ job: JobSummary; x: number; y: number } | null>(null);
	let dropOn = $state<string | null>(null);
	/** Set when a drag ends, so the click that follows it does not open the job. */
	let dragged = false;

	function pressJob(e: PointerEvent, job: JobSummary) {
		if (e.button !== 0 || e.pointerType === 'touch') return;
		press = { job, x: e.clientX, y: e.clientY };
	}

	function moveJob(e: PointerEvent) {
		if (!press) return;
		if (!dragging) {
			if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < 6) return;
			showProjects = true;
		}
		dragging = { job: press.job, x: e.clientX, y: e.clientY };
		const under = document.elementFromPoint(e.clientX, e.clientY);
		dropOn = under?.closest<HTMLElement>('[data-project]')?.dataset.project ?? null;
	}

	/** Ends a drag; the click the browser sends after it is swallowed. */
	function endDrag() {
		const was = dragging !== null;
		press = null;
		dragging = null;
		dropOn = null;
		if (!was) return;
		dragged = true;
		setTimeout(() => (dragged = false));
	}

	async function dropJob() {
		const drag = dragging;
		const target = app.projects.find((p) => p.project.id === dropOn);
		endDrag();
		if (!drag || !target) return;
		try {
			await api.linkJob(target.project.id, drag.job.id, null);
			toast(m.shell.addedTo(target.project.name));
			await refreshProjects();
		} catch (e) {
			toast(describeError(e, m));
		}
	}
</script>

<svelte:window
	onpointermove={moveJob}
	onpointerup={() => void dropJob()}
	onpointercancel={endDrag}
	onkeydown={(e) => {
		if (e.key === 'Escape' && dragging) endDrag();
	}}
/>

{#if dragging}
	<div class="drag-ghost" style:left="{dragging.x + 12}px" style:top="{dragging.y + 8}px">
		<Icon name={dragging.job.task} size={14} /><span class="ellipsis">{dragging.job.name}</span>
	</div>
{/if}

{#if menu}
	{@const at = menu}
	<JobMenu
		job={at.job}
		x={at.x}
		y={at.y}
		onrename={() => (renaming = { id: at.job.id, name: at.job.name })}
		onclose={() => (menu = null)}
	/>
{/if}

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
			title="{collapsed ? m.shell.expand : m.shell.collapse} ({shortcut(m.shell.sidebarShortcut)})"
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
		title={collapsed ? `${m.nav.newJob} (${shortcut(m.shell.newJobShortcut)})` : undefined}
		onclick={() => go({ name: 'new' })}
	>
		<Icon name="plus" size={16} />
		{#if !collapsed}
			<span class="grow">{m.nav.newJob}</span>
			<span class="kbd">{shortcut(m.shell.newJobShortcut)}</span>
		{/if}
	</button>

	<div class="nav-group" class:dropping={dragging !== null}>
		<div class="tool-row">
			<button
				class="tool projects"
				class:current={app.view.name === 'projects' && !openProject}
				aria-current={app.view.name === 'projects' && !openProject ? 'page' : undefined}
				aria-label={collapsed ? m.nav.projects : undefined}
				title={collapsed ? m.nav.projects : undefined}
				onclick={() => go({ name: 'projects' })}
				><Icon name="project" size={16} />{#if !collapsed}{m.nav.projects}{/if}</button
			>
			{#if !collapsed && listed.length > 0}
				<button
					class="fold unfold"
					aria-label={showProjects ? m.shell.hideProjects : m.shell.showProjects}
					title={showProjects ? m.shell.hideProjects : m.shell.showProjects}
					aria-expanded={showProjects}
					aria-controls="project-list"
					onclick={() => (showProjects = !showProjects)}
					><span class="turn" class:open={showProjects}><Icon name="chevron" size={14} /></span
					></button
				>
			{/if}
		</div>
		{#if !collapsed && showProjects && listed.length > 0}
			<ul class="projects-list" id="project-list" aria-label={m.nav.projects}>
				{#each listed as p (p.project.id)}
					<li>
						<button
							class="project"
							class:current={openProject === p.project.id}
							class:drop={dropOn === p.project.id}
							aria-current={openProject === p.project.id ? 'page' : undefined}
							data-project={p.project.id}
							onclick={() => go({ name: 'projects', projectId: p.project.id })}
							><span class="ellipsis">{p.project.name}</span><span class="num">{p.documents}</span
							></button
						>
					</li>
				{/each}
			</ul>
			{#if dragging}<p class="drop-hint">{m.shell.dropOnProject}</p>{/if}
		{/if}
	</div>
	<button
		class="tool workflows"
		class:current={app.view.name === 'workflows'}
		aria-current={app.view.name === 'workflows' ? 'page' : undefined}
		aria-label={collapsed ? m.nav.workflows : undefined}
		title={collapsed ? m.nav.workflows : undefined}
		onclick={() => go({ name: 'workflows' })}
		><Icon name="workflow" size={16} />{#if !collapsed}{m.nav.workflows}{/if}</button
	>
	{#if collapsed}
		<!-- The rail hides the job list; the Jobs page stays one click away. -->
		<button
			class="tool"
			class:current={allJobsOpen}
			aria-current={allJobsOpen ? 'page' : undefined}
			aria-label={m.allJobs.title}
			title={m.allJobs.title}
			onclick={() => go({ name: 'allJobs' })}><Icon name="list" size={16} /></button
		>
	{/if}

	<div class="jobs" id="job-list" hidden={collapsed}>
		{#each groups as group (group.key)}
			<section class="group" aria-labelledby="group-{group.key}">
				<h2 class="group-head" id="group-{group.key}">
					<span>{group.label}</span><span class="num">{group.count}</span>
				</h2>
				<ul>
					{#each group.jobs as job (job.id)}
						<li>
							{#if renaming?.id === job.id}
								<input
									class="rename"
									aria-label={m.jobs.nameLabel}
									bind:value={renaming.name}
									onkeydown={renameKeys}
									onblur={rename}
									{@attach chosen}
								/>
							{:else}
								<button
									class="job"
									class:current={openJob === job.id}
									class:lifted={dragging?.job.id === job.id}
									aria-current={openJob === job.id ? 'page' : undefined}
									title={group.key === 'finished' && listed.length > 0
										? m.shell.dragHint
										: undefined}
									onpointerdown={group.key === 'finished' ? (e) => pressJob(e, job) : undefined}
									onclick={() => {
										if (!dragged) go({ name: 'jobs', jobId: job.id });
									}}
									oncontextmenu={(e) => openMenu(e, job)}
								>
									<span class="job-name">
										<Icon name={job.task} size={14} />
										<span class="ellipsis">{job.name}</span>
									</span>
									<span
										class="job-meta"
										class:warn={UNFINISHED.includes(job.status) || group.key === 'waiting'}
										class:danger={job.status === 'completed_with_errors' && group.key !== 'waiting'}
									>
										{#if group.key === 'waiting'}
											{m.workflows.stepState[app.waiting[job.id]?.state ?? 'review']}
										{:else if group.key === 'finished'}
											{job.status === 'completed' ? m.task[job.task] : m.status[job.status]}
										{:else}
											{m.status[job.status]} ·
											<span class="num"
												>{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span
											>
										{/if}
									</span>
									{#if group.key !== 'finished' && group.key !== 'waiting'}
										<span class="job-bar" aria-hidden="true"
											><i style:width="{share(job)}%"></i></span
										>
									{/if}
								</button>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{:else}
			<p class="no-jobs">{m.shell.noJobs}</p>
		{/each}
		{#if app.jobs.length > 0}
			<button
				class="all-jobs"
				class:current={allJobsOpen}
				aria-current={allJobsOpen ? 'page' : undefined}
				onclick={() => go({ name: 'allJobs' })}
				><Icon name="list" size={14} /><span class="ellipsis"
					>{m.allJobs.link(app.jobs.length)}</span
				></button
			>
		{/if}
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
		<button
			class="tool"
			class:current={app.view.name === 'glossary'}
			aria-current={app.view.name === 'glossary' ? 'page' : undefined}
			aria-label={collapsed ? m.nav.glossary : undefined}
			title={collapsed ? m.nav.glossary : undefined}
			onclick={() => go({ name: 'glossary' })}
			><Icon name="glossary" size={16} />{#if !collapsed}{m.nav.glossary}{/if}</button
		>
		{#if !app.info?.app_store}
			<SupportLink variant="tool" iconOnly={collapsed} />
		{/if}
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
	.nav-group {
		display: grid;
		gap: 2px;
		margin-bottom: calc(-1 * var(--space-3));
	}
	.tool-row {
		display: flex;
		align-items: center;
		gap: 2px;
	}
	.tool-row .tool {
		flex: 1;
	}
	.collapsed .tool-row .tool {
		flex: none;
	}
	.unfold {
		margin-left: 0;
	}
	.turn {
		display: grid;
		transition: transform 120ms ease-out;
	}
	.turn.open {
		transform: rotate(90deg);
	}
	/* The projects under Projects, indented under its icon; a long list scrolls on its own. */
	.projects-list {
		max-height: 13rem;
		overflow: hidden auto;
		padding-left: 1.625rem;
	}
	.projects-list li,
	.project .ellipsis {
		min-width: 0;
	}
	.project {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		min-height: var(--control-sm);
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
	.project .num {
		font-family: var(--font-mono);
		font-size: 0.75rem;
		color: var(--text-muted);
	}
	.project:hover {
		background: var(--wash);
	}
	.project.current {
		background: var(--surface-0);
		box-shadow: var(--shadow-raised);
		color: var(--text-bright);
		font-weight: 500;
	}
	/* While a job is dragged, each project shows it can take it; the one under the pointer is
	   ringed in ink. */
	.dropping .project {
		box-shadow: inset 0 0 0 1px var(--line-strong);
	}
	.dropping .project.drop {
		background: var(--surface-0);
		box-shadow: inset 0 0 0 1.5px var(--ink);
		color: var(--text-bright);
	}
	.drop-hint {
		padding: var(--space-1) 0.625rem 0 2.25rem;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.job.lifted {
		opacity: 0.5;
	}
	.drag-ghost {
		position: fixed;
		z-index: 60;
		display: flex;
		align-items: center;
		gap: var(--space-2);
		max-width: 16rem;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius-control);
		background: var(--ink);
		color: var(--on-ink);
		font-size: var(--type-sm);
		box-shadow: var(--shadow-flyout);
		pointer-events: none;
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
		font-size: 0.75rem;
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
	.rename {
		width: 100%;
		min-height: 2.25rem;
		font-size: var(--type-sm);
		font-weight: 500;
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
	/* The way to every job, under the most recent: quiet, like a group's heading. */
	.all-jobs {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
		min-height: var(--control-sm);
		margin-top: calc(-1 * var(--space-2));
		padding: 0 0.625rem;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.all-jobs:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.all-jobs.current {
		background: var(--surface-0);
		box-shadow: var(--shadow-raised);
		color: var(--text-bright);
		font-weight: 500;
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
