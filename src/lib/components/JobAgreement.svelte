<script lang="ts">
	// How far the models of an analysis agree over the whole job: one row per field (and pair of
	// set-ups, with more than two), with how it was measured, the share of agreement, Cohen's
	// kappa for a choice and how many answers were compared. The numbers are the engine's.
	import TermLink from './TermLink.svelte';
	import { agreementRows } from '#lib/agreement.js';
	import type { JobAgreement } from '#lib/bindings/JobAgreement.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { messages } from '#lib/i18n/index.js';

	let { agreement, setups }: { agreement: JobAgreement; setups: Setup[] } = $props();
	const m = messages();

	const rows = $derived(agreementRows(agreement, setups, m));
	/** With two set-ups there is one pair: no column to name it. */
	const pairs = $derived(setups.length > 2);
	const anyChoice = $derived(rows.some((r) => r.measure === 'choice'));
	const anyOverlap = $derived(rows.some((r) => r.measure === 'overlap'));
</script>

<section aria-labelledby="agreement-head" class="agreement">
	<h2 id="agreement-head">{m.agreement.title}<TermLink term="agreement" /></h2>
	<p class="intro">{m.agreement.intro}</p>
	<div class="frame">
		<table aria-label={m.agreement.title}>
			<thead>
				<tr>
					<th scope="col">{m.agreement.field}</th>
					{#if pairs}<th scope="col">{m.agreement.pair}</th>{/if}
					<th scope="col">{m.agreement.measure}</th>
					<th scope="col" class="num">{m.agreement.agreement}</th>
					<th scope="col" class="num">{m.agreement.kappa}</th>
					<th scope="col" class="num">{m.agreement.compared}</th>
				</tr>
			</thead>
			<tbody>
				{#each rows as row (`${row.key} ${row.pair.join('')}`)}
					<tr class:first={row.span > 0}>
						{#if row.span > 0}
							<th scope={row.span > 1 ? 'rowgroup' : 'row'} rowspan={row.span}>{row.label}</th>
						{/if}
						{#if pairs}
							<td>
								<span class="pair" aria-hidden="true">
									<i class="siglum">{row.pair[0]}</i><i class="siglum">{row.pair[1]}</i>
								</span><span class="sr-only">{m.agreement.pairOf(row.pair[0], row.pair[1])}</span>
							</td>
						{/if}
						<td class="muted">{m.agreement.measures[row.measure]}</td>
						<td class="num value">{row.agreement}</td>
						<td class="num" class:muted={row.measure === 'overlap'}>{row.kappa}</td>
						<td class="num muted">{row.pieces}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
	{#if anyChoice}<p class="note">{m.agreement.kappaNote}</p>{/if}
	{#if anyOverlap}<p class="note">{m.agreement.overlapNote}</p>{/if}
</section>

<style>
	.agreement {
		display: grid;
		gap: var(--space-3);
	}
	.intro,
	.note {
		margin: 0;
		max-width: 48rem;
		font-size: var(--type-sm);
		color: var(--text-muted);
	}
	.note {
		font-size: var(--type-xs);
	}
	/* Scrolls sideways on a narrow window rather than squeezing the numbers. */
	.frame {
		overflow-x: auto;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--type-sm);
	}
	th,
	td {
		padding: var(--space-2) var(--space-4);
		text-align: left;
		vertical-align: top;
		white-space: nowrap;
	}
	thead th {
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-muted);
		border-bottom: 1px solid var(--line);
	}
	tbody th {
		font-weight: 500;
		color: var(--text-bright);
		white-space: normal;
	}
	/* A hairline between fields, none between one field's pairs. */
	tbody tr.first:not(:first-child) > * {
		border-top: 1px solid var(--line-soft);
	}
	.num {
		text-align: right;
	}
	.value {
		color: var(--text-bright);
		font-weight: 500;
	}
	.pair {
		display: inline-flex;
		gap: var(--space-1);
	}
</style>
