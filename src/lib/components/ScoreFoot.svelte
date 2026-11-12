<script lang="ts">
	// The foot of the review's text: each set-up's error rate against the pages reviewed, or the
	// single model's, or the model and what the page cost; and what the marks in the text mean.
	import Missing from './Missing.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { ComparisonReport } from '#lib/bindings/ComparisonReport.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { formatPercent, messages } from '#lib/i18n/index.js';
	import { pieceCost } from '#lib/pieces.js';

	let {
		report,
		setups,
		ownScore,
		lead,
		compared,
		correction,
		corrected
	}: {
		report: ComparisonReport | null;
		setups: Setup[];
		/** A single model's error rate, said in words, once pages are reviewed. */
		ownScore: string | null;
		/** The first set-up's piece shown. */
		lead: PieceView | undefined;
		/** Whether other readings are compared with the text, their differences marked. */
		compared: boolean;
		/** A correction's review, whose marks are changes rather than disagreements. */
		correction: boolean;
		/** Whether the text shown holds the researcher's corrections. */
		corrected: boolean;
	} = $props();
	const m = messages();
</script>

<span class="mono">
	{#if report && setups.length > 1}
		{m.review.cer}{m.common.colon}
		{#each report.scores as s, i (s.setup_id)}
			<span
				class="score"
				title={s.wer !== null
					? `${m.review.wer}${m.common.colon} ${formatPercent(s.wer)}`
					: undefined}
				>{siglum(i).toUpperCase()}
				{#if s.cer !== null}{formatPercent(s.cer)}{:else}<Missing />{/if}</span
			>
		{/each}
		{#if report.scores.some((s) => s.cer !== null)}<span class="hint"
				>· {m.review.scoredPages(report.scores[0]?.scored_pages ?? 0)}</span
			>{:else}<span class="hint">· {m.review.noReference}</span>{/if}
	{:else if ownScore}
		<span title={m.scores.note}>{ownScore}</span>
	{:else}
		{lead?.model_version ?? setups[0]?.model ?? ''}{pieceCost(lead, m)}
	{/if}
</span>
{#if compared || corrected}
	<span class="legend">
		{#if compared}<span
				><i class="key marker" aria-hidden="true"></i>{correction
					? m.review.legendChanged
					: m.review.legendDiffer}</span
			>{/if}
		<span><i class="key hand" aria-hidden="true"></i>{m.review.legendCorrected}</span>
	</span>
{/if}

<style>
	.score {
		margin-left: var(--space-2);
		color: var(--text-bright);
	}
	.hint {
		font-family: var(--font-ui);
		margin-left: var(--space-1);
	}
	.legend {
		display: inline-flex;
		gap: var(--space-4);
	}
	.legend > span {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.key {
		display: inline-block;
		width: 0.875rem;
	}
	.key.marker {
		height: 0.625rem;
		border-radius: 2px;
		background: var(--marker);
	}
	.key.hand {
		height: 2px;
		background: var(--hand);
	}
</style>
