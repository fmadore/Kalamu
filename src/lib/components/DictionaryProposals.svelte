<script lang="ts">
	// Merges proposed by a model for one field of the dictionary. Asking runs an ordinary job
	// (its cost, progress and answers are in Jobs); the proposals are read back against the rows
	// sent, and each waits for the researcher: merged as chosen, turned down, or kept apart.
	import { providerName as providerNameOf } from '#lib/catalogText.js';
	import Icon from './Icon.svelte';
	import type { Item } from '#lib/bindings/Item.js';
	import type { Pace } from '#lib/bindings/Pace.js';
	import type { Proposal } from '#lib/bindings/Proposal.js';
	import type { ProposalPlan } from '#lib/bindings/ProposalPlan.js';
	import type { ProposalsView } from '#lib/bindings/ProposalsView.js';
	import { Action } from '#lib/action.svelte.js';
	import { formatDate, formatUsd, messages } from '#lib/i18n/index.js';
	import { modelsFor, providerDefault, rulesFor } from '#lib/newJob.js';
	import { app, go, hasKey, refreshJobs } from '#lib/state.svelte.js';
	import { api, askConsent, providersOf } from '#lib/tauri.js';

	let {
		projectId,
		namespace,
		fieldLabel,
		scope,
		onchange
	}: {
		projectId: string;
		namespace: string;
		fieldLabel: string;
		/** The collection shown, or '' for the whole project. */
		scope: string;
		/** A proposal was merged or kept apart: the dictionary reads its rows again. */
		onchange: () => Promise<void>;
	} = $props();

	const m = messages();
	const p = $derived(m.dictionary.proposals);
	/** Models read the rows as text. */
	const TEXT = 'text/plain';

	let view = $state<ProposalsView | null>(null);
	const action = new Action({ after: () => load() });
	let asking = $state(false);
	let plan = $state<ProposalPlan | null>(null);
	let provider = $state('');
	let model = $state('');
	let economy = $state(false);
	let cost = $state<{ usd: number | null; economy: number | null } | null>(null);
	let starting = $state(false);
	let showDismissed = $state(false);
	/** Per proposal: the members left out of the merge, and the label as edited. */
	let left = $state<Record<string, string[]>>({});
	let labels = $state<Record<string, string>>({});

	const catalog = $derived(app.catalog);
	const rule = $derived(rulesFor(app.rules, 'analysis'));
	const providers = $derived(
		(catalog?.providers ?? []).filter(
			(pr) => catalog && modelsFor(catalog, rule, pr.id, TEXT).length > 0
		)
	);
	const models = $derived(catalog ? modelsFor(catalog, rule, provider, TEXT) : []);
	const providerName = $derived.by(() => {
		const found = providers.find((pr) => pr.id === provider);
		return found ? providerNameOf(found) : provider;
	});
	const requests = $derived((view?.requests ?? []).filter((r) => r.namespace === namespace));
	const open = $derived(
		(view?.proposals ?? []).filter((x) => x.namespace === namespace && !x.dismissed)
	);
	const dismissed = $derived(
		(view?.proposals ?? []).filter((x) => x.namespace === namespace && x.dismissed)
	);

	async function load() {
		try {
			view = await api.dictionaryProposals(projectId);
		} catch (e) {
			action.fail(e);
		}
	}

	/** After a merge or a pair kept apart, the dictionary's rows are read again too. */
	async function changed() {
		await onchange();
		await load();
	}

	async function ask() {
		action.error = null;
		try {
			plan = await api.dictionaryPlan(projectId, scope || null, namespace);
			if (!catalog) return;
			const keyed = providers.find((pr) => hasKey(pr.id)) ?? providers[0];
			provider = keyed?.id ?? '';
			model = providerDefault(catalog, rule, provider, TEXT);
			asking = true;
		} catch (e) {
			action.fail(e);
		}
	}

	function setup() {
		return {
			provider,
			model,
			preset_id: 'dictionary.merges',
			label: null,
			options: { temperature: null, diarize: false, timestamps: false, speakers: null }
		};
	}

	// What the groups will cost with the model chosen; economy only where it batches.
	$effect(() => {
		const sources = plan ? $state.snapshot(plan.sources) : [];
		if (!model || sources.length === 0) {
			cost = null;
			return;
		}
		void api
			.estimateCost({ task: 'analysis', sources, setups: [setup()], sample_pages: null })
			.then(({ rows }) => {
				const r = rows[0]?.estimate;
				cost = r ? { usd: r.usd, economy: r.economy_usd } : null;
				if (r?.economy_usd === null) economy = false;
			})
			.catch(() => (cost = null));
	});

	async function start() {
		action.error = null;
		starting = true;
		try {
			const pace: Pace = economy ? 'economy' : 'fast';
			if (!(await askConsent(providersOf([setup()]), app.catalog))) return;
			await api.dictionaryPropose(
				projectId,
				scope || null,
				namespace,
				p.jobName(fieldLabel),
				setup(),
				pace
			);
			asking = false;
			plan = null;
			await refreshJobs();
			await load();
		} catch (e) {
			action.fail(e);
		} finally {
			starting = false;
		}
	}

	function chosen(x: Proposal): Item[] {
		const out = left[x.key] ?? [];
		return x.members.filter((mb) => !out.includes(mb.key)).map((mb) => mb.item);
	}

	function toggle(x: Proposal, key: string) {
		const out = left[x.key] ?? [];
		left[x.key] = out.includes(key) ? out.filter((k) => k !== key) : [...out, key];
	}

	function merge(event: SubmitEvent, x: Proposal) {
		event.preventDefault();
		const label = (labels[x.key] ?? x.label).trim();
		void action.run(() => api.dictionaryMerge(projectId, namespace, chosen(x), label), changed);
	}

	$effect(() => {
		void app.jobs;
		void projectId;
		void load();
	});
</script>

<section class="proposals" aria-labelledby="proposals-title">
	<div class="head">
		<h3 id="proposals-title">{p.title}</h3>
		{#if !asking}
			<button class="btn sm" onclick={ask}><Icon name="merge" size={14} />{p.ask}</button>
		{/if}
	</div>
	{#if action.error}<p class="chip danger" role="alert">{action.error}</p>{/if}

	{#if asking && plan}
		<div class="ask card">
			<p>
				{p.plan(fieldLabel, plan.rows, plan.sources.length)}
				<button class="link" onclick={() => go({ name: 'prompts', presetId: 'dictionary.merges' })}
					>{p.prompt}</button
				>
			</p>
			<div class="pickers">
				<label class="pick">
					<span>{m.newJob.provider}</span>
					<select
						bind:value={provider}
						onchange={() => catalog && (model = providerDefault(catalog, rule, provider, TEXT))}
					>
						{#each providers as pr (pr.id)}
							<option value={pr.id}>{pr.name}</option>
						{/each}
					</select>
				</label>
				<label class="pick">
					<span>{m.newJob.model}</span>
					<select bind:value={model}>
						{#each models as md (md.id)}
							<option value={md.id}>{md.name}</option>
						{/each}
					</select>
				</label>
				{#if cost?.economy !== null && cost?.economy !== undefined}
					<label class="check">
						<input type="checkbox" bind:checked={economy} />
						<span>{p.economy}</span>
					</label>
				{/if}
			</div>
			{#if provider && !hasKey(provider)}
				<p class="warning">{p.noKey(providerName)}</p>
			{/if}
			<p class="muted">{p.sends(providerName)}</p>
			<p class="num">
				{#if cost === null || cost.usd === null}
					{p.costUnknown}
				{:else}
					{p.cost(formatUsd(economy && cost.economy !== null ? cost.economy : cost.usd))}
				{/if}
			</p>
			<div class="actions">
				<button
					class="btn primary sm"
					disabled={starting || !model || !hasKey(provider)}
					onclick={start}>{p.start}</button
				>
				<button class="btn ghost sm" onclick={() => (asking = false)}>{m.dictionary.cancel}</button>
			</div>
		</div>
	{/if}

	{#each requests as r (r.id)}
		<p class="request muted">
			<span
				>{p.request(
					m.status[r.status],
					r.answered,
					r.groups,
					r.proposals,
					formatDate(r.created_at)
				)}</span
			>
			{#if r.unknown > 0}<span class="chip">{p.unknown(r.unknown)}</span>{/if}
			<button class="btn ghost sm" onclick={() => go({ name: 'jobs', jobId: r.job_id })}
				>{p.openJob}</button
			>
			<button
				class="btn ghost sm"
				title={p.forgetHint}
				onclick={() => action.run(() => api.dictionaryForgetRequest(projectId, r.id))}
				>{p.forget}</button
			>
		</p>
	{/each}

	{#if requests.length > 0 && open.length === 0}
		<p class="muted">{p.none}</p>
	{/if}

	<ul class="list">
		{#each open as x (x.key)}
			<li class="proposal card">
				<form onsubmit={(e) => merge(e, x)}>
					<div class="top">
						<input
							type="text"
							value={labels[x.key] ?? x.label}
							oninput={(e) => (labels[x.key] = e.currentTarget.value)}
							aria-label={m.dictionary.mergeLabel}
						/>
						<span class="chip certainty {x.certainty}" title={p.certaintyHint}
							>{p.certainty[x.certainty as keyof typeof p.certainty] ?? x.certainty}</span
						>
					</div>
					{#if x.reason}<p class="reason">{x.reason}</p>{/if}
					<ul class="members">
						{#each x.members as mb (mb.key)}
							<li>
								<label>
									<input
										type="checkbox"
										checked={!(left[x.key] ?? []).includes(mb.key)}
										onchange={() => toggle(x, mb.key)}
									/>
									<span>{mb.label}</span>
									{#if mb.forms.length > 1 || mb.forms[0] !== mb.label}
										<span class="muted forms">{mb.forms.join('; ')}</span>
									{/if}
								</label>
							</li>
						{/each}
					</ul>
					<div class="actions">
						<button
							class="btn primary sm"
							type="submit"
							disabled={chosen(x).length < 2 || !(labels[x.key] ?? x.label).trim()}
							>{m.dictionary.mergeConfirm}</button
						>
						<button
							class="btn sm"
							type="button"
							onclick={() =>
								action.run(() => api.dictionaryDismiss(projectId, namespace, x.key, true))}
							>{p.dismiss}</button
						>
						{#if x.members.length === 2}
							<button
								class="btn ghost sm"
								type="button"
								title={m.dictionary.keepApartHint}
								onclick={() =>
									action.run(
										() =>
											api.dictionaryKeepApart(
												projectId,
												namespace,
												x.members[0].item,
												x.members[1].item
											),
										changed
									)}>{m.dictionary.keepApart}</button
							>
						{/if}
					</div>
				</form>
			</li>
		{/each}
	</ul>

	{#if dismissed.length > 0}
		<button class="btn ghost sm" onclick={() => (showDismissed = !showDismissed)}
			>{p.dismissedCount(dismissed.length)}</button
		>
		{#if showDismissed}
			<ul class="dismissed">
				{#each dismissed as x (x.key)}
					<li>
						<span>{x.members.map((mb) => mb.label).join(' · ')}</span>
						<button
							class="btn ghost sm"
							onclick={() =>
								action.run(() => api.dictionaryDismiss(projectId, namespace, x.key, false))}
							>{p.takeBack}</button
						>
					</li>
				{/each}
			</ul>
		{/if}
	{/if}
</section>

<style>
	.proposals {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-2);
		margin-block: var(--space-3);
	}
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}
	h3 {
		margin: 0;
	}
	/* A long button label wraps in a narrow window. */
	.head .btn {
		white-space: normal;
		text-align: left;
	}
	.ask {
		display: grid;
		gap: var(--space-2);
	}
	.ask p {
		margin: 0;
	}
	.pickers,
	.actions,
	.request,
	.top {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.request {
		margin: 0;
	}
	.pick,
	.check {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.request span:first-child {
		flex: 1 1 auto;
	}
	.list,
	.members,
	.dismissed {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	.proposal form {
		display: grid;
		gap: var(--space-2);
	}
	.top input {
		flex: 1 1 16rem;
		min-width: 0;
	}
	.reason {
		margin: 0;
	}
	.members label {
		display: flex;
		align-items: baseline;
		gap: var(--space-2);
	}
	.forms {
		overflow-wrap: anywhere;
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
	.dismissed li {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}
</style>
