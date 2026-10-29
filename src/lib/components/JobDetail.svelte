<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { JobScores } from '#lib/bindings/JobScores.js';
	import type { PieceState } from '#lib/bindings/PieceState.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { describeScore, scoredTask } from '#lib/scores.js';
	import { api, ask, revealInFolder } from '#lib/tauri.js';
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

	/** Each set-up's score over the whole job, in a sentence, where pages were reviewed. */
	const totals = $derived(
		job && scores
			? scores.total.map((s) => describeScore(s, job?.task ?? 'text_extraction', m))
			: []
	);

	/** One set-up's score on one document, in a sentence. */
	function documentScore(sourceId: string, setupId: string): string | null {
		const score = scores?.sources
			.find((s) => s.source_id === sourceId)
			?.scores.find((s) => s.setup_id === setupId);
		return score && job ? describeScore(score, job.task, m) : null;
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
									{#if job.setups.length > 1}<i class="siglum" title={setup.label}>{siglum(i)}</i
										>{/if}
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
						<button
							class="btn sm"
							onclick={() => go({ name: 'review', jobId, sourceId: source.id })}
						>
							{m.jobs.review}
						</button>
					</div>
				{/each}
			</div>
		</section>

		{#if live?.preview}
			<section aria-labelledby="preview-head" class="files">
				<h2 id="preview-head">{m.jobs.livePreview}</h2>
				<pre class="preview">{live.preview}</pre>
			</section>
		{/if}

		{#if written.length > 0}
			<section aria-labelledby="written-head" class="files">
				<h2 id="written-head">{m.jobs.outputs}</h2>
				<ul class="written">
					{#each written as path (path)}
						{@const name = path.split(/[\\/]/).pop()}
						<li>
							<button class="link mono" title={m.jobs.openFile} onclick={() => open(path)}
								><Icon name="file" size={14} />{name}</button
							>
							<button
								class="btn ghost sm icon"
								title={m.jobs.openFolder}
								aria-label="{m.jobs.openFolder}: {name}"
								onclick={() => revealInFolder(path)}><Icon name="folder" size={14} /></button
							>
						</li>
					{/each}
				</ul>
			</section>
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
	/* The job's facts as one strip, cells divided by hairlines. */
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
	.scores > .note,
	.doc-score {
		font-size: var(--type-xs);
		font-weight: 400;
		color: var(--text-muted);
	}
	.doc-score {
		margin: 0;
	}
	.fail,
	.err {
		color: var(--danger-text);
	}
	.cut {
		color: var(--warn-text);
	}
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
		font-size: var(--type-xs);
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
	.written {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-1);
	}
	.written li {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	/* The job's name being typed, in place of the title and at its size. */
	.rename {
		min-width: min(28rem, 100%);
		font-size: var(--type-xl);
		font-weight: 600;
	}
	.link {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		border: 0;
		background: none;
		color: var(--text-bright);
		cursor: pointer;
		padding: 0;
	}
	.link:hover {
		text-decoration: underline;
	}
	@media (max-width: 820px) {
		.body {
			padding: var(--space-4);
		}
		.row {
			grid-template-columns: minmax(0, 1fr);
		}
		.row .btn {
			justify-self: start;
		}
	}
</style>
