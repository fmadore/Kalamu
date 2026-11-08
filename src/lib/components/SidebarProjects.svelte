<script lang="ts">
	// The projects listed under Projects in the sidebar, each opening its workspace. While a
	// finished job is dragged, each shows it can take it, and the one under the pointer is ringed.
	import type { ProjectSummary } from '#lib/bindings/ProjectSummary.js';
	import { messages } from '#lib/i18n/index.js';
	import type { JobDrag } from '#lib/jobDrag.svelte.js';
	import { app, go } from '#lib/state.svelte.js';

	let { projects, drag }: { projects: ProjectSummary[]; drag: JobDrag } = $props();
	const m = messages();

	const openProject = $derived(app.view.name === 'projects' ? app.view.projectId : undefined);
</script>

<ul
	class="projects-list"
	class:dropping={drag.dragging !== null}
	id="project-list"
	aria-label={m.nav.projects}
>
	{#each projects as p (p.project.id)}
		<li>
			<button
				class="project"
				class:current={openProject === p.project.id}
				class:drop={drag.dropOn === p.project.id}
				aria-current={openProject === p.project.id ? 'page' : undefined}
				data-project={p.project.id}
				onclick={() => go({ name: 'projects', projectId: p.project.id })}
				><span class="ellipsis">{p.project.name}</span><span class="num">{p.documents}</span
				></button
			>
		</li>
	{/each}
</ul>
{#if drag.dragging}<p class="drop-hint">{m.shell.dropOnProject}</p>{/if}

<style>
	/* The projects under Projects, indented under its icon; a long list scrolls on its own. */
	.projects-list {
		list-style: none;
		margin: 0;
		padding: 0 0 0 1.625rem;
		display: grid;
		gap: 2px;
		max-height: 13rem;
		overflow: hidden auto;
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
</style>
