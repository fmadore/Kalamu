<script lang="ts">
	// Workflows: the saved ones and their runs, one workflow being edited or run, or one run.
	import RunDetail from './RunDetail.svelte';
	import WorkflowEditor from './WorkflowEditor.svelte';
	import WorkflowList from './WorkflowList.svelte';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { FromProject } from '#lib/state.svelte.js';
	import type { Example } from '#lib/workflows.js';

	let {
		workflowId,
		example,
		runId,
		paths,
		sources,
		project
	}: {
		workflowId?: string;
		example?: Example;
		runId?: string;
		paths?: string[];
		sources?: SourceFile[];
		project?: FromProject;
	} = $props();
</script>

{#if runId}
	{#key runId}
		<RunDetail {runId} />
	{/key}
{:else if workflowId}
	{#key `${workflowId}/${example ?? ''}`}
		<WorkflowEditor {workflowId} {example} {paths} {sources} {project} />
	{/key}
{:else}
	<WorkflowList {paths} {sources} {project} />
{/if}
