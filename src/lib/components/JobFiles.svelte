<script lang="ts">
	// A job's files as a table: each file's progress under every set-up, the errors met, its
	// error rate where pages were reviewed, and the way into its review.
	import ProgressBar from './ProgressBar.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { JobScores } from '#lib/bindings/JobScores.js';
	import type { PieceState } from '#lib/bindings/PieceState.js';
	import { messages } from '#lib/i18n/index.js';
	import { describeScore } from '#lib/scores.js';
	import { go } from '#lib/state.svelte.js';

	let {
		jobId,
		job,
		pieces,
		scores
	}: {
		jobId: string;
		job: Job;
		/** Every piece's state, read again as progress arrives. */
		pieces: PieceState[];
		/** Error rates against the pages reviewed so far, for tasks that produce text. */
		scores: JobScores | null;
	} = $props();
	const m = messages();

	/** One set-up's score on one document, in a sentence. */
	function documentScore(sourceId: string, setupId: string): string | null {
		const score = scores?.sources
			.find((s) => s.source_id === sourceId)
			?.scores.find((s) => s.setup_id === setupId);
		return score ? describeScore(score, job.task, m) : null;
	}

	function counts(setupId: string, sourceId: string) {
		const mine = pieces.filter((p) => p.setup_id === setupId && p.source_id === sourceId);
		return {
			total: mine.length,
			done: mine.filter((p) => p.status === 'done' || p.status === 'cached').length,
			failed: mine.filter((p) => p.status === 'failed').length,
			errors: [...new Set(mine.flatMap((p) => (p.error ? [p.error] : [])))]
		};
	}
</script>

<section aria-labelledby="files-head" class="files">
	<h2 id="files-head">{m.jobs.files}</h2>
	<div class="table">
		{#each job.sources as source (source.id)}
			<div class="row">
				<span class="file mono">{source.file_name}</span>
				<div class="progress">
					{#each job.setups as setup, i (setup.id)}
						{@const c = counts(setup.id, source.id)}
						<div class="setup-progress">
							{#if job.setups.length > 1}<i class="siglum" title={setup.label}>{siglum(i)}</i>{/if}
							<ProgressBar
								done={c.done}
								failed={c.failed}
								total={Math.max(c.total, 1)}
								label="{source.file_name} {setup.label}"
							/>
							<span class="count mono">{c.total > 0 ? `${c.done}/${c.total}` : '—'}</span>
						</div>
						{#each c.errors.slice(0, 3) as e (e)}
							<p class="err">{e}</p>
						{/each}
						{@const score = documentScore(source.id, setup.id)}
						{#if score}<p class="doc-score">{score}</p>{/if}
					{/each}
				</div>
				<button class="btn sm" onclick={() => go({ name: 'review', jobId, sourceId: source.id })}>
					{m.jobs.review}
				</button>
			</div>
		{/each}
	</div>
</section>

<style>
	.files {
		display: grid;
		gap: var(--space-3);
	}
	.table {
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		overflow: hidden;
	}
	.row {
		display: grid;
		grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr) auto;
		gap: var(--space-3) var(--space-5);
		align-items: center;
		padding: var(--space-3) var(--space-4);
		border-top: 1px solid var(--line-soft);
	}
	.row:first-child {
		border-top: 0;
	}
	.file {
		overflow-wrap: anywhere;
		color: var(--text-bright);
	}
	.progress {
		display: grid;
		gap: var(--space-2);
	}
	.setup-progress {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) 3.5rem;
		gap: var(--space-3);
		align-items: center;
	}
	.setup-progress:not(:has(.siglum)) {
		grid-template-columns: minmax(0, 1fr) 3.5rem;
	}
	.count {
		color: var(--text-muted);
		text-align: right;
	}
	.err {
		color: var(--danger-text);
		font-size: var(--type-xs);
	}
	.doc-score {
		margin: 0;
		font-size: var(--type-xs);
		font-weight: 400;
		color: var(--text-muted);
	}
	@media (max-width: 820px) {
		.row {
			grid-template-columns: minmax(0, 1fr);
		}
		.row .btn {
			justify-self: start;
		}
	}
</style>
