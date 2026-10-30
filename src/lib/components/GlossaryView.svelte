<script lang="ts">
	// The glossary: the words the app uses, explained for colleagues new to these tools, and what
	// each export format suits. Set as a reader's glossary: the word in bold, its definition in
	// the reading face, the words it leans on after it. A link from elsewhere in the app opens the
	// page on its word, marked with the highlighter's edge; a field above keeps the words that
	// hold what is typed.
	import { tick } from 'svelte';
	import Icon from './Icon.svelte';
	import { anchor, FORMATS, filterTerms, SEE_ALSO, type Term } from '#lib/glossary.js';
	import { messages } from '#lib/i18n/index.js';

	/** The view as opened: a new object on every link followed, so the same word can be gone to
	 * again. */
	let { opening = {} }: { opening?: { term?: Term | 'formats' } } = $props();
	const m = messages();

	let query = $state('');
	const shown = $derived(filterTerms(m, query));
	/** The word gone to, from a link or a "See also". */
	let marked = $state<Term | 'formats' | null>(null);

	async function show(target: Term | 'formats') {
		query = '';
		marked = target;
		await tick();
		const el = document.getElementById(anchor(target));
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		el?.scrollIntoView({ block: 'start', behavior: still ? 'auto' : 'smooth' });
		el?.focus({ preventScroll: true });
	}

	// Opened on a word from a link elsewhere in the app.
	$effect(() => {
		if (opening.term) void show(opening.term);
	});
</script>

<header class="masthead">
	<div>
		<h1>{m.glossary.title}</h1>
		<p class="lede">{m.glossary.lede}</p>
	</div>
</header>

<div class="sheet glossary">
	<label class="filter">
		<span class="glass" aria-hidden="true"><Icon name="search" size={14} /></span>
		<input
			type="search"
			bind:value={query}
			placeholder={m.glossary.filter}
			aria-label={m.glossary.filter}
		/>
	</label>

	<section class="card block" aria-labelledby="glossary-words">
		<h2 id="glossary-words">{m.glossary.wordsTitle}</h2>
		{#if shown.length === 0}
			<p class="muted">{m.glossary.noMatch}</p>
		{/if}
		<dl class="entries">
			{#each shown as t (t)}
				{@const entry = m.glossary.terms[t]}
				<div class="entry" id={anchor(t)} tabindex="-1" class:marked={marked === t}>
					<dt>{entry.word}</dt>
					<dd>
						<p>{entry.text}</p>
						{#if SEE_ALSO[t]}
							<p class="see">
								{m.glossary.seeAlso}:
								{#each SEE_ALSO[t] ?? [] as other, i (other)}{#if i > 0}{', '}{/if}<a
										href="#{anchor(other)}"
										onclick={(e) => {
											e.preventDefault();
											void show(other);
										}}>{m.glossary.terms[other].word}</a
									>{/each}
							</p>
						{/if}
					</dd>
				</div>
			{/each}
		</dl>
	</section>

	<section
		class="card block"
		aria-labelledby="glossary-formats"
		id={anchor('formats')}
		tabindex="-1"
		class:marked={marked === 'formats'}
	>
		<h2 id="glossary-formats">{m.glossary.formatsTitle}</h2>
		<dl class="entries">
			{#each FORMATS as f (f)}
				<div class="entry">
					<dt class:mono={f === 'provenance'}>
						{f === 'provenance' ? 'provenance.json' : m.format[f]}
					</dt>
					<dd><p>{m.glossary.formats[f]}</p></dd>
				</div>
			{/each}
		</dl>
	</section>
</div>

<style>
	.glossary {
		max-width: 52rem;
	}
	.block {
		padding: var(--space-4) 0 0;
		display: grid;
		gap: var(--space-3);
		scroll-margin-top: var(--space-4);
	}
	.filter {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		max-width: 24rem;
	}
	.glass {
		display: grid;
		color: var(--text-muted);
	}
	.filter input {
		flex: 1;
		min-height: var(--control-sm);
	}
	.entries {
		display: grid;
		margin: 0;
	}
	/* The word beside its definition, as a reader's glossary sets them; stacked when narrow. */
	.entry {
		display: grid;
		grid-template-columns: minmax(9rem, 14rem) minmax(0, 1fr);
		gap: var(--space-1) var(--space-5);
		padding: var(--space-3) var(--space-3) var(--space-3) var(--space-2);
		border-top: 1px solid var(--line-soft);
		border-radius: var(--radius-control);
		scroll-margin-top: var(--space-4);
		outline: none;
	}
	.entry:first-child {
		border-top: 0;
	}
	/* The word gone to, marked with the highlighter's edge, as a table of contents marks its page. */
	.marked:is(.entry, .block) {
		background: var(--surface-1);
		box-shadow: inset 3px 0 0 var(--marker);
	}
	dt {
		font-size: var(--type-md);
		font-weight: 600;
		color: var(--text-bright);
		line-height: 1.45;
	}
	dd {
		margin: 0;
		display: grid;
		gap: var(--space-1);
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.6;
		color: var(--text-body);
	}
	.see {
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.see a {
		color: inherit;
		text-decoration: underline;
		text-underline-offset: 0.2em;
	}
	.see a:hover {
		color: var(--text-bright);
	}
	@media (max-width: 820px) {
		.entry {
			grid-template-columns: 1fr;
		}
	}
</style>
