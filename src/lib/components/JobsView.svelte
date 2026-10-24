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

<div class="layout">
	<section class="list" aria-labelledby="jobs-title">
		<h1 id="jobs-title">{m.jobs.title}</h1>
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
						class="job card"
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
		gap: var(--space-5);
		align-items: start;
	}
	.list {
		display: grid;
		gap: var(--space-3);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	.job {
		width: 100%;
		text-align: left;
		padding: var(--space-3);
		display: grid;
		gap: var(--space-2);
		cursor: pointer;
	}
	.job.active {
		border-color: var(--accent-border);
		box-shadow: inset 0 0 0 1px var(--accent-border);
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
</style>
