<script lang="ts">
	// Adds a job run earlier to a project: each of its files is shown with the document it is
	// taken for, and the researcher confirms or changes it. Same content in another file is
	// offered, never chosen for them. Nothing is sent again.
	import Dialog from './Dialog.svelte';
	import type { DocumentRow } from '#lib/bindings/DocumentRow.js';
	import type { SourceMatch } from '#lib/bindings/SourceMatch.js';
	import { Action } from '#lib/action.svelte.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { app } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		collectionId,
		documents,
		onclose
	}: {
		projectId: string;
		collectionId?: string;
		documents: DocumentRow[];
		onclose: (attached: number) => void;
	} = $props();
	const m = messages();

	const NEW = 'new';
	const SKIP = 'skip';
	let jobId = $state('');
	let plan = $state<SourceMatch[]>([]);
	/** For each source: a document id, `new` or `skip`. */
	let choices = $state<Record<string, string>>({});
	const action = new Action();

	const titles = $derived(new Map(documents.map((d) => [d.document.id, d.document.title])));
	const chosenCount = $derived(Object.values(choices).filter((c) => c !== SKIP).length);
	const anyShared = $derived(plan.some((p) => p.same_content.length > 0));

	async function loadPlan() {
		action.error = null;
		plan = [];
		choices = {};
		if (!jobId) return;
		try {
			plan = await api.attachPlan(projectId, jobId);
			choices = Object.fromEntries(plan.map((p) => [p.source_id, p.linked ?? p.proposed ?? NEW]));
		} catch (e) {
			action.fail(e);
		}
	}

	async function confirm() {
		await action.run(async () => {
			const picked = plan
				.filter((p) => choices[p.source_id] !== SKIP)
				.map((p) => ({
					source_id: p.source_id,
					document: choices[p.source_id] === NEW ? null : choices[p.source_id]
				}));
			const attached = await api.attachJob(projectId, jobId, picked, collectionId ?? null);
			onclose(attached.length);
		});
	}
</script>

<Dialog labelledby="attach-title" size="medium" onescape={() => onclose(0)}>
	<h2 id="attach-title">{m.projects.attach.title}</h2>
	<p class="muted">{m.projects.attach.lede}</p>

	<label class="field">
		<span>{m.projects.attach.job}</span>
		<select bind:value={jobId} onchange={loadPlan}>
			<option value="">{m.projects.attach.chooseJob}</option>
			{#each app.jobs as job (job.id)}
				<option value={job.id}
					>{job.name} · {m.task[job.task]} · {formatDateTime(job.created_at)}</option
				>
			{/each}
		</select>
	</label>

	{#if plan.length > 0}
		{#if anyShared}<p class="notice">{m.projects.attach.sameContentHint}</p>{/if}
		<table class="data-table">
			<thead>
				<tr><th>{m.projects.attach.file}</th><th>{m.projects.attach.document}</th></tr>
			</thead>
			<tbody>
				{#each plan as p (p.source_id)}
					<tr>
						<td><span class="mono file" title={p.path}>{p.file_name}</span></td>
						<td>
							<select bind:value={choices[p.source_id]} aria-label={p.file_name}>
								{#if p.linked}
									<option value={p.linked}
										>{m.projects.attach.already(titles.get(p.linked) ?? '')}</option
									>
								{/if}
								{#if p.proposed && p.proposed !== p.linked}
									<option value={p.proposed}
										>{m.projects.attach.proposed(titles.get(p.proposed) ?? '')}</option
									>
								{/if}
								{#each p.same_content.filter((d) => d.id !== p.linked) as d (d.id)}
									<option value={d.id}>{m.projects.attach.sameContent(d.title)}</option>
								{/each}
								<option value={NEW}>{m.projects.attach.newDocument}</option>
								<option value={SKIP}>{m.projects.attach.skip}</option>
							</select>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}

	{#if action.error}<p class="chip danger" role="alert">{action.error}</p>{/if}

	<div class="actions">
		<button class="btn" onclick={() => onclose(0)}>{m.projects.cancel}</button>
		<button class="btn primary" disabled={action.busy || chosenCount === 0} onclick={confirm}
			>{m.projects.attach.confirm}</button
		>
	</div>
</Dialog>

<style>
	td select {
		width: 100%;
	}
	.file {
		overflow-wrap: anywhere;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
