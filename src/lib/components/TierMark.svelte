<script lang="ts">
	// Where a model stands among its provider's, drawn as three rising bars, as many filled as
	// its tier is high: three for the most capable, one for the lightest. Decorative: the tier's
	// name is always written beside it.
	import type { Tier } from '#lib/bindings/Tier.js';

	let { tier }: { tier: Tier } = $props();
	const filled = $derived(tier === 'top' ? 3 : tier === 'balanced' ? 2 : 1);
</script>

<svg class="tier" viewBox="0 0 14 14" width="14" height="14" aria-hidden="true">
	{#each [5, 9, 13] as height, i (i)}
		<rect x={1 + i * 4.5} y={13.5 - height} width="3" {height} rx="0.75" class:on={i < filled} />
	{/each}
</svg>

<style>
	.tier {
		flex: none;
		vertical-align: -0.15em;
	}
	rect {
		fill: var(--line-strong);
	}
	rect.on {
		fill: currentColor;
	}
</style>
