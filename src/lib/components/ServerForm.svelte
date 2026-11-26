<script lang="ts">
	// Adding a server, or changing one, in three steps and nothing to describe by hand: its name
	// and address (and a key, only when it asks for one), Connect; the server says what it is and
	// lists its models, the researcher ticks those to use; Try and add tries each ticked model in
	// turn and saves the server with those that read the trial's picture or filled its form. A
	// model is offered only for what its trial showed, which the shell keeps; a model added before
	// at the same address keeps its trial and is not tried again.
	import type { FoundServer } from '#lib/bindings/FoundServer.js';
	import type { FoundServerModel } from '#lib/bindings/FoundServerModel.js';
	import type { OwnServer } from '#lib/bindings/OwnServer.js';
	import type { TrialServer } from '#lib/bindings/TrialServer.js';
	import { describeError, isAppError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { osNames } from '#lib/platform.js';
	import {
		didSomething,
		foundWords,
		hostOf,
		keptModel,
		modelDraft,
		MOST_AT_ONCE,
		serverDraft,
		unencrypted,
		visionWords,
		type ModelChoices
	} from '#lib/servers.js';
	import { api } from '#lib/tauri.js';
	import TrialLine, { type TrialState } from './TrialLine.svelte';

	let {
		editing = null,
		onsaved,
		onclose
	}: {
		/** The server changed; none when one is added. */
		editing?: OwnServer | null;
		onsaved: (server: OwnServer) => void;
		onclose: () => void;
	} = $props();

	const m = messages();
	const s = m.servers;
	const id = $props.id();

	// svelte-ignore state_referenced_locally
	const start = editing;
	let name = $state(start?.name ?? '');
	let address = $state(start?.address ?? '');
	let asksKey = $state(start?.key ?? false);
	let key = $state('');

	let found = $state<FoundServer | null>(null);
	let connecting = $state(false);
	let connectError = $state<string | null>(null);
	let ticked = $state<Record<string, boolean>>({});
	let choices = $state<Record<string, ModelChoices>>({});
	let atOnce = $state(start?.at_once ?? 1);
	let trials = $state<Record<string, TrialState>>({});
	let running = $state(false);
	let error = $state<string | null>(null);
	let saved = $state<OwnServer | null>(null);

	/** The key typed, sent only while the server is said to ask for one. */
	const typedKey = () => (asksKey && key.trim() ? key.trim() : null);
	const chosen = $derived(found?.models.filter((x) => ticked[x.id] && !x.elsewhere) ?? []);
	/** Models ticked that must be tried: those without a stored trial that still holds. */
	const toTry = $derived(found ? chosen.filter((x) => !keptModel(start, found!, x.id)) : []);

	async function connect() {
		connecting = true;
		connectError = null;
		error = null;
		found = null;
		trials = {};
		try {
			const answer = await api.findServer(address, typedKey(), start?.id ?? null);
			ticked = Object.fromEntries(
				answer.models.filter((x) => x.added && !x.elsewhere).map((x) => [x.id, true])
			);
			choices = Object.fromEntries(
				answer.models.map((x) => {
					const own = start?.models.find((o) => o.id === x.id);
					return [
						x.id,
						{
							context: x.context,
							max_output: x.max_output,
							input_price: own?.input_per_mtok ?? null,
							output_price: own?.output_per_mtok ?? null
						}
					];
				})
			);
			found = answer;
		} catch (e) {
			connectError = describeError(e, m);
		} finally {
			connecting = false;
		}
	}

	/** The address changed: what was found there no longer stands. */
	function addressChanged() {
		found = null;
		trials = {};
		connectError = null;
	}

	async function tryOne(server: TrialServer, x: FoundServerModel): Promise<void> {
		const trial = crypto.randomUUID();
		trials[x.id] = { stage: 'trying', trial };
		try {
			const result = await api.tryServerModel(
				trial,
				server,
				{ id: x.id, context: Number(choices[x.id].context), vision: x.vision, digest: x.digest },
				typedKey()
			);
			trials[x.id] = { stage: 'done', result };
		} catch (e) {
			trials[x.id] =
				isAppError(e) && e.id === 'server.trial_stopped'
					? { stage: 'stopped' }
					: { stage: 'failed', error: describeError(e, m) };
		}
	}

	async function tryAndAdd() {
		const at = found;
		if (!at) return;
		error = null;
		if (!name.trim()) {
			error = describeError({ id: 'server.no_name' }, m);
			return;
		}
		running = true;
		const models = chosen;
		const server: TrialServer = {
			id: start?.id ?? null,
			name: name.trim(),
			address: at.address,
			dialect: at.dialect,
			asks_key: asksKey
		};
		trials = Object.fromEntries(
			models.map((x): [string, TrialState] => {
				const kept = keptModel(start, at, x.id);
				return [x.id, kept ? { stage: 'kept', model: kept } : { stage: 'waiting' }];
			})
		);
		for (const x of models) {
			if (trials[x.id]?.stage === 'waiting') await tryOne(server, x);
		}
		const keep = models.filter((x) => {
			const t = trials[x.id];
			return t?.stage === 'kept' || (t?.stage === 'done' && didSomething(t.result));
		});
		if (keep.length === 0 && models.length > 0) {
			error = s.notSaved;
			running = false;
			return;
		}
		try {
			saved = await api.saveServer(
				serverDraft({
					editing: start,
					name,
					found: at,
					asksKey,
					atOnce,
					models: keep.map((x) => modelDraft(x.id, choices[x.id]))
				}),
				typedKey()
			);
			onsaved(saved);
		} catch (e) {
			error = describeError(e, m);
		} finally {
			running = false;
		}
	}

	function stop(trial: string) {
		void api.cancelServerTrial(trial).catch(() => undefined);
	}

	const sizeOf = (x: FoundServerModel) =>
		x.size_bytes !== null ? m.settings.size(x.size_bytes) : null;
	const locked = $derived(running || saved !== null);
</script>

<section class="form" aria-labelledby="{id}-title">
	<h3 id="{id}-title">{start ? s.editTitle(start.name) : s.addTitle}</h3>

	<form
		class="part who"
		onsubmit={(e) => {
			e.preventDefault();
			void connect();
		}}
	>
		<label class="field">
			<span>{s.name}</span>
			<input type="text" bind:value={name} autocomplete="off" disabled={locked} />
			<small>{s.nameHint}</small>
		</label>
		<label class="field">
			<span>{s.address}</span>
			<input
				type="text"
				bind:value={address}
				placeholder="http://localhost:11434"
				autocomplete="off"
				spellcheck="false"
				inputmode="url"
				disabled={locked}
				oninput={addressChanged}
			/>
			<small>{s.addressHint}</small>
		</label>
		<label class="inline">
			<input type="checkbox" bind:checked={asksKey} disabled={locked} />
			{s.asksKey}
		</label>
		{#if asksKey}
			<label class="field">
				<span>{s.key}</span>
				<input
					type="password"
					bind:value={key}
					autocomplete="off"
					spellcheck="false"
					disabled={locked}
				/>
				<small>
					{start?.key ? s.keyKept(osNames().keychain) : s.keyHint(osNames().keychain)}
				</small>
			</label>
		{/if}
		<div class="row">
			<button
				type="submit"
				class="btn"
				class:primary={found === null}
				disabled={connecting || locked}
			>
				{connecting ? s.connecting : s.connect}
			</button>
			{#if !saved}
				<button type="button" class="btn ghost" disabled={running} onclick={onclose}>
					{m.common.cancel}
				</button>
			{/if}
		</div>
		{#if connectError}<p class="err small" role="alert">{connectError}</p>{/if}
	</form>

	{#if found}
		{@const at = found}
		<div class="part what">
			<p class="found"><strong>{foundWords(m, at)}</strong></p>
			{#if unencrypted(at)}
				<p class="notice" role="note">{asksKey ? s.unencryptedKey : s.unencrypted}</p>
			{/if}
			<h4>{s.modelsTitle}</h4>
			{#if at.models.length === 0}
				<p class="muted small">{s.noModels}</p>
			{:else}
				<p class="muted small">{s.modelsHint}</p>
				<ul class="models">
					{#each at.models as x (x.id)}
						{@const size = sizeOf(x)}
						<li class="model" class:off={x.elsewhere !== null}>
							<label class="tick">
								<input
									type="checkbox"
									bind:checked={ticked[x.id]}
									disabled={x.elsewhere !== null || locked}
									aria-describedby="{id}-{x.id}-about"
								/>
								<span class="mono id">{x.id}</span>
								{#if x.added}<span class="chip">{s.added}</span>{/if}
							</label>
							<p class="muted small about" id="{id}-{x.id}-about">
								{#if x.elsewhere !== null}
									{s.elsewhere(hostOf(x.elsewhere))}
								{:else}
									{size ? `${size} · ` : ''}{visionWords(m, x.vision)}
								{/if}
							</p>
							{#if ticked[x.id] && x.elsewhere === null}
								<details class="more">
									<summary>{s.more}</summary>
									<div class="limits">
										<label class="field">
											<span>{s.context}</span>
											<input
												type="number"
												min="1"
												step="1"
												bind:value={choices[x.id].context}
												disabled={locked}
											/>
											<small>{s.contextHint}</small>
										</label>
										<label class="field">
											<span>{s.maxOutput}</span>
											<input
												type="number"
												min="1"
												step="1"
												bind:value={choices[x.id].max_output}
												disabled={locked}
											/>
											<small>{s.maxOutputHint}</small>
										</label>
										<label class="field">
											<span>{s.priceIn}</span>
											<input
												type="number"
												min="0"
												step="any"
												bind:value={choices[x.id].input_price}
												disabled={locked}
											/>
										</label>
										<label class="field">
											<span>{s.priceOut}</span>
											<input
												type="number"
												min="0"
												step="any"
												bind:value={choices[x.id].output_price}
												disabled={locked}
											/>
										</label>
										<p class="muted small wide">{s.priceHint}</p>
									</div>
								</details>
							{/if}
							{#if trials[x.id]}
								<TrialLine state={trials[x.id]} model={x.id} onstop={stop} />
							{/if}
						</li>
					{/each}
				</ul>
				<details class="more">
					<summary>{s.more}</summary>
					<label class="field narrow">
						<span>{s.atOnce}</span>
						<input
							type="number"
							min="1"
							max={MOST_AT_ONCE}
							step="1"
							bind:value={atOnce}
							disabled={locked}
						/>
						<small>{s.atOnceHint}</small>
					</label>
				</details>
			{/if}
		</div>

		<div class="part go">
			{#if error}<p class="err small" role="alert">{error}</p>{/if}
			{#if saved}
				<p class="ok small" role="status">{s.saved(saved.name)}</p>
				<div class="row">
					<button type="button" class="btn primary" onclick={onclose}>{s.done}</button>
				</div>
			{:else}
				<div class="row">
					<button
						type="button"
						class="btn primary"
						disabled={running || chosen.length === 0}
						onclick={tryAndAdd}
					>
						{toTry.length > 0 || chosen.length === 0 ? s.tryAndAdd : m.settings.save}
					</button>
				</div>
			{/if}
		</div>
	{/if}
</section>

<style>
	.form {
		display: grid;
		gap: var(--space-4);
		padding: var(--space-4) 0;
		border-top: 1px solid var(--line);
		min-width: 0;
	}
	h3 {
		margin: 0;
		font-size: var(--type-md);
	}
	h4 {
		margin: 0;
		font-size: var(--type-sm);
		font-weight: 600;
	}
	.part {
		display: grid;
		gap: var(--space-3);
		max-width: 44rem;
		min-width: 0;
	}
	.part p {
		margin: 0;
	}
	.who .field {
		max-width: 28rem;
	}
	.small {
		font-size: var(--type-sm);
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		font-size: var(--type-sm);
		width: fit-content;
	}
	.err {
		color: var(--danger);
	}
	.ok {
		color: var(--ok);
	}
	.models {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		border-top: 1px solid var(--line);
	}
	.model {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3) 0;
		border-bottom: 1px solid var(--line);
		min-width: 0;
	}
	.tick {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
		cursor: pointer;
	}
	.off .tick {
		cursor: not-allowed;
	}
	/* A model's name on the server can be long (`hf.co/bartowski/…:Q4_K_M`): it wraps anywhere
	   rather than pushing the page sideways. */
	.id {
		min-width: 0;
		overflow-wrap: anywhere;
		color: var(--text-bright);
		font-weight: 500;
	}
	.off .id {
		color: var(--text-muted);
	}
	/* Under the name, level with it past the box. */
	.model > .about,
	.model > :global(details),
	.model > :global(.trial) {
		padding-left: calc(1rem + var(--space-2));
	}
	.more summary {
		width: fit-content;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		cursor: pointer;
	}
	.limits {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--space-3);
		margin-top: var(--space-3);
	}
	.limits .wide {
		grid-column: 1 / -1;
	}
	.more > .field {
		margin-top: var(--space-3);
	}
	.narrow {
		max-width: 18rem;
	}
	@media (max-width: 560px) {
		.limits {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
