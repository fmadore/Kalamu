<script lang="ts">
	// An answer that quotes the text: one card per entry, a theme's name over its description and
	// its quotations as the engine found them in the text, each with who speaks and when. A
	// quotation plays in its transcript's review from its turn, or, without a recording, is shown
	// where it stands in the text beside the answers. An entry's fields are typed anew, an entry or
	// one quotation removed, each in the corrected answers and taken back from its toast; a
	// quotation's words are never changed. In a comparison each set-up's cards stand under its
	// siglum, not collated. What the engine could not find in the text is said under them.
	import Icon from './Icon.svelte';
	import { siglum } from '#lib/apparatus.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import {
		cardsOf,
		citedOf,
		draftChanges,
		entryDraft,
		quoteStart,
		saidBy,
		type Card
	} from '#lib/cited.js';
	import type { CitedAnswers } from '#lib/citedAnswers.svelte.js';
	import { choiceLabel, messages } from '#lib/i18n/index.js';

	let {
		field,
		setups,
		answersOf,
		comparing,
		cited
	}: {
		field: Field;
		setups: Setup[];
		/** A set-up's answers as the researcher left them; none before it answered. */
		answersOf: (s: Setup) => FieldAnswer[] | null;
		comparing: boolean;
		cited: CitedAnswers;
	} = $props();
	const m = messages();
	const c = m.cited;

	const own = $derived(field.kind.type === 'cited' ? field.kind.fields : []);
	const isEditing = (s: Setup, entry: number) =>
		cited.editing?.setupId === s.id &&
		cited.editing.key === field.key &&
		cited.editing.entry === entry;

	function startEditing(s: Setup, entry: number) {
		const value = citedOf(answersOf(s) ?? [], field.key)?.entries[entry];
		if (!value) return;
		const draft = entryDraft(value, own);
		cited.editing = { setupId: s.id, key: field.key, entry, draft, before: { ...draft } };
	}

	async function saveEditing(s: Setup, entry: number) {
		const e = cited.editing;
		if (!e) return;
		const changes = draftChanges(e.draft, e.before).map((d) => ({
			kind: 'answer' as const,
			entry,
			key: d.key,
			value: d.value
		}));
		const at = { entry };
		if (changes.length > 0 && !(await cited.change(s, field.key, changes, m.review.saved, at, at)))
			return;
		// The form goes, and the focus with it to the entry's Edit.
		cited.editing = null;
		await cited.focus(s.id, field.key, at, true);
	}

	function removeEntry(s: Setup, card: Card, entry: number, count: number) {
		const said = card.title ? c.entryRemoved(card.title) : c.thisRemoved;
		const after = count > 1 ? { entry: Math.min(entry, count - 2) } : null;
		void cited.change(s, field.key, [{ kind: 'remove_entry', entry }], said, after, { entry });
	}

	function removeQuote(s: Setup, card: Card, entry: number, quotation: number) {
		const left = card.quotes.length - 1;
		const after = { entry, quote: left > 0 ? Math.min(quotation, left - 1) : undefined };
		void cited.change(
			s,
			field.key,
			[{ kind: 'remove_quotation', entry, quotation }],
			c.quoteRemoved,
			after,
			{ entry, quote: quotation }
		);
	}

	function keys(e: KeyboardEvent) {
		if (e.key === 'Escape' && cited.editing) {
			e.stopPropagation();
			const at = { entry: cited.editing.entry };
			const s = cited.editing.setupId;
			cited.editing = null;
			void cited.focus(s, field.key, at, true);
		}
	}
</script>

{#each setups as s, i (s.id)}
	{@const answers = answersOf(s)}
	{@const value = answers ? citedOf(answers, field.key) : null}
	{@const cards = cardsOf(value, own)}
	{#if answers}
		<div
			class="cited"
			data-cited="{s.id}/{field.key}"
			tabindex="-1"
			aria-label={comparing ? `${field.label} · ${s.label}` : field.label}
			role="group"
		>
			{#if comparing}
				<p class="who"><i class="siglum">{siglum(i)}</i> <span class="mono">{s.label}</span></p>
			{/if}
			{#if cards.length === 0}
				<p class="muted none">{m.review.noneFound}</p>
			{/if}
			<ol class="cards">
				{#each cards as card, e (e)}
					<li class="card" data-entry={e}>
						{#if isEditing(s, e) && cited.editing}
							{@const edit = cited.editing}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div class="edit" onkeydown={keys}>
								{#each own as f, k (f.key)}
									<label class="field">
										<span>{f.label}</span>
										{#if f.kind.type === 'choice'}
											<select bind:value={edit.draft[f.key]}>
												<option value=""></option>
												{#each f.kind.values as v (v)}<option value={v}>{choiceLabel(f, v)}</option
													>{/each}
											</select>
										{:else if k === 0 && own.length > 1}
											<!-- svelte-ignore a11y_autofocus -->
											<input type="text" bind:value={edit.draft[f.key]} autofocus />
										{:else}
											<textarea rows="2" bind:value={edit.draft[f.key]}></textarea>
										{/if}
									</label>
								{/each}
								<div class="edit-actions">
									<button
										class="btn sm"
										onclick={() => {
											cited.editing = null;
											void cited.focus(s.id, field.key, { entry: e }, true);
										}}>{m.common.cancel}</button
									>
									<button
										class="btn sm primary"
										disabled={cited.busy}
										onclick={() => saveEditing(s, e)}>{m.common.save}</button
									>
								</div>
							</div>
						{:else}
							{#if card.title}<h3>{card.title}</h3>{/if}
							{#if card.notesFirst}
								{#each card.notes as note (note.key)}<p class="note">{note.value}</p>{/each}
							{/if}
							{#if card.quotes.length === 0}
								<p class="muted none">{c.noQuote}</p>
							{:else}
								<ul class="quotes" aria-label={c.quotes}>
									{#each card.quotes as q, n (n)}
										{@const said = saidBy(q)}
										{@const plays = cited.recording(q) !== null}
										<li data-quote={n}>
											<blockquote lang="">{q.quote}</blockquote>
											<div class="said">
												<button
													class="btn sm"
													data-act="open"
													aria-label={said
														? plays
															? c.listenTo(said)
															: c.showInTextOf(said)
														: undefined}
													onclick={() => cited.open(q)}
													><Icon name={plays ? 'play' : 'file'} size={12} />{plays
														? c.listen
														: c.showInText}</button
												>
												{#if said}<span class="by" aria-hidden="true">{said}</span>{/if}
												<button
													class="btn ghost sm icon remove"
													aria-label={c.removeQuote(quoteStart(q.quote))}
													title={c.removeQuote(quoteStart(q.quote))}
													disabled={cited.busy}
													onclick={() => removeQuote(s, card, e, n)}
													><Icon name="close" size={14} /></button
												>
											</div>
										</li>
									{/each}
								</ul>
							{/if}
							{#if !card.notesFirst}
								{#each card.notes as note (note.key)}<p class="note">{note.value}</p>{/each}
							{/if}
							<div class="card-actions">
								<button
									class="btn ghost sm"
									data-act="edit"
									aria-label={card.title ? c.editEntry(card.title) : c.editThis}
									disabled={cited.busy}
									onclick={() => startEditing(s, e)}><Icon name="edit" size={12} />{c.edit}</button
								>
								<button
									class="btn ghost sm"
									aria-label={card.title ? c.removeEntry(card.title) : c.removeThis}
									disabled={cited.busy}
									onclick={() => removeEntry(s, card, e, cards.length)}
									><Icon name="close" size={12} />{c.remove}</button
								>
							</div>
						{/if}
					</li>
				{/each}
			</ol>
			{#if value && value.left_out > 0}<p class="left">{c.leftOut(value.left_out)}</p>{/if}
			{#if value && value.dropped > 0}<p class="left">{c.dropped(value.dropped)}</p>{/if}
		</div>
	{/if}
{/each}

<style>
	.cited {
		display: grid;
		gap: var(--space-3);
	}
	.cited + .cited {
		margin-top: var(--space-4);
	}
	.cited:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: 2px;
	}
	.who {
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.who .siglum {
		margin-right: 0.35em;
	}
	.cards {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-3);
	}
	.card {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		background: var(--surface-1);
	}
	h3 {
		margin: 0;
		font-family: var(--font-ui);
		font-size: var(--type-md);
		font-weight: 600;
		color: var(--text-bright);
	}
	.note {
		margin: 0;
		font-size: var(--type-md);
		color: var(--text-body);
	}
	.quotes {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-3);
	}
	blockquote {
		margin: 0;
		padding-left: var(--space-3);
		border-left: 3px solid var(--line-strong);
		font-size: var(--type-md);
		font-style: italic;
		color: var(--text-bright);
	}
	/* French guillemets keep their no-break spaces. */
	:global(:root:lang(fr)) blockquote {
		quotes: '«\a0' '\a0»';
	}
	blockquote::before {
		content: open-quote;
	}
	blockquote::after {
		content: close-quote;
	}
	.said {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		margin-top: var(--space-1);
		padding-left: var(--space-3);
		font-family: var(--font-ui);
	}
	.by {
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.remove {
		margin-left: auto;
	}
	.btn.icon {
		width: var(--control-sm);
		padding: 0;
	}
	.card-actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
		justify-content: flex-end;
		font-family: var(--font-ui);
	}
	.edit {
		display: grid;
		gap: var(--space-2);
		font-family: var(--font-ui);
	}
	.edit input,
	.edit textarea,
	.edit select {
		width: 100%;
	}
	.edit-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.none,
	.left {
		margin: 0;
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-muted);
	}
</style>
