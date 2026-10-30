<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import JobFacts from './JobFacts.svelte';
	import JobFiles from './JobFiles.svelte';
	import JobOutputs from './JobOutputs.svelte';
	import type { Job } from '#lib/bindings/Job.js';
	import type { JobScores } from '#lib/bindings/JobScores.js';
	import type { PieceState } from '#lib/bindings/PieceState.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { scoredTask } from '#lib/scores.js';
	import { api, ask } from '#lib/tauri.js';
	import { app, go, liveJob, refreshJobs } from '#lib/state.svelte.js';

	let { jobId }: { jobId: string } = $props();
	const m = messages();

	let job = $state<Job | null>(null);
	let pieces = $state<PieceState[]>([]);
	let error = $state<string | null>(null);
	/** The files the job wrote that are still on disk. */
	let written = $state<string[]>([]);
	/** The name being typed while the job is renamed. */
	let draftName = $state<string | null>(null);
	/** Error rates against the pages reviewed so far, for tasks that produce text. */
	let scores = $state<JobScores | null>(null);

	const summary = $derived(app.jobs.find((j) => j.id === jobId));
	const live = $derived(app.live[jobId]);
	const running = $derived(summary?.status === 'running');
	const stopped = $derived(summary?.status === 'interrupted' || summary?.status === 'paused');
	/** What the job has cost over all its runs: live while it runs, stored otherwise. */
	const spent = $derived(running && live ? live.spent : (summary?.cost_usd ?? 0));
	/** Pieces the provider's batch service holds, in economy mode. */
	const held = $derived(pieces.filter((p) => p.status === 'submitted').length);
	/** When the last of the batches is due, as the engine last reported. */
	const dueBy = $derived(Math.max(0, ...Object.values(live?.batches ?? {}).map((b) => b.dueBy)));

	async function load() {
		try {
			job = await api.getJob(jobId);
			pieces = await api.jobPieces(jobId);
		} catch (e) {
			error = describeError(e, m);
		}
		// Scores are an extra: the page stands without them.
		if (job && scoredTask(job.task)) {
			scores = await api.jobScores(jobId).catch(() => null);
		}
	}

	// The written files, read again whenever the job reports new ones.
	$effect(() => {
		void live?.written.length;
		void api
			.jobOutputs(jobId)
			.then((paths) => (written = paths ?? []))
			.catch(() => undefined);
	});

	async function open(path: string) {
		error = null;
		try {
			await api.openOutput(jobId, path);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function rename() {
		const name = draftName?.trim();
		draftName = null;
		if (!name || !job || name === job.name) return;
		try {
			await api.renameJob(jobId, name);
			job.name = name;
			await refreshJobs();
		} catch (e) {
			error = describeError(e, m);
		}
	}

	function renameKeys(e: KeyboardEvent) {
		if (e.key === 'Enter') void rename();
		else if (e.key === 'Escape') draftName = null;
	}

	// Reload piece states as progress arrives, at most twice a second.
	let timer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		void summary?.pieces_finished;
		void summary?.pieces_failed;
		void summary?.status;
		if (timer) return;
		timer = setTimeout(() => {
			timer = null;
			void api
				.jobPieces(jobId)
				.then((p) => (pieces = p))
				.catch(() => undefined);
		}, 500);
	});

	async function act(action: () => Promise<void>) {
		error = null;
		try {
			liveJob(jobId);
			await action();
			await refreshJobs();
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function remove() {
		if (!(await ask(m.jobs.deleteConfirm))) return;
		await act(() => api.deleteJob(jobId));
		go({ name: 'jobs' });
	}

	/** A new analysis job over the text this job wrote. */
	async function summarise() {
		error = null;
		try {
			const paths = await api.jobTextOutputs(jobId);
			if (paths.length === 0) {
				error = m.jobs.noTextOutputs;
				return;
			}
			go({ name: 'new', task: 'analysis', paths });
		} catch (e) {
			error = describeError(e, m);
		}
	}

	onMount(() => {
		void load();
	});
</script>

{#if job && summary}
	<header class="masthead">
		<div>
			{#if draftName !== null}
				<!-- svelte-ignore a11y_autofocus -->
				<input
					class="rename"
					type="text"
					aria-label={m.jobs.nameLabel}
					bind:value={draftName}
					onkeydown={renameKeys}
					onblur={rename}
					autofocus
				/>
			{:else}
				<h1>{job.name}</h1>
				<button
					class="btn ghost sm icon"
					title={m.jobs.rename}
					aria-label={m.jobs.rename}
					onclick={() => (draftName = job?.name ?? '')}><Icon name="edit" size={14} /></button
				>
			{/if}
			<span
				class="chip"
				class:warn={stopped || summary.status === 'pending'}
				class:ok={summary.status === 'completed'}
				class:danger={summary.status === 'completed_with_errors'}
				class:hand={running}>{m.status[summary.status]}</span
			>
		</div>
		<div class="actions">
			{#if running}
				<button class="btn" onclick={() => act(() => api.pauseJob(jobId))}
					><Icon name="pause" size={12} />{m.jobs.pause}</button
				>
				<button class="btn danger" onclick={() => act(() => api.cancelJob(jobId))}
					>{m.jobs.cancel}</button
				>
			{:else}
				<button class="btn ghost danger" onclick={remove}>{m.jobs.delete}</button>
				{#if summary.pieces_failed > 0}
					<button class="btn" onclick={() => act(() => api.retryFailed(jobId))}
						>{m.jobs.retry}</button
					>
				{/if}
				{#if summary.status === 'completed' && summary.task !== 'analysis'}
					<button class="btn" onclick={summarise}>{m.jobs.summarise}</button>
				{/if}
				{#if summary.status !== 'completed'}
					<button class="btn primary" onclick={() => act(() => api.startJob(jobId))}
						><Icon name="play" size={12} />{m.jobs.resume}</button
					>
				{/if}
			{/if}
		</div>
	</header>

	{#if held > 0 && running && dueBy > 0}
		<p class="banner calm" role="status">{m.jobs.waitingNotice(held, formatDateTime(dueBy))}</p>
	{:else if held > 0 && !running}
		<p class="banner" role="status">{m.jobs.heldNotice(held)}</p>
	{:else if stopped}
		<p class="banner" role="status">
			{summary.status === 'paused' ? m.jobs.pausedNotice : m.jobs.interruptedNotice}
		</p>
	{/if}

	<div class="body">
		{#if error}<p class="chip danger">{error}</p>{/if}

		<JobFacts {job} {summary} {spent} {scores} />

		<JobFiles {jobId} {job} {pieces} {scores} />

		{#if live?.preview}
			<section aria-labelledby="preview-head" class="files">
				<h2 id="preview-head">{m.jobs.livePreview}</h2>
				<pre class="preview">{live.preview}</pre>
			</section>
		{/if}

		{#if written.length > 0}
			<JobOutputs {written} onopen={open} />
		{/if}
	</div>
{/if}

<style>
	.actions {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
		justify-content: flex-end;
	}
	.banner {
		padding: var(--space-3) var(--space-5);
		background: var(--warn-wash);
		border-bottom: 1px solid var(--warn-border);
		color: var(--warn-text);
		font-size: var(--type-sm);
	}
	.banner.calm {
		background: var(--surface-1);
		border-bottom-color: var(--line);
		color: var(--text-body);
	}
	.body {
		display: grid;
		gap: var(--space-6);
		max-width: 72rem;
		padding: var(--space-5) var(--space-5) var(--space-7);
	}
	.files {
		display: grid;
		gap: var(--space-3);
	}
	.preview {
		margin: 0;
		padding: var(--space-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.6;
		white-space: pre-wrap;
		max-height: 16rem;
		overflow: auto;
	}
	/* The job's name being typed, in place of the title and at its size. */
	.rename {
		min-width: min(28rem, 100%);
		font-size: var(--type-xl);
		font-weight: 600;
	}
	@media (max-width: 820px) {
		.body {
			padding: var(--space-4);
		}
	}
</style>
