<script lang="ts">
	// New job's suggestion beside a long document (#215): try the model on a few pages spread
	// through it, alone or with a second model, before reading every page; once chosen, what will
	// be read and how to read every page again. Shown only for a long document: nothing changes
	// for a short one or for a researcher who passes it by.
	import { messages } from '#lib/i18n/index.js';

	let {
		pages,
		n,
		chosen,
		two,
		comparable,
		ontry,
		onevery
	}: {
		/** The pages of the longest document. */
		pages: number;
		/** The pages tried. */
		n: number;
		/** The suggestion was taken. */
		chosen: boolean;
		/** Two models or more read the sample. */
		two: boolean;
		/** Another model can be used, so the comparison is offered. */
		comparable: boolean;
		ontry: (two: boolean) => void;
		onevery: () => void;
	} = $props();
	const m = messages();
	const t = m.newJob.trySample;
</script>

<section class="notice offer" aria-labelledby="sample-offer-title">
	<div class="text">
		<h2 id="sample-offer-title">{t.title}</h2>
		<p>{chosen ? (two ? t.chosenTwo(n) : t.chosen(n)) : t.offer(pages, n)}</p>
	</div>
	<div class="actions">
		{#if chosen}
			<button class="btn sm" onclick={onevery}>{t.every}</button>
		{:else}
			<button class="btn sm primary" onclick={() => ontry(false)}>{t.one(n)}</button>
			{#if comparable}
				<button class="btn sm" onclick={() => ontry(true)}>{t.two(n)}</button>
			{/if}
		{/if}
	</div>
</section>

<style>
	/* A suggestion, not a warning: the calm ground of the form's panels, under the files and
	   in line with them, past the steps' numbers. */
	.offer {
		margin-left: 2.75rem;
		justify-content: space-between;
		align-items: flex-start;
		background: var(--surface-1);
		border: 1px solid var(--line);
		color: var(--text-body);
	}
	.text {
		display: grid;
		gap: var(--space-1);
		flex: 1 1 18rem;
		min-width: 0;
	}
	h2 {
		margin: 0;
		font-size: var(--type-sm);
		font-weight: 600;
	}
	p {
		margin: 0;
		max-width: 70ch;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
</style>
