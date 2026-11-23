<script lang="ts">
	// The sidebar's foot: the job that waits for the researcher to go on, paused or cut short by
	// a quit or a crash, seen from any screen with its Resume. The one stopped last is shown;
	// the others stopped too are one click away under it, each with its own Resume, so the foot
	// stays short however many there are. Resuming asks the engine as the job's page does
	// (`start_job`); once the job runs it leaves the foot, and the focus goes to the next one to
	// resume, or to the job in the list, never to nothing. In the rail, only the latest, as an
	// icon named by its tooltip.
	import { tick } from 'svelte';
	import Icon from './Icon.svelte';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import { Action } from '#lib/action.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import { toResume } from '#lib/jobList.js';
	import { api } from '#lib/tauri.js';
	import { app, liveJob, refreshJobs, toast } from '#lib/state.svelte.js';

	let {
		collapsed,
		fallback
	}: {
		collapsed: boolean;
		/** Where the focus goes when nothing nearer is left to take it. */
		fallback: () => HTMLElement | null;
	} = $props();
	const m = messages();

	const jobs = $derived(toResume(app.jobs, app.waiting));
	const others = $derived(jobs.slice(1));
	let showOthers = $state(false);
	const shown = $derived(showOthers ? jobs : jobs.slice(0, 1));

	/** The job being resumed, while the engine is asked. */
	let busy = $state<string | null>(null);
	/** What went wrong last, and for which job: said under the foot until the next attempt. The
	 * rail has no room for it, so there it is the toast's failure, read out and kept the same. */
	let failure = $state<{ id: string; text: string } | null>(null);
	const action = new Action({
		report: (text) => {
			if (collapsed) toast(text, { kind: 'error' });
			else if (busy) failure = { id: busy, text };
		}
	});
	const failed = $derived(failure && jobs.some((j) => j.id === failure?.id) ? failure.text : '');

	/** The Resume buttons by job, to move the focus between them. */
	const buttons = $state<Record<string, HTMLButtonElement>>({});

	async function resume(job: JobSummary, button: HTMLButtonElement) {
		if (busy) return;
		busy = job.id;
		failure = null;
		const next = jobs.find((j) => j.id !== job.id);
		const ok = await action.run(async () => {
			liveJob(job.id);
			await api.startJob(job.id);
			await refreshJobs();
		});
		busy = null;
		if (!ok) return;
		toast(m.shell.resumed(job.name));
		await tick();
		// The row went with the job: its focus goes to the next to resume, the job in the list,
		// or the tools, so a keyboard or a screen reader is never left at the top of the page.
		if (button.isConnected) return;
		const inList = document.querySelector<HTMLElement>(
			`#job-list:not([hidden]) [data-job="${CSS.escape(job.id)}"]`
		);
		((next && buttons[next.id]) || inList || fallback())?.focus();
	}
</script>

{#if jobs.length > 0}
	{#if collapsed}
		{@const job = jobs[0]}
		<button
			bind:this={buttons[job.id]}
			class="tool resume-icon"
			aria-label={m.shell.resumeJob(job.name)}
			aria-disabled={busy === job.id}
			title="{m.shell.resumeJob(job.name)} · {m.status[job.status]} · {m.jobs.progress(
				job.pieces_finished,
				job.pieces_total
			)}"
			onclick={(e) => void resume(job, e.currentTarget)}
		>
			<Icon name="play" size={16} />
		</button>
	{:else}
		<section class="resume" aria-labelledby="resume-head">
			<!-- With others, the head is where they are shown, so the foot keeps to one job's
			     height in a short window. -->
			<div class="head">
				<h2 id="resume-head">{m.shell.toResume}</h2>
				{#if others.length > 0}
					<button
						class="others"
						aria-expanded={showOthers}
						aria-controls="resume-list"
						onclick={() => (showOthers = !showOthers)}
					>
						{m.shell.resumeOthers(others.length)}<span class="turn" class:open={showOthers}>
							<Icon name="chevron" size={14} />
						</span>
					</button>
				{/if}
			</div>
			<ul id="resume-list">
				{#each shown as job (job.id)}
					<li>
						<span class="name">
							<Icon name={job.task} size={14} />
							<span class="ellipsis">{job.name}</span>
						</span>
						<span class="meta" id="resume-meta-{job.id}">
							{m.status[job.status]} ·
							<span class="num">{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span>
						</span>
						<button
							bind:this={buttons[job.id]}
							class="btn sm"
							aria-label={m.shell.resumeJob(job.name)}
							aria-describedby="resume-meta-{job.id} resume-failure"
							aria-disabled={busy === job.id}
							onclick={(e) => void resume(job, e.currentTarget)}
						>
							<Icon name="play" size={12} />{m.jobs.resume}
						</button>
					</li>
				{/each}
			</ul>
			<!-- Always there while a job is offered, empty until something fails, so the words
			     put in are read out at once in both webviews. A live region rather than an alert:
			     the toast is the window's one alert. -->
			<p
				class="failure"
				class:quiet={!failed}
				id="resume-failure"
				aria-live="assertive"
				aria-atomic="true"
			>
				{failed}
			</p>
		</section>
	{/if}
{/if}

<style>
	.resume {
		display: grid;
		gap: var(--space-1);
		padding-bottom: var(--space-3);
		margin-bottom: var(--space-1);
		border-bottom: 1px solid var(--line);
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: 1.5rem;
		padding: 0 0 0 var(--space-2);
	}
	/* Like the job list's group heads. */
	h2 {
		font-size: var(--type-xs);
		font-weight: 500;
		letter-spacing: 0;
		color: var(--text-muted);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	/* The name, how far the job got, then its Resume under them: each as wide as the foot, so
	   neither the name nor the state is cut beside the button. */
	li {
		display: grid;
		gap: 0.2rem;
		padding: 0 0 0 0.625rem;
	}
	li .btn {
		justify-self: start;
		margin: 0.2rem 0 0 1.375rem;
	}
	.name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
		font-size: var(--type-sm);
		font-weight: 500;
		color: var(--text-bright);
	}
	.name :global(.icon) {
		flex: none;
		color: var(--text-secondary);
	}
	.meta {
		padding-left: 1.375rem;
		font-size: var(--type-xs);
		color: var(--warn-text);
	}
	.btn[aria-disabled='true'] {
		opacity: 0.45;
		cursor: progress;
	}
	.others {
		display: flex;
		align-items: center;
		gap: 0.125rem;
		min-height: 1.5rem;
		padding: 0 var(--space-1) 0 var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-secondary);
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.others:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.turn {
		display: grid;
		transition: transform 120ms ease-out;
	}
	.turn.open {
		transform: rotate(90deg);
	}
	.failure {
		padding: 0 0.625rem;
		font-size: var(--type-xs);
		color: var(--danger-text);
	}
	/* Kept in the page while empty (a live region hidden away may not be read once filled), its
	   place in the grid taken back. */
	.failure.quiet {
		margin-top: calc(-1 * var(--space-1));
	}
	/* The rail's Resume: a tool like the others round it. */
	.resume-icon {
		display: grid;
		place-items: center;
		width: 2.25rem;
		min-height: var(--control-md);
		padding: 0;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--warn-text);
		cursor: pointer;
	}
	.resume-icon:hover {
		background: var(--wash);
	}
	.resume-icon[aria-disabled='true'] {
		opacity: 0.45;
		cursor: progress;
	}
</style>
