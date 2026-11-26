// A provider's or model's words in the interface language. The catalogue is data refreshed from
// the repository, so its names and descriptions carry their own translations; what a language
// lacks stays in English.

import type { Catalog } from './bindings/Catalog.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { OwnInfo } from './bindings/OwnInfo.js';
import type { ProviderInfo } from './bindings/ProviderInfo.js';
import { locale } from './i18n/index.js';

/** A model's description in the interface language. */
export function modelDescription(model: ModelInfo): string {
	return model.translations?.[locale()]?.description ?? model.description;
}

/** What a model is for in a few words, for the model list; its description when the list
 *  gives no summary (an older or downloaded list). */
export function modelSummary(model: ModelInfo): string {
	const own = model.translations?.[locale()];
	return own?.summary ?? (own?.description ? undefined : model.summary) ?? modelDescription(model);
}

/** A provider's name in the interface language. */
export function providerName(provider: ProviderInfo): string {
	return provider.translations?.[locale()]?.name ?? provider.name;
}

/** A provider's name by its id, or the id when the catalogue does not list it. */
export function providerNameOf(catalog: Catalog, id: string): string {
	const provider = catalog.providers.find((p) => p.id === id);
	return provider ? providerName(provider) : id;
}

/** Where a server of the researcher's own is and how it is reached; nothing for any other
 *  provider, or one the catalogue does not list. */
export function ownOf(catalog: Catalog | null | undefined, id: string): OwnInfo | undefined {
	return catalog?.providers.find((p) => p.id === id)?.own;
}

/**
 * Whether what a provider reads stays on this computer: an engine inside Kalamu, or a server of
 * the researcher's own at a local address. A server on the network, like a company's service,
 * receives the files.
 */
export function staysHere(catalog: Catalog | null | undefined, id: string): boolean {
	const provider = catalog?.providers.find((p) => p.id === id);
	return provider?.local === true || provider?.own?.this_computer === true;
}

/** A model's name by its provider and id, or the id when the catalogue does not list it. */
export function modelNameOf(catalog: Catalog, provider: string, id: string): string {
	const model = catalog.models.find((m) => m.provider === provider && m.id === id);
	return model?.translations?.[locale()]?.name ?? model?.name ?? id;
}
