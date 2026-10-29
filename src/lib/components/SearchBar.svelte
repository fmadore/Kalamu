<script lang="ts" module>
	/** A match as the list shows it: the page or time it is at, and the words around it. */
	export type ListedMatch = {
		/** The piece it is in, as its heading in the list. */
		group: string;
		/** In a recording, when its line begins. */
		time: string | null;
		before: string;
		words: string;
		after: string;
	};
</script>

<script lang="ts">
	// The search under a review's masthead: the words looked for, how many were found, and,
	// where matches can be gone to, buttons to step through them (Enter, Shift Enter) and a list
	// of them by page or time. Typing opens the list; going to a match closes it. Escape closes
	// the list, then the search.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';

	/** At most this many matches are listed; all are counted and stepped through. */
	const LISTED = 300;

	let {
		query = $bindable(''),
		input = $bindable(null),
		placeholder,
		summary,
		matches = null,
		current = -1,
		onpick,
		onclose
	}: {
		query?: string;
		input?: HTMLInputElement | null;
		placeholder: string;
		/** How many were found, or which is gone to. */
		summary: string;
		/** The matches to go to; null where the search only filters. */
		matches?: ListedMatch[] | null;
		current?: number;
		onpick?: (i: number) => void;
		onclose: () => void;
	} = $props();
	const m = messages();

	let listed = $state(false);
	let box = $state<HTMLElement | null>(null);
	const count = $derived(matches?.length ?? 0);

	function pick(i: number) {
		listed = false;
		onpick?.(i);
	}

	function step(delta: number) {
		if (count === 0) return;
		const from = current < 0 ? (delta > 0 ? -1 : 0) : current;
		pick((from + delta + count) % count);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			step(e.shiftKey ? -1 : 1);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			if (listed) listed = false;
			else onclose();
		}
	}

	/** A click outside the search closes the list. */
	function onpointerdown(e: PointerEvent) {
		if (listed && !box?.contains(e.target as Node)) listed = false;
	}
</script>

<svelte:window {onpointerdown} />

<div class="search" role="search" bind:this={box}>
	<span class="glass" aria-hidden="true"><Icon name="search" size={14} /></span>
	<input
		type="search"
		bind:this={input}
		bind:value={query}
		{placeholder}
		aria-label={placeholder}
		oninput={() => (listed = matches !== null)}
		{onkeydown}
	/>
	<span class="summary" aria-live="polite">{summary}</span>
	{#if matches}
		<span class="steps">
			<button
				class="btn sm icon"
				aria-label={m.review.previousMatch}
				title="{m.review.previousMatch} (Shift ↵)"
				disabled={count === 0}
				onclick={() => step(-1)}><Icon name="up" size={14} /></button
			>
			<button
				class="btn sm icon"
				aria-label={m.review.nextMatch}
				title="{m.review.nextMatch} (↵)"
				disabled={count === 0}
				onclick={() => step(1)}><Icon name="down" size={14} /></button
			>
			<button
				class="btn sm toggle"
				aria-expanded={listed && count > 0}
				aria-controls="matches"
				title={m.review.listMatchesHint}
				disabled={count === 0}
				onclick={() => (listed = !listed)}
				><Icon name="list" size={14} />{m.review.listMatches}</button
			>
		</span>
	{/if}
	<button
		class="btn sm icon ghost close"
		aria-label={m.review.closeSearch}
		title={m.review.closeSearch}
		onclick={onclose}><Icon name="close" size={14} /></button
	>

	{#if matches && listed && count > 0}
		<div class="list" id="matches" role="region" aria-label={m.review.matches}>
			<ol>
				{#each matches.slice(0, LISTED) as hit, i (i)}
					{#if i === 0 || matches[i - 1].group !== hit.group}
						<li class="group" aria-hidden="true">{hit.group}</li>
					{/if}
					<li>
						<button
							class="match"
							aria-current={i === current ? 'true' : undefined}
							onclick={() => pick(i)}
						>
							{#if hit.time}<span class="time num">{hit.time}</span>{/if}
							<span class="words"
								>{hit.before}<mark>{hit.words}</mark>{hit.after}<span class="sr-only">
									· {hit.group}</span
								></span
							>
						</button>
					</li>
				{/each}
			</ol>
			{#if count > LISTED}<p class="more">{m.review.moreMatches(LISTED, count)}</p>{/if}
		</div>
	{/if}
</div>

<style>
	.search {
		position: relative;
		z-index: 9;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		padding: var(--space-2) var(--space-5);
		border-bottom: 1px solid var(--line);
		background: var(--surface-0);
	}
	.glass {
		display: grid;
		color: var(--text-muted);
	}
	input {
		flex: 0 1 24rem;
		min-width: 12rem;
		min-height: var(--control-sm);
	}
	/* The close button clears and closes; the field needs no clear button of its own. */
	input::-webkit-search-cancel-button {
		display: none;
	}
	.summary {
		font-size: var(--type-xs);
		color: var(--text-muted);
		white-space: nowrap;
	}
	.steps {
		display: inline-flex;
		gap: var(--space-1);
	}
	.close {
		margin-left: auto;
	}
	.toggle[aria-expanded='true'] {
		background: var(--wash);
		border-color: var(--line-strong);
	}
	/* The matches, under the search and over the review. */
	.list {
		position: absolute;
		top: calc(100% + var(--space-1));
		left: var(--space-5);
		width: min(36rem, calc(100% - 2 * var(--space-5)));
		max-height: min(32rem, 60vh);
		overflow: auto;
		padding: var(--space-2) 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-flyout);
	}
	ol {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.group {
		position: sticky;
		top: calc(-1 * var(--space-2));
		padding: var(--space-2) var(--space-4) var(--space-1);
		background: var(--surface-2);
		font-family: var(--font-mono);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.match {
		display: flex;
		align-items: baseline;
		gap: var(--space-3);
		width: 100%;
		padding: var(--space-1) var(--space-4);
		border: 0;
		background: transparent;
		color: var(--text-body);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.match:hover,
	.match:focus-visible {
		background: var(--wash);
	}
	.match:focus-visible {
		outline-offset: -2px;
	}
	.match[aria-current='true'] {
		box-shadow: inset 3px 0 0 var(--ink);
	}
	.time {
		flex: none;
		font-family: var(--font-mono);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.words {
		font-family: var(--font-text);
		font-size: var(--type-md);
		line-height: 1.45;
		overflow-wrap: anywhere;
	}
	mark {
		background: var(--found);
		color: inherit;
		border-radius: 2px;
		box-shadow: 0 0 0 2px var(--found);
	}
	.more {
		padding: var(--space-2) var(--space-4) var(--space-1);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	@media (max-width: 820px) {
		.search {
			padding: var(--space-2) var(--space-4);
		}
		.list {
			left: var(--space-4);
			width: calc(100% - 2 * var(--space-4));
		}
	}
</style>
