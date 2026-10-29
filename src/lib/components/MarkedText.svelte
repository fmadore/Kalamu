<script lang="ts">
	// A text with the places where the witnesses differ marked. The reading text comes in
	// numbered lines, a number in the margin every fifth line and on every line that differs; a
	// compared text runs on as written. Where one text has words the other lacks, a narrow mark
	// stands for them. A place pointed at is shown linked in both texts.
	import { numbered as numberedLine, type Entry, type Line } from '#lib/apparatus.js';

	let {
		lines = null,
		parts = [],
		entries,
		marked,
		selected,
		hovered,
		corrected = () => false,
		emptyLabel,
		controls = null,
		onpick,
		onpoint,
		onaway
	}: {
		/** The reading text, numbered by line. */
		lines?: Line[] | null;
		/** A compared text, when there are no lines. */
		parts?: Line['parts'];
		entries: Entry[];
		/** Whether the differences are highlighted, or only underlined when corrected. */
		marked: boolean;
		selected: number | null;
		hovered: number | null;
		/** Whether a place is the researcher's correction, marked in their hand. */
		corrected?: (entry: number) => boolean;
		/** What a mark with no words in it stands for, for screen readers. */
		emptyLabel: string;
		/** The id of the readings a place opens, if it opens them. */
		controls?: string | null;
		onpick: (entry: number) => void;
		onpoint: (e: PointerEvent, entry: number) => void;
		onaway: () => void;
	} = $props();
</script>

<!-- Inline, with no white space between the parts: the text is set as written. -->
<!-- prettier-ignore -->
{#snippet text(part: Line['parts'][number])}{#if part.entry !== null}{@const id = part.entry}<button class="mark" class:empty={part.text === ''} class:marked class:corr={corrected(id)} class:on={selected === id} class:linked={hovered === id} data-entry={id} aria-expanded={controls ? selected === id : undefined} aria-controls={controls && selected === id ? controls : undefined} aria-label={part.text === '' ? emptyLabel : undefined} onclick={() => onpick(id)} onpointerenter={(e) => onpoint(e, id)} onpointerleave={onaway}>{part.text}</button>{:else}{part.text}{/if}{/snippet}

{#if lines}
	<div class="text numbered" lang="">
		{#each lines as line, li (li)}
			<div class="line" class:blank={line.number === 0}>
				{#if numberedLine(line.number, entries)}<span class="n num" aria-hidden="true"
						>{line.number}</span
					>{/if}
				<!-- prettier-ignore -->
				<span class="t">{#each line.parts as part, k (k)}{@render text(part)}{/each}</span>
			</div>
		{/each}
	</div>
{:else}
	<!-- prettier-ignore -->
	<div class="text run" lang="">{#each parts as part, k (k)}{@render text(part)}{/each}</div>
{/if}

<style>
	.text {
		max-width: 42rem;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.7;
		color: var(--text-body);
	}
	/* The numbers hang in the page's margin, so the text lines up with the other panes'. */
	.numbered {
		margin-left: calc(var(--space-4) - var(--space-6));
	}
	.run {
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	/* Each line sits in a grid with its number, on the line's first baseline. */
	.line {
		display: grid;
		grid-template-columns: 2.25rem minmax(0, 1fr);
		column-gap: var(--space-3);
		align-items: baseline;
		min-height: 1.7em;
	}
	.line.blank {
		min-height: 0.7em;
	}
	.t {
		grid-column: 2;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.n {
		grid-column: 1;
		justify-self: end;
		font-family: var(--font-mono);
		font-size: 0.6875rem;
		color: var(--text-faint);
	}
	.mark {
		display: inline;
		padding: 0;
		border: 0;
		border-radius: 2px;
		background: none;
		color: inherit;
		font: inherit;
		line-height: inherit;
		text-align: inherit;
		white-space: pre-wrap;
		cursor: pointer;
		transition:
			background-color 120ms ease-out,
			box-shadow 120ms ease-out;
	}
	.mark.marked {
		background: var(--marker);
		color: var(--on-marker);
		box-shadow: 0 0 0 2px var(--marker);
	}
	.mark.corr {
		background: none;
		color: inherit;
		box-shadow: none;
		text-decoration: underline;
		text-decoration-color: var(--hand);
		text-decoration-thickness: 2px;
		text-underline-offset: 0.25em;
	}
	.mark.on,
	.mark:hover,
	.mark.linked {
		box-shadow:
			0 0 0 2px var(--marker),
			0 0 0 3.5px var(--text-bright);
	}
	.mark.corr.on,
	.mark.corr:hover,
	.mark.corr.linked {
		box-shadow: none;
		background: var(--hand-wash);
	}
	/* Words one text has and the other lacks: a mark the width of a narrow letter. */
	.mark.empty::before {
		content: '';
		display: inline-block;
		width: 0.3em;
		height: 1em;
		margin: 0 0.1em;
		vertical-align: -0.15em;
		border-radius: 1px;
		background: var(--text-faint);
	}
	.mark.empty.marked {
		box-shadow: none;
		background: none;
	}
	.mark.empty.marked::before {
		background: var(--marker);
		box-shadow: 0 0 0 1px var(--on-marker);
	}
	@media (max-width: 820px) {
		.numbered {
			margin-left: calc(var(--space-2) - var(--space-4));
		}
	}
</style>
