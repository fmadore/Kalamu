<script lang="ts">
	// One list answer of an analysis's review, a field of free names or of choices: its names in
	// the reading face, each with the sigla of the set-ups that found it when not all of them did,
	// then Add for what the models missed (`AddItems`). A name opens its menu (`ListedNames`) and
	// is found in the document while pointed at or focused; the names are one tab stop. A name no
	// model listed is the researcher's, underlined in their blue and said to be.
	import AddItems from './AddItems.svelte';
	import { byHand, type Reading } from '#lib/analysis.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import { choiceLabel, messages } from '#lib/i18n/index.js';
	import { rovingStop } from '#lib/keys.js';
	import type { ListedNames } from '#lib/listedNames.svelte.js';

	let {
		field: f,
		readings,
		names,
		models,
		comparing,
		count,
		onhover
	}: {
		field: Field;
		/** The names the set-ups gave, collated. */
		readings: Reading[];
		names: ListedNames;
		/** Each set-up's own answers, from the model: a name none of them lists is the researcher's. */
		models: FieldAnswer[][];
		comparing: boolean;
		/** How many set-ups are compared. */
		count: number;
		/** The name pointed at or focused, found in the document; null when none is. */
		onhover: (value: string | null) => void;
	} = $props();
	const m = messages();

	const named = $derived(f.kind.type === 'list');
	/** A name as shown: a default's choice in the interface language, a name as listed. */
	const shown = (value: string) => (named ? value : choiceLabel(f, value));
	const values = $derived(readings.map((r) => r.value));
	const stop = $derived(rovingStop(values, names.stops[f.key]));

	/** For a field of choices, those its answers do not list yet; null for free names. */
	function choicesLeft(listed: string[]): string[] | null {
		if (f.kind.type !== 'choices') return null;
		const has = (c: string) => listed.some((v) => v.trim().toLowerCase() === c.toLowerCase());
		return f.kind.values.filter((c) => !has(c));
	}
</script>

{#if readings.length > 0}
	<p class="items" dir="auto" oncontextmenu={(e) => names.openMenu(e, f)}>
		{#each readings as r, k (r.value)}{@const yours = byHand(
				f.key,
				r.value,
				models
			)}{#if k > 0}<span class="sep">{m.common.semicolon}</span>{/if}<button
				type="button"
				class="item"
				class:yours
				class:partial={comparing && r.sigla.length < count}
				data-item={r.value}
				title={yours ? m.review.yoursHint : undefined}
				aria-label={yours
					? m.review.yours(
							comparing && r.sigla.length < count
								? `${shown(r.value)} ${r.sigla.join(' ')}`
								: shown(r.value)
						)
					: undefined}
				tabindex={stop === r.value ? 0 : -1}
				aria-haspopup="menu"
				aria-describedby={named ? 'item-keys' : 'choice-keys'}
				onclick={(e) => names.openMenu(e, f)}
				onkeydown={(e) => names.rove(e, f.key, values, r.value)}
				onfocus={() => {
					onhover(r.value);
					names.stops[f.key] = r.value;
				}}
				onblur={() => onhover(null)}
				onpointerenter={() => onhover(r.value)}
				onpointerleave={() => onhover(null)}
			>
				{shown(r.value)}{#if comparing && r.sigla.length < count}<sup class="found">
						{r.sigla.join(' ')}
					</sup>{/if}
			</button>{/each}
	</p>
{/if}
<AddItems
	field={f.key}
	label={f.label}
	choices={choicesLeft(values)}
	choiceText={(c) => choiceLabel(f, c)}
	onadd={(text) => names.add(f.key, text)}
/>

<style>
	/* A name reads as the text around it; pointed at or focused, it takes the highlighter, as
	   where it is found in the document. */
	.items .item {
		display: inline;
		padding: 0;
		border: 0;
		border-radius: 2px;
		background: none;
		color: inherit;
		font: inherit;
		line-height: inherit;
		text-align: inherit;
		/* A name in its own direction, whatever the list's. */
		unicode-bidi: plaintext;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	/* A name no model listed: the researcher's, underlined in their blue as a correction is. */
	.items .item.yours {
		text-decoration: underline;
		text-decoration-color: var(--hand);
		text-decoration-thickness: 2px;
		text-underline-offset: 0.25em;
	}
	.items .item:hover,
	.items .item:focus-visible {
		background: var(--marker);
		color: var(--on-marker);
		box-shadow: 0 0 0 2px var(--marker);
	}
	.items .item:focus-visible {
		outline-offset: 3px;
	}
	/* The sigla of the set-ups that found a name, when not all of them did. */
	.found {
		font-family: var(--font-mono);
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: uppercase;
		color: var(--text-muted);
		margin-left: 0.15em;
	}
	.sep {
		color: var(--text-faint);
	}
</style>
