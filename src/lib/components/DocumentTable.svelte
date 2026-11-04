<script lang="ts">
	// A project's documents, found and sorted, each with its collections and what has been read
	// from it. Selected documents can be grouped, ungrouped, removed, or sent to New job, whose
	// job is then tied to them.
	import Icon from './Icon.svelte';
	import TagInput from './TagInput.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { DocumentRow } from '#lib/bindings/DocumentRow.js';
	import type { TagView } from '#lib/bindings/TagView.js';
	import { Action } from '#lib/action.svelte.js';
	import { formatDate, formatDateTime, messages } from '#lib/i18n/index.js';
	import {
		selectionPaths,
		shownDocuments,
		sortDocuments,
		tagsOf,
		taskFor,
		type DocumentSort
	} from '#lib/projects.js';
	import { Selection } from '#lib/selection.svelte.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { api, ask } from '#lib/tauri.js';

	let {
		projectId,
		projectName,
		rows,
		collections,
		tags,
		collectionId,
		onchange,
		onerror
	}: {
		projectId: string;
		projectName: string;
		rows: DocumentRow[];
		collections: CollectionView[];
		tags: TagView[];
		collectionId?: string;
		onchange: () => Promise<void>;
		onerror: (message: string) => void;
	} = $props();
	const m = messages();

	let query = $state('');
	let sort = $state<DocumentSort>('title');
	const selection = new Selection();
	/** A change to the documents chosen: the choice is cleared and the project read again. */
	const action = new Action({
		after: async () => {
			selection.clear();
			await onchange();
		},
		report: (message) => onerror(message)
	});
	let target = $state('');
	let tag = $state('');
	let untagging = $state('');
	/** The new name typed for the tag filtered on, while renaming it. */
	let renaming = $state<string | null>(null);

	// A tag no document carries any longer stops filtering.
	const filterTag = $derived(tags.some((t) => t.name === tag) ? tag : '');
	const shown = $derived(sortDocuments(shownDocuments(rows, collectionId, query, filterTag), sort));
	const chosen = $derived(selection.of(shown, (r) => r.document.id));
	const allChosen = $derived(shown.length > 0 && chosen.length === shown.length);
	const names = $derived(new Map(collections.map((c) => [c.collection.id, c.collection.name])));
	const chosenTags = $derived(tagsOf(chosen));

	async function addTo(id: string) {
		target = '';
		if (!id) return;
		const ids = chosen.map((r) => r.document.id);
		await action.run(() => api.addToCollection(projectId, id, ids));
	}

	async function addTag(name: string) {
		const ids = chosen.map((r) => r.document.id);
		await action.run(() => api.tagDocuments(projectId, ids, name));
	}

	async function removeTag(name: string) {
		untagging = '';
		if (!name) return;
		const ids = chosen.map((r) => r.document.id);
		await action.run(() => api.untagDocuments(projectId, ids, name));
	}

	/** Renames the tag filtered on across the project, and keeps filtering on it. */
	async function renameTag(e: SubmitEvent) {
		e.preventDefault();
		const from = filterTag;
		const to = renaming?.trim() ?? '';
		renaming = null;
		if (!from || !to || to === from) return;
		await action.run(
			() => api.renameTag(projectId, from, to),
			async (renamed) => {
				tag = renamed.name;
				await onchange();
				toast(m.projects.tagRenamed(renamed.changed));
			}
		);
	}

	async function removeFromCollection() {
		if (!collectionId) return;
		const ids = chosen.map((r) => r.document.id);
		await action.run(() => api.removeFromCollection(projectId, collectionId, ids));
	}

	async function removeDocuments() {
		if (!(await ask(m.projects.removeDocumentsConfirm(chosen.length)))) return;
		const ids = chosen.map((r) => r.document.id);
		await action.run(() => api.deleteDocuments(projectId, ids));
	}

	function newJob() {
		go({
			name: 'new',
			task: taskFor(chosen),
			paths: selectionPaths(chosen),
			project: { id: projectId, name: projectName, collectionId }
		});
	}

	function open(id: string) {
		go({ name: 'projects', projectId, collectionId, documentId: id });
	}
</script>

{#if rows.length === 0}
	<p class="muted">{m.projects.noDocuments}</p>
{:else}
	<div class="table-filters">
		<label class="table-search">
			<Icon name="search" size={14} />
			<input
				type="text"
				bind:value={query}
				placeholder={m.projects.search}
				aria-label={m.projects.search}
			/>
		</label>
		{#if tags.length > 0}
			<select bind:value={tag} aria-label={m.projects.tagFilter}>
				<option value="">{m.projects.anyTag}</option>
				{#each tags as t (t.name)}
					<option value={t.name}>{m.projects.tagCount(t.name, t.documents)}</option>
				{/each}
			</select>
			{#if filterTag && renaming !== null}
				<form class="rename-tag" onsubmit={renameTag}>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						bind:value={renaming}
						aria-label={m.projects.renameTagTo(filterTag)}
						placeholder={m.projects.renameTagTo(filterTag)}
						autofocus
						onkeydown={(e) => e.key === 'Escape' && (renaming = null)}
					/>
					<button class="btn sm" disabled={!renaming.trim()}>{m.projects.renameTag}</button>
				</form>
			{:else if filterTag}
				<button class="btn ghost sm" onclick={() => (renaming = filterTag)}
					><Icon name="edit" size={14} />{m.projects.renameTag}</button
				>
			{/if}
		{/if}
		<label class="sort">
			<span>{m.projects.sortBy}</span>
			<select bind:value={sort}>
				<option value="title">{m.projects.sortTitle}</option>
				<option value="added">{m.projects.sortAdded}</option>
				<option value="run">{m.projects.sortRun}</option>
			</select>
		</label>
	</div>

	{#if chosen.length > 0}
		<div class="selection-bar" role="toolbar" aria-label={m.projects.selected(chosen.length)}>
			<span class="num">{m.projects.selected(chosen.length)}</span>
			<button class="btn ghost sm" onclick={() => selection.clear()}
				>{m.projects.clearSelection}</button
			>
			<span class="grow"></span>
			<button class="btn primary sm" disabled={selectionPaths(chosen).length === 0} onclick={newJob}
				><Icon name="plus" size={14} />{m.projects.newJobWith}</button
			>
			<button
				class="btn sm"
				disabled={selectionPaths(chosen).length === 0}
				onclick={() =>
					go({
						name: 'workflows',
						paths: selectionPaths(chosen),
						project: { id: projectId, name: projectName, collectionId }
					})}><Icon name="workflow" size={14} />{m.projects.workflowWith}</button
			>
			<button
				class="btn sm"
				onclick={() =>
					go({
						name: 'projects',
						projectId,
						collectionId,
						pane: 'queue',
						selection: chosen.map((r) => r.document.id)
					})}><Icon name="flag" size={14} />{m.queue.checkSelection}</button
			>
			{#if collections.length > 0}
				<select
					class="sm"
					bind:value={target}
					onchange={() => addTo(target)}
					aria-label={m.projects.addTo}
				>
					<option value="">{m.projects.addTo}</option>
					{#each collections.filter((c) => c.collection.id !== collectionId) as c (c.collection.id)}
						<option value={c.collection.id}>{c.collection.name}</option>
					{/each}
				</select>
			{/if}
			<TagInput known={tags.map((t) => t.name)} onadd={addTag} />
			{#if chosenTags.length > 0}
				<select
					bind:value={untagging}
					onchange={() => removeTag(untagging)}
					aria-label={m.projects.removeTag}
				>
					<option value="">{m.projects.removeTag}</option>
					{#each chosenTags as name (name)}
						<option value={name}>{name}</option>
					{/each}
				</select>
			{/if}
			{#if collectionId}
				<button class="btn sm" onclick={removeFromCollection}
					>{m.projects.removeFromCollection}</button
				>
			{/if}
			<button class="btn ghost danger sm" onclick={removeDocuments}
				>{m.projects.removeDocuments}</button
			>
		</div>
	{/if}

	{#if shown.length === 0}
		<p class="muted">{m.projects.noMatch}</p>
	{:else}
		<table class="data-table">
			<thead>
				<tr>
					<th class="check">
						<input
							type="checkbox"
							checked={allChosen}
							aria-label={m.projects.selectAll}
							onchange={() => selection.toggleAll(shown.map((r) => r.document.id))}
						/>
					</th>
					<th>{m.projects.columnTitle}</th>
					<th>{m.projects.columnKind}</th>
					<th>{m.projects.columnCollections}</th>
					<th>{m.projects.columnResults}</th>
					<th>{m.projects.columnDate}</th>
				</tr>
			</thead>
			<tbody>
				{#each shown as row (row.document.id)}
					{@const doc = row.document}
					<tr class:chosen={selection.has(doc.id)}>
						<td class="check">
							<input
								type="checkbox"
								checked={selection.has(doc.id)}
								aria-label={m.projects.select(doc.title)}
								onchange={() => selection.toggle(doc.id)}
							/>
						</td>
						<td
							><div class="title">
								<button class="link" onclick={() => open(doc.id)}>{doc.title}</button>
								<span class="file mono" title={doc.source.path}>{doc.source.file_name}</span>
								{#if row.missing}
									<span class="chip danger" title={m.projects.missingHint}
										>{m.projects.missing}</span
									>
								{/if}
								{#if row.same_content > 0}
									<span class="chip warn">{m.projects.sameContent(row.same_content)}</span>
								{/if}
							</div></td
						>
						<td class="muted">{m.projects.kind[doc.source.kind]}</td>
						<td>
							<div class="chips">
								{#each row.collections as id (id)}
									<span class="chip">{names.get(id)}</span>
								{/each}
								{#each doc.tags as name (name)}
									<button
										class="chip tag"
										class:on={name === filterTag}
										aria-pressed={name === filterTag}
										onclick={() => (tag = name === filterTag ? '' : name)}
										><Icon name="tag" size={11} />{name}</button
									>
								{/each}
							</div>
						</td>
						<td class="results">
							{#if row.tasks.length === 0}
								<span class="muted">{m.projects.noResults}</span>
							{:else}
								{#each row.tasks as t (t.task)}
									<span
										class="result"
										class:reviewed={t.reviewed > 0}
										title={m.projects.taskResults(m.task[t.task], t.results, t.reviewed)}
									>
										<Icon name={t.task} size={13} />
										<span class="num">{t.results}</span>
										{#if t.reviewed > 0}<Icon name="check" size={12} />{/if}
										{#if t.preferred}<Icon name="star" size={12} filled />{/if}
										<span class="sr-only"
											>{m.projects.taskResults(m.task[t.task], t.results, t.reviewed)}{t.preferred
												? `, ${m.projects.preferredMark}`
												: ''}</span
										>
									</span>
								{/each}
							{/if}
						</td>
						<td class="num muted date" title={formatDateTime(doc.created_at)}
							>{formatDate(doc.created_at)}</td
						>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
{/if}

<style>
	.rename-tag {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
	.rename-tag input {
		width: 14rem;
	}
	.sort {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	/* Collections and tags in one row that wraps, centred on each other: in the line of text a
	   chip that starts with an icon sat lower than one without. */
	.chips {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1);
	}
	.chip.tag {
		border: 0;
		cursor: pointer;
		gap: 0.25rem;
		font-family: inherit;
	}
	.chip.tag:hover,
	.chip.tag.on {
		background: var(--wash);
		color: var(--text-bright);
	}
	.selection-bar select {
		min-height: var(--control-sm);
		font-size: var(--type-xs);
	}
	.grow {
		flex: 1;
	}
	.title {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-1) var(--space-2);
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font-weight: 500;
		text-align: left;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
		text-underline-offset: 0.18em;
	}
	.file {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.results {
		white-space: nowrap;
	}
	.result {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		margin-right: var(--space-2);
		color: var(--text-secondary);
	}
	.result.reviewed {
		color: var(--ok-text);
	}
	.date {
		white-space: nowrap;
	}
	@media (max-width: 760px) {
		th:nth-child(3),
		td:nth-child(3),
		th:nth-child(6),
		td:nth-child(6) {
			display: none;
		}
		/* The table fits the window: results wrap, each count kept with its mark, and long
		   names break. */
		.results {
			white-space: normal;
		}
		td {
			overflow-wrap: anywhere;
		}
	}
</style>
