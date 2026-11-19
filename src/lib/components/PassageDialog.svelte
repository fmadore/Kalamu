<script lang="ts">
	// Asking a model to correct a passage selected in the text review, and what it proposes.
	// First the request: the passage and the context sent around it, the model and the correction
	// prompt (the last used, else the job's own when it is a correction, else the first provider
	// with a key), who it goes to and what it costs. Then the wait, which can be stopped. Then the
	// proposal, its changes marked against the passage: accepted as it is or edited first, which
	// writes it into the researcher's corrections, or discarded. Nothing changes the text before
	// it is accepted; the model's own output never changes. Asked through the engine, which
	// records it and keeps its answer, so the same request again costs nothing.
	import AskStatus from './AskStatus.svelte';
	import Dialog from './Dialog.svelte';
	import DiffSpans from './DiffSpans.svelte';
	import ModelPickers from './ModelPickers.svelte';
	import type { Correction } from '#lib/bindings/Correction.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PassageAsk } from '#lib/bindings/PassageAsk.js';
	import type { PassageProposal } from '#lib/bindings/PassageProposal.js';
	import { messages } from '#lib/i18n/index.js';
	import { ModelChoice, ModelRequest, readSetup } from '#lib/modelAsk.svelte.js';
	import {
		PASSAGE_LIMITS,
		charCount,
		passageAround,
		passageProblem,
		type Chosen
	} from '#lib/passage.js';
	import { hasKey } from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';
	import { api } from '#lib/tauri.js';

	let {
		job,
		setupId,
		sourceId,
		index,
		base,
		chosen,
		onaccepted,
		onclose
	}: {
		job: Job;
		/** The set-up whose text was selected. */
		setupId: string;
		sourceId: string;
		index: number;
		/** The piece's text as the review shows it. */
		base: string;
		chosen: Chosen;
		/** The proposal written into the corrections: its id and the correction it replaced. */
		onaccepted: (id: string, before: Correction) => void;
		onclose: () => void;
	} = $props();
	const m = messages();
	const p = m.passage;
	/** Corrections read the text as text. */
	const TEXT = ['text/plain'];
	const LAST_KEY = 'kalamu.passage.setup';

	// svelte-ignore state_referenced_locally
	const around = passageAround(base, chosen.start, chosen.end);
	const problem = passageProblem(around.passage);
	const length = charCount(around.passage);

	// The set-up to start from: the last one used, the job's own when it corrects text, or the
	// first provider with a key and its usual model.
	// svelte-ignore state_referenced_locally
	const own = job.task === 'text_correction' ? job.setups[0] : undefined;
	const choice = new ModelChoice(
		'text_correction',
		TEXT,
		hasKey,
		[
			recall(LAST_KEY, readSetup, null),
			own ? { provider: own.provider, model: own.model, preset: own.prompt.preset_id } : null
		],
		'text_correction.general'
	);
	const request = new ModelRequest<PassageProposal, Correction>(p, {
		cancel: api.cancelPassage,
		accept: api.acceptPassage,
		discard: api.discardPassage,
		changed: 'passage.changed'
	});
	let editor = $state<HTMLTextAreaElement | null>(null);

	function askOf(id: string): PassageAsk {
		return {
			id,
			job_id: job.id,
			setup_id: setupId,
			source_id: sourceId,
			index,
			before: around.before,
			passage: around.passage,
			after: around.after,
			at: around.at,
			provider: choice.provider,
			model: choice.model,
			preset_id: choice.preset || null
		};
	}

	$effect(() => choice.loadPresets());

	// What the request would cost, and whether it is answered already, as the choice changes.
	$effect(() => {
		const ask = askOf('estimate');
		return request.estimateWith(problem || !ask.model ? null : () => api.passageEstimate(ask));
	});

	$effect(() => {
		if (request.editing && editor) editor.focus();
	});

	function ask() {
		void request.ask({
			provider: choice.provider,
			modelName: choice.modelName,
			remember: () =>
				keep(LAST_KEY, { provider: choice.provider, model: choice.model, preset: choice.preset }),
			send: (id) => api.askPassage(askOf(id)),
			base: around.passage
		});
	}
</script>

<Dialog labelledby="passage-title" size="medium" onescape={() => request.escape(onclose)}>
	<h2 id="passage-title">{p.title}</h2>

	<figure class="sent">
		<figcaption class="muted">
			{p.passage} · <span class="num">{p.chars(length)}</span>
		</figcaption>
		<blockquote lang="">
			{#if around.before}<span class="context" title={p.context}>{around.before}</span>{/if}<mark
				class="passage">{around.passage}</mark
			>{#if around.after}<span class="context" title={p.context}>{around.after}</span>{/if}
		</blockquote>
		{#if around.before || around.after}
			<p class="muted small"><span class="swatch" aria-hidden="true"></span>{p.context}</p>
		{/if}
	</figure>

	{#if chosen.cut}<p class="notice" role="note">{p.cut}</p>{/if}

	{#if problem === 'short'}
		<p class="chip danger" role="alert">{p.tooShort(PASSAGE_LIMITS.min)}</p>
	{:else if problem === 'long'}
		<p class="chip danger" role="alert">{p.tooLong(length, PASSAGE_LIMITS.max)}</p>
	{/if}

	{#if request.stage === 'ask'}
		<ModelPickers {choice} />
		<p class="muted">
			{choice.local
				? p.sentOffline(choice.modelName)
				: p.sentCloud(choice.providerName, PASSAGE_LIMITS.context)}
		</p>
		<AskStatus {choice} estimate={request.estimate} texts={p} />
	{:else if request.stage === 'waiting'}
		<p class="waiting">{p.waiting(choice.modelName)}</p>
	{:else if request.stage === 'answer' && request.answer}
		{@const proposal = request.answer}
		{#if request.editing}
			<label class="field">
				<span>{p.editLabel}</span>
				<textarea bind:this={editor} bind:value={request.draft} rows="5" spellcheck="false" lang=""
				></textarea>
			</label>
		{:else}
			<figure class="proposal">
				<figcaption class="muted">{p.proposal}</figcaption>
				<!-- prettier-ignore -->
				<p class="diff" lang=""><DiffSpans spans={request.spans} answer={proposal.answer ?? ''} removed={p.removed} added={p.added} /></p>
			</figure>
		{/if}
		{#each proposal.notes as note (note)}
			<p class="notice" role="note">{p.notes[note]}</p>
		{/each}
	{:else if request.stage === 'changed'}
		<p class="notice" role="alert">{p.changed}</p>
	{/if}

	{#if request.error}<p class="chip danger" role="alert">{request.error}</p>{/if}
	{#if request.estimateError && request.stage === 'ask'}<p class="chip danger" role="alert">
			{request.estimateError}
		</p>{/if}
	<p class="sr-only" aria-live="polite">{request.said}</p>

	<div class="actions">
		{#if request.stage === 'ask'}
			<button class="btn" onclick={onclose}>{m.common.cancel}</button>
			<button
				class="btn primary"
				disabled={problem !== null ||
					request.estimateError !== null ||
					!choice.model ||
					!choice.keyed ||
					!choice.installed}
				onclick={ask}>{p.ask}</button
			>
		{:else if request.stage === 'waiting'}
			<button class="btn" onclick={() => request.stop()}>{p.stop}</button>
		{:else if request.stage === 'answer'}
			<button class="btn" onclick={() => request.discard(onclose)}>{p.discard}</button>
			{#if !request.editing}
				<button class="btn" onclick={() => (request.editing = true)}>{p.edit}</button>
			{/if}
			<button
				class="btn primary"
				disabled={request.editing && !request.draft.trim()}
				onclick={() => request.accept(onaccepted)}>{p.accept}</button
			>
		{:else}
			<button class="btn primary" onclick={() => request.discard(onclose)}>{p.lookAgain}</button>
		{/if}
	</div>
	{#if request.stopped && request.stage === 'ask'}<p class="muted small" role="status">
			{p.stopped}
		</p>{/if}
</Dialog>

<style>
	h2 {
		margin: 0;
		font-size: var(--type-lg);
	}
	figure {
		margin: 0;
		display: grid;
		gap: var(--space-2);
	}
	blockquote,
	.diff {
		margin: 0;
		max-height: 12rem;
		overflow: auto;
		padding: var(--space-3);
		border-radius: var(--radius-control);
		background: var(--surface-1);
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.6;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		color: var(--text-body);
	}
	.context {
		color: var(--text-faint);
	}
	.passage {
		background: var(--hand-wash);
		color: var(--text-bright);
		border-radius: 2px;
	}
	.swatch {
		display: inline-block;
		width: 0.75rem;
		height: 0.75rem;
		margin-right: var(--space-2);
		vertical-align: -0.1em;
		border-radius: 2px;
		background: var(--text-faint);
	}
	.small {
		margin: 0;
		font-size: var(--type-xs);
	}
	textarea {
		width: 100%;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.6;
	}
	.waiting {
		color: var(--text-secondary);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	p {
		margin: 0;
	}
</style>
