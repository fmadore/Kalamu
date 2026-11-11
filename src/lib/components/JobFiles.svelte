<script lang="ts">
	// A job's files as a table: each file's progress under every set-up, the errors met, its
	// error rate where pages were reviewed, and the way into its review. A long job's files are
	// drawn a page at a time; the store counts the pieces of the files drawn. The files with a
	// failure can be listed alone, so they are reached however far down a long job they sit.
	import Missing from './Missing.svelte';
	import MoreRows from './MoreRows.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { FileProgress } from '#lib/bindings/FileProgress.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { SourceFile } from '#lib/bindings/SourceFile.js';
	import type { JobProgress } from '#lib/bindings/JobProgress.js';
	import type { JobScores } from '#lib/bindings/JobScores.js';
	import type { SetupScore } from '#lib/bindings/SetupScore.js';
	import { messages } from '#lib/i18n/index.js';
	import type { Paging } from '#lib/paging.svelte.js';
	import { describeScore } from '#lib/scores.js';
	import { go } from '#lib/state.svelte.js';

	let {
		jobId,
		job,
		sources,
		failedCount,
		filtering,
		onfilter,
		progress,
		paging,
		scores
	}: {
		jobId: string;
		job: Job;
		/** The files listed, in the job's order: every one, or those with a failure. */
		sources: readonly SourceFile[];
		/** How many of the job's files have a failed piece, drawn or not. */
		failedCount: number;
		/** Whether only the files with a failure are listed. */
		filtering: boolean;
		onfilter: (onlyFailed: boolean) => void;
		/** The drawn files' pieces counted under each set-up, read again as progress arrives. */
		progress: JobProgress;
		/** How many of the job's files are drawn. */
		paging: Paging;
		/** Error rates against the pages reviewed so far, for tasks that produce text. */
		scores: JobScores | null;
	} = $props();
	const m = messages();

	const key = (sourceId: string, setupId: string) => `${sourceId}/${setupId}`;
	/** Each drawn file's counts under each set-up, looked up by row. */
	const counted = $derived(
		new Map<string, FileProgress>(progress.files.map((f) => [key(f.source_id, f.setup_id), f]))
	);
	/** Each document's score under each set-up. */
	const scored = $derived(
		new Map<string, SetupScore>(
			(scores?.sources ?? []).flatMap((s) =>
				s.scores.map((score) => [key(s.source_id, score.setup_id), score] as const)
			)
		)
	);
	const drawn = $derived(paging.of(sources));
	let list = $state<HTMLElement | null>(null);

	/** One set-up's score on one document, in a sentence. */
	function documentScore(sourceId: string, setupId: string): string | null {
		const score = scored.get(key(sourceId, setupId));
		return score ? describeScore(score, job.task, m) : null;
	}

	const NONE = { total: 0, done: 0, failed: 0, errors: [] as string[] };
</script>

<section aria-labelledby="files-head" class="files">
	<div class="files-head">
		<h2 id="files-head">{m.jobs.files}</h2>
		{#if failedCount > 0}
			<select
				value={filtering ? 'failed' : 'all'}
				onchange={(e) => onfilter(e.currentTarget.value === 'failed')}
				aria-label={m.jobs.filesShown}
			>
				<option value="all">{m.jobs.allFiles(job.sources.length)}</option>
				<option value="failed">{m.jobs.failedFiles(failedCount)}</option>
			</select>
		{/if}
	</div>
	<!-- Always there, so a screen reader hears the count when the list is filtered. -->
	<p class="sr-only" role="status">
		{filtering ? m.jobs.failedListed(sources.length, job.sources.length) : ''}
	</p>
	<div class="table" bind:this={list}>
		{#each drawn as source, i (source.id)}
			<div class="row" data-row={i}>
				<span class="file mono">{source.file_name}</span>
				<div class="progress">
					{#each job.setups as setup, i (setup.id)}
						{@const c = counted.get(key(source.id, setup.id)) ?? NONE}
						<div class="setup-progress">
							{#if job.setups.length > 1}<i class="siglum" title={setup.label}>{siglum(i)}</i>{/if}
							<ProgressBar
								done={c.done}
								failed={c.failed}
								total={Math.max(c.total, 1)}
								label="{source.file_name} {setup.label}"
							/>
							<span class="count mono"
								>{#if c.total > 0}{c.done}/{c.total}{:else}<Missing />{/if}</span
							>
						</div>
						{#each c.errors as e (e)}
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
	<MoreRows {paging} total={sources.length} table={list} />
</section>

<style>
	.files {
		display: grid;
		gap: var(--space-3);
	}
	.files-head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
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
