<script lang="ts">
	// What can be done with the documents chosen in a project's table: send them to New job or a
	// workflow, check them, ask which of them speak of the researcher's question, group, tag,
	// ungroup or remove them. While the table is filtered by an answer, the researcher's own
	// decision on them, Keep or Set aside, is one click: one of two tags, the other taken off, so
	// the model's answer is never changed.
	import Icon from './Icon.svelte';
	import ScreeningDialog from './ScreeningDialog.svelte';
	import TagInput from './TagInput.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { DocumentRow } from '#lib/bindings/DocumentRow.js';
	import type { TagView } from '#lib/bindings/TagView.js';
	import type { Action } from '#lib/action.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import { selectionPaths, tagsOf, taskFor } from '#lib/projects.js';
	import { decisionTags } from '#lib/screening.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { ask } from '#lib/dialogs.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		projectName,
		collectionId,
		collections,
		tags,
		chosen,
		action,
		deciding,
		onclear,
		onchange
	}: {
		projectId: string;
		projectName: string;
		collectionId?: string;
		collections: CollectionView[];
		tags: TagView[];
		chosen: DocumentRow[];
		/** Runs a change to the documents chosen, then clears the choice and reads the project. */
		action: Action;
		/** The table is filtered by an answer: the researcher's decision is offered. */
		deciding: boolean;
		onclear: () => void;
		onchange: () => Promise<void>;
	} = $props();
	const m = messages();

	let target = $state('');
	let untagging = $state('');
	let screening = $state<string[] | null>(null);
	const chosenTags = $derived(tagsOf(chosen));
	const ids = () => chosen.map((r) => r.document.id);
	const project = $derived({ id: projectId, name: projectName, collectionId });

	async function addTo(id: string) {
		target = '';
		if (!id) return;
		const of = ids();
		await action.run(() => api.addToCollection(projectId, id, of));
	}

	async function addTag(name: string) {
		const of = ids();
		await action.run(() => api.tagDocuments(projectId, of, name));
	}

	async function removeTag(name: string) {
		untagging = '';
		if (!name) return;
		const of = ids();
		await action.run(() => api.untagDocuments(projectId, of, name));
	}

	/** Keep or Set aside: one tag given, the other taken off. */
	async function decide(keep: boolean) {
		const of = ids();
		const { add, remove } = decisionTags(keep, m);
		await action.run(
			async () => {
				await api.untagDocuments(projectId, of, remove);
				await api.tagDocuments(projectId, of, add);
			},
			async () => {
				onclear();
				await onchange();
				toast(m.screening.decided(add, of.length));
			}
		);
	}

	async function removeFromCollection() {
		if (!collectionId) return;
		const from = collectionId;
		const of = ids();
		const name = collections.find((c) => c.collection.id === from)?.collection.name ?? '';
		await action.run(
			() => api.removeFromCollection(projectId, from, of),
			async () => {
				onclear();
				await onchange();
				// Put back from the toast: the documents themselves stay in the project.
				toast(m.projects.removedFromCollection(of.length, name), {
					action: {
						label: m.common.undo,
						run: () => void action.run(() => api.addToCollection(projectId, from, of))
					}
				});
			}
		);
	}

	async function removeDocuments() {
		if (!(await ask(m.projects.removeDocumentsConfirm(chosen.length)))) return;
		const of = ids();
		await action.run(() => api.deleteDocuments(projectId, of));
	}

	function newJob() {
		go({ name: 'new', task: taskFor(chosen), paths: selectionPaths(chosen), project });
	}
</script>

<div class="selection-bar" role="toolbar" aria-label={m.projects.selected(chosen.length)}>
	<span class="num">{m.projects.selected(chosen.length)}</span>
	<button class="btn ghost sm" onclick={onclear}>{m.projects.clearSelection}</button>
	<span class="grow"></span>
	{#if deciding}
		<button class="btn sm" onclick={() => decide(true)}>
			<Icon name="check" size={14} />{m.screening.keep}
		</button>
		<button class="btn sm" onclick={() => decide(false)}>{m.screening.setAside}</button>
	{/if}
	<button class="btn primary sm" disabled={selectionPaths(chosen).length === 0} onclick={newJob}>
		<Icon name="plus" size={14} />{m.projects.newJobWith}
	</button>
	<button
		class="btn sm"
		disabled={selectionPaths(chosen).length === 0}
		onclick={() => go({ name: 'workflows', paths: selectionPaths(chosen), project })}
	>
		<Icon name="workflow" size={14} />{m.projects.workflowWith}
	</button>
	<button
		class="btn sm"
		onclick={() =>
			go({ name: 'projects', projectId, collectionId, pane: 'queue', selection: ids() })}
	>
		<Icon name="flag" size={14} />{m.queue.checkSelection}
	</button>
	<button class="btn sm" onclick={() => (screening = ids())}>
		<Icon name="search" size={14} />{m.screening.action}
	</button>
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
		<button class="btn sm" onclick={removeFromCollection}>{m.projects.removeFromCollection}</button>
	{/if}
	<button class="btn ghost danger sm" onclick={removeDocuments}>
		{m.projects.removeDocuments}
	</button>
</div>

{#if screening}
	<ScreeningDialog {project} documents={screening} onclose={() => (screening = null)} />
{/if}

<style>
	.selection-bar select {
		min-height: var(--control-sm);
		font-size: var(--type-xs);
	}
	.grow {
		flex: 1;
	}
</style>
