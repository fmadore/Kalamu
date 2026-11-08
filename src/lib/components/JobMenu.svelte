<script lang="ts">
	// What can be done to a job from the job list, opened by a right click or the keyboard's menu
	// key: rename it, add it to a project, delete it. At the pointer, kept inside the window; closed
	// by Escape, a click elsewhere or a choice.
	import Icon from './Icon.svelte';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import type { ProjectSummary } from '#lib/bindings/ProjectSummary.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { deleteJob } from '#lib/jobActions.svelte.js';
	import { toast } from '#lib/state.svelte.js';
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

	let box = $state<HTMLElement | null>(null);
	let projects = $state<ProjectSummary[] | null>(null);
	const running = $derived(job.status === 'running');

	$effect(() => {
		void api
			.listProjects()
			.then((p) => (projects = p))
			.catch(() => (projects = []));
	});

	$effect(() => {
		if (!box) return;
		box.showPopover();
		const { offsetWidth: w, offsetHeight: h } = box;
		box.style.left = `${Math.max(8, Math.min(x, innerWidth - w - 8))}px`;
		box.style.top = `${Math.max(8, Math.min(y, innerHeight - h - 8))}px`;
		items()[0]?.focus();
	});

	const items = () =>
		box ? [...box.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')] : [];

	function keys(e: KeyboardEvent) {
		const all = items();
		const at = all.indexOf(document.activeElement as HTMLButtonElement);
		const to =
			e.key === 'ArrowDown'
				? (at + 1) % all.length
				: e.key === 'ArrowUp'
					? (at - 1 + all.length) % all.length
					: e.key === 'Home'
						? 0
						: e.key === 'End'
							? all.length - 1
							: -1;
		if (to < 0) return;
		e.preventDefault();
		all[to]?.focus();
	}

	function close() {
		box?.hidePopover();
	}

	async function addTo(p: ProjectSummary) {
		close();
		try {
			await api.linkJob(p.project.id, job.id, null);
			toast(m.shell.addedTo(p.project.name));
		} catch (e) {
			toast(describeError(e, m));
		}
	}

	async function remove() {
		close();
		try {
			await deleteJob(job.id);
		} catch (e) {
			toast(describeError(e, m));
		}
	}
</script>

<div
	bind:this={box}
	class="menu"
	popover="auto"
	role="menu"
	tabindex="-1"
	aria-label={m.shell.jobMenu(job.name)}
	onkeydown={keys}
	ontoggle={(e) => {
		if (e.newState === 'closed') onclose();
	}}
>
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
</div>

<style>
	/* In the top layer, placed at the pointer by the effect above. */
	.menu {
		position: fixed;
		inset: auto;
		margin: 0;
		width: 15rem;
		padding: var(--space-1);
		border: 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	/* One column no wider than the menu: a long project name is cut short, never scrolled. */
	.menu:popover-open {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 1px;
	}
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
	button:hover:not(:disabled),
	button:focus-visible {
		background: var(--wash);
		outline: none;
	}
	button:disabled {
		color: var(--text-faint);
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
