<script lang="ts">
	// The sidebar's list of jobs, grouped by what they need from the researcher: workflow steps
	// that wait for them first, since nothing moves until they act. Finished jobs are the most
	// recent few, then a link to the Jobs page, which lists them all; what runs, stopped early or
	// waits is always listed in full, and so is the job open in the main area. A job's menu (a
	// right click or the keyboard's menu key) renames it, adds it to a project or deletes it; a
	// finished job can be dragged onto a project.
	import Icon from './Icon.svelte';
	import JobMenu from './JobMenu.svelte';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import { messages } from '#lib/i18n/index.js';
	import { focusName, RowRename } from '#lib/jobActions.svelte.js';
	import type { JobDrag } from '#lib/jobDrag.svelte.js';
	import { UNFINISHED, finishedShown, groupOf, type GroupKey } from '#lib/jobList.js';
	import { app, go } from '#lib/state.svelte.js';

	let { collapsed, drag }: { collapsed: boolean; drag: JobDrag } = $props();
	const m = messages();

	/** The job whose menu is open, and where. */
	let menu = $state<{ job: JobSummary; x: number; y: number } | null>(null);
	const rename = new RowRename();

	function openMenu(e: MouseEvent, job: JobSummary) {
		e.preventDefault();
		// From the keyboard's menu key there is no pointer: open under the job.
		const at = (e.currentTarget as HTMLElement).getBoundingClientRect();
		const keyboard = e.button !== 2 && e.clientX === 0 && e.clientY === 0;
		menu = { job, x: keyboard ? at.left + 12 : e.clientX, y: keyboard ? at.bottom : e.clientY };
	}

	/** The job the main area shows, whether as its detail or in review. */
	const openJob = $derived(
		app.view.name === 'review'
			? app.view.jobId
			: app.view.name === 'jobs'
				? (app.view.jobId ?? app.jobs[0]?.id)
				: undefined
	);

	const labels: Record<GroupKey, string> = {
		waiting: m.shell.toCheck,
		running: m.shell.running,
		unfinished: m.shell.unfinished,
		finished: m.shell.finished
	};
	const groups = $derived.by(() => {
		const by: Record<GroupKey, JobSummary[]> = {
			waiting: [],
			running: [],
			unfinished: [],
			finished: []
		};
		for (const job of app.jobs) by[groupOf(job, app.waiting)].push(job);
		return (Object.keys(by) as GroupKey[])
			.filter((key) => by[key].length > 0)
			.map((key) => ({
				key,
				label: labels[key],
				count: by[key].length,
				jobs: key === 'finished' ? finishedShown(by[key], openJob) : by[key]
			}));
	});
	const allJobsOpen = $derived(app.view.name === 'allJobs');
	/** Whether there is a project to drag a finished job onto. */
	const droppable = $derived(app.projects.some((p) => !p.project.archived));

	const share = (j: JobSummary) => (100 * j.pieces_finished) / Math.max(j.pieces_total, 1);
</script>

{#if menu}
	{@const at = menu}
	<JobMenu
		job={at.job}
		x={at.x}
		y={at.y}
		onrename={() => rename.start(at.job)}
		onclose={() => (menu = null)}
	/>
{/if}

<div class="jobs" id="job-list" hidden={collapsed}>
	{#each groups as group (group.key)}
		<section class="group" aria-labelledby="group-{group.key}">
			<h2 class="group-head" id="group-{group.key}">
				<span>{group.label}</span><span class="num">{group.count}</span>
			</h2>
			<ul>
				{#each group.jobs as job (job.id)}
					<li>
						{#if rename.editing?.id === job.id}
							<input
								class="rename"
								aria-label={m.jobs.nameLabel}
								bind:value={rename.editing.name}
								onkeydown={rename.keys}
								onblur={rename.commit}
								{@attach focusName}
							/>
						{:else}
							<button
								class="job"
								class:current={openJob === job.id}
								class:lifted={drag.dragging?.job.id === job.id}
								aria-current={openJob === job.id ? 'page' : undefined}
								title={group.key === 'finished' && droppable ? m.shell.dragHint : undefined}
								onpointerdown={group.key === 'finished' ? (e) => drag.press(e, job) : undefined}
								onclick={() => {
									if (!drag.dragged) go({ name: 'jobs', jobId: job.id });
								}}
								oncontextmenu={(e) => openMenu(e, job)}
							>
								<span class="job-name">
									<Icon name={job.task} size={14} />
									<span class="ellipsis">{job.name}</span>
								</span>
								<span
									class="job-meta"
									class:warn={UNFINISHED.includes(job.status) || group.key === 'waiting'}
									class:danger={job.status === 'completed_with_errors' && group.key !== 'waiting'}
								>
									{#if group.key === 'waiting'}
										{m.workflows.stepState[app.waiting[job.id]?.state ?? 'review']}
									{:else if group.key === 'finished'}
										{job.status === 'completed' ? m.task[job.task] : m.status[job.status]}
									{:else}
										{m.status[job.status]} ·
										<span class="num">{m.jobs.progress(job.pieces_finished, job.pieces_total)}</span
										>
									{/if}
								</span>
								{#if group.key !== 'finished' && group.key !== 'waiting'}
									<span class="job-bar" aria-hidden="true"><i style:width="{share(job)}%"></i></span
									>
								{/if}
							</button>
						{/if}
					</li>
				{/each}
			</ul>
		</section>
	{:else}
		<p class="no-jobs">{m.shell.noJobs}</p>
	{/each}
	{#if app.jobs.length > 0}
		<button
			class="all-jobs"
			class:current={allJobsOpen}
			aria-current={allJobsOpen ? 'page' : undefined}
			onclick={() => go({ name: 'allJobs' })}
			><Icon name="list" size={14} /><span class="ellipsis">{m.allJobs.link(app.jobs.length)}</span
			></button
		>
	{/if}
</div>

<style>
	.jobs {
		flex: 1;
		min-height: 0;
		overflow: auto;
		display: grid;
		align-content: start;
		gap: var(--space-4);
		margin: 0 calc(-1 * var(--space-1));
		padding: 0 var(--space-1);
	}
	.jobs[hidden] {
		display: none;
	}
	.group-head {
		display: flex;
		justify-content: space-between;
		padding: 0 var(--space-2) var(--space-1);
		font-size: var(--type-xs);
		font-weight: 500;
		letter-spacing: 0;
		color: var(--text-muted);
	}
	.group-head .num {
		font-family: var(--font-mono);
		font-size: 0.75rem;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 2px;
	}
	.job {
		width: 100%;
		display: grid;
		gap: 0.3rem;
		padding: var(--space-2) 0.625rem;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-bright);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.rename {
		width: 100%;
		min-height: 2.25rem;
		font-size: var(--type-sm);
		font-weight: 500;
	}
	.job:hover {
		background: var(--wash);
	}
	.job.current {
		background: var(--surface-0);
		box-shadow: var(--shadow-raised);
	}
	.job.lifted {
		opacity: 0.5;
	}
	.job-name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
		font-size: var(--type-sm);
		font-weight: 500;
	}
	.job-name :global(.icon) {
		color: var(--text-secondary);
	}
	.job-meta {
		padding-left: 1.375rem;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.job-meta.warn {
		color: var(--warn-text);
	}
	.job-meta.danger {
		color: var(--danger-text);
	}
	.job-bar {
		display: block;
		height: 3px;
		margin-left: 1.375rem;
		border-radius: 2px;
		background: var(--line);
		overflow: hidden;
	}
	.job-bar i {
		display: block;
		height: 100%;
		background: var(--ink);
		transition: width 400ms ease-out;
	}
	/* The way to every job, under the most recent: quiet, like a group's heading. */
	.all-jobs {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
		min-height: var(--control-sm);
		margin-top: calc(-1 * var(--space-2));
		padding: 0 0.625rem;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.all-jobs:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.all-jobs.current {
		background: var(--surface-0);
		box-shadow: var(--shadow-raised);
		color: var(--text-bright);
		font-weight: 500;
	}
	.no-jobs {
		padding: 0 var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-muted);
	}
</style>
