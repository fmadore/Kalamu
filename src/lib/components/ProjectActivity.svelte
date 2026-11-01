<script lang="ts">
	// The jobs that read a project's documents, newest first, each opening its own page where it
	// is paused, resumed, retried or deleted as any job is.
	import Icon from './Icon.svelte';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { app, go } from '#lib/state.svelte.js';

	let { jobs }: { jobs: string[] } = $props();
	const m = messages();

	const listed = $derived(
		jobs.map((id) => app.jobs.find((j) => j.id === id)).filter((j) => j !== undefined)
	);
</script>

<div class="activity">
	<h2>{m.projects.activity}</h2>
	<p class="muted">{m.projects.activityLede}</p>
	{#if listed.length === 0}
		<p class="muted">{m.projects.noActivity}</p>
	{:else}
		<ul>
			{#each listed as job (job.id)}
				<li>
					<button onclick={() => go({ name: 'jobs', jobId: job.id })}>
						<span class="name"
							><Icon name={job.task} size={14} /><span class="ellipsis">{job.name}</span></span
						>
						<span class="meta num">
							{m.task[job.task]} · {m.status[job.status]} ·
							{m.jobs.progress(job.pieces_finished, job.pieces_total)}
							{#if job.cost_usd !== null}· {formatUsd(job.cost_usd)}{/if}
							· {formatDateTime(job.created_at)}
						</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.activity {
		display: grid;
		gap: var(--space-2);
	}
	ul {
		list-style: none;
		margin: var(--space-2) 0 0;
		padding: 0;
		border-top: 1px solid var(--line);
	}
	button {
		display: grid;
		gap: 2px;
		width: 100%;
		padding: var(--space-2);
		border: 0;
		border-bottom: 1px solid var(--line-soft);
		background: none;
		text-align: left;
		cursor: pointer;
	}
	button:hover {
		background: var(--wash);
	}
	.name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-bright);
		font-weight: 500;
		min-width: 0;
	}
	.ellipsis {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.meta {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
</style>
