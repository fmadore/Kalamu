<script lang="ts">
	import { onMount } from 'svelte';
	import ProgressBar from './ProgressBar.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PieceState } from '#lib/bindings/PieceState.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { api, ask, revealInFolder } from '#lib/tauri.js';
	import { app, go, liveJob, refreshJobs } from '#lib/state.svelte.js';

	let { jobId }: { jobId: string } = $props();
	const m = messages();

	let job = $state<Job | null>(null);
	let pieces = $state<PieceState[]>([]);
	let error = $state<string | null>(null);

	const summary = $derived(app.jobs.find((j) => j.id === jobId));
	const live = $derived(app.live[jobId]);
	const running = $derived(summary?.status === 'running');

	async function load() {
		try {
			job = await api.getJob(jobId);
			pieces = await api.jobPieces(jobId);
		} catch (e) {
			error = describeError(e, m);
		}
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

	function counts(setupId: string, sourceId: string) {
		const mine = pieces.filter((p) => p.setup_id === setupId && p.source_id === sourceId);
		return {
			total: mine.length,
			done: mine.filter((p) => p.status === 'done' || p.status === 'cached').length,
			failed: mine.filter((p) => p.status === 'failed').length,
			errors: [...new Set(mine.flatMap((p) => (p.error ? [p.error] : [])))]
		};
	}

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

	/** A new summary job over the text this job wrote. */
	async function summarise() {
		error = null;
		try {
			const paths = await api.jobTextOutputs(jobId);
			if (paths.length === 0) {
				error = m.jobs.noTextOutputs;
				return;
			}
			go({ name: 'new', task: 'summary', paths });
		} catch (e) {
			error = describeError(e, m);
		}
	}

	onMount(() => {
		void load();
	});
</script>

{#if job && summary}
	<div class="detail card">
		<header>
			<div>
				<h2>{job.name}</h2>
				<p class="muted small meta">
					<span>{m.task[job.task]}</span>
					{#each job.setups as s, i (s.id)}
						<span
							>{#if job.setups.length > 1}<i class="siglum">{siglum(i)}</i>
							{/if}{s.model}</span
						>
					{/each}
					<span>{m.status[summary.status]}</span>
				</p>
			</div>
			<div class="actions">
				{#if running}
					<button class="btn" onclick={() => act(() => api.pauseJob(jobId))}>{m.jobs.pause}</button>
					<button class="btn danger" onclick={() => act(() => api.cancelJob(jobId))}
						>{m.jobs.cancel}</button
					>
				{:else}
					{#if summary.status !== 'completed'}
						<button class="btn primary" onclick={() => act(() => api.startJob(jobId))}
							>{m.jobs.resume}</button
						>
					{/if}
					{#if summary.pieces_failed > 0}
						<button class="btn" onclick={() => act(() => api.retryFailed(jobId))}
							>{m.jobs.retry}</button
						>
					{/if}
					{#if summary.status === 'completed' && summary.task !== 'summary'}
						<button class="btn" onclick={summarise}>{m.jobs.summarise}</button>
					{/if}
					<button class="btn ghost danger" onclick={remove}>{m.jobs.delete}</button>
				{/if}
			</div>
		</header>

		<div class="overall">
			<ProgressBar
				done={summary.pieces_finished}
				failed={summary.pieces_failed}
				total={Math.max(summary.pieces_total, 1)}
				label={m.jobs.progress(summary.pieces_finished, summary.pieces_total)}
			/>
			<div class="row muted small">
				<span>{m.jobs.progress(summary.pieces_finished, summary.pieces_total)}</span>
				{#if summary.pieces_failed > 0}<span class="fail"
						>{m.jobs.failed(summary.pieces_failed)}</span
					>{/if}
				{#if live && live.spent > 0}<span>{m.jobs.spent(formatUsd(live.spent))}</span>{/if}
			</div>
		</div>

		{#if error}<p class="chip danger">{error}</p>{/if}

		<h3>{m.jobs.files}</h3>
		<ul class="sources">
			{#each job.sources as source (source.id)}
				<li>
					<div class="row between">
						<strong class="file">{source.file_name}</strong>
						<button
							class="btn sm"
							onclick={() => go({ name: 'review', jobId, sourceId: source.id })}
						>
							{m.jobs.review}
						</button>
					</div>
					{#each job.setups as setup, i (setup.id)}
						{@const c = counts(setup.id, source.id)}
						<div class="setup-progress">
							<span class="small label-cell"
								>{#if job.setups.length > 1}<i class="siglum">{siglum(i)}</i>
									{setup.label}{/if}</span
							>
							<ProgressBar
								done={c.done}
								failed={c.failed}
								total={Math.max(c.total, 1)}
								label="{source.file_name} {setup.label}"
							/>
							<span class="muted small num">{c.total > 0 ? `${c.done}/${c.total}` : '—'}</span>
						</div>
						{#each c.errors.slice(0, 3) as e (e)}
							<p class="err small">{e}</p>
						{/each}
					{/each}
				</li>
			{/each}
		</ul>

		{#if live?.preview}
			<h3>{m.jobs.livePreview}</h3>
			<pre class="preview">{live.preview}</pre>
		{/if}

		{#if live && live.written.length > 0}
			<h3>{m.jobs.outputs}</h3>
			<ul class="written">
				{#each [...new Set(live.written)] as path (path)}
					<li>
						<button class="link small" onclick={() => revealInFolder(path)}
							>{path.split(/[\\/]/).pop()}</button
						>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
{/if}

<style>
	.detail {
		padding: var(--space-4) 0 0;
		display: grid;
		gap: var(--space-4);
	}
	header {
		display: flex;
		justify-content: space-between;
		gap: var(--space-4);
		align-items: start;
	}
	.actions {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
		justify-content: flex-end;
	}
	.overall {
		display: grid;
		gap: var(--space-2);
	}
	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.between {
		justify-content: space-between;
	}
	.small {
		font-size: var(--type-sm);
	}
	.fail,
	.err {
		color: var(--danger);
	}
	.err {
		margin-left: var(--space-2);
	}
	.sources,
	.written {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	.sources li {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
	}
	.file {
		overflow-wrap: anywhere;
	}
	.setup-progress {
		display: grid;
		grid-template-columns: 11rem 1fr 3rem;
		gap: var(--space-3);
		align-items: center;
	}
	.label-cell {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 600;
	}
	/* Task, set-ups and status, separated by middle dots. */
	.meta {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		column-gap: var(--space-2);
	}
	.meta > span + span::before {
		content: '·';
		margin-right: var(--space-2);
	}
	.meta .siglum {
		margin-right: 0.25em;
	}
	.preview {
		margin: 0;
		padding: var(--space-3);
		background: var(--paper);
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
		font-family: var(--font-text);
		white-space: pre-wrap;
		max-height: 14rem;
		overflow: auto;
	}
	.link {
		border: 0;
		background: none;
		color: var(--accent-soft);
		cursor: pointer;
		padding: 0;
		text-decoration: underline;
	}
</style>
