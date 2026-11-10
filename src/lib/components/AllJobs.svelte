<script lang="ts">
	// Every job in one table, for when the sidebar lists only the most recent: found by its name
	// or a file it read, filtered by task and status, sorted by date or name. A row opens its job;
	// its menu (the ⋯ button or a right click) renames it, adds it to a project or deletes it, and
	// rows chosen together can be added to a project or deleted at once.
	import Icon from './Icon.svelte';
	import JobMenu from './JobMenu.svelte';
	import MoreRows from './MoreRows.svelte';
	import type { JobSummary } from '#lib/bindings/JobSummary.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatDate, formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import {
		DEFAULT_JOB_LIST_VIEW,
		UNFINISHED,
		readJobListView,
		shownJobs,
		sortJobs,
		type JobSort,
		type StatusFilter
	} from '#lib/jobList.js';
	import { focusName, RowRename } from '#lib/jobActions.svelte.js';
	import { Paging, rowIndex } from '#lib/paging.svelte.js';
	import { Selection } from '#lib/selection.svelte.js';
	import { app, go, refreshJobs, refreshProjects, toast } from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';
	import { api, ask } from '#lib/tauri.js';

	const m = messages();

	// The filters and sort as the page was left, remembered on this computer.
	const VIEW_KEY = 'kalamu.jobs.view';
	const left = recall(VIEW_KEY, readJobListView, DEFAULT_JOB_LIST_VIEW);
	let query = $state('');
	let task = $state<Task | ''>(left.task);
	let status = $state<StatusFilter>(left.status);
	let sort = $state<JobSort>(left.sort);
	$effect(() => keep(VIEW_KEY, { task, status, sort }));
	const selection = new Selection();
	let target = $state('');

	/** The names of each job's files, for the search; read again when jobs come or go. */
	let sources = $state<ReadonlyMap<string, string[]>>(new Map());
	const listed = $derived(app.jobs.map((j) => j.id).join());
	$effect(() => {
		void listed;
		api
			.jobSources()
			.then((all) => (sources = new Map(all.map((s) => [s.job_id, s.names]))))
			.catch(() => undefined);
	});

	// The choices of the filters: only the tasks and states some job has.
	const tasks = $derived([...new Set(app.jobs.map((j) => j.task))]);
	const states = $derived([...new Set(app.jobs.map((j) => j.status))]);
	const anyWaiting = $derived(app.jobs.some((j) => app.waiting[j.id]));
	// A filter no job answers any longer, kept from an earlier visit, stops filtering.
	const taskShown = $derived(task && tasks.includes(task) ? task : '');
	const statusShown = $derived(
		status === 'waiting'
			? anyWaiting
				? status
				: ''
			: status && states.includes(status)
				? status
				: ''
	);

	const shown = $derived(
		sortJobs(
			shownJobs(app.jobs, { query, task: taskShown, status: statusShown }, sources, app.waiting),
			sort
		)
	);
	// A long list is drawn a page at a time, from its first page again when filtered or sorted.
	const paging = new Paging();
	const drawn = $derived(paging.of(shown));
	let table = $state<HTMLTableElement | null>(null);
	$effect(() => {
		void [query, taskShown, statusShown, sort];
		paging.reset();
	});
	const chosen = $derived(selection.of(shown, (j) => j.id));
	const allChosen = $derived(shown.length > 0 && chosen.length === shown.length);
	const anyRunning = $derived(chosen.some((j) => j.status === 'running'));
	const projects = $derived(app.projects.filter((p) => !p.project.archived));

	/** The job whose menu is open, and where. */
	let menu = $state<{ job: JobSummary; x: number; y: number } | null>(null);
	const rename = new RowRename();

	function openMenu(e: MouseEvent, job: JobSummary) {
		e.preventDefault();
		e.stopPropagation();
		const at = (e.currentTarget as HTMLElement).getBoundingClientRect();
		// From the ⋯ button or the keyboard's menu key there is no pointer to open at.
		const pointer = e.type === 'contextmenu' && (e.clientX !== 0 || e.clientY !== 0);
		menu = { job, x: pointer ? e.clientX : at.left, y: pointer ? e.clientY : at.bottom };
	}

	/** A click on a row opens its job, unless it was on one of the row's own controls. */
	function openRow(e: MouseEvent, job: JobSummary) {
		if ((e.target as HTMLElement).closest('input, button, select, a')) return;
		go({ name: 'jobs', jobId: job.id });
	}

	async function addTo(projectId: string) {
		target = '';
		const project = projects.find((p) => p.project.id === projectId);
		if (!project) return;
		const ids = chosen.map((j) => j.id);
		try {
			for (const id of ids) await api.linkJob(projectId, id, null);
			toast(m.allJobs.addedTo(ids.length, project.project.name));
			selection.clear();
			await refreshProjects();
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
	}

	async function remove() {
		const ids = chosen.map((j) => j.id);
		if (ids.length === 0 || anyRunning) return;
		if (!(await ask(m.allJobs.deleteConfirm(ids.length)))) return;
		try {
			for (const id of ids) await api.deleteJob(id);
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
		selection.clear();
		await refreshJobs();
	}

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

<header class="masthead">
	<div>
		<h1>{m.allJobs.title}</h1>
		{#if app.jobs.length > 0}
			<p class="lede num">{m.allJobs.count(app.jobs.length)}</p>
		{/if}
	</div>
</header>

<div class="sheet all-jobs">
	{#if app.jobs.length === 0}
		<div class="empty">
			<p class="muted">{m.jobs.empty}</p>
			<button class="btn primary" onclick={() => go({ name: 'new' })}
				><Icon name="plus" />{m.jobs.empty_new}</button
			>
		</div>
	{:else}
		<div class="table-filters">
			<label class="table-search">
				<Icon name="search" size={14} />
				<input
					type="text"
					bind:value={query}
					data-find
					placeholder={m.allJobs.search}
					aria-label={m.allJobs.search}
				/>
			</label>
			<select bind:value={() => taskShown, (v) => (task = v)} aria-label={m.allJobs.columnTask}>
				<option value="">{m.allJobs.anyTask}</option>
				{#each tasks as t (t)}
					<option value={t}>{m.task[t]}</option>
				{/each}
			</select>
			<select
				bind:value={() => statusShown, (v) => (status = v)}
				aria-label={m.allJobs.columnStatus}
			>
				<option value="">{m.allJobs.anyStatus}</option>
				{#if anyWaiting}<option value="waiting">{m.shell.toCheck}</option>{/if}
				{#each states as s (s)}
					<option value={s}>{m.status[s]}</option>
				{/each}
			</select>
			<label class="sort">
				<span>{m.allJobs.sortBy}</span>
				<select bind:value={sort}>
					<option value="newest">{m.allJobs.sortNewest}</option>
					<option value="oldest">{m.allJobs.sortOldest}</option>
					<option value="name">{m.allJobs.sortName}</option>
				</select>
			</label>
		</div>

		{#if chosen.length > 0}
			<div class="selection-bar" role="toolbar" aria-label={m.allJobs.selected(chosen.length)}>
				<span class="num">{m.allJobs.selected(chosen.length)}</span>
				<button class="btn ghost sm" onclick={() => selection.clear()}
					>{m.allJobs.clearSelection}</button
				>
				<span class="grow"></span>
				{#if projects.length > 0}
					<select
						class="sm"
						bind:value={target}
						onchange={() => addTo(target)}
						aria-label={m.allJobs.addTo}
					>
						<option value="">{m.allJobs.addTo}</option>
						{#each projects as p (p.project.id)}
							<option value={p.project.id}>{p.project.name}</option>
						{/each}
					</select>
				{/if}
				<button
					class="btn ghost danger sm"
					disabled={anyRunning}
					title={anyRunning ? m.allJobs.runningKept : undefined}
					onclick={remove}>{m.allJobs.delete}</button
				>
			</div>
		{/if}

		{#if shown.length === 0}
			<p class="muted">{m.allJobs.noMatch}</p>
		{:else}
			<table class="data-table jobs-table" bind:this={table} aria-rowcount={shown.length + 1}>
				<thead>
					<tr aria-rowindex={1}>
						<th class="check">
							<input
								type="checkbox"
								checked={allChosen}
								aria-label={m.allJobs.selectAll}
								onchange={() => selection.toggleAll(shown.map((j) => j.id))}
							/>
						</th>
						<th>{m.allJobs.columnName}</th>
						<th class="task">{m.allJobs.columnTask}</th>
						<th class="state">{m.allJobs.columnStatus}</th>
						<th class="pieces">{m.allJobs.columnPieces}</th>
						<th class="cost">{m.allJobs.columnCost}</th>
						<th class="date">{m.allJobs.columnDate}</th>
						<th class="more"><span class="sr-only">{m.allJobs.actions}</span></th>
					</tr>
				</thead>
				<tbody>
					{#each drawn as job, i (job.id)}
						{@const waits = app.waiting[job.id]}
						<!-- The row opens its job on a click; from the keyboard, its name does. -->
						<tr
							data-row={i}
							aria-rowindex={rowIndex(i)}
							class:chosen={selection.has(job.id)}
							onclick={(e) => openRow(e, job)}
							oncontextmenu={(e) => openMenu(e, job)}
						>
							<td class="check">
								<input
									type="checkbox"
									checked={selection.has(job.id)}
									aria-label={m.allJobs.select(job.name)}
									onchange={() => selection.toggle(job.id)}
								/>
							</td>
							<td class="name">
								{#if rename.editing?.id === job.id}
									<input
										class="rename"
										type="text"
										aria-label={m.jobs.nameLabel}
										bind:value={rename.editing.name}
										onkeydown={rename.keys}
										onblur={rename.commit}
										{@attach focusName}
									/>
								{:else}
									<button class="link" onclick={() => go({ name: 'jobs', jobId: job.id })}
										><Icon name={job.task} size={14} /><span>{job.name}</span></button
									>
								{/if}
								<span class="meta">
									<span class="num">{m.allJobs.files(job.source_count)}</span>
									{#if job.workflow}
										<span class="workflow"><Icon name="workflow" size={12} />{job.workflow}</span>
									{/if}
								</span>
							</td>
							<td class="task muted">{m.task[job.task]}</td>
							<td class="state">
								{#if waits}
									<span class="chip warn">{m.workflows.stepState[waits.state]}</span>
								{:else if job.status === 'running'}
									<span>{m.status.running}</span>
									<span class="bar" aria-hidden="true"><i style:width="{share(job)}%"></i></span>
								{:else}
									<span
										class:chip={job.status !== 'completed' && job.status !== 'cancelled'}
										class:warn={UNFINISHED.includes(job.status)}
										class:danger={job.status === 'completed_with_errors'}
										class:muted={job.status === 'cancelled'}>{m.status[job.status]}</span
									>
								{/if}
							</td>
							<td class="pieces num">
								{job.pieces_finished}/{job.pieces_total}
								{#if job.pieces_failed > 0}
									<span class="failed">{m.jobs.failed(job.pieces_failed)}</span>
								{/if}
							</td>
							<td class="cost num"
								>{#if job.cost_usd !== null}{formatUsd(job.cost_usd)}{:else}<span
										class="muted"
										aria-hidden="true">—</span
									>{/if}</td
							>
							<td class="date num muted" title={formatDateTime(job.created_at)}
								>{formatDate(job.created_at)}</td
							>
							<td class="more">
								<button
									class="btn ghost sm icon"
									aria-label={m.shell.jobMenu(job.name)}
									title={m.shell.jobMenu(job.name)}
									aria-haspopup="menu"
									onclick={(e) => openMenu(e, job)}><Icon name="more" size={16} /></button
								>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
			<MoreRows {paging} total={shown.length} {table} />
		{/if}
	{/if}
</div>

<style>
	/* One column whose content may shrink, so a long name wraps instead of widening the page. */
	.all-jobs {
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-4);
	}
	.empty {
		display: grid;
		justify-items: start;
		gap: var(--space-3);
	}
	.sort {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.selection-bar select {
		min-height: var(--control-sm);
		font-size: var(--type-xs);
	}
	.grow {
		flex: 1;
	}
	tbody tr {
		cursor: pointer;
	}
	tbody tr:hover td {
		background: var(--wash);
	}
	.data-table td {
		vertical-align: middle;
	}
	.name {
		overflow-wrap: break-word;
	}
	.link span {
		min-width: 0;
	}
	.link {
		display: inline-flex;
		align-items: baseline;
		gap: var(--space-2);
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font-weight: 500;
		text-align: left;
		cursor: pointer;
	}
	.link :global(.icon) {
		align-self: center;
		color: var(--text-secondary);
	}
	.link:hover span {
		text-decoration: underline;
		text-underline-offset: 0.18em;
	}
	.rename {
		width: 100%;
		min-height: var(--control-sm);
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0 var(--space-3);
		margin-top: 2px;
		padding-left: calc(14px + var(--space-2));
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The icon runs with the workflow's name, which may wrap under it. */
	.workflow :global(.icon) {
		display: inline-block;
		margin-right: var(--space-1);
		vertical-align: -1px;
	}
	.date,
	.cost {
		white-space: nowrap;
	}
	/* A long state ("Done with errors") wraps inside its column rather than overflowing it. */
	.state .chip {
		white-space: normal;
	}
	.pieces {
		white-space: nowrap;
		color: var(--text-secondary);
	}
	.failed {
		display: block;
		font-size: var(--type-xs);
		color: var(--danger-text);
	}
	.bar {
		display: block;
		width: 5rem;
		height: 3px;
		margin-top: var(--space-1);
		border-radius: 2px;
		background: var(--line);
		overflow: hidden;
	}
	.bar i {
		display: block;
		height: 100%;
		background: var(--ink);
		transition: width 400ms ease-out;
	}
	.more {
		text-align: right;
	}
	/* Fixed columns: the name takes what is left, wrapping, so the table never grows wider than
	   the page. Fewer columns as the page narrows (the sidebar folded or not), measured on the
	   page itself; the task still shows by the name's icon. */
	.jobs-table {
		table-layout: fixed;
	}
	.jobs-table th.check {
		width: 2.25rem;
	}
	.jobs-table th.task {
		width: 11rem;
	}
	.jobs-table th.state {
		width: 10rem;
	}
	.jobs-table th.pieces {
		width: 6.5rem;
	}
	.jobs-table th.cost {
		width: 5.5rem;
	}
	.jobs-table th.date {
		width: 6.75rem;
	}
	.jobs-table th.more {
		width: 2.75rem;
	}
	.all-jobs {
		container-type: inline-size;
	}
	@container (max-width: 52rem) {
		.task,
		.cost {
			display: none;
		}
	}
	@container (max-width: 36rem) {
		.pieces {
			display: none;
		}
		.jobs-table th.state {
			width: 8.5rem;
		}
	}
	@container (max-width: 26rem) {
		.date {
			display: none;
		}
		.jobs-table th.state {
			width: 7rem;
		}
	}
</style>
