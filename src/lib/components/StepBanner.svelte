<script lang="ts">
	// A job that is a step of a workflow says so where the researcher already is: on the job's
	// page, and in its review when the next step waits for them. Continue is offered here, so a
	// colleague who has checked the text never has to find the workflow's own page.
	import Icon from './Icon.svelte';
	import type { JobStep } from '#lib/bindings/JobStep.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { app, go, refreshJobs, refreshWaiting } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let { jobId, inReview = false }: { jobId: string; inReview?: boolean } = $props();

	const m = messages();
	const w = m.workflows;
	let found = $state<JobStep | null>(null);
	let error = $state<string | null>(null);
	let busy = $state(false);

	$effect(() => {
		void app.runs;
		const id = jobId;
		void api
			.jobStep(id)
			.then((s) => (found = s))
			.catch(() => (found = null));
	});

	const where = $derived(found ? found.view.steps[found.step]?.state : undefined);
	const waits = $derived(where === 'review' || where === 'attention');
	/** The steps that read this one, by name. */
	const next = $derived(
		found
			? found.view.run.steps
					.filter((s) => s.input.kind === 'step' && s.input.step === found!.step)
					.map((s) => s.name)
			: []
	);

	async function proceed() {
		if (!found) return;
		error = null;
		busy = true;
		try {
			await api.approveStep(found.view.run.id, found.step);
			await Promise.all([refreshWaiting(), refreshJobs()]);
			app.runs++;
		} catch (e) {
			error = describeError(e, m);
		} finally {
			busy = false;
		}
	}
</script>

{#if found && (waits || !inReview)}
	{@const { run } = found.view}
	<div class="step-banner" class:waits role="status">
		<Icon name="workflow" size={16} />
		<p>
			<strong>{w.banner.step(found.step + 1, run.steps.length, run.name)}</strong>
			{#if where === 'review'}
				{w.banner.review(next.join(', '))}
			{:else if where === 'attention'}
				{w.banner.attention(next.join(', '))}
			{:else if next.length > 0 && where !== 'done' && where !== 'done_with_errors'}
				{run.steps[found.step].review
					? w.banner.nextAfterCheck(next.join(', '))
					: w.banner.next(next.join(', '))}
			{/if}
			{#if error}<span class="error">{error}</span>{/if}
		</p>
		<span class="buttons">
			{#if where === 'review'}
				<button class="btn sm primary" onclick={proceed} disabled={busy}
					><Icon name="check" size={14} />{w.continue}</button
				>
			{:else if where === 'attention'}
				<button class="btn sm primary" onclick={proceed} disabled={busy}>{w.continueAnyway}</button>
			{/if}
			<button class="btn sm ghost" onclick={() => go({ name: 'workflows', runId: run.id })}
				>{w.banner.see}</button
			>
		</span>
	</div>
{/if}

<style>
	.step-banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-5);
		background: var(--surface-1);
		border-bottom: 1px solid var(--line);
		color: var(--text-body);
		font-size: var(--type-sm);
	}
	.step-banner.waits {
		background: var(--warn-wash);
		border-bottom-color: var(--warn-border);
		color: var(--warn-text);
	}
	p {
		flex: 1;
		margin: 0;
		min-width: 0;
	}
	strong {
		margin-right: var(--space-1);
	}
	.buttons {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.error {
		display: block;
		color: var(--danger-text);
	}
	@media (max-width: 760px) {
		.step-banner {
			flex-wrap: wrap;
		}
	}
</style>
