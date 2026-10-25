<script lang="ts">
	import JobDetail from './JobDetail.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import type { JobStatus } from '#lib/bindings/JobStatus.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { app, go } from '#lib/state.svelte.js';

	let { jobId }: { jobId?: string } = $props();
	const m = messages();

	const unfinished = (s: JobStatus) => s === 'interrupted' || s === 'paused' || s === 'pending';
	const selected = $derived(jobId ?? app.jobs[0]?.id);
	const resumable = $derived(app.jobs.filter((j) => unfinished(j.status)));
</script>

<header class="masthead">
	<div>
		<h1 id="jobs-title">{m.jobs.title}</h1>
		<p class="lede">{m.jobs.lede(app.jobs.length)}</p>
	</div>
</header>

<div class="layout">
	<section class="list" aria-labelledby="jobs-title">
		{#if resumable.length > 0 && app.info?.interrupted.length}
			<div class="notice">
				<strong>{m.jobs.resumeTitle}</strong>
				<p class="small">{m.jobs.resumeHint}</p>
			</div>
		{/if}
		{#if app.jobs.length === 0}
			<p class="muted">{m.jobs.empty}</p>
		{/if}
		<ul>
			{#each app.jobs as job (job.id)}
				<li>
					<button
						class="job"
						class:active={job.id === selected}
						aria-current={job.id === selected ? 'true' : undefined}
						onclick={() => go({ name: 'jobs', jobId: job.id })}
					>
						<div class="top">
							<strong class="name">{job.name}</strong>
							<span
								class="chip"
								class:warn={unfinished(job.status)}
								class:ok={job.status === 'completed'}
								class:danger={job.status === 'completed_with_errors'}
							>
								{m.status[job.status]}
							</span>
						</div>
						<ProgressBar
							done={job.pieces_finished}
							failed={job.pieces_failed}
							total={Math.max(job.pieces_total, 1)}
							label={m.jobs.progress(job.pieces_finished, job.pieces_total)}
						/>
						<div class="meta muted">
							<span>{m.task[job.task]}</span>
							<span>{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span>
							{#if job.setup_count > 1}<span>{m.jobs.comparison}</span>{/if}
							<span>{formatDateTime(job.created_at)}</span>
						</div>
					</button>
				</li>
			{/each}
		</ul>
	</section>

	<section class="detail">
		{#if selected}
			{#key selected}
				<JobDetail jobId={selected} />
			{/key}
		{/if}
	</section>
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(18rem, 24rem) 1fr;
		gap: var(--space-6);
		align-items: start;
		padding: var(--space-5) var(--space-6) var(--space-7);
	}
	.list {
		display: grid;
		gap: var(--space-3);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		border-top: 1px solid var(--rule);
	}
	li {
		border-bottom: 1px solid var(--line);
	}
	.job {
		width: 100%;
		text-align: left;
		padding: var(--space-3);
		display: grid;
		gap: var(--space-2);
		cursor: pointer;
		border: 0;
		background: transparent;
		color: var(--text-body);
		transition: background-color 120ms ease-out;
	}
	.job:hover {
		background: var(--accent-wash);
	}
	.job.active {
		background: var(--tint);
	}
	.name {
		font-family: var(--font-text);
		font-size: 1.125rem;
		font-weight: 500;
		color: var(--text-bright);
	}
	.top {
		display: flex;
		justify-content: space-between;
		gap: var(--space-2);
		align-items: center;
	}
	.name {
		overflow-wrap: anywhere;
	}
	.meta {
		font-variant-numeric: tabular-nums;
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-3);
		font-size: var(--type-xs);
	}
	.notice {
		padding: var(--space-3);
		border: 1px solid var(--warn-border);
		background: var(--warn-wash);
		border-radius: var(--radius-control);
		display: grid;
		gap: var(--space-1);
	}
	.small {
		font-size: var(--type-sm);
	}
	.detail {
		position: sticky;
		top: 0;
	}
	@media (max-width: 1100px) {
		.layout {
			grid-template-columns: 1fr;
		}
		.detail {
			position: static;
		}
	}
	@media (max-width: 820px) {
		.layout {
			padding: var(--space-4) var(--space-4) var(--space-6);
			gap: var(--space-5);
		}
	}
</style>
