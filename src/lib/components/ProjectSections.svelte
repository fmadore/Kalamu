<script lang="ts">
	// A project's sections down the side of its screen: all its documents, each collection and a
	// new one named in place, then its search, what needs checking, its dictionary and its
	// activity. The one shown reads as the current page.
	import Icon from './Icon.svelte';
	import type { ProjectView } from '#lib/bindings/ProjectView.js';
	import { messages } from '#lib/i18n/index.js';
	import type { ProjectPane } from '#lib/state.svelte.js';

	let {
		view,
		pane,
		collectionId,
		naming = $bindable(),
		onshow,
		onpane,
		onname
	}: {
		view: ProjectView;
		pane: ProjectPane;
		collectionId?: string;
		/** The collection being named, null for a new one; shared with the rename in the toolbar. */
		naming: null | { id: string | null; name: string };
		/** Shows the documents of collection `id`, or all of them. */
		onshow: (id?: string) => void;
		onpane: (to: ProjectPane) => void;
		/** Saves the collection's name typed. */
		onname: (e: SubmitEvent) => void;
	} = $props();
	const m = messages();
</script>

<nav class="sections" aria-label={m.projects.collections}>
	<button
		class="section"
		class:current={pane === 'documents' && !collectionId}
		aria-current={pane === 'documents' && !collectionId ? 'page' : undefined}
		onclick={() => onshow()}
	>
		<Icon name="project" size={14} /><span class="grow">{m.projects.allDocuments}</span>
		<span class="num">{view.documents.length}</span>
	</button>

	<h2 class="group-head">{m.projects.collections}</h2>
	{#each view.collections as c (c.collection.id)}
		<button
			class="section"
			class:current={pane === 'documents' && collectionId === c.collection.id}
			aria-current={pane === 'documents' && collectionId === c.collection.id ? 'page' : undefined}
			onclick={() => onshow(c.collection.id)}
		>
			<Icon name="collection" size={14} /><span class="grow ellipsis">{c.collection.name}</span>
			<span class="num">{c.documents}</span>
		</button>
	{/each}
	{#if naming && !naming.id}
		<form class="naming" onsubmit={onname}>
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
		class="section search"
		class:current={pane === 'search'}
		aria-current={pane === 'search' ? 'page' : undefined}
		onclick={() => onpane('search')}
	>
		<Icon name="search" size={14} /><span class="grow">{m.search.title}</span>
	</button>
	<button
		class="section"
		class:current={pane === 'queue'}
		aria-current={pane === 'queue' ? 'page' : undefined}
		onclick={() => onpane('queue')}
	>
		<Icon name="flag" size={14} /><span class="grow">{m.queue.title}</span>
	</button>
	<button
		class="section"
		class:current={pane === 'dictionary'}
		aria-current={pane === 'dictionary' ? 'page' : undefined}
		onclick={() => onpane('dictionary')}
	>
		<Icon name="glossary" size={14} /><span class="grow">{m.dictionary.title}</span>
	</button>
	<button
		class="section activity"
		class:current={pane === 'activity'}
		aria-current={pane === 'activity' ? 'page' : undefined}
		onclick={() => onpane('activity')}
	>
		<Icon name="list" size={14} /><span class="grow">{m.projects.activity}</span>
		<span class="num">{view.jobs.length}</span>
	</button>
</nav>

<style>
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
	.section.search {
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
	.naming {
		padding: var(--space-1) 0;
	}
	.naming input {
		width: 100%;
	}
	@media (max-width: 900px) {
		/* The sections wrap onto a second line rather than scroll sideways. */
		.sections {
			display: flex;
			flex-wrap: wrap;
			border-right: 0;
			border-bottom: 1px solid var(--line);
			padding: var(--space-2) var(--space-4);
		}
		.group-head {
			display: none;
		}
		.section.search {
			margin-top: 0;
		}
	}
</style>
