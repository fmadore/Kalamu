<script lang="ts">
	import TermLink from './TermLink.svelte';
	// New job's summary, in view while the form scrolls: what the job will cost, at the pace
	// chosen, model by model; what still stops it; the one action, Start; and saving the form as
	// a pipeline.
	import type { EstimateRow } from '#lib/bindings/EstimateRow.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';

	let {
		estimates,
		cost,
		total,
		video,
		error,
		blocker,
		ready,
		starting,
		canSave,
		naming = $bindable(),
		onstart,
		onsave
	}: {
		/** The estimate for each model of the job, a merging model on a row of its own. */
		estimates: EstimateRow[];
		/** A row's cost at the pace chosen; null when it cannot be known. */
		cost: (row: EstimateRow) => number | null;
		/** The whole job's cost at the pace chosen; null when it cannot be known. */
		total: number | null;
		/** Videos are added, whose cost is known only once they are read. */
		video: boolean;
		error: string | null;
		/** Why Start is unavailable, when the form cannot say it where the user is looking. */
		blocker: string | null;
		ready: boolean;
		starting: boolean;
		/** The form has set-ups to save as a pipeline. */
		canSave: boolean;
		/** The name being typed for a new pipeline, while saving one. */
		naming: string | null;
		onstart: () => void;
		/** Saves the form as a pipeline under the name typed. */
		onsave: () => void;
	} = $props();
	const m = messages();
</script>

<aside class="summary" aria-labelledby="estimate-label">
	<div class="estimate">
		<span id="estimate-label" class="label">{m.newJob.estimate}<TermLink term="tokens" /></span>
		<strong class="total mono"
			>{estimates.length > 0 && total !== null ? formatUsd(total) : '—'}</strong
		>
		{#if video}
			<p class="hint">{m.newJob.estimateVideo}</p>
		{:else if estimates.length > 0}
			<ul class="rows">
				{#each estimates as row, i (`${i}/${row.provider}/${row.model}/${row.merge}`)}
					<li>
						<span class="mono">{row.merge ? m.newJob.mergeRow(row.model) : row.model}</span>
						<span class="mono"
							>{cost(row) !== null ? formatUsd(cost(row) ?? 0) : m.newJob.estimateUnknown}</span
						>
					</li>
				{/each}
			</ul>
			<p class="hint">{m.newJob.estimateNote}</p>
		{/if}
	</div>

	{#if error}
		<p class="chip danger wrap-chip" role="alert">{error}</p>
	{/if}

	<div class="go">
		{#if blocker && !starting}
			<p class="blocker">{blocker}</p>
		{/if}
		<button class="btn primary lg" disabled={!ready || starting} onclick={onstart}>
			{starting ? m.newJob.starting : m.newJob.start}<span class="kbd"
				>{m.newJob.startShortcut}</span
			>
		</button>
		{#if naming === null}
			<button class="btn" disabled={!canSave} onclick={() => (naming = '')}
				>{m.newJob.pipelineSave}</button
			>
		{:else}
			<form
				class="naming"
				onsubmit={(e) => {
					e.preventDefault();
					onsave();
				}}
			>
				<!-- svelte-ignore a11y_autofocus -->
				<input
					type="text"
					bind:value={naming}
					placeholder={m.newJob.pipelineName}
					aria-label={m.newJob.pipelineName}
					autofocus
				/>
				<div class="naming-actions">
					<button class="btn" type="submit" disabled={!naming.trim()}>{m.common.save}</button>
					<button class="btn ghost" type="button" onclick={() => (naming = null)}
						>{m.common.cancel}</button
					>
				</div>
			</form>
		{/if}
	</div>
</aside>

<style>
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 60ch;
	}
	.summary {
		position: sticky;
		top: 0;
		align-self: start;
		display: grid;
		gap: var(--space-5);
		min-height: calc(100vh - 3.5rem);
		align-content: start;
		padding: var(--space-5);
		border-left: 1px solid var(--line);
		background: var(--surface-1);
	}
	.estimate {
		display: grid;
		gap: var(--space-2);
	}
	.total {
		font-size: 1.875rem;
		font-weight: 500;
		letter-spacing: -0.02em;
		color: var(--text-bright);
	}
	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
		border-top: 1px solid var(--line);
	}
	.rows li {
		display: flex;
		justify-content: space-between;
		gap: var(--space-3);
		padding: var(--space-2) 0;
		border-bottom: 1px solid var(--line);
		font-size: var(--type-xs);
	}
	.rows li > :first-child {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.wrap-chip {
		white-space: normal;
	}
	.go {
		display: grid;
		gap: var(--space-2);
	}
	/* What still stops Start, said where Start is, in the footer too. */
	.blocker {
		margin: 0;
		font-size: var(--type-xs);
		color: var(--warn-text);
	}
	.naming {
		display: grid;
		gap: var(--space-2);
	}
	.naming-actions {
		display: flex;
		gap: var(--space-2);
	}
	@media (max-width: 1100px) {
		.summary {
			position: sticky;
			bottom: 0;
			top: auto;
			min-height: 0;
			grid-template-columns: minmax(0, 1fr) auto;
			align-items: center;
			padding: var(--space-3) var(--space-5);
			border-left: 0;
			border-top: 1px solid var(--line);
			box-shadow: 0 -10px 24px -18px rgba(23, 22, 18, 0.35);
		}
		.summary .rows,
		.summary .hint {
			display: none;
		}
		.total {
			font-size: var(--type-xl);
		}
	}
</style>
