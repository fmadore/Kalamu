<script lang="ts">
	// A job's facts as one strip, cells divided by hairlines: its task, progress, models, error
	// rates where pages were reviewed, pace, cost and date.
	import { siglum } from '#lib/apparatus.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { JobScores } from '#lib/bindings/JobScores.js';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { describeScore } from '#lib/scores.js';

	let {
		job,
		summary,
		spent,
		scores
	}: {
		job: Job;
		summary: JobSummary;
		/** What the job has cost over all its runs. */
		spent: number;
		/** Error rates against the pages reviewed so far, for tasks that produce text. */
		scores: JobScores | null;
	} = $props();
	const m = messages();

	/** Each set-up's score over the whole job, in a sentence, where pages were reviewed. */
	const totals = $derived(scores ? scores.total.map((s) => describeScore(s, job.task, m)) : []);
</script>

<dl class="facts">
	<div>
		<dt>{m.jobs.task}</dt>
		<dd>{m.task[job.task]}</dd>
	</div>
	<div>
		<dt>{m.jobs.pieces}</dt>
		<dd class="mono">
			{m.jobs.progress(summary.pieces_finished, summary.pieces_total)}
			{#if summary.pieces_failed > 0}<span class="fail"
					>· {m.jobs.failed(summary.pieces_failed)}</span
				>{/if}
			{#if summary.pieces_truncated > 0}<span class="cut" title={m.jobs.truncatedNote}
					>· {m.jobs.truncated(summary.pieces_truncated)}</span
				>{/if}
		</dd>
	</div>
	<div class="wide">
		<dt>{m.jobs.models}</dt>
		<dd class="models">
			{#each job.setups as s, i (s.id)}
				<span
					>{#if job.setups.length > 1}<i class="siglum">{siglum(i)}</i>{/if}<span class="mono"
						>{s.model}</span
					></span
				>
			{/each}
		</dd>
	</div>
	{#if totals.some((t) => t !== null)}
		<div class="wide">
			<dt>{m.scores.title}</dt>
			<dd class="scores">
				{#each totals as total, i (i)}
					{#if total}
						<span
							>{#if job.setups.length > 1}<i class="siglum">{siglum(i)}</i>{/if}{total}</span
						>
					{/if}
				{/each}
				<span class="note">{m.scores.note}</span>
			</dd>
		</div>
	{/if}
	{#if job.settings.pace === 'economy'}
		<div>
			<dt>{m.jobs.pace}</dt>
			<dd>{m.jobs.paceName.economy}</dd>
		</div>
	{/if}
	{#if spent > 0}
		<div>
			<dt>{m.jobs.cost}</dt>
			<dd class="mono">{formatUsd(spent)}</dd>
		</div>
	{/if}
	<div>
		<dt>{m.jobs.created}</dt>
		<dd>{formatDateTime(summary.created_at)}</dd>
	</div>
</dl>

<style>
	.facts {
		display: flex;
		flex-wrap: wrap;
		margin: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
	}
	.facts > div {
		flex: 1 1 10rem;
		display: grid;
		gap: var(--space-1);
		align-content: start;
		padding: var(--space-3) var(--space-4);
		border-left: 1px solid var(--line);
		margin-left: -1px;
	}
	.facts > .wide {
		flex: 2 1 14rem;
	}
	dt {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	dd {
		margin: 0;
		color: var(--text-bright);
		font-weight: 500;
	}
	.models {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-3);
	}
	.models > span {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	/* Each set-up's error rate as a sentence, then how it was counted. */
	.scores {
		display: grid;
		gap: var(--space-1);
	}
	.scores > span {
		display: inline-flex;
		align-items: baseline;
		gap: var(--space-2);
	}
	.scores > .note {
		font-size: var(--type-xs);
		font-weight: 400;
		color: var(--text-muted);
	}
	.fail {
		color: var(--danger-text);
	}
	.cut {
		color: var(--warn-text);
	}
</style>
