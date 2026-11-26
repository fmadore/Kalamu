<script lang="ts" module>
	import type { ModelTrial } from '#lib/bindings/ModelTrial.js';
	import type { OwnModel } from '#lib/bindings/OwnModel.js';

	/** Where a model's trial stands, in Settings › Other servers. */
	export type TrialState =
		| { stage: 'waiting' }
		| { stage: 'trying'; trial: string }
		| { stage: 'done'; result: ModelTrial }
		| { stage: 'stopped' }
		/** The trial could not be made at all (the server gone, the model no longer on it). */
		| { stage: 'failed'; error: string }
		/** Added before at this address: its stored trial holds, and it is not tried again. */
		| { stage: 'kept'; model: OwnModel };
</script>

<script lang="ts">
	// One model's trial as it goes: waiting its turn, trying (with a Stop, since a model can take
	// minutes to load), then its two results, *Reads pages* and *Fills an analysis's form*, each
	// yes or no with what follows from it; or why there was no answer, with the server's own words.
	import { describeError } from '#lib/errors.js';
	import { formatDay, messages } from '#lib/i18n/index.js';

	let {
		state,
		model,
		onstop
	}: {
		state: TrialState;
		/** The model's name on the server, for the sentence when it is left out. */
		model: string;
		onstop: (trial: string) => void;
	} = $props();

	const m = messages();
	const s = m.servers;
</script>

{#snippet results(does: { reads_pages: boolean; fills_forms: boolean })}
	<dl class="results">
		<div>
			<dt>{s.readsPages}</dt>
			<dd>
				<span class="chip" class:ok={does.reads_pages}>{does.reads_pages ? s.yes : s.no}</span>
				<span class="muted">{does.reads_pages ? s.readsPagesYes : s.readsPagesNo}</span>
			</dd>
		</div>
		<div>
			<dt>{s.fillsForms}</dt>
			<dd>
				<span class="chip" class:ok={does.fills_forms}>{does.fills_forms ? s.yes : s.no}</span>
				<span class="muted">{does.fills_forms ? s.fillsFormsYes : s.fillsFormsNo}</span>
			</dd>
		</div>
	</dl>
	{#if does.reads_pages && does.fills_forms}
		<p class="muted small">{s.both}</p>
	{:else if !does.reads_pages && !does.fills_forms}
		<p class="warning small">
			{s.neither}
			{describeError({ id: 'server.did_nothing', params: { model } }, m)}
		</p>
	{/if}
{/snippet}

<!-- Read out as the trial moves on. -->
<div class="trial" role="status">
	{#if state.stage === 'waiting'}
		<p class="muted small">{s.waiting}</p>
	{:else if state.stage === 'trying'}
		{@const trial = state.trial}
		<div class="trying">
			<span class="pulse" aria-hidden="true"></span>
			<p class="small">{s.trying}</p>
			<button type="button" class="btn sm" onclick={() => onstop(trial)}>{s.stop}</button>
		</div>
	{:else if state.stage === 'stopped'}
		<p class="muted small">{s.stopped}</p>
	{:else if state.stage === 'failed'}
		<p class="err small">{state.error}</p>
	{:else if state.stage === 'kept'}
		{@render results(state.model)}
		{#if state.model.tried_at}<p class="muted small">
				{s.kept(formatDay(state.model.tried_at))}
			</p>{/if}
	{:else if state.result.failure}
		<p class="err small">
			{s.noAnswer(describeError({ id: state.result.failure }, m))}
		</p>
		{#if state.result.words}
			<figure class="words">
				<figcaption class="muted small">{s.said}</figcaption>
				<blockquote class="own-direction" lang="">{state.result.words}</blockquote>
			</figure>
		{/if}
	{:else}
		{@render results(state.result)}
	{/if}
</div>

<style>
	.trial {
		display: grid;
		gap: var(--space-2);
		min-width: 0;
	}
	.trial p {
		margin: 0;
	}
	.small {
		font-size: var(--type-sm);
	}
	.trying {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
	}
	.trying p {
		flex: 1 1 16rem;
		color: var(--text-secondary);
	}
	/* A slow beat while the server loads the model: something is happening, without a figure
	   that would promise how long. */
	.pulse {
		width: 8px;
		height: 8px;
		border-radius: 4px;
		background: var(--text-secondary);
		animation: pulse 1.4s ease-in-out infinite;
		flex: none;
	}
	@keyframes pulse {
		50% {
			opacity: 0.25;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.pulse {
			animation: none;
		}
	}
	/* Each result on its line: the question, then yes or no and what follows from it, the
	   questions in one column so the answers line up. */
	.results {
		display: grid;
		grid-template-columns: max-content minmax(0, 1fr);
		gap: var(--space-1) var(--space-3);
		margin: 0;
		font-size: var(--type-sm);
	}
	.results > div {
		display: contents;
	}
	.results dt {
		color: var(--text-secondary);
		font-weight: 500;
	}
	.results dd {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-1) var(--space-2);
		margin: 0;
	}
	.chip {
		min-width: 2.5rem;
		justify-content: center;
	}
	.err {
		color: var(--danger);
	}
	.warning {
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--warn-border);
		background: var(--warn-wash);
		border-radius: var(--radius-control);
	}
	.words {
		display: grid;
		gap: var(--space-1);
		margin: 0;
	}
	blockquote {
		margin: 0;
		max-height: 8rem;
		overflow: auto;
		padding: var(--space-2) var(--space-3);
		border-left: 2px solid var(--line-strong);
		background: var(--surface-1);
		font-size: var(--type-sm);
		overflow-wrap: anywhere;
		white-space: pre-wrap;
	}
	/* Stacked where the window leaves the answers too little room beside the questions. */
	@media (max-width: 960px) {
		.results {
			grid-template-columns: minmax(0, 1fr);
		}
		.results dd {
			margin-bottom: var(--space-1);
		}
	}
</style>
