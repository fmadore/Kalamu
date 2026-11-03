<script lang="ts">
	// One project: its collections down the side, the documents of the one chosen (or all) in a
	// table, what in them needs checking, its dictionary of the names its analyses found, and its
	// activity, the jobs that read them. Files dropped on the window join it.
	import { onMount } from 'svelte';
	import AttachDialog from './AttachDialog.svelte';
	import DictionaryView from './DictionaryView.svelte';
	import DocumentTable from './DocumentTable.svelte';
	import Icon from './Icon.svelte';
	import ProjectActivity from './ProjectActivity.svelte';
	import ReviewQueue from './ReviewQueue.svelte';
	import type { ProjectView } from '#lib/bindings/ProjectView.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { api, ask, onDrop, pickDocuments, pickFolder } from '#lib/tauri.js';

	let {
		projectId,
		collectionId,
		opening,
		selection = []
	}: {
		projectId: string;
		collectionId?: string;
		/** The pane to open on: the list of what to check. */
		opening?: 'queue';
		/** Documents chosen in the table, for the list of what to check. */
		selection?: string[];
	} = $props();
	const m = messages();

	let view = $state<ProjectView | null>(null);
	let error = $state<string | null>(null);
	// svelte-ignore state_referenced_locally
	let pane = $state<'documents' | 'queue' | 'dictionary' | 'activity'>(opening ?? 'documents');
	let editing = $state(false);
	let draftName = $state('');
	let draftDescription = $state('');
	let naming = $state<null | { id: string | null; name: string }>(null);
	let attaching = $state(false);
	let adding = $state(false);

	// Asked again for the list of what to check (from the table's selection), it opens.
	$effect(() => {
		if (opening) pane = opening;
	});

	const project = $derived(view?.project);
	const collection = $derived(view?.collections.find((c) => c.collection.id === collectionId));

	async function load() {
		try {
			view = await api.projectView(projectId);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function act(action: () => Promise<unknown>) {
		error = null;
		try {
			await action();
			await load();
		} catch (e) {
			error = describeError(e, m);
		}
	}

	function show(id?: string) {
		pane = 'documents';
		go({ name: 'projects', projectId, collectionId: id });
	}

	function startEdit() {
		draftName = project?.name ?? '';
		draftDescription = project?.description ?? '';
		editing = true;
	}

	async function saveEdit(e: SubmitEvent) {
		e.preventDefault();
		await act(() => api.updateProject(projectId, draftName, draftDescription));
		if (!error) editing = false;
	}

	async function remove() {
		if (!project || !(await ask(m.projects.deleteConfirm(project.name)))) return;
		try {
			await api.deleteProject(projectId);
			go({ name: 'projects' });
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function saveCollection(e: SubmitEvent) {
		e.preventDefault();
		if (!naming) return;
		const { id, name } = naming;
		naming = null;
		if (id) {
			await act(() => api.renameCollection(projectId, id, name));
		} else {
			let created: string | undefined;
			await act(async () => (created = (await api.createCollection(projectId, name)).id));
			if (created) show(created);
		}
	}

	async function removeCollection() {
		if (!collection) return;
		if (!(await ask(m.projects.deleteCollectionConfirm(collection.collection.name)))) return;
		await act(() => api.deleteCollection(projectId, collection.collection.id));
		show();
	}

	/** Files chosen, dropped or found in a folder, read and added as documents. */
	async function addPaths(paths: string[]) {
		if (paths.length === 0) return;
		adding = true;
		error = null;
		try {
			const inspected = await api.inspectFiles(paths);
			const sources = inspected.map((f) => f.source).filter((s): s is SourceFile => !!s);
			const unread = inspected.length - sources.length;
			if (sources.length > 0) {
				await api.addDocuments(projectId, sources, collectionId ?? null);
				toast(m.projects.addedFiles(sources.length));
			}
			if (unread > 0) error = m.projects.notReadable(unread);
			await load();
		} catch (e) {
			error = describeError(e, m);
		} finally {
			adding = false;
		}
	}

	async function addFolder() {
		const folder = await pickFolder();
		if (!folder) return;
		// Every kind of file a task reads: documents and images, recordings and videos.
		const [texts, recordings] = await Promise.all([
			api.expandPaths([folder], 'analysis'),
			api.expandPaths([folder], 'audio_transcription')
		]);
		await addPaths([...new Set([...texts, ...recordings])]);
	}

	onMount(() => {
		void load();
		let unlisten: (() => void) | undefined;
		void onDrop((paths) => void addPaths(paths)).then((u) => (unlisten = u));
		return () => unlisten?.();
	});
</script>

<header class="masthead">
	<div>
		<button class="back" onclick={() => go({ name: 'projects' })}
			><Icon name="back" size={14} />{m.projects.back}</button
		>
		<h1>{project?.name ?? ''}</h1>
		{#if project?.description}<p class="lede">{project.description}</p>{/if}
	</div>
	{#if project}
		<div class="tools">
			<button class="btn ghost sm" onclick={startEdit}
				><Icon name="edit" size={14} />{m.projects.edit}</button
			>
			<button
				class="btn ghost sm"
				onclick={() => act(() => api.archiveProject(projectId, !project.archived))}
				><Icon name={project.archived ? 'restore' : 'archive'} size={14} />{project.archived
					? m.projects.restore
					: m.projects.archive}</button
			>
			<button class="btn ghost danger sm" onclick={remove}>{m.projects.delete}</button>
		</div>
	{/if}
</header>

{#if error}<p class="chip danger banner" role="alert">{error}</p>{/if}
{#if project?.archived}<p class="notice banner">{m.projects.archivedNotice}</p>{/if}

{#if editing}
	<form class="edit" onsubmit={saveEdit}>
		<label class="field">
			<span>{m.projects.name}</span>
			<input type="text" bind:value={draftName} required />
		</label>
		<label class="field">
			<span>{m.projects.description}</span>
			<textarea rows="2" bind:value={draftDescription} placeholder={m.projects.descriptionHint}
			></textarea>
		</label>
		<div class="row">
			<button type="button" class="btn ghost" onclick={() => (editing = false)}
				>{m.projects.cancel}</button
			>
			<button class="btn primary" disabled={!draftName.trim()}>{m.projects.save}</button>
		</div>
	</form>
{/if}

{#if view}
	<div class="workspace">
		<nav class="sections" aria-label={m.projects.collections}>
			<button
				class="section"
				class:current={pane === 'documents' && !collectionId}
				aria-current={pane === 'documents' && !collectionId ? 'page' : undefined}
				onclick={() => show()}
			>
				<Icon name="project" size={14} /><span class="grow">{m.projects.allDocuments}</span>
				<span class="num">{view.documents.length}</span>
			</button>

			<h2 class="group-head">{m.projects.collections}</h2>
			{#each view.collections as c (c.collection.id)}
				<button
					class="section"
					class:current={pane === 'documents' && collectionId === c.collection.id}
					aria-current={pane === 'documents' && collectionId === c.collection.id
						? 'page'
						: undefined}
					onclick={() => show(c.collection.id)}
				>
					<Icon name="collection" size={14} /><span class="grow ellipsis">{c.collection.name}</span>
					<span class="num">{c.documents}</span>
				</button>
			{/each}
			{#if naming && !naming.id}
				<form class="naming" onsubmit={saveCollection}>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						bind:value={naming.name}
						aria-label={m.projects.collectionName}
						autofocus
						onkeydown={(e) => e.key === 'Escape' && (naming = null)}
					/>
				</form>
			{:else}
				<button class="section add" onclick={() => (naming = { id: null, name: '' })}
					><Icon name="plus" size={14} />{m.projects.newCollection}</button
				>
			{/if}

			<button
				class="section queue"
				class:current={pane === 'queue'}
				aria-current={pane === 'queue' ? 'page' : undefined}
				onclick={() => (pane = 'queue')}
			>
				<Icon name="flag" size={14} /><span class="grow">{m.queue.title}</span>
			</button>
			<button
				class="section"
				class:current={pane === 'dictionary'}
				aria-current={pane === 'dictionary' ? 'page' : undefined}
				onclick={() => (pane = 'dictionary')}
			>
				<Icon name="glossary" size={14} /><span class="grow">{m.dictionary.title}</span>
			</button>
			<button
				class="section activity"
				class:current={pane === 'activity'}
				aria-current={pane === 'activity' ? 'page' : undefined}
				onclick={() => (pane = 'activity')}
			>
				<Icon name="list" size={14} /><span class="grow">{m.projects.activity}</span>
				<span class="num">{view.jobs.length}</span>
			</button>
		</nav>

		<section class="main">
			{#if pane === 'activity'}
				<ProjectActivity jobs={view.jobs} />
			{:else if pane === 'queue'}
				{#key selection.join(',')}
					<ReviewQueue {projectId} collections={view.collections} {selection} />
				{/key}
			{:else if pane === 'dictionary'}
				<DictionaryView
					{projectId}
					projectName={view.project.name}
					collections={view.collections}
				/>
			{:else}
				<div class="toolbar">
					{#if collection}
						{#if naming?.id === collection.collection.id}
							<form class="naming" onsubmit={saveCollection}>
								<!-- svelte-ignore a11y_autofocus -->
								<input
									type="text"
									bind:value={naming.name}
									aria-label={m.projects.collectionName}
									autofocus
									onkeydown={(e) => e.key === 'Escape' && (naming = null)}
								/>
							</form>
						{:else}
							<h2>{collection.collection.name}</h2>
							<button
								class="btn ghost sm"
								onclick={() =>
									(naming = { id: collection.collection.id, name: collection.collection.name })}
								>{m.projects.renameCollection}</button
							>
							<button class="btn ghost danger sm" onclick={removeCollection}
								>{m.projects.deleteCollection}</button
							>
						{/if}
					{:else}
						<h2>{m.projects.allDocuments}</h2>
					{/if}
					<span class="grow"></span>
					<button
						class="btn sm"
						disabled={adding}
						onclick={async () => addPaths(await pickDocuments())}
						><Icon name="add-file" size={14} />{m.projects.addFiles}</button
					>
					<button class="btn sm" disabled={adding} onclick={addFolder}
						><Icon name="folder" size={14} />{m.projects.addFolder}</button
					>
					<button class="btn sm" onclick={() => (attaching = true)}
						><Icon name="link" size={14} />{m.projects.attachJob}</button
					>
				</div>
				<DocumentTable
					{projectId}
					projectName={view.project.name}
					rows={view.documents}
					collections={view.collections}
					tags={view.tags}
					{collectionId}
					onchange={load}
					onerror={(e) => (error = e)}
				/>
			{/if}
		</section>
	</div>
{/if}

{#if attaching && view}
	<AttachDialog
		{projectId}
		{collectionId}
		documents={view.documents}
		onclose={(attached) => {
			attaching = false;
			if (attached > 0) {
				toast(m.projects.attach.attached(attached));
				void load();
			}
		}}
	/>
{/if}

<style>
	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
	}
	.banner {
		margin: var(--space-3) var(--space-5) 0;
	}
	.edit {
		display: grid;
		gap: var(--space-3);
		max-width: 36rem;
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--line);
	}
	.row {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	/* Down to the foot of the window, so the column of sections keeps its rule. */
	.workspace {
		display: grid;
		grid-template-columns: 14rem minmax(0, 1fr);
		min-height: calc(100vh - 3.5rem);
	}
	.sections {
		display: grid;
		align-content: start;
		gap: 1px;
		padding: var(--space-4) var(--space-2);
		border-right: 1px solid var(--line);
		min-width: 0;
	}
	.group-head {
		margin: var(--space-4) var(--space-2) var(--space-1);
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-muted);
	}
	.section {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-height: var(--control-md);
		padding: 0 var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: none;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
		min-width: 0;
	}
	.section:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.section.current {
		background: var(--wash);
		color: var(--text-bright);
		font-weight: 500;
	}
	.section.add {
		color: var(--text-muted);
	}
	.section.queue {
		margin-top: var(--space-4);
	}
	.section .num {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.grow {
		flex: 1;
		min-width: 0;
	}
	.ellipsis {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.naming {
		padding: var(--space-1) 0;
	}
	.naming input {
		width: 100%;
	}
	.main {
		display: grid;
		align-content: start;
		gap: var(--space-4);
		padding: var(--space-4) var(--space-5) var(--space-7);
		min-width: 0;
	}
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.toolbar h2 {
		font-size: 1rem;
	}
	@media (max-width: 900px) {
		.workspace {
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: auto 1fr;
		}
		.sections {
			grid-auto-flow: column;
			grid-auto-columns: max-content;
			overflow-x: auto;
			border-right: 0;
			border-bottom: 1px solid var(--line);
			padding: var(--space-2) var(--space-4);
		}
		.group-head {
			display: none;
		}
		.section.queue {
			margin-top: 0;
		}
	}
</style>
