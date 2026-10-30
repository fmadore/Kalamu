<script lang="ts">
	// A word explained: a small help mark beside a heading or label that opens the glossary on that
	// word. Named for screen readers by the question it answers.
	import Icon from './Icon.svelte';
	import type { Term } from '#lib/glossary.js';
	import { messages } from '#lib/i18n/index.js';
	import { go } from '#lib/state.svelte.js';

	let { term }: { term: Term | 'formats' } = $props();
	const m = messages();
	const word = $derived(term === 'formats' ? m.glossary.formatsTitle : m.glossary.terms[term].word);
</script>

<button
	type="button"
	class="term"
	aria-label={m.glossary.whatIs(word)}
	title={m.glossary.whatIs(word)}
	onclick={() => go({ name: 'glossary', term })}><Icon name="help" size={14} /></button
>

<style>
	/* Smaller than an icon button, so a heading keeps its height; the icon centred in it. */
	.term {
		display: inline-grid;
		place-items: center;
		width: 1.375rem;
		height: 1.375rem;
		margin-left: var(--space-1);
		padding: 0;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-muted);
		vertical-align: middle;
		cursor: pointer;
		transition:
			background-color 120ms ease-out,
			color 120ms ease-out;
	}
	.term:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
</style>
