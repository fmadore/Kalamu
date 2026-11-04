<script lang="ts">
	// The bar over the review's third pane: which reading it shows, chosen among the others when
	// there are several, and the button that closes it.
	import Icon from './Icon.svelte';
	import type { Witness } from '#lib/compare.svelte.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		compared,
		witnesses,
		nameOf,
		onchoose,
		onclose
	}: {
		compared: Witness | undefined;
		witnesses: Witness[];
		/** Who reads what a siglum names. */
		nameOf: (siglum: string) => string;
		onchoose: (key: string) => void;
		onclose: () => void;
	} = $props();
	const m = messages();
</script>

{#if compared}
	<span class="compared">
		<i class="siglum">{compared.siglum}</i>
		{#if witnesses.length > 1}
			<select
				class="mono"
				aria-label={m.review.compareWith}
				value={compared.key}
				onchange={(e) => onchoose(e.currentTarget.value)}
			>
				{#each witnesses as w (w.key)}
					<option value={w.key}>{nameOf(w.siglum)}</option>
				{/each}
			</select>
		{:else}
			<span class="mono">{nameOf(compared.siglum)}</span>
		{/if}
	</span>
{/if}
<button
	class="btn sm icon ghost close-compare"
	aria-label={m.review.closeCompare}
	title={m.review.closeCompare}
	onclick={onclose}><Icon name="close" size={14} /></button
>

<style>
	.compared {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.compared select {
		min-height: var(--control-sm);
		max-width: 16rem;
		font-size: var(--type-xs);
	}
	.close-compare {
		margin-left: auto;
	}
</style>
