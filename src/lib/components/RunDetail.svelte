<script lang="ts">
	// One run: each step with its state and job, and what the researcher can do where a step
	// waits for them. Continuing is their decision, recorded with the time; nothing else counts
	// as one.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import type { RunView } from '#lib/bindings/RunView.js';
	import type { StepView } from '#lib/bindings/StepView.js';
	import { Action } from '#lib/action.svelte.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { app, go, refreshJobs } from '#lib/state.svelte.js';
	import { api, ask } from '#lib/tauri.js';
	import { tone } from '#lib/workflows.js';

	let { runId }: { runId: string } = $props();

	const m = messages();
	const w = m.workflows;
	let view = $state<RunView | null>(null);
	const action = new Action({ after: () => Promise.all([load(), refreshJobs()]) });

	async function load() {
		try {
			view = await api.getRun(runId);
		} catch (e) {
			action.fail(e);
		}
	}

	onMount(() => void load());
	$effect(() => {
		void app.runs;
		void load();
	});

	const approve = (step: number) => action.run(() => api.approveStep(runId, step));

	async function cancel() {
		if (await ask(w.cancelConfirm)) await action.run(() => api.cancelRun(runId));
	}

	async function forget() {
		if (!(await ask(w.forgetConfirm))) return;
		try {
			await api.deleteRun(runId);
			go({ name: 'workflows' });
		} catch (e) {
			action.fail(e);
		}
	}

	function notice(step: StepView, index: number): string | null {
		const input = view?.run.steps[index].input;
		switch (step.state) {
			case 'waiting':
				return input?.kind === 'step' ? w.waitingFor(input.step + 1) : null;
			case 'review':
				return w.reviewNotice;
			case 'attention':
				return w.attentionNotice;
			case 'paused':
				return w.pausedNotice;
			case 'blocked':
				return w.blockedNotice;
			case 'failed': {
				const id = view?.run.steps[index].progress.error ?? '';
				return w.failedNotice(describeError({ id }, m));
			}
			case 'running':
			case 'done':
			case 'done_with_errors':
			case 'cancelled':
				return null;
		}
	}
</script>

{#if view}
	{@const { run, status, steps } = view}
	<header class="masthead">
		<div class="title">
			<button class="btn ghost sm" onclick={() => go({ name: 'workflows' })}
				><Icon name="back" />{w.back}</button
			>
			<h1>{run.name}</h1>
			<p class="lede">
				<span class="chip {tone(status)}">{w.runStatus[status]}</span>
				{w.runOf(run.workflow_name, run.version)} · {w.files(run.sources.length)} ·
				{w.started(formatDateTime(run.created_at))}
			</p>
		</div>
		<div class="buttons">
			{#if status === 'running' || status === 'needs_you'}
				<button class="btn ghost danger" onclick={cancel} disabled={action.busy}>{w.cancel}</button>
			{/if}
			<button class="btn ghost" onclick={forget}>{w.forget}</button>
		</div>
	</header>

	<div class="sheet">
		{#if action.error}<p class="chip danger" role="alert">{action.error}</p>{/if}
		<ol class="steps">
			{#each steps as step, i (i)}
				{@const frozen = run.steps[i]}
				{@const said = notice(step, i)}
				{@const approval = frozen.progress.approval}
				<li class="card step">
					<div class="head">
						<span class="what">
							<Icon name={frozen.task} size={16} />
							<strong>{frozen.name}</strong>
							<span class="muted">{m.task[frozen.task]}</span>
						</span>
						<span class="chip {tone(step.state)}">{w.stepState[step.state]}</span>
					</div>
					<p class="muted reads">
						{w.reads}:
						{frozen.input.kind === 'sources'
							? w.readsFiles
							: w.readsStep(frozen.input.step + 1, run.steps[frozen.input.step]?.name ?? '')}
					</p>
					{#if step.job}
						{@const job = step.job}
						<p class="num facts">
							{m.status[job.status]} · {job.pieces_finished}/{job.pieces_total}
							{#if job.cost_usd !== null}· {formatUsd(job.cost_usd)}{/if}
						</p>
					{/if}
					{#if said}<p class="notice">{said}</p>{/if}
					{#if approval}
						<p class="muted approved">
							{approval.partial
								? w.approvedPartial(formatDateTime(approval.at))
								: w.approved(formatDateTime(approval.at))}
						</p>
					{/if}
					<div class="actions">
						{#if step.job}
							{@const job = step.job}
							<button class="btn sm" onclick={() => go({ name: 'jobs', jobId: job.id })}
								>{w.openJob}</button
							>
						{/if}
						{#if step.state === 'review'}
							<button class="btn sm primary" onclick={() => approve(i)} disabled={action.busy}
								><Icon name="check" />{w.continue}</button
							>
						{:else if step.state === 'attention'}
							<button class="btn sm primary" onclick={() => approve(i)} disabled={action.busy}
								>{w.continueAnyway}</button
							>
						{/if}
					</div>
				</li>
			{/each}
		</ol>
	</div>
{:else if action.error}
	<div class="sheet"><p class="chip danger" role="alert">{action.error}</p></div>
{:else}
	<div class="sheet"><p class="muted">{m.common.loading}</p></div>
{/if}

<style>
	.title {
		display: grid;
		gap: var(--space-2);
		justify-items: start;
	}
	.lede {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.buttons {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
	}
	.steps {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-3);
	}
	.step {
		display: grid;
		gap: var(--space-2);
		padding-top: var(--space-3);
	}
	.head {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.what {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex: 1;
		min-width: 0;
		flex-wrap: wrap;
	}
	.reads,
	.facts,
	.approved {
		margin: 0;
		font-size: var(--type-sm);
	}
	.actions {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.actions:empty {
		display: none;
	}
</style>
