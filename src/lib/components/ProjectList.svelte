<script lang="ts">
	// Every project, newest change first, and a form to start one. Archived projects are kept
	// whole and listed on request.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import type { ProjectSummary } from '#lib/bindings/ProjectSummary.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { go } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	const m = messages();
	let projects = $state<ProjectSummary[]>([]);
	let loaded = $state(false);
	let error = $state<string | null>(null);
	let showArchived = $state(false);
	let creating = $state(false);
	let name = $state('');
	let description = $state('');

	const shown = $derived(projects.filter((p) => showArchived || !p.project.archived));
	const archivedCount = $derived(projects.filter((p) => p.project.archived).length);

	async function load() {
		try {
			projects = await api.listProjects();
		} catch (e) {
			error = describeError(e, m);
		} finally {
			loaded = true;
		}
	}

	async function create(e: SubmitEvent) {
		e.preventDefault();
		error = null;
		try {
			const project = await api.createProject(name, description);
			go({ name: 'projects', projectId: project.id });
		} catch (err) {
			error = describeError(err, m);
		}
	}

	onMount(() => void load());
</script>

<header class="masthead">
	<div>
		<h1>{m.projects.title}</h1>
		<p class="lede">{m.projects.lede}</p>
	</div>
	{#if !creating}
		<button class="btn primary" onclick={() => (creating = true)}
			><Icon name="plus" />{m.projects.newProject}</button
		>
	{/if}
</header>

<div class="sheet">
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}

	{#if creating}
		<form class="card create" onsubmit={create} aria-labelledby="new-project">
			<h2 id="new-project">{m.projects.newProject}</h2>
			<label class="field">
				<span>{m.projects.name}</span>
				<!-- svelte-ignore a11y_autofocus -->
				<input type="text" bind:value={name} required autofocus />
			</label>
			<label class="field">
				<span>{m.projects.description}</span>
				<textarea rows="2" bind:value={description} placeholder={m.projects.descriptionHint}
				></textarea>
			</label>
			<div class="actions">
				<button type="button" class="btn ghost" onclick={() => (creating = false)}
					>{m.projects.cancel}</button
				>
				<button class="btn primary" disabled={!name.trim()}>{m.projects.create}</button>
			</div>
		</form>
	{/if}

	{#if loaded && projects.length === 0 && !creating}
		<p class="muted">{m.projects.empty}</p>
	{/if}

	{#if shown.length > 0}
		<ul class="list">
			{#each shown as { project, documents, collections, jobs } (project.id)}
				<li>
					<button class="project" onclick={() => go({ name: 'projects', projectId: project.id })}>
						<span class="name">
							<Icon name="project" size={16} />
							<span class="ellipsis">{project.name}</span>
							{#if project.archived}<span class="chip">{m.projects.archived}</span>{/if}
						</span>
						{#if project.description}
							<span class="description">{project.description}</span>
						{/if}
						<span class="meta num">
							{m.projects.counts(documents, collections, jobs)} ·
							{m.projects.updated(formatDateTime(project.updated_at))}
						</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}

	{#if archivedCount > 0}
		<label class="toggle">
			<input type="checkbox" bind:checked={showArchived} />
			{m.projects.showArchived} <span class="num muted">({archivedCount})</span>
		</label>
	{/if}
</div>

<style>
	.create {
		display: grid;
		gap: var(--space-3);
		padding-top: var(--space-4);
		max-width: 36rem;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		border-top: 1px solid var(--line);
	}
	.project {
		display: grid;
		gap: var(--space-1);
		width: 100%;
		padding: var(--space-3) var(--space-2);
		border: 0;
		border-bottom: 1px solid var(--line-soft);
		background: none;
		text-align: left;
		cursor: pointer;
	}
	.project:hover {
		background: var(--wash);
	}
	.name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-bright);
		font-weight: 600;
		min-width: 0;
	}
	.ellipsis {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.description {
		color: var(--text-secondary);
		font-size: var(--type-sm);
		max-width: 75ch;
	}
	.meta {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.toggle {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
</style>
