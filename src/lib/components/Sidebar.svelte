<script lang="ts">
	// The sidebar: the app's name, New job, Projects with the projects listed under it
	// (SidebarProjects), the list of jobs (SidebarJobs), and the tools at its foot (Prompts,
	// Settings, Glossary, support). A finished job dragged onto a listed project is added to it,
	// as its menu's Add to a project does. It folds to a rail of icons named by their tooltips,
	// leaving the room to the work.
	import Icon from './Icon.svelte';
	import SidebarJobs from './SidebarJobs.svelte';
	import SidebarProjects from './SidebarProjects.svelte';
	import SupportLink from './SupportLink.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { JobDrag } from '#lib/jobDrag.svelte.js';
	import { shortcut } from '#lib/platform.js';
	import { app, go, refreshProjects } from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';

	let {
		collapsed = $bindable()
	}: {
		/** Folded to the rail; the page keeps it, and its shortcut, on this computer. */
		collapsed: boolean;
	} = $props();

	const m = messages();

	const allJobsOpen = $derived(app.view.name === 'allJobs');
	const offlineReady = $derived(app.offline?.models.some((s) => s.installed) ?? false);

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

	/** A finished job carried onto a project: the projects unfold to take it. */
	const drag = new JobDrag(() => (showProjects = true));
</script>

<svelte:window
	onpointermove={drag.move}
	onpointerup={() => void drag.drop()}
	onpointercancel={drag.end}
	onkeydown={(e) => {
		if (e.key === 'Escape' && drag.dragging) drag.end();
	}}
/>

{#if drag.dragging}
	{@const held = drag.dragging}
	<div class="drag-ghost" style:left="{held.x + 12}px" style:top="{held.y + 8}px">
		<Icon name={held.job.task} size={14} /><span class="ellipsis">{held.job.name}</span>
	</div>
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

	<div class="nav-group">
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
			<SidebarProjects projects={listed} {drag} />
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

	<SidebarJobs {collapsed} {drag} />

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
