<script lang="ts">
	// Under a review's request: what keeps it from going (no key, the model not downloaded), then
	// what it costs (about $x, free on this computer, already answered, no published price). Each
	// dialog says these in its own words.
	import type { PassageEstimate } from '#lib/bindings/PassageEstimate.js';
	import { formatUsd } from '#lib/i18n/index.js';
	import type { ModelChoice } from '#lib/modelAsk.svelte.js';

	let {
		choice,
		estimate,
		texts,
		testid
	}: {
		choice: ModelChoice;
		estimate: PassageEstimate | null;
		texts: {
			noKey: (provider: string) => string;
			notInstalled: (model: string) => string;
			costAbout: (usd: string) => string;
			costStored: string;
			costOffline: string;
			costUnknown: string;
		};
		/** Names the cost line for tests. */
		testid?: string;
	} = $props();
</script>

{#if !choice.keyed}
	<p class="notice" role="status">{texts.noKey(choice.providerName)}</p>
{:else if !choice.installed}
	<p class="notice" role="status">{texts.notInstalled(choice.modelName)}</p>
{/if}
{#if estimate}
	<p class="num" data-testid={testid}>
		{#if estimate.cached}{texts.costStored}{:else if estimate.offline}{texts.costOffline}{:else if estimate.usd !== null}{texts.costAbout(
				formatUsd(estimate.usd)
			)}{:else}{texts.costUnknown}{/if}
	</p>
{/if}

<style>
	p {
		margin: 0;
	}
</style>
