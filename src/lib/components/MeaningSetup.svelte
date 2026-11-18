<script lang="ts">
	// The search by meaning's set-up, where it is needed: the project's model, a local one not yet
	// downloaded pointing to Settings, the researcher's consent for a cloud one, what preparing
	// will send and cost before anything is sent, the preparation's progress with pause and
	// cancel, what was spent, and what is kept on this computer with its removal. Nothing is sent
	// from here but by Prepare and Update; everything shown is read from Rust.
	import { onMount } from 'svelte';
	import ProgressBar from './ProgressBar.svelte';
	import type { EmbeddingSpace } from '#lib/bindings/EmbeddingSpace.js';
	import type { MeaningStatus } from '#lib/bindings/MeaningStatus.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, formatUsd, messages } from '#lib/i18n/index.js';
	import { meaningStep, usdText } from '#lib/meaning.js';
	import { providerNameOf } from '#lib/catalogText.js';
	import { app, go, toast } from '#lib/state.svelte.js';
	import { api, listenWhileMounted, on } from '#lib/tauri.js';

	let {
		projectId,
		status,
		reload
	}: {
		projectId: string;
		status: MeaningStatus;
		/** Reads the status again. */
		reload: () => Promise<void>;
	} = $props();
	const m = messages();
	const s = $derived(m.meaning.setup);

	let error = $state<string | null>(null);
	let busy = $state(false);
	const step = $derived(meaningStep(status));
	const model = $derived(status.model);
	const counts = $derived(status.counts);
	const number = (n: number) => n.toLocaleString(m.locale.tag);
	const providerName = (id: string) => (app.catalog ? providerNameOf(app.catalog, id) : id);
	const sameSpace = (a: EmbeddingSpace, b: EmbeddingSpace | undefined) =>
		!!b && a.provider === b.provider && a.model === b.model && a.dimensions === b.dimensions;
	const lastRun = $derived(status.runs[0]);

	async function act(work: () => Promise<unknown>) {
		error = null;
		busy = true;
		try {
			await work();
			await reload();
		} catch (e) {
			error = describeError(e, m);
		} finally {
			busy = false;
		}
	}

	function choose(value: string) {
		const [provider, ...rest] = value.split('/');
		if (!provider || rest.length === 0) return;
		void act(() => api.chooseMeaningModel(projectId, provider, rest.join('/')));
	}

	function remove(space: EmbeddingSpace) {
		void act(async () => {
			await api.removeMeaningSpace(projectId, space);
			toast(s.removed);
		});
	}

	// Progress after each batch, and the status read again while a run goes on, so a missed
	// event or a run started elsewhere still shows.
	onMount(() => {
		const stop = listenWhileMounted(() =>
			on.meaningProgress((p) => {
				if (p.project_id !== projectId) return;
				void reload();
			})
		);
		const timer = setInterval(() => {
			if (status.running) void reload();
		}, 1500);
		return () => {
			stop();
			clearInterval(timer);
		};
	});
</script>

<section class="setup" aria-label={s.title}>
	{#if step === 'no_models'}
		<p class="muted">{s.noModels}</p>
	{:else}
		<label class="pick">
			<span>{s.model}</span>
			<select
				value={status.choice ? `${status.choice.space.provider}/${status.choice.space.model}` : ''}
				disabled={busy || step === 'running'}
				onchange={(e) => choose(e.currentTarget.value)}
			>
				{#if !status.choice}<option value="" disabled>{s.chooseModel}</option>{/if}
				{#each status.models as mod (`${mod.provider}/${mod.id}`)}
					<option value={`${mod.provider}/${mod.id}`}>{mod.name}</option>
				{/each}
			</select>
		</label>
		{#if model}
			<p class="model-facts muted">
				{model.local ? s.local : s.cloud(providerName(model.provider))}
				· {model.multilingual ? s.languages.many : s.languages.english}
				{#if !model.local && model.input_price !== null}
					· <span class="num">{s.price(formatUsd(model.input_price))}</span>
				{/if}
			</p>
		{/if}

		{#if step === 'gone'}
			<p class="chip warn">{s.gone}</p>
		{:else if step === 'blocked'}
			{#if status.blocked === 'provider.not_installed'}
				<p class="notice">
					{s.notInstalled}
					<button class="link" onclick={() => go({ name: 'settings', tab: 'offline' })}
						>{s.openOffline}</button
					>
				</p>
			{:else if status.blocked === 'provider.missing_key'}
				<p class="notice">
					{s.missingKey}
					<button class="link" onclick={() => go({ name: 'settings', tab: 'keys' })}
						>{s.openKeys}</button
					>
				</p>
			{:else}
				<p class="chip danger">{describeError({ id: status.blocked }, m)}</p>
			{/if}
		{:else if step === 'consent' && model}
			<div class="consent" role="group" aria-labelledby="meaning-consent-title">
				<h3 id="meaning-consent-title">{s.consentTitle(providerName(model.provider))}</h3>
				<p>{s.consentBody(providerName(model.provider))}</p>
				<button
					class="btn sm"
					disabled={busy}
					onclick={() => act(() => api.consentMeaning(projectId))}>{s.consent}</button
				>
			</div>
		{/if}
		{#if model && !model.local && status.choice?.consented_at}
			<p class="muted small">
				{s.consented(providerName(model.provider), formatDateTime(status.choice.consented_at))}
			</p>
		{/if}

		{#if status.running}
			<div class="running" role="status">
				<p class="num">{s.running(status.running.done, status.running.total)}</p>
				<ProgressBar
					done={status.running.done}
					total={Math.max(status.running.total, 1)}
					label={s.title}
				/>
				<div class="row">
					<button class="btn sm" onclick={() => act(() => api.pauseMeaning(projectId))}
						>{s.pause}</button
					>
					<button class="btn sm danger" onclick={() => act(() => api.cancelMeaning(projectId))}
						>{s.cancel}</button
					>
				</div>
			</div>
		{:else if counts && model && (step === 'prepare' || step === 'ready')}
			{#if counts.pieces === 0}
				<p class="muted">{s.nothingRead}</p>
			{:else if counts.chunks === 0}
				<p class="ok-line">
					{status.prepared_at ? s.ready(formatDateTime(status.prepared_at)) : s.readyNever}
				</p>
			{:else}
				<div class="estimate">
					<h3>{s.estimate}</h3>
					<ul class="num">
						<li>{s.pieces(counts.pieces)}</li>
						{#if counts.prepared > 0}<li>{s.prepared(counts.prepared)}</li>{/if}
						{#if counts.missing > 0}<li>{s.missing(counts.missing)}</li>{/if}
						{#if counts.changed > 0}<li>{s.changed(counts.changed)}</li>{/if}
						<li>
							{model.local ? s.chunksLocal(counts.chunks) : s.chunks(counts.chunks)} ·
							{s.chars(number(counts.chars))}
						</li>
						{#if !model.local}
							<li>
								{s.tokens(number(counts.tokens))}
								{#if status.estimated_cost_usd !== null}
									· {status.estimated_cost_usd > 0 && status.estimated_cost_usd < 0.001
										? usdText(status.estimated_cost_usd, m)
										: s.cost(formatUsd(status.estimated_cost_usd))}
								{:else}
									· {s.noPrice}
								{/if}
							</li>
						{/if}
					</ul>
					{#if counts.skipped > 0}<p class="muted small">{s.skipped(counts.skipped)}</p>{/if}
					<p class="muted small">{model.local ? s.localNote : s.estimateNote}</p>
					<button
						class="btn sm"
						disabled={busy}
						onclick={() => act(() => api.prepareMeaning(projectId))}
					>
						{lastRun?.state === 'paused' || lastRun?.state === 'interrupted'
							? s.resume
							: counts.prepared > 0
								? s.update
								: s.prepare}
					</button>
				</div>
			{/if}
		{/if}
	{/if}
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}

	{#if status.runs.length > 0 || status.spaces.length > 0}
		<details class="record">
			<summary>{s.runs}</summary>
			{#if status.spent_usd !== null}
				<p class="num">{s.spent(usdText(status.spent_usd, m))}</p>
			{/if}
			{#if status.choice && status.choice.queries > 0}
				<p class="muted num">{s.queries(status.choice.queries)}</p>
			{/if}
			<ul class="runs num">
				{#each status.runs.slice(0, 8) as run (run.id)}
					<li>
						<span>{formatDateTime(run.started_at)}</span>
						<span>{run.space.model}</span>
						<span class:danger-text={run.state === 'failed'}>{s.state[run.state]}</span>
						<span>{s.run(run.chunks, number(run.tokens))}</span>
						{#if run.cost_usd !== null}<span>{usdText(run.cost_usd, m)}</span>{/if}
						{#if run.error}<span class="danger-text">{describeError({ id: run.error }, m)}</span
							>{/if}
					</li>
				{/each}
			</ul>
			{#if status.spaces.length > 0}
				<h4>{s.kept}</h4>
				<ul class="spaces">
					{#each status.spaces as size (`${size.space.provider}/${size.space.model}/${size.space.dimensions}`)}
						<li>
							<span class="num">{size.space.model}</span>
							<span class="muted num">{s.keptLine(size.chunks, s.size(size.bytes))}</span>
							{#if sameSpace(size.space, status.choice?.space)}
								<span class="chip">{s.inUse}</span>
							{/if}
							<button
								class="btn sm danger"
								disabled={busy || step === 'running'}
								onclick={() => remove(size.space)}>{s.remove}</button
							>
						</li>
					{/each}
				</ul>
				<p class="muted small">{s.lostNote}</p>
			{/if}
		</details>
	{/if}
</section>

<style>
	.setup {
		display: grid;
		gap: var(--space-2);
		max-width: 44rem;
		padding: var(--space-3) 0;
		border-top: 1px solid var(--line);
		border-bottom: 1px solid var(--line);
	}
	.pick {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.pick select {
		max-width: 100%;
	}
	.model-facts,
	.small {
		font-size: var(--type-xs);
	}
	.consent,
	.estimate,
	.running {
		display: grid;
		gap: var(--space-2);
	}
	.consent h3,
	.estimate h3,
	.record h4 {
		font-size: var(--type-sm);
		font-weight: 600;
		color: var(--text-bright);
	}
	.consent p {
		font-size: var(--type-sm);
	}
	.consent .btn,
	.estimate .btn {
		justify-self: start;
	}
	.estimate ul,
	.runs,
	.spaces {
		margin: 0;
		padding-left: 1.1rem;
		font-size: var(--type-sm);
	}
	.runs li,
	.spaces li {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
	.spaces {
		list-style: none;
		padding: 0;
	}
	.row {
		display: flex;
		gap: var(--space-2);
	}
	.notice {
		font-size: var(--type-sm);
	}
	.ok-line {
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.record summary {
		cursor: pointer;
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.record {
		display: grid;
		gap: var(--space-2);
	}
	.danger-text {
		color: var(--danger-text);
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font: inherit;
		font-weight: 500;
		text-decoration: underline;
		text-underline-offset: 0.18em;
		cursor: pointer;
	}
</style>
