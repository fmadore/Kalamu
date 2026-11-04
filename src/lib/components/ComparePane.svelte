<script lang="ts">
	// Another reading of the page in the review's third pane, its differences from the text shown
	// marked as they are there: choosing one opens its readings over the text shown.
	import MarkedText from './MarkedText.svelte';
	import { witnessParts, type Entry } from '#lib/apparatus.js';
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';
	import type { Witness } from '#lib/compare.svelte.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		compared,
		base,
		spans,
		entries,
		marked,
		selected,
		hovered,
		onpick,
		onhover
	}: {
		compared: Witness | undefined;
		/** The text shown, which the witness is compared with. */
		base: string;
		/** Each witness's differences from the text shown, once worked out. */
		spans: Record<string, DiffSpan[]>;
		entries: Entry[];
		marked: boolean;
		selected: number | null;
		hovered: number | null;
		onpick: (id: number) => void;
		/** The difference under the pointer, or null once it leaves. */
		onhover: (id: number | null) => void;
	} = $props();
	const m = messages();

	const parts = $derived(
		compared && spans[compared.key] ? witnessParts(base, spans[compared.key], entries) : null
	);
</script>

{#if parts}
	<MarkedText
		{parts}
		{entries}
		{marked}
		{selected}
		{hovered}
		emptyLabel={m.review.leftOutHere}
		{onpick}
		onpoint={(_, id) => onhover(id)}
		onaway={() => onhover(null)}
	/>
{:else}
	<p class="muted">{m.common.loading}</p>
{/if}
