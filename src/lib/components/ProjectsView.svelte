<script lang="ts">
	// Projects: the list of them, one project's documents, or one document with its results.
	import DocumentView from './DocumentView.svelte';
	import ProjectList from './ProjectList.svelte';
	import ProjectWorkspace from './ProjectWorkspace.svelte';

	let {
		projectId,
		collectionId,
		documentId,
		pane,
		selection
	}: {
		projectId?: string;
		collectionId?: string;
		documentId?: string;
		pane?: 'queue';
		selection?: string[];
	} = $props();
</script>

{#if !projectId}
	<ProjectList />
{:else if documentId}
	{#key documentId}
		<DocumentView {projectId} {documentId} />
	{/key}
{:else}
	{#key projectId}
		<ProjectWorkspace {projectId} {collectionId} opening={pane} {selection} />
	{/key}
{/if}
