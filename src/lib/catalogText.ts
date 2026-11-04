// A provider's or model's words in the interface language. The catalogue is data refreshed from
// the repository, so its names and descriptions carry their own translations; what a language
// lacks stays in English.

import type { Catalog } from './bindings/Catalog.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { ProviderInfo } from './bindings/ProviderInfo.js';
import { locale } from './i18n/index.js';

/** A model's description in the interface language. */
export function modelDescription(model: ModelInfo): string {
	return model.translations?.[locale()]?.description ?? model.description;
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
