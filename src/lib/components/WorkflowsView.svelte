<script lang="ts">
	// Workflows: the saved ones and their runs, one workflow being edited or run, or one run.
	import RunDetail from './RunDetail.svelte';
	import WorkflowEditor from './WorkflowEditor.svelte';
	import WorkflowList from './WorkflowList.svelte';
	import type { FromProject } from '#lib/state.svelte.js';

	let {
		workflowId,
		runId,
		paths,
		project
	}: { workflowId?: string; runId?: string; paths?: string[]; project?: FromProject } = $props();
</script>

{#if runId}
	{#key runId}
		<RunDetail {runId} />
	{/key}
{:else if workflowId}
	{#key workflowId}
		<WorkflowEditor {workflowId} {paths} {project} />
	{/key}
{:else}
	<WorkflowList {paths} {project} />
{/if}
