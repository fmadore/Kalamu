<script lang="ts">
	// The text review's way between pieces: the previous one, a list of them all (each marked
	// once reviewed), the next one. Turning may wait on a correction being kept or let go, so the
	// list stays on the piece shown until the review turns.
	import Icon from './Icon.svelte';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		indices,
		index,
		views,
		labelOf,
		onstep,
		ongoto
	}: {
		/** The pieces' indices, in order. */
		indices: number[];
		/** The piece shown. */
		index: number;
		views: PieceView[];
		labelOf: (v: PieceView | undefined) => string;
		onstep: (delta: number) => void;
		ongoto: (index: number) => void;
	} = $props();
	const m = messages();
</script>

<button
	class="btn sm icon ghost"
	aria-label={m.review.previous}
	title={m.review.previous}
	disabled={indices.indexOf(index) <= 0}
	onclick={() => onstep(-1)}
>
	<Icon name="previous" />
</button>
<select
	class="pages"
	aria-label={m.review.title}
	value={index}
	onchange={(e) => {
		const list = e.currentTarget;
		ongoto(Number(list.value));
		// Kept on the piece shown while the researcher is asked about a correction.
		list.value = String(index);
	}}
>
	{#each indices as i (i)}
		{@const v = views.find((x) => x.index === i)}
		<option value={i}>
			{views.some((x) => x.index === i && x.reviewed)
				? m.review.pieceReviewed(labelOf(v))
				: labelOf(v)}
		</option>
	{/each}
</select>
<button
	class="btn sm icon ghost"
	aria-label={m.review.next}
	title={m.review.next}
	disabled={indices.indexOf(index) >= indices.length - 1}
	onclick={() => onstep(1)}
>
	<Icon name="chevron" />
</button>

<style>
	.pages {
		min-height: var(--control-sm);
		max-width: 14rem;
		font-family: var(--font-mono);
		font-size: var(--type-xs);
	}
</style>
