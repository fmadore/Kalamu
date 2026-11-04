<script lang="ts">
	// A word explained: a small help mark beside a heading or label that opens the word's
	// definition in a bubble beside it, so the form around it stays as it is. The bubble closes on
	// Escape or a click elsewhere, and links to the whole glossary. Named for screen readers by the
	// question it answers.
	import Icon from './Icon.svelte';
	import { FORMATS, PLACES, type Term } from '#lib/glossary.js';
	import { messages } from '#lib/i18n/index.js';
	import { go } from '#lib/state.svelte.js';

	let { term }: { term: Term | 'formats' } = $props();
	const m = messages();
	const id = $props.id();
	const word = $derived(term === 'formats' ? m.glossary.formatsTitle : m.glossary.terms[term].word);

	let mark = $state<HTMLButtonElement | null>(null);
	let bubble = $state<HTMLElement | null>(null);

	/** Under the mark, or over it when the window has no room below; kept inside the window. */
	function place(e: ToggleEvent) {
		if (e.newState !== 'open' || !mark || !bubble) return;
		const at = mark.getBoundingClientRect();
		const { offsetWidth: w, offsetHeight: h } = bubble;
		const below = innerHeight - at.bottom > h + 16 || at.top < h + 16;
		bubble.style.top = `${below ? at.bottom + 6 : at.top - h - 6}px`;
		bubble.style.left = `${Math.max(8, Math.min(at.left - 12, innerWidth - w - 8))}px`;
	}

	// The mark often sits in a <label>: a click on the bubble's text would otherwise reach the
	// label's control.
	function keepClick(e: MouseEvent) {
		if (!(e.target as HTMLElement).closest('a, button')) e.preventDefault();
	}

	function openGlossary() {
		bubble?.hidePopover();
		go({ name: 'glossary', term });
	}

	/** Where the thing the word names is kept, such as the prompts. */
	const kept = $derived(term === 'formats' ? undefined : PLACES[term]);
</script>

<button
	bind:this={mark}
	type="button"
	class="term"
	aria-label={m.glossary.whatIs(word)}
	title={m.glossary.whatIs(word)}
	popovertarget={id}><Icon name="help" size={14} /></button
><span
	bind:this={bubble}
	{id}
	class="bubble"
	popover="auto"
	role="dialog"
	aria-label={word}
	tabindex="-1"
	ontoggle={place}
	onclick={keepClick}
	onkeydown={() => {}}
>
	<strong class="word">{word}</strong>
	{#if term === 'formats'}
		<span class="formats">
			{#each FORMATS as f (f)}
				<span class="format"
					><b class:mono={f === 'provenance'}
						>{f === 'provenance' ? 'provenance.json' : m.format[f]}</b
					>
					{m.glossary.formats[f]}</span
				>
			{/each}
		</span>
	{:else}
		<span class="text">{m.glossary.terms[term].text}</span>
	{/if}
	<span class="links">
		{#if kept === 'prompts'}
			<button
				type="button"
				class="more"
				onclick={() => {
					bubble?.hidePopover();
					go({ name: 'prompts' });
				}}>{m.glossary.openPrompts}</button
			>
		{/if}
		<button type="button" class="more" onclick={openGlossary}>{m.glossary.openGlossary}</button>
	</span>
</span>

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

	/* In the top layer, placed by `place`; reset from the heading or label it sits in. */
	.bubble {
		position: fixed;
		inset: auto;
		margin: 0;
		width: min(24rem, calc(100vw - 16px));
		max-height: min(24rem, calc(100vh - 16px));
		overflow-y: auto;
		padding: var(--space-3);
		border: 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		font-weight: normal;
		letter-spacing: normal;
		text-transform: none;
		line-height: 1.5;
		color: var(--text-body);
		text-align: start;
		cursor: auto;
	}
	.bubble:popover-open {
		display: grid;
		gap: var(--space-2);
		animation: rise 120ms ease-out;
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.bubble:popover-open {
			animation: none;
		}
	}
	.word {
		color: var(--text-bright);
		font-weight: 600;
	}
	.formats {
		display: grid;
		gap: var(--space-2);
	}
	.format b {
		color: var(--text-bright);
		font-weight: 600;
	}
	.mono {
		font-family: var(--font-mono);
	}
	.links {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-4);
	}
	.more {
		justify-self: start;
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font: inherit;
		text-decoration: underline;
		text-underline-offset: 2px;
		cursor: pointer;
	}
</style>
