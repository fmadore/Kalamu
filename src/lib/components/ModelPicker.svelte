<script lang="ts">
	// A set-up's provider and model. Listed by provider, a provider field chooses whose models the
	// model field lists, under their tiers; listed by tier, one model field lists every
	// provider's, the most capable first, each named with its provider, and choosing one chooses
	// its provider too. A provider without a key is offered but cannot be chosen, either way.
	import type { Catalog } from '#lib/bindings/Catalog.js';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import { providerNameOf } from '#lib/catalogText.js';
	import { messages } from '#lib/i18n/index.js';
	import { hasKey } from '#lib/state.svelte.js';
	import { acrossProviders, byTier, type ModelsBy } from '#lib/tiers.js';

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
	const keyOf = (mod: ModelInfo) => `${mod.provider}/${mod.id}`;
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
	<label class="field">
		<span>{m.newJob.model}</span>
		<select bind:value={model}>
			{#each byTier(models(provider)) as group (group.key)}
				<optgroup label={group.tier ? m.newJob.tiers.group[group.tier] : ''}>
					{#each group.models as mod (mod.id)}
						<option value={mod.id}>{mod.name}{suffix(mod)}</option>
					{/each}
				</optgroup>
			{/each}
		</select>
	</label>
{:else}
	<label class="field">
		<span>{m.newJob.model}</span>
		<select
			value={`${provider}/${model}`}
			onchange={(e) => {
				const chosen = all.find((mod) => keyOf(mod) === e.currentTarget.value);
				if (chosen) {
					provider = chosen.provider;
					model = chosen.id;
				}
			}}
		>
			{#each acrossProviders(all, catalog.providers) as group (group.key)}
				<optgroup label={group.tier ? m.newJob.tiers.group[group.tier] : m.newJob.tiers.local}>
					{#each group.models as mod (keyOf(mod))}
						<option value={keyOf(mod)} disabled={!hasKey(mod.provider)}
							>{group.tier
								? m.newJob.tiers.withProvider(mod.name, providerNameOf(catalog, mod.provider))
								: mod.name}{suffix(mod)}</option
						>
					{/each}
				</optgroup>
			{/each}
		</select>
	</label>
{/if}

<style>
	select {
		width: 100%;
		min-width: 0;
	}
</style>
