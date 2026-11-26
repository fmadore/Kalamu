<script lang="ts">
	// Settings › Other servers: the researcher's own servers, each with where it is, whether it
	// holds a key, and its models with what each is offered for and the day it was tried; a model
	// tried again after it was updated on the server; a server changed or removed; and the form
	// that adds one. Until a server is added, two sentences say what this is for.
	import { onMount } from 'svelte';
	import type { OwnModel } from '#lib/bindings/OwnModel.js';
	import type { OwnServer } from '#lib/bindings/OwnServer.js';
	import { ownOf } from '#lib/catalogText.js';
	import { ask } from '#lib/dialogs.js';
	import { describeError, isAppError } from '#lib/errors.js';
	import { guideUrl } from '#lib/guide.js';
	import { formatDay, messages } from '#lib/i18n/index.js';
	import { didSomething, draftOfSaved, offeredFor } from '#lib/servers.js';
	import { app, refreshKeys, toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';
	import ExternalLink from './ExternalLink.svelte';
	import ServerForm from './ServerForm.svelte';
	import TrialLine, { type TrialState } from './TrialLine.svelte';

	const m = messages();
	const s = m.servers;

	let servers = $state<OwnServer[]>([]);
	let loaded = $state(false);
	/** The form: a server added (`'new'`), or the id of the server changed. */
	let form = $state<string | null>(null);
	/** A model tried again from the list, by server and model. */
	let again = $state<Record<string, TrialState>>({});
	const againKey = (server: OwnServer, model: OwnModel) => JSON.stringify([server.id, model.id]);

	async function reread() {
		try {
			servers = await api.listServers();
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		} finally {
			loaded = true;
		}
	}
	onMount(reread);

	/** A server saved or removed: the list, and the catalogue every picker reads, read again. */
	async function changed() {
		await reread();
		await Promise.all([
			refreshKeys().catch(() => undefined),
			(async () => {
				app.catalog = await api.getCatalog();
				app.rules = await api.taskRules();
			})().catch(() => undefined)
		]);
	}

	/** Host and port, as the address shows them. */
	function hostOf(address: string): string {
		try {
			return new URL(address).host;
		} catch {
			return address;
		}
	}

	async function remove(server: OwnServer) {
		const agreed = await ask(s.removeAsk(server.name), { ok: s.remove, cancel: m.common.cancel });
		if (!agreed) return;
		try {
			await api.removeServer(server.id);
			toast(s.removed(server.name));
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
		await changed();
	}

	/** Tries a model again as the server has it now, then saves what it showed: a model that
	 *  now does neither is left out; one that did not answer keeps its earlier trial. */
	async function tryAgain(server: OwnServer, model: OwnModel) {
		const at = againKey(server, model);
		const trial = crypto.randomUUID();
		again[at] = { stage: 'trying', trial };
		try {
			const found = await api.findServer(server.address, null, server.id);
			const now = found.models.find((x) => x.id === model.id);
			if (!now) {
				again[at] = { stage: 'failed', error: s.gone(model.id) };
				return;
			}
			const result = await api.tryServerModel(
				trial,
				{
					id: server.id,
					name: server.name,
					address: server.address,
					dialect: server.dialect,
					asks_key: server.key
				},
				{ id: model.id, context: model.context, vision: now.vision, digest: now.digest },
				null
			);
			again[at] = { stage: 'done', result };
			if (result.failure) return;
			await api.saveServer(draftOfSaved(server, didSomething(result) ? [] : [model.id]), null);
			await changed();
		} catch (e) {
			again[at] =
				isAppError(e) && e.id === 'server.trial_stopped'
					? { stage: 'stopped' }
					: { stage: 'failed', error: describeError(e, m) };
		}
	}

	function stop(trial: string) {
		void api.cancelServerTrial(trial).catch(() => undefined);
	}

	const busy = (server: OwnServer) =>
		server.models.some((model) => again[againKey(server, model)]?.stage === 'trying');
</script>

<section class="card block" aria-labelledby="servers-title">
	<h2 id="servers-title">{s.title}</h2>
	<p class="muted small intro">
		{s.intro}
		<ExternalLink href={guideUrl('other-servers')}>{s.guide}</ExternalLink>
	</p>

	{#if loaded && servers.length === 0 && form === null}
		<p class="small">{s.empty}</p>
	{/if}

	{#if servers.length > 0}
		<h3 class="list-title">{s.listTitle}</h3>
		<ul class="servers">
			{#each servers as server (server.id)}
				{#if form === server.id}
					<li>
						<ServerForm
							editing={server}
							onsaved={() => void changed()}
							onclose={() => (form = null)}
						/>
					</li>
				{:else}
					{@const own = ownOf(app.catalog, server.id)}
					<li class="server" aria-labelledby="server-{server.id}">
						<div class="head">
							<h4 id="server-{server.id}">{server.name}</h4>
							{#if own}
								<span class="chip">{own.this_computer ? s.onThisComputer : s.onNetwork}</span>
							{/if}
							{#if server.key}<span class="chip ok">{s.keySaved}</span>{/if}
						</div>
						<p class="muted small mono host">{hostOf(server.address)}</p>
						<ul class="models">
							{#each server.models as model (model.id)}
								{@const state = again[againKey(server, model)]}
								<li class="model">
									<div class="line">
										<span class="mono id">{model.id}</span>
										<button
											type="button"
											class="btn sm ghost"
											aria-label={s.tryAgainLabel(model.id)}
											disabled={state?.stage === 'trying' || form !== null}
											onclick={() => void tryAgain(server, model)}
										>
											{s.tryAgain}
										</button>
									</div>
									<p class="muted small">
										{s.offeredFor(offeredFor(m, model))}{model.tried_at
											? ` · ${s.triedOn(formatDay(model.tried_at))}`
											: ''}
									</p>
									{#if state}
										<TrialLine {state} model={model.id} onstop={stop} />
									{/if}
								</li>
							{/each}
						</ul>
						<div class="actions">
							<button
								type="button"
								class="btn sm"
								disabled={form !== null || busy(server)}
								onclick={() => (form = server.id)}
							>
								{s.edit}
							</button>
							<button
								type="button"
								class="btn sm ghost danger"
								disabled={form !== null || busy(server)}
								onclick={() => void remove(server)}
							>
								{s.remove}
							</button>
						</div>
					</li>
				{/if}
			{/each}
		</ul>
	{/if}

	{#if form === 'new'}
		<ServerForm onsaved={() => void changed()} onclose={() => (form = null)} />
	{:else if form === null}
		<div>
			<button
				type="button"
				class="btn"
				class:primary={servers.length === 0}
				onclick={() => (form = 'new')}
			>
				{s.add}
			</button>
		</div>
	{/if}
</section>

<style>
	.block {
		padding: var(--space-4) 0 var(--space-5);
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-3);
	}
	.block > p {
		margin: 0;
	}
	.small {
		font-size: var(--type-sm);
	}
	.intro {
		max-width: 44rem;
	}
	.list-title {
		margin: var(--space-2) 0 0;
		font-size: var(--type-md);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.servers {
		display: grid;
	}
	.server {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-4) 0;
		border-top: 1px solid var(--line);
		min-width: 0;
	}
	.server p {
		margin: 0;
	}
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-2);
	}
	h4 {
		margin: 0;
		font-size: var(--type-md);
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.host {
		overflow-wrap: anywhere;
	}
	.models {
		display: grid;
		gap: var(--space-1);
		max-width: 44rem;
		padding-left: var(--space-3);
		border-left: 2px solid var(--line);
	}
	.model {
		display: grid;
		gap: var(--space-1);
		padding: var(--space-2) 0;
		min-width: 0;
	}
	.model p {
		margin: 0;
	}
	.line {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--space-2) var(--space-3);
		min-width: 0;
	}
	/* A model's name on the server can be long: it wraps anywhere. */
	.id {
		min-width: 0;
		overflow-wrap: anywhere;
		color: var(--text-bright);
		font-weight: 500;
	}
	.line .btn {
		flex: none;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		margin-top: var(--space-1);
	}
</style>
