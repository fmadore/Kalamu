<script lang="ts">
	// Every project, newest change first, and a form to start one. Archived projects are kept
	// whole and listed on request.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import type { ProjectSummary } from '#lib/bindings/ProjectSummary.js';
	import { Action } from '#lib/action.svelte.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import type { ProjectFileOpened } from '#lib/bindings/ProjectFileOpened.js';
	import { pickProjectFile } from '#lib/projectFile.js';
	import { go, refreshJobs, refreshProjects } from '#lib/state.svelte.js';
	import { api, listenWhileMounted, on } from '#lib/tauri.js';
	import CheckProjectDialog from './CheckProjectDialog.svelte';
	import ProgressBar from './ProgressBar.svelte';

	const m = messages();
	let projects = $state<ProjectSummary[]>([]);
	let loaded = $state(false);
	/** Reading and starting projects, what went wrong said above the list. */
	const action = new Action();
	let showArchived = $state(false);
	let creating = $state(false);
	let name = $state('');
	let description = $state('');

	/** A project file being opened, how far it has got, and what came in once it is open. */
	let opening = $state<{ done: number; total: number } | null>(null);
	let opened = $state<ProjectFileOpened | null>(null);
	/** A project file being checked without opening it. */
	let checking = $state<string | null>(null);

	const shown = $derived(projects.filter((p) => showArchived || !p.project.archived));
	const archivedCount = $derived(projects.filter((p) => p.project.archived).length);

	async function load() {
		try {
			projects = await api.listProjects();
		} catch (e) {
			action.fail(e);
		} finally {
			loaded = true;
		}
	}

	async function create(e: SubmitEvent) {
		e.preventDefault();
		await action.run(
			() => api.createProject(name, description),
			(project) => go({ name: 'projects', projectId: project.id })
		);
	}

	/** Opens a project file as a new project, beside the others, and says what came in. */
	async function openFile() {
		const path = await pickProjectFile();
		if (!path) return;
		opened = null;
		opening = { done: 0, total: 0 };
		await action.run(
			() => api.openProjectFile(path),
			async (found) => {
				opened = found;
				await load();
				// Its jobs, copied in, are listed at once with the others.
				void refreshProjects();
				void refreshJobs();
			}
		);
		opening = null;
	}

	/** Checks a project file without opening it, in a dialog that says what it found. */
	async function checkFile() {
		checking = await pickProjectFile();
	}

	onMount(() => {
		void load();
		return listenWhileMounted(() =>
			on.projectFileProgress((p) => {
				if (opening) opening = p;
			})
		);
	});
</script>

<header class="masthead">
	<div>
		<h1>{m.projects.title}</h1>
		<p class="lede">{m.projects.lede}</p>
	</div>
	{#if !creating}
		<div class="tools">
			<button class="btn" onclick={openFile} disabled={!!opening}>
				<Icon name="folder" />{m.projectFile.open}
			</button>
			<button class="btn" onclick={checkFile} disabled={!!opening}>
				<Icon name="check" />{m.projectFile.check}
			</button>
			<button class="btn primary" onclick={() => (creating = true)}>
				<Icon name="plus" />{m.projects.newProject}
			</button>
		</div>
	{/if}
</header>

<div class="sheet">
	{#if action.error}<p class="chip danger" role="alert">{action.error}</p>{/if}

	{#if opening}
		<div class="card opening" role="status">
			<span>{m.projectFile.opening}</span>
			<ProgressBar done={opening.done} total={opening.total || 1} label={m.projectFile.opening} />
		</div>
	{/if}
	{#if opened}
		{@const project = opened}
		<div class="card opened" role="status">
			<p>{m.projectFile.opened(project.name, project.documents, project.jobs)}</p>
			{#if !project.with_originals}<p class="muted">{m.projectFile.openedWithout}</p>{/if}
			{#if project.missing_texts.length > 0}
				<div class="missing">
					<p>{m.projectFile.missingTexts(project.missing_texts.length)}</p>
					<ul>
						{#each project.missing_texts as text, i (i)}
							<li>
								{m.projectFile.missingText(
									text.documents.length > 0 ? text.documents.join(', ') : text.file_name,
									text.jobs.join(', ')
								)}
							</li>
						{/each}
					</ul>
				</div>
			{/if}
			<div>
				<button
					class="btn primary sm"
					onclick={() => go({ name: 'projects', projectId: project.project_id })}
				>
					<Icon name="project" size={14} />{project.name}
				</button>
			</div>
		</div>
	{/if}

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
				<button type="button" class="btn ghost" onclick={() => (creating = false)}>
					{m.projects.cancel}
				</button>
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

{#if checking}
	<CheckProjectDialog path={checking} onclose={() => (checking = null)} />
{/if}

<style>
	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.opening,
	.opened {
		display: grid;
		gap: var(--space-2);
		max-width: 48rem;
		padding-top: var(--space-3);
	}
	.opened p {
		margin: 0;
	}
	.missing {
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.missing ul {
		margin: var(--space-1) 0 0;
		padding-left: var(--space-4);
	}
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
