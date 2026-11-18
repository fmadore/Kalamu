<script lang="ts">
	// Asking a model to correct a passage selected in the text review, and what it proposes.
	// First the request: the passage and the context sent around it, the model and the correction
	// prompt (the last used, else the job's own when it is a correction, else the first provider
	// with a key), who it goes to and what it costs. Then the wait, which can be stopped. Then the
	// proposal, its changes marked against the passage: accepted as it is or edited first, which
	// writes it into the researcher's corrections, or discarded. Nothing changes the text before
	// it is accepted; the model's own output never changes. Asked through the engine, which
	// records it and keeps its answer, so the same request again costs nothing.
	import Dialog from './Dialog.svelte';
	import type { Correction } from '#lib/bindings/Correction.js';
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PassageAsk } from '#lib/bindings/PassageAsk.js';
	import type { PassageEstimate } from '#lib/bindings/PassageEstimate.js';
	import type { PassageProposal } from '#lib/bindings/PassageProposal.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import { providerName as providerNameOf } from '#lib/catalogText.js';
	import { askConsent } from '#lib/dialogs.js';
	import { describeError, isAppError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { rulesFor } from '#lib/newJob.js';
	import { findModel, modelsFor, providerDefault } from '#lib/newJobModels.js';
	import {
		PASSAGE_LIMITS,
		charCount,
		passageAround,
		passageProblem,
		type Chosen
	} from '#lib/passage.js';
	import { app, hasKey, isInstalled } from '#lib/state.svelte.js';
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

	const catalog = $derived(app.catalog);
	const rule = $derived(rulesFor(app.rules, 'text_correction'));
	const providers = $derived(
		(catalog?.providers ?? []).filter(
			(pr) => catalog && modelsFor(catalog, rule, pr.id, TEXT).length > 0
		)
	);
	let presets = $state<Preset[]>([]);

	type Last = { provider: string; model: string; preset: string | null };
	function readLast(v: unknown): Last | null {
		const s = v as Partial<Last> | null;
		return typeof s?.provider === 'string' && typeof s.model === 'string'
			? { provider: s.provider, model: s.model, preset: s.preset ?? null }
			: null;
	}

	/** The set-up to start from: the last one used, the job's own when it corrects text, or the
	 * first provider with a key and its usual model. */
	function startingSetup(): Last {
		const offered = (s: Last | null) =>
			s !== null &&
			catalog !== null &&
			modelsFor(catalog, rule, s.provider, TEXT).some((x) => x.id === s.model);
		const last = recall(LAST_KEY, readLast, null);
		if (offered(last) && last) return last;
		const own = job.task === 'text_correction' ? job.setups[0] : undefined;
		const fromJob = own
			? { provider: own.provider, model: own.model, preset: own.prompt.preset_id }
			: null;
		if (offered(fromJob) && fromJob) return fromJob;
		const keyed = providers.find((pr) => hasKey(pr.id)) ?? providers[0];
		const id = keyed?.id ?? '';
		return {
			provider: id,
			model: catalog ? providerDefault(catalog, rule, id, TEXT) : '',
			preset: null
		};
	}
	const first = startingSetup();
	let provider = $state(first.provider);
	let model = $state(first.model);
	let preset = $state<string>(first.preset ?? 'text_correction.general');
	const models = $derived(catalog ? modelsFor(catalog, rule, provider, TEXT) : []);

	const providerName = $derived.by(() => {
		const found = catalog?.providers.find((pr) => pr.id === provider);
		return found ? providerNameOf(found) : provider;
	});
	const modelName = $derived(
		(catalog ? findModel(catalog, provider, model)?.name : undefined) ?? model
	);
	const local = $derived(catalog?.providers.find((pr) => pr.id === provider)?.local === true);
	const keyed = $derived(hasKey(provider));
	const installed = $derived(isInstalled(provider, model));

	type Stage = 'ask' | 'waiting' | 'proposal' | 'changed';
	let stage = $state<Stage>('ask');
	let estimate = $state<PassageEstimate | null>(null);
	let error = $state<string | null>(null);
	/** Why the request would be refused, from the engine: a text changed since, say. */
	let estimateError = $state<string | null>(null);
	/** The researcher stopped waiting for the last request. */
	let stopped = $state(false);
	let proposal = $state<PassageProposal | null>(null);
	let spans = $state<DiffSpan[]>([]);
	let editing = $state(false);
	let draft = $state('');
	/** Read out as it changes: the proposal ready. */
	let said = $state('');
	let asking = $state<string | null>(null);
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
			provider,
			model,
			preset_id: preset || null
		};
	}

	$effect(() => {
		void api
			.listPresets('text_correction')
			.then((list) => (presets = list))
			.catch(() => (presets = []));
	});

	// What the request would cost, and whether it is answered already, as the choice changes.
	$effect(() => {
		const ask = askOf('estimate');
		estimateError = null;
		if (stage !== 'ask' || problem || !ask.model) {
			estimate = null;
			return;
		}
		let gone = false;
		api
			.passageEstimate(ask)
			.then((e) => {
				if (!gone) estimate = e;
			})
			.catch((e) => {
				if (gone) return;
				estimate = null;
				estimateError = describeError(e, m);
			});
		return () => (gone = true);
	});

	$effect(() => {
		if (editing && editor) editor.focus();
	});

	async function ask() {
		error = null;
		stopped = false;
		if (!(await askConsent([provider], app.catalog))) return;
		keep(LAST_KEY, { provider, model, preset });
		const id = crypto.randomUUID();
		asking = id;
		stage = 'waiting';
		said = p.waiting(modelName);
		try {
			const answer = await api.askPassage(askOf(id));
			if (answer.status === 'answered' && answer.answer !== null) {
				proposal = answer;
				draft = answer.answer;
				spans = await api.diffTexts(around.passage, answer.answer).catch(() => []);
				stage = 'proposal';
				said = p.ready;
			} else if (answer.status === 'cancelled') {
				stage = 'ask';
				stopped = true;
				said = p.stopped;
			} else {
				stage = 'ask';
				error = p.failed(answer.error ?? '');
				said = '';
			}
		} catch (e) {
			stage = 'ask';
			error = describeError(e, m);
			said = '';
		} finally {
			asking = null;
		}
	}

	function stop() {
		if (asking) void api.cancelPassage(asking);
	}

	async function accept() {
		if (!proposal) return;
		error = null;
		const text = editing ? draft : (proposal.answer ?? '');
		try {
			const before = await api.acceptPassage(proposal.ask.id, text);
			onaccepted(proposal.ask.id, before);
		} catch (e) {
			if (isAppError(e) && e.id === 'passage.changed') {
				stage = 'changed';
				said = p.changed;
			} else {
				error = describeError(e, m);
			}
		}
	}

	/** Leaves the proposal unused, and closes. */
	function discard() {
		if (proposal) void api.discardPassage(proposal.ask.id).catch(() => undefined);
		onclose();
	}

	function escape() {
		if (stage === 'waiting') stop();
		if (stage === 'proposal' || stage === 'changed') discard();
		else onclose();
	}
</script>

<Dialog labelledby="passage-title" size="medium" onescape={escape}>
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

	{#if stage === 'ask'}
		<div class="pickers">
			<label class="field">
				<span>{m.newJob.provider}</span>
				<select
					bind:value={provider}
					onchange={() => catalog && (model = providerDefault(catalog, rule, provider, TEXT))}
				>
					{#each providers as pr (pr.id)}
						<option value={pr.id}>{providerNameOf(pr)}</option>
					{/each}
				</select>
			</label>
			<label class="field">
				<span>{m.newJob.model}</span>
				<select bind:value={model}>
					{#each models as md (md.id)}
						<option value={md.id}>{md.name}</option>
					{/each}
				</select>
			</label>
			<label class="field wide">
				<span>{m.newJob.prompt}</span>
				<select bind:value={preset}>
					{#each presets as pr (pr.id)}
						<option value={pr.id}>{pr.name}</option>
					{/each}
				</select>
			</label>
		</div>
		<p class="muted">
			{local ? p.sentOffline(modelName) : p.sentCloud(providerName, PASSAGE_LIMITS.context)}
		</p>
		{#if !keyed}
			<p class="notice" role="status">{p.noKey(providerName)}</p>
		{:else if !installed}
			<p class="notice" role="status">{p.notInstalled(modelName)}</p>
		{/if}
		{#if estimate}
			<p class="num">
				{#if estimate.cached}{p.costStored}{:else if estimate.offline}{p.costOffline}{:else if estimate.usd !== null}{p.costAbout(
						formatUsd(estimate.usd)
					)}{:else}{p.costUnknown}{/if}
			</p>
		{/if}
	{:else if stage === 'waiting'}
		<p class="waiting">{p.waiting(modelName)}</p>
	{:else if stage === 'proposal' && proposal}
		{#if editing}
			<label class="field">
				<span>{p.editLabel}</span>
				<textarea bind:this={editor} bind:value={draft} rows="5" spellcheck="false" lang=""
				></textarea>
			</label>
		{:else}
			<figure class="proposal">
				<figcaption class="muted">{p.proposal}</figcaption>
				<!-- prettier-ignore -->
				<p class="diff" lang="">{#each spans as s, i (i)}{#if s.kind === 'removed'}<del><span class="sr-only">{p.removed}: </span>{s.text}</del>{:else if s.kind === 'added'}<ins><span class="sr-only">{p.added}: </span>{s.text}</ins>{:else}{s.text}{/if}{:else}{proposal.answer}{/each}</p>
			</figure>
		{/if}
		{#each proposal.notes as note (note)}
			<p class="notice" role="note">{p.notes[note]}</p>
		{/each}
	{:else if stage === 'changed'}
		<p class="notice" role="alert">{p.changed}</p>
	{/if}

	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	{#if estimateError && stage === 'ask'}<p class="chip danger" role="alert">
			{estimateError}
		</p>{/if}
	<p class="sr-only" aria-live="polite">{said}</p>

	<div class="actions">
		{#if stage === 'ask'}
			<button class="btn" onclick={onclose}>{m.common.cancel}</button>
			<button
				class="btn primary"
				disabled={problem !== null || estimateError !== null || !model || !keyed || !installed}
				onclick={ask}>{p.ask}</button
			>
		{:else if stage === 'waiting'}
			<button class="btn" onclick={stop}>{p.stop}</button>
		{:else if stage === 'proposal'}
			<button class="btn" onclick={discard}>{p.discard}</button>
			{#if !editing}
				<button class="btn" onclick={() => (editing = true)}>{p.edit}</button>
			{/if}
			<button class="btn primary" disabled={editing && !draft.trim()} onclick={accept}
				>{p.accept}</button
			>
		{:else}
			<button class="btn primary" onclick={discard}>{p.lookAgain}</button>
		{/if}
	</div>
	{#if stopped && stage === 'ask'}<p class="muted small" role="status">{p.stopped}</p>{/if}
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
	del {
		color: var(--danger-text);
		text-decoration-thickness: 2px;
	}
	ins {
		text-decoration: none;
		background: var(--hand-wash);
		color: var(--text-bright);
		border-bottom: 2px solid var(--hand);
	}
	.pickers {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--space-3);
	}
	.pickers .wide {
		grid-column: 1 / -1;
	}
	.pickers select {
		width: 100%;
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
