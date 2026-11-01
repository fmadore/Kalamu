<script lang="ts">
	// The jobs that read a project's documents, newest first, each opening its own page where it
	// is paused, resumed, retried or deleted as any job is. Filters narrow them by task, state,
	// model, workflow and date, and the jobs shown are costed together.
	import Icon from './Icon.svelte';
	import {
		activityChoices,
		filterActivity,
		isFiltered,
		openFilter,
		summedCost,
		type ActivityFilter,
		type Period
	} from '#lib/activity.js';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { app, go } from '#lib/state.svelte.js';

	let { jobs }: { jobs: string[] } = $props();
	const m = messages();
	const periods: Period[] = ['any', 'week', 'month', 'year'];

	let filter = $state<ActivityFilter>({ ...openFilter });

	const listed = $derived(
		jobs.map((id) => app.jobs.find((j) => j.id === id)).filter((j) => j !== undefined)
	);
	const choices = $derived(activityChoices(listed));
	const shown = $derived(filterActivity(listed, filter));
	const cost = $derived(summedCost(shown));
</script>

<div class="activity">
	<h2>{m.projects.activity}</h2>
	<p class="muted">{m.projects.activityLede}</p>
	{#if listed.length === 0}
		<p class="muted">{m.projects.noActivity}</p>
	{:else}
		<div class="filters" role="group" aria-label={m.projects.activityFilters}>
			<select bind:value={filter.task} aria-label={m.projects.filterBy.task}>
				<option value="">{m.projects.anyTask}</option>
				{#each choices.tasks as task (task)}
					<option value={task}>{m.task[task]}</option>
				{/each}
			</select>
			<select bind:value={filter.status} aria-label={m.projects.filterBy.status}>
				<option value="">{m.projects.anyStatus}</option>
				{#each choices.statuses as status (status)}
					<option value={status}>{m.status[status]}</option>
				{/each}
			</select>
			<select bind:value={filter.model} aria-label={m.projects.filterBy.model}>
				<option value="">{m.projects.anyModel}</option>
				{#each choices.models as model (model)}
					<option value={model}>{model}</option>
				{/each}
			</select>
			{#if choices.workflows.length > 0}
				<select bind:value={filter.workflow} aria-label={m.projects.filterBy.workflow}>
					<option value="">{m.projects.anyWorkflow}</option>
					{#each choices.workflows as workflow (workflow)}
						<option value={workflow}>{workflow}</option>
					{/each}
				</select>
			{/if}
			<select bind:value={filter.period} aria-label={m.projects.filterBy.period}>
				{#each periods as period (period)}
					<option value={period}>{m.projects.period[period]}</option>
				{/each}
			</select>
			{#if isFiltered(filter)}
				<button class="clear" onclick={() => (filter = { ...openFilter })}
					>{m.projects.clearFilters}</button
				>
			{/if}
		</div>

		<p class="summary num" aria-live="polite">
			{m.projects.activityShown(shown.length, listed.length)}
			{#if cost.total !== null}· {m.projects.activityCost(formatUsd(cost.total))}{/if}
			{#if cost.unpriced > 0 && cost.unpriced < shown.length}
				· {m.projects.activityUnpriced(cost.unpriced)}
			{/if}
		</p>

		{#if shown.length === 0}
			<p class="muted">{m.projects.noActivityMatch}</p>
		{:else}
			<ul>
				{#each shown as job (job.id)}
					<li>
						<button class="job" onclick={() => go({ name: 'jobs', jobId: job.id })}>
							<span class="name"
								><Icon name={job.task} size={14} /><span class="ellipsis">{job.name}</span></span
							>
							<span class="meta num">
								{m.task[job.task]} · {m.status[job.status]} ·
								{m.jobs.progress(job.pieces_finished, job.pieces_total)}
								{#if job.workflow !== null}· {job.workflow}{/if}
								{#if job.cost_usd !== null}· {formatUsd(job.cost_usd)}{/if}
								· {formatDateTime(job.created_at)}
							</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	{/if}
</div>

<style>
	.activity {
		display: grid;
		gap: var(--space-2);
	}
	.filters {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
		font-size: var(--type-sm);
	}
	.clear {
		border: 0;
		background: none;
		color: var(--text-secondary);
		text-decoration: underline;
		cursor: pointer;
	}
	.summary {
		margin: 0;
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	ul {
		list-style: none;
		margin: var(--space-2) 0 0;
		padding: 0;
		border-top: 1px solid var(--line);
	}
	.job {
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
	.job:hover {
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
