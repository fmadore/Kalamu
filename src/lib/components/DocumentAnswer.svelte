<script lang="ts">
	// Under a document's title, while the project's table is filtered by an answer: what the
	// analysis answered, why, and the passages it quoted, each with its page, readable without
	// opening the job; then the researcher's own decision, Keep or Set aside, two buttons the
	// keyboard reaches in the row's order, the one given pressed. Pressing it again takes it off.
	// A document carrying both (given in two languages) shows neither pressed and says so;
	// either button settles it.
	import { saidBy } from '#lib/cited.js';
	import { messages } from '#lib/i18n/index.js';
	import type { Decision, RowAnswer } from '#lib/screening.js';

	let {
		title,
		answer,
		decision,
		busy,
		ondecide
	}: {
		/** The document's title, naming the buttons for a screen reader. */
		title: string;
		answer: RowAnswer;
		decision: Decision | null;
		busy: boolean;
		/** Keep (true) or Set aside (false); the same again takes it off. */
		ondecide: (keep: boolean) => void;
	} = $props();
	const m = messages();
	const t = m.screening;
</script>

<div class="answer">
	<p class="said own-direction">
		<span class="chip">{answer.label}</span>
		{#if answer.reason}<span class="reason">{answer.reason}</span>{/if}
	</p>
	{#if answer.passages.length > 0}
		<ul class="passages" aria-label={t.passagesOf(title)}>
			{#each answer.passages as passage, i (i)}
				{@const where = saidBy(passage, m)}
				<li class="own-direction">
					<q>{passage.quote}</q>
					{#if where}<span class="where">{where}</span>{/if}
				</li>
			{/each}
		</ul>
	{/if}
	<div class="decide">
		<button
			class="btn sm"
			class:on={decision === 'keep'}
			aria-pressed={decision === 'keep'}
			aria-label={t.keepThis(title)}
			disabled={busy}
			onclick={() => ondecide(true)}
		>
			{t.keep}
		</button>
		<button
			class="btn sm"
			class:on={decision === 'aside'}
			aria-pressed={decision === 'aside'}
			aria-label={t.setAsideThis(title)}
			disabled={busy}
			onclick={() => ondecide(false)}
		>
			{t.setAside}
		</button>
		{#if decision === 'both'}
			<span class="chip warn">{t.decisionConflict}</span>
		{/if}
	</div>
</div>

<style>
	/* The answer reads as a note under the title: quieter, the passages as the text's own words. */
	.answer {
		flex-basis: 100%;
		display: grid;
		gap: var(--space-1);
		margin-top: var(--space-1);
		max-width: 70ch;
		font-size: var(--type-sm);
		color: var(--text-secondary);
		white-space: normal;
	}
	p {
		margin: 0;
		line-height: 1.5;
	}
	.said .chip {
		margin-right: var(--space-1);
	}
	.reason {
		color: var(--text-body);
	}
	.passages {
		display: grid;
		gap: var(--space-1);
		margin: 0;
		padding-left: var(--space-3);
		border-left: 2px solid var(--line);
		list-style: none;
	}
	q {
		color: var(--text-body);
		font-family: var(--font-text);
	}
	.where {
		margin-left: var(--space-1);
		color: var(--text-muted);
		font-size: var(--type-xs);
		white-space: nowrap;
	}
	.decide {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
	}
	.btn.on {
		background: var(--wash);
		color: var(--text-bright);
		font-weight: 500;
	}
</style>
