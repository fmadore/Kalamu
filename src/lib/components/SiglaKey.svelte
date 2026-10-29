<script lang="ts">
	// The key to a comparison's sigla, under its text or answers: each set-up with its letter,
	// model and cost. The first is corrected and reviewed from the masthead; the others here.
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { siglum } from '#lib/apparatus.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';

	let {
		setups,
		viewOf,
		answered,
		corrected,
		onedit,
		onreviewed
	}: {
		setups: Setup[];
		viewOf: (setup: Setup) => PieceView | undefined;
		/** Whether a set-up has an answer here to correct. */
		answered: (view: PieceView) => boolean;
		corrected: (view: PieceView) => boolean;
		onedit: (setup: Setup, view: PieceView) => void;
		onreviewed: (setup: Setup, view: PieceView, reviewed: boolean) => void;
	} = $props();
	const m = messages();
</script>

<section class="key" aria-labelledby="sigla-head">
	<h2 id="sigla-head">{m.review.models}</h2>
	{#each setups as s, i (s.id)}
		{@const v = viewOf(s)}
		<div class="witness">
			<i class="siglum">{siglum(i)}</i>
			<span class="name mono">{s.label}</span>
			<span class="meta muted mono">
				{#if v?.status === 'failed'}<span class="failed">{m.status.failed}</span>
				{:else}{v?.model_version ?? s.model}{v?.cost_usd ? ` · ${formatUsd(v.cost_usd)}` : ''}{/if}
			</span>
			{#if i > 0 && v && answered(v)}
				<span class="tools">
					{#if corrected(v)}<span class="chip hand">{m.review.corrected}</span>{/if}
					<button class="btn sm" onclick={() => onedit(s, v)}>{m.review.edit}</button>
					<label>
						<input
							type="checkbox"
							checked={v.reviewed}
							onchange={(e) => onreviewed(s, v, e.currentTarget.checked)}
						/>
						{m.review.markReviewed}
					</label>
				</span>
			{/if}
		</div>
	{/each}
</section>

<style>
	.key {
		max-width: 44rem;
		margin-top: var(--space-6);
		padding-top: var(--space-4);
		border-top: 1px solid var(--line);
		display: grid;
		gap: var(--space-1);
		font-family: var(--font-ui);
	}
	h2 {
		margin-bottom: var(--space-1);
		font-size: var(--type-sm);
	}
	.witness {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		min-height: 2.5rem;
	}
	.name {
		font-size: var(--type-sm);
		color: var(--text-bright);
	}
	.meta {
		font-size: var(--type-xs);
	}
	.failed {
		color: var(--danger-text);
	}
	.tools {
		margin-left: auto;
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	label {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
</style>
