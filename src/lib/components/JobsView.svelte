<script lang="ts">
	// One job at a time: the sidebar lists them, this shows the one chosen (the newest by default).
	import Icon from './Icon.svelte';
	import JobDetail from './JobDetail.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { app, go } from '#lib/state.svelte.js';

	let { jobId }: { jobId?: string } = $props();
	const m = messages();

	const selected = $derived(jobId ?? app.jobs[0]?.id);
</script>

{#if selected}
	{#key selected}
		<JobDetail jobId={selected} />
	{/key}
{:else}
	<header class="masthead">
		<div><h1>{m.jobs.title}</h1></div>
	</header>
	<div class="empty">
		<p class="muted">{m.jobs.empty}</p>
		<button class="btn primary" onclick={() => go({ name: 'new' })}
			><Icon name="plus" />{m.jobs.empty_new}</button
		>
	</div>
{/if}

<style>
	.empty {
		display: grid;
		justify-items: start;
		gap: var(--space-3);
		padding: var(--space-6);
	}
</style>
