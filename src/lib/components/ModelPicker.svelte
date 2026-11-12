<script lang="ts">
	// A set-up's provider and model. Listed by provider, a provider field chooses whose models the
	// model menu lists, under their tiers; listed by tier, one model menu lists every provider's,
	// the most capable first, each named with its provider, and choosing one chooses its provider
	// too. A provider without a key is offered but cannot be chosen, either way.
	import type { Catalog } from '#lib/bindings/Catalog.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import { providerNameOf } from '#lib/catalogText.js';
	import { messages } from '#lib/i18n/index.js';
	import { hasKey } from '#lib/state.svelte.js';
	import { acrossProviders, byTier, type ModelGroup, type ModelsBy } from '#lib/tiers.js';
	import ModelMenu from './ModelMenu.svelte';

	let {
		catalog,
		by,
		providers,
		models,
		defaultFor,
		provider = $bindable(),
		model = $bindable(),
		suffix = () => ''
	}: {
		catalog: Catalog;
		by: ModelsBy;
		/** The providers offered, in order. */
		providers: string[];
		/** A provider's models for this set-up, in the catalogue's order. */
		models: (provider: string) => ModelInfo[];
		/** The model chosen when the provider is changed. */
		defaultFor: (provider: string) => string;
		provider: string;
		model: string;
		/** What follows a model's name, such as "(preview)". */
		suffix?: (model: ModelInfo) => string;
	} = $props();

	const m = messages();
	/** The provider a set-up names is listed even when it is offered nothing new, so a pipeline
	 * that names it shows its model as it is. */
	const listed = $derived(
		provider && !providers.includes(provider) ? [...providers, provider] : providers
	);
	const all = $derived(listed.flatMap((id) => models(id)));
	const chosen = $derived(all.find((mod) => mod.provider === provider && mod.id === model));
	const headingOf = (group: ModelGroup) =>
		group.tier ? m.newJob.tiers.group[group.tier] : m.newJob.tiers.local;
	const nameOf = (mod: ModelInfo) => `${mod.name}${suffix(mod)}`;
	const isLocal = (mod: ModelInfo) =>
		catalog.providers.some((p) => p.id === mod.provider && p.local);
</script>

{#if by === 'provider'}
	<label class="field">
		<span>{m.newJob.provider}</span>
		<select bind:value={provider} onchange={() => (model = defaultFor(provider))}>
			{#each providers as id (id)}
				<option value={id} disabled={!hasKey(id)}>{providerNameOf(catalog, id)}</option>
			{/each}
		</select>
	</label>
	<ModelMenu
		label={m.newJob.model}
		groups={byTier(models(provider))}
		{chosen}
		{headingOf}
		{nameOf}
		onchoose={(mod) => (model = mod.id)}
	/>
{:else}
	<ModelMenu
		label={m.newJob.model}
		groups={acrossProviders(all, catalog.providers)}
		{chosen}
		{headingOf}
		{nameOf}
		providerOf={(mod) => (isLocal(mod) ? null : providerNameOf(catalog, mod.provider))}
		unavailable={(mod) =>
			hasKey(mod.provider) ? null : m.newJob.tiers.noKey(providerNameOf(catalog, mod.provider))}
		onchoose={(mod) => {
			provider = mod.provider;
			model = mod.id;
		}}
	/>
{/if}

<style>
	select {
		width: 100%;
		min-width: 0;
	}
</style>
