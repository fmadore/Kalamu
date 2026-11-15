<script lang="ts">
	// What can be done to a job from the job list, opened by its ⋯ button, a right click or the
	// keyboard's menu key: rename it, add it to a project, delete it. At the pointer or under the
	// button, kept inside the window; closed by Escape, a click elsewhere or a choice, the focus
	// going back to where it was.
	import ContextMenu from './ContextMenu.svelte';
	import Icon from './Icon.svelte';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import type { ProjectSummary } from '#lib/bindings/ProjectSummary.js';
	import { Action } from '#lib/action.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import { deleteJob } from '#lib/jobActions.svelte.js';
	import { refreshProjects, toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		job,
		x,
		y,
		onrename,
		onclose
	}: {
		job: JobSummary;
		/** Where it opens, in the window. */
		x: number;
		y: number;
		onrename: () => void;
		onclose: () => void;
	} = $props();
	const m = messages();
	/** Adding to a project and deleting, what went wrong said in an error toast. */
	const action = new Action({ report: (message) => toast(message, { kind: 'error' }) });

	let menu = $state<ContextMenu | null>(null);
	let projects = $state<ProjectSummary[] | null>(null);
	const running = $derived(job.status === 'running');

	$effect(() => {
		void api
			.listProjects()
			.then((p) => (projects = p))
			.catch(() => (projects = []));
	});

	function close() {
		menu?.close();
	}

	async function addTo(p: ProjectSummary) {
		close();
		await action.run(async () => {
			await api.linkJob(p.project.id, job.id, null);
			toast(m.shell.addedTo(p.project.name));
			// The sidebar counts each project's documents: this one has more.
			void refreshProjects();
		});
	}

	async function remove() {
		close();
		await action.run(() => deleteJob(job.id));
	}
</script>

<ContextMenu bind:this={menu} {x} {y} label={m.shell.jobMenu(job.name)} width="15rem" {onclose}>
	<button
		type="button"
		role="menuitem"
		onclick={() => {
			close();
			onrename();
		}}><Icon name="edit" size={14} />{m.jobs.rename}</button
	>
	<div class="group" role="group" aria-label={m.shell.addToProject}>
		<span class="label" aria-hidden="true"
			><Icon name="project" size={14} />{m.shell.addToProject}</span
		>
		{#if projects === null}
			<span class="note">{m.common.loading}</span>
		{:else if projects.length === 0}
			<span class="note">{m.shell.noProjects}</span>
		{:else}
			<div class="projects">
				{#each projects as p (p.project.id)}
					<button type="button" role="menuitem" class="project" onclick={() => addTo(p)}
						><span class="ellipsis">{p.project.name}</span></button
					>
				{/each}
			</div>
		{/if}
	</div>
	<button
		type="button"
		role="menuitem"
		class="danger"
		disabled={running}
		title={running ? m.shell.pauseFirst : undefined}
		onclick={remove}>{m.jobs.delete}</button
	>
</ContextMenu>

<style>
	button {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
		min-height: 2rem;
		padding: 0 var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-bright);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	button:hover:not(:disabled) {
		background: var(--wash);
	}
	/* The keyboard's place, told apart from the pointer's: the app's focus ring, inside the row. */
	button:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(-1 * var(--focus-width));
	}
	/* Unavailable, yet legible: muted beside the bright items, with no hover. */
	button:disabled {
		color: var(--text-muted);
		cursor: default;
	}
	.danger:not(:disabled) {
		color: var(--warn-text);
	}
	.group {
		display: grid;
		padding: var(--space-1) 0;
		margin: var(--space-1) 0;
		border-block: 1px solid var(--line-soft);
	}
	.label {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-1) var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.projects {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		max-height: 12rem;
		overflow: hidden auto;
	}
	.project {
		min-width: 0;
		padding-left: calc(var(--space-2) + 14px + var(--space-2));
	}
	.note {
		padding: var(--space-1) var(--space-2) var(--space-1)
			calc(var(--space-2) + 14px + var(--space-2));
		color: var(--text-muted);
	}
	.ellipsis {
		min-width: 0;
	}
</style>
