<script lang="ts">
	// One document: its original, its collections, and every result read from it across jobs,
	// task by task, each opening its review. The researcher can mark the result they rely on as
	// preferred; a newer run never takes its place.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import UseTextMenu from './UseTextMenu.svelte';
	import type { DocumentDetail } from '#lib/bindings/DocumentDetail.js';
	import type { DocumentResult } from '#lib/bindings/DocumentResult.js';
	import type { ProjectView } from '#lib/bindings/ProjectView.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { rulesFor } from '#lib/newJob.js';
	import { app, go } from '#lib/state.svelte.js';
	import { api, ask, revealInFolder } from '#lib/tauri.js';

	let { projectId, documentId }: { projectId: string; documentId: string } = $props();
	const m = messages();

	let detail = $state<DocumentDetail | null>(null);
	let project = $state<ProjectView | null>(null);
	let error = $state<string | null>(null);
	let draftTitle = $state<string | null>(null);

	/** The collection the table showed, to go back to it. */
	const from = $derived(app.view.name === 'projects' ? app.view.collectionId : undefined);
	const byTask = $derived.by(() => {
		const groups = new Map<Task, DocumentResult[]>();
		for (const r of detail?.results ?? []) {
			groups.set(r.task, [...(groups.get(r.task) ?? []), r]);
		}
		return [...groups.entries()];
	});

	async function load() {
		try {
			[detail, project] = await Promise.all([
				api.documentDetail(projectId, documentId),
				api.projectView(projectId)
			]);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function act(action: () => Promise<unknown>) {
		error = null;
		try {
			await action();
			await load();
		} catch (e) {
			error = describeError(e, m);
		}
	}

	async function rename() {
		const title = draftTitle?.trim();
		draftTitle = null;
		if (!title || title === detail?.document.title) return;
		await act(() => api.renameDocument(projectId, documentId, title));
	}

	function renameKeys(e: KeyboardEvent) {
		if (e.key === 'Enter') void rename();
		else if (e.key === 'Escape') draftTitle = null;
	}

	function toggleCollection(id: string, member: boolean) {
		void act(() =>
			member
				? api.removeFromCollection(projectId, id, [documentId])
				: api.addToCollection(projectId, id, [documentId])
		);
	}

	function prefer(r: DocumentResult) {
		void act(() =>
			r.preferred
				? api.setPreferred(projectId, documentId, r.task, null, null)
				: api.setPreferred(projectId, documentId, r.task, r.job_id, r.setup_id)
		);
	}

	async function detach(r: DocumentResult) {
		if (!(await ask(m.projects.document.detachConfirm))) return;
		await act(() => api.detachJob(projectId, documentId, r.job_id, r.source_id));
	}

	async function merge(id: string, title: string) {
		if (!(await ask(m.projects.document.mergeConfirm(title)))) return;
		await act(() => api.mergeDocuments(projectId, id, documentId));
	}

	onMount(() => void load());
</script>

<header class="masthead">
	<div>
		<button class="back" onclick={() => go({ name: 'projects', projectId, collectionId: from })}
			><Icon name="back" size={14} />{project?.project.name ?? m.projects.back}</button
		>
		{#if draftTitle !== null}
			<!-- svelte-ignore a11y_autofocus -->
			<input
				type="text"
				class="title-input"
				bind:value={draftTitle}
				aria-label={m.projects.document.title}
				autofocus
				onkeydown={renameKeys}
				onblur={rename}
			/>
		{:else}
			<h1>{detail?.document.title ?? ''}</h1>
			{#if detail}
				<button
					class="btn ghost sm icon"
					aria-label={m.projects.document.rename}
					title={m.projects.document.rename}
					onclick={() => (draftTitle = detail?.document.title ?? '')}
					><Icon name="edit" size={14} /></button
				>
			{/if}
		{/if}
	</div>
</header>

{#if error}<p class="chip danger banner" role="alert">{error}</p>{/if}

{#if detail && project}
	{@const doc = detail.document}
	<div class="sheet">
		<section class="card block" aria-labelledby="source-title">
			<h2 id="source-title">{m.projects.document.source}</h2>
			<p class="path mono">{doc.source.path}</p>
			<p class="facts">
				<span>{m.projects.kind[doc.source.kind]}</span>
				{#if doc.source.page_count}<span class="num">{m.newJob.pages(doc.source.page_count)}</span
					>{/if}
				{#if detail.missing}
					<span class="chip danger">{m.projects.missing}</span>
				{:else if doc.source.kind !== 'youtube'}
					<button class="btn ghost sm" onclick={() => revealInFolder(doc.source.path)}
						><Icon name="folder" size={14} />{m.projects.document.reveal}</button
					>
				{/if}
			</p>
			{#if detail.missing}<p class="muted">{m.projects.missingHint}</p>{/if}
		</section>

		<section class="card block" aria-labelledby="collections-title">
			<h2 id="collections-title">{m.projects.document.collections}</h2>
			{#if project.collections.length === 0}
				<p class="muted">{m.projects.document.notInCollection}</p>
			{:else}
				<div class="memberships">
					{#each project.collections as c (c.collection.id)}
						{@const member = detail.collections.includes(c.collection.id)}
						<label class="member">
							<input
								type="checkbox"
								checked={member}
								onchange={() => toggleCollection(c.collection.id, member)}
							/>
							{c.collection.name}
						</label>
					{/each}
				</div>
			{/if}
		</section>

		{#if detail.same_content.length > 0}
			<section class="card block" aria-labelledby="same-title">
				<h2 id="same-title">{m.projects.document.sameContentTitle}</h2>
				<p class="muted">{m.projects.document.sameContentLede}</p>
				<ul class="same">
					{#each detail.same_content as other (other.id)}
						<li>
							<button
								class="link"
								onclick={() => go({ name: 'projects', projectId, documentId: other.id })}
								>{other.title}</button
							>
							<span class="mono muted path">{other.path}</span>
							<button class="btn sm" onclick={() => merge(other.id, other.title)}
								><Icon name="merge" size={14} />{m.projects.document.merge}</button
							>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		<section class="card block" aria-labelledby="results-title">
			<h2 id="results-title">{m.projects.document.results}</h2>
			{#if byTask.length === 0}
				<p class="muted">{m.projects.document.noResults}</p>
			{/if}
			{#each byTask as [task, results] (task)}
				<div class="task">
					<h3><Icon name={task} size={14} />{m.task[task]}</h3>
					<table>
						<thead>
							<tr>
								<th>{m.projects.document.job}</th>
								<th>{m.projects.document.setup}</th>
								<th>{m.projects.document.progress}</th>
								<th>{m.projects.document.reviewed}</th>
								<th>{m.projects.document.cost}</th>
								<th><span class="sr-only">{m.projects.document.preferred}</span></th>
								<th><span class="sr-only">{m.projects.document.open}</span></th>
							</tr>
						</thead>
						<tbody>
							{#each results as r (`${r.job_id}/${r.setup_id}`)}
								<tr class:preferred={r.preferred}>
									<td>
										<span class="job">{r.job_name}</span>
										<span class="muted small num"
											>{formatDateTime(r.created_at)} · {m.status[r.job_status]}</span
										>
									</td>
									<td class="mono small">{r.model}</td>
									<td class="num">{m.projects.document.pieces(r.finished, r.pieces, r.failed)}</td>
									<td class="num"
										>{m.projects.document.reviewedCount(r.reviewed, r.pieces, r.corrected)}</td
									>
									<td class="num">{r.cost_usd !== null ? formatUsd(r.cost_usd) : '–'}</td>
									<td>
										<button
											class="btn ghost sm"
											class:on={r.preferred}
											aria-pressed={r.preferred}
											title={m.projects.document.preferHint}
											onclick={() => prefer(r)}
											><Icon name="star" size={14} filled={r.preferred} />{r.preferred
												? m.projects.document.preferred
												: m.projects.document.prefer}</button
										>
									</td>
									<td class="open">
										<button
											class="btn sm"
											onclick={() => go({ name: 'review', jobId: r.job_id, sourceId: r.source_id })}
											>{m.projects.document.open}</button
										>
										<button
											class="btn ghost sm"
											onclick={() => go({ name: 'jobs', jobId: r.job_id })}
											>{m.projects.document.openJob}</button
										>
										{#if rulesFor(app.rules, r.task).yields_text && r.finished > 0}
											<UseTextMenu
												small
												jobId={r.job_id}
												setupId={r.setup_id}
												sourceId={r.source_id}
												project={{ id: projectId, name: project?.project.name ?? '' }}
												onerror={(e) => (error = e)}
											/>
										{/if}
										<button
											class="btn ghost sm icon"
											aria-label={m.projects.document.detach}
											title={m.projects.document.detach}
											onclick={() => detach(r)}><Icon name="unlink" size={14} /></button
										>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/each}
		</section>
	</div>
{/if}

<style>
	.banner {
		margin: var(--space-3) var(--space-5) 0;
	}
	.title-input {
		min-width: 18rem;
	}
	.block {
		display: grid;
		gap: var(--space-2);
		padding-top: var(--space-4);
	}
	.path {
		overflow-wrap: anywhere;
		color: var(--text-secondary);
	}
	.facts {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.memberships {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-4);
	}
	.member {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
	}
	.same {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	.same li {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font-weight: 500;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
		text-underline-offset: 0.18em;
	}
	.task {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-2);
	}
	.task h3 {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--type-sm);
	}
	th {
		text-align: left;
		font-weight: 500;
		font-size: var(--type-xs);
		color: var(--text-muted);
		padding: var(--space-1) var(--space-2);
		border-bottom: 1px solid var(--line);
	}
	td {
		padding: var(--space-2);
		border-bottom: 1px solid var(--line-soft);
		vertical-align: top;
	}
	.job {
		display: block;
		color: var(--text-bright);
		font-weight: 500;
	}
	.small {
		font-size: var(--type-xs);
	}
	tr.preferred td {
		background: var(--hand-wash);
	}
	.on {
		color: var(--hand);
	}
	.open {
		white-space: nowrap;
	}
</style>
