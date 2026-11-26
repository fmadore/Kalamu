// How New job lists models: under each provider, as the provider and model fields always did,
// or by tier, one list of every provider's models with the most capable first. The choice is a
// convenience remembered on this computer.

import type { ModelInfo } from './bindings/ModelInfo.js';
import type { ProviderInfo } from './bindings/ProviderInfo.js';
import type { Tier } from './bindings/Tier.js';
import { keep, recall } from './stored.js';

export type ModelsBy = 'provider' | 'tier';

/** From the most capable to the lightest. */
export const TIERS: readonly Tier[] = ['top', 'balanced', 'light'];

/** A heading of a model list and the models under it: a tier's, those that run on this
 * computer, which no tier compares with the providers' own, or the researcher's servers'. */
export interface ModelGroup {
	key: string;
	tier: Tier | null;
	models: ModelInfo[];
}

/** A model's tier; `balanced` for an entry that names none, as the catalogue reads it. */
export const tierOf = (model: ModelInfo): Tier => model.tier ?? 'balanced';

const rank = (model: ModelInfo) => TIERS.indexOf(tierOf(model));

/** The models under their tiers, most capable first, each tier keeping the order given; a tier
 * with none is left out. */
export function byTier(models: ModelInfo[]): ModelGroup[] {
	return TIERS.map((tier) => ({
		key: tier,
		tier,
		models: models.filter((m) => tierOf(m) === tier)
	})).filter((g) => g.models.length > 0);
}

/**
 * Every provider's models in one list: the cloud models under their tiers, then those that run
 * on this computer under one heading of their own, most capable first, then the researcher's
 * servers' under one heading, server by server. A Whisper model is classed among Whisper's, not
 * against Claude's, and a server's model, which no tier describes, against no one's.
 */
export function acrossProviders(models: ModelInfo[], providers: ProviderInfo[]): ModelGroup[] {
	const local = new Set(providers.filter((p) => p.local).map((p) => p.id));
	const own = providers.filter((p) => p.own).map((p) => p.id);
	const onComputer = models.filter((m) => local.has(m.provider));
	const onServers = own.flatMap((id) => models.filter((m) => m.provider === id));
	const groups = byTier(models.filter((m) => !local.has(m.provider) && !own.includes(m.provider)));
	if (onComputer.length > 0) {
		// A stable sort keeps each family together inside a tier.
		groups.push({
			key: 'local',
			tier: null,
			models: [...onComputer].sort((a, b) => rank(a) - rank(b))
		});
	}
	if (onServers.length > 0) groups.push({ key: 'own', tier: null, models: onServers });
	return groups;
}

const KEY = 'kalamu.newJob.modelsBy';

/** How models were last listed; by provider at first. */
export function recallModelsBy(): ModelsBy {
	return recall(KEY, (v) => (v === 'tier' || v === 'provider' ? v : null), 'provider');
}

export function keepModelsBy(by: ModelsBy): void {
	keep(KEY, by);
}
