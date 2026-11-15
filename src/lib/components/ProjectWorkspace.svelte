<script lang="ts">
	// One project: its collections down the side, the documents of the one chosen (or all) in a
	// table, a search through the text of every document, what in them needs checking, its
	// dictionary of the names its analyses found, and its activity, the jobs that read them.
	// It opens on the pane it was left on, remembered on this computer. Files dropped on the
	// window join it.
	import { onMount } from 'svelte';
	import AttachDialog from './AttachDialog.svelte';
	import DictionaryView from './DictionaryView.svelte';
	import DocumentTable from './DocumentTable.svelte';
	import ExportDialog from './ExportDialog.svelte';
	import Icon from './Icon.svelte';
	import ProjectActivity from './ProjectActivity.svelte';
	import ProjectEditForm from './ProjectEditForm.svelte';
	import ProjectSearch from './ProjectSearch.svelte';
	import ProjectSections from './ProjectSections.svelte';
	import ReviewQueue from './ReviewQueue.svelte';
	import type { ProjectView } from '#lib/bindings/ProjectView.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import { changed, type Draft } from '#lib/described.js';
	import { Action } from '#lib/action.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		app,
		go,
		refreshProjects,
		returnTo,
		settle,
		takeFiles,
		toast,
		type ProjectPane
	} from '#lib/state.svelte.js';
	import { keepFor, recallFor } from '#lib/stored.js';
	import { ask, pickDocuments, pickFolder } from '#lib/dialogs.js';
	import { api, listenWhileMounted, onDrop } from '#lib/tauri.js';

	let {
		projectId,
		collectionId,
		opening,
		selection = []
	}: {
		projectId: string;
		collectionId?: string;
		/** The pane to open on; the one it was left on when not given. */
		opening?: ProjectPane;
		/** Documents chosen in the table, for the list of what to check. */
		selection?: string[];
	} = $props();
	const m = messages();

	let view = $state<ProjectView | null>(null);
	const action = new Action({
		after: async () => {
			await load();
			// The sidebar lists the projects by name: a renamed or archived one changes there too.
			void refreshProjects();
		}
	});
	const PANE_KEY = 'kalamu.project.pane';
	const PANES: ProjectPane[] = ['documents', 'search', 'queue', 'dictionary', 'activity'];
	let pane = $state<ProjectPane>(
		// svelte-ignore state_referenced_locally
		opening ??
			recallFor(PANE_KEY, projectId, (v) => PANES.find((p) => p === v) ?? null) ??
			'documents'
	);
	let editing = $state(false);
	let draftName = $state('');
	let draftDescription = $state('');
	let draftDefaults = $state<Draft>({});
	let naming = $state<null | { id: string | null; name: string }>(null);
	let attaching = $state(false);
	let exporting = $state(false);
	let adding = $state(false);

	// Asked again for a pane (the list of what to check, from the table's selection), it opens.
	$effect(() => {
		if (opening) {
			pane = opening;
			keepFor(PANE_KEY, projectId, opening, 100);
		}
	});

	/** Shows a pane, remembered on this computer and in the view, so Back returns to it. */
	function showPane(to: ProjectPane) {
		pane = to;
		keepFor(PANE_KEY, projectId, to, 100);
		if (app.view.name === 'projects') settle({ ...app.view, pane: to });
	}

	const project = $derived(view?.project);
	const collection = $derived(view?.collections.find((c) => c.collection.id === collectionId));

	async function load() {
		try {
			view = await api.projectView(projectId);
		} catch (e) {
			action.fail(e);
		}
	}

	function show(id?: string) {
		keepFor(PANE_KEY, projectId, 'documents', 100);
		go({ name: 'projects', projectId, collectionId: id, pane: 'documents' });
	}

	function startEdit() {
		draftName = project?.name ?? '';
		draftDescription = project?.description ?? '';
		draftDefaults = { ...(project?.source_defaults ?? {}) };
		editing = true;
	}

	async function saveEdit(e: SubmitEvent) {
		e.preventDefault();
		const defaults = $state.snapshot(draftDefaults);
		const defaultsChanged = changed(defaults, project?.source_defaults ?? {});
		await action.run(async () => {
			await api.updateProject(projectId, draftName, draftDescription);
			if (defaultsChanged) await api.setSourceDefaults(projectId, defaults);
		});
		if (!action.error) editing = false;
	}

	async function remove() {
		if (!project || !(await ask(m.projects.deleteConfirm(project.name)))) return;
		try {
			await api.deleteProject(projectId);
			go({ name: 'projects' });
		} catch (e) {
			action.fail(e);
		}
	}

	async function saveCollection(e: SubmitEvent) {
		e.preventDefault();
		if (!naming) return;
		const { id, name } = naming;
		naming = null;
		if (id) {
			await action.run(() => api.renameCollection(projectId, id, name));
		} else {
			let created: string | undefined;
			await action.run(async () => (created = (await api.createCollection(projectId, name)).id));
			if (created) show(created);
		}
	}

	async function removeCollection() {
		if (!collection) return;
		if (!(await ask(m.projects.deleteCollectionConfirm(collection.collection.name)))) return;
		await action.run(() => api.deleteCollection(projectId, collection.collection.id));
		show();
	}

	/** Files chosen, dropped or found in a folder, read and added as documents; `refused` files
	 * were already found to be of no kind Kalamu reads. */
	async function addPaths(paths: string[], refused = 0) {
		if (paths.length === 0 && refused === 0) return;
		adding = true;
		action.error = null;
		try {
			const inspected = paths.length > 0 ? await api.inspectFiles(paths) : [];
			const sources = inspected.map((f) => f.source).filter((s): s is SourceFile => !!s);
			const unread = inspected.length - sources.length + refused;
			if (sources.length > 0) {
				await api.addDocuments(projectId, sources, collectionId ?? null);
				toast(m.projects.addedFiles(sources.length));
			}
			if (unread > 0) action.error = m.projects.notReadable(unread);
			await load();
		} catch (e) {
			action.fail(e);
		} finally {
			adding = false;
		}
	}

	/** Files and folders chosen or dropped, a folder opened to the files in it of every kind a
	 * task reads (documents and images, recordings and videos), then added. */
	async function addExpanded(paths: string[]) {
		if (paths.length === 0) return;
		try {
			const [texts, recordings] = await Promise.all([
				api.expandPaths(paths, 'analysis'),
				api.expandPaths(paths, 'audio_transcription')
			]);
			// A file neither reads is said to be left out; a folder with nothing in it is not.
			const refused = texts.refused.filter(
				(r) => !r.folder && recordings.refused.some((o) => o.path === r.path)
			).length;
			await addPaths([...new Set([...texts.paths, ...recordings.paths])], refused);
		} catch (e) {
			action.fail(e);
		}
	}

	async function addFolder() {
		const folder = await pickFolder();
		if (folder) await addExpanded([folder]);
	}

	onMount(() => {
		void load();
		// The pane opened on is set in the view, for Back to come back to it whatever is
		// remembered by then.
		if (app.view.name === 'projects' && !app.view.pane) settle({ ...app.view, pane });
		const release = takeFiles(m.projects.dropHere);
		const stop = listenWhileMounted(() => onDrop((paths) => void addExpanded(paths)));
		return () => {
			release();
			stop();
		};
	});
</script>

<header class="masthead">
	<div>
		<button class="back" onclick={() => returnTo({ name: 'projects' })}
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
			<button class="btn ghost sm" onclick={() => (exporting = true)}
				><Icon name="download" size={14} />{m.projects.describe.export}</button
			>
			<button
				class="btn ghost sm"
				onclick={() => action.run(() => api.archiveProject(projectId, !project.archived))}
				><Icon name={project.archived ? 'restore' : 'archive'} size={14} />{project.archived
					? m.projects.restore
					: m.projects.archive}</button
			>
			<button class="btn ghost danger sm" onclick={remove}>{m.projects.delete}</button>
		</div>
	{/if}
</header>

{#if action.error}<p class="chip danger banner" role="alert">{action.error}</p>{/if}
{#if project?.archived}<p class="notice banner">{m.projects.archivedNotice}</p>{/if}

{#if editing}
	<ProjectEditForm
		bind:name={draftName}
		bind:description={draftDescription}
		bind:defaults={draftDefaults}
		onsubmit={saveEdit}
		oncancel={() => (editing = false)}
	/>
{/if}

{#if view}
	<div class="workspace">
		<ProjectSections
			{view}
			{pane}
			{collectionId}
			bind:naming
			onshow={show}
			onpane={showPane}
			onname={saveCollection}
		/>

		<section class="main">
			{#if pane === 'activity'}
				<ProjectActivity jobs={view.jobs} />
			{:else if pane === 'search'}
				<ProjectSearch {projectId} collections={view.collections} tags={view.tags} />
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
					onerror={(e) => (action.error = e)}
				/>
			{/if}
		</section>
	</div>
{/if}

{#if exporting}
	<ExportDialog
		{projectId}
		{collectionId}
		name={collection?.collection.name ?? project?.name ?? ''}
		onclose={() => (exporting = false)}
	/>
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
	/* Down to the foot of the window, so the column of sections keeps its rule. */
	.workspace {
		display: grid;
		grid-template-columns: 14rem minmax(0, 1fr);
		min-height: calc(100vh - 3.5rem);
	}
	.grow {
		flex: 1;
		min-width: 0;
	}
	.naming {
		padding: var(--space-1) 0;
	}
	.naming input {
		width: 100%;
	}
	.main {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
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
		font-size: var(--type-lg);
	}
	@media (max-width: 900px) {
		.workspace {
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: auto 1fr;
		}
	}
</style>
