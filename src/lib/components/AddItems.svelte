<script lang="ts">
	// What a model missed, added to one list answer of an analysis's review: a compact Add at the
	// end of the names opens a field in place (a name, or several between semicolons, pasted or
	// typed), or for a field of choices the choices not yet listed. Enter adds and keeps it open
	// for the next one; Escape or Done closes it, the focus going back to Add. A name already
	// listed is pointed out rather than added twice; what was added is said by the toast, which
	// offers to take it back.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';

	let {
		field,
		label,
		choices = null,
		choiceText = (c) => c,
		onadd
	}: {
		/** The field's key, to find its Add again. */
		field: string;
		/** The field's label, as its heading shows it. */
		label: string;
		/** For a field of choices, those not yet listed; null for free names. */
		choices?: string[] | null;
		/** A choice as shown, from the value stored. */
		choiceText?: (choice: string) => string;
		/** Adds what was typed or chosen; what was refused and why, or null when it failed. */
		onadd: (
			text: string
		) => Promise<{ added: string[]; already: string[]; refused: string[] } | null>;
	} = $props();
	const m = messages();

	let open = $state(false);
	let draft = $state('');
	let message = $state('');
	let busy = $state(false);
	let opener = $state<HTMLButtonElement | null>(null);
	let input = $state<HTMLInputElement | HTMLSelectElement | null>(null);
	let closer = $state<HTMLButtonElement | null>(null);
	const hintId = $derived(`add-hint-${field}`);

	// Opened: the focus into the field; once every choice is listed, on Close.
	$effect(() => {
		if (open) (input ?? closer)?.focus();
	});

	// A choice taken is no longer offered: the first one left is offered next.
	$effect(() => {
		if (choices && !choices.includes(draft)) draft = choices[0] ?? '';
	});

	function close() {
		open = false;
		message = '';
		draft = '';
		queueMicrotask(() => opener?.focus());
	}

	async function submit(e?: SubmitEvent) {
		e?.preventDefault();
		if (busy) return;
		if (!draft.trim()) {
			message = choices ? m.review.allChosen : m.review.addEmpty;
			return;
		}
		busy = true;
		const result = await onadd(draft);
		busy = false;
		if (!result) return;
		const notes = [
			result.already.length > 0 ? m.review.alreadyListed(result.already) : '',
			result.refused.length > 0 ? m.review.notAChoice(result.refused) : ''
		];
		message = notes.filter(Boolean).join(' ');
		if (result.added.length > 0 && !choices) draft = '';
		(input ?? closer)?.focus();
	}

	/** A column pasted from a spreadsheet or a list keeps one name per line: a one-line field
	 * would run them together, so its lines are set between semicolons. */
	function paste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text/plain') ?? '';
		const field = e.currentTarget;
		if (!/[\r\n]/.test(text) || !(field instanceof HTMLInputElement)) return;
		e.preventDefault();
		const lines = text
			.split(/\r?\n|\r/)
			.map((l) => l.trim())
			.filter(Boolean)
			.join('; ');
		const from = field.selectionStart ?? draft.length;
		const to = field.selectionEnd ?? from;
		draft = draft.slice(0, from) + lines + draft.slice(to);
	}

	function keydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			close();
		} else if (e.key === 'Enter' && e.target instanceof HTMLSelectElement) {
			// A select does not submit its form on Enter in every engine.
			e.preventDefault();
			void submit();
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<form class="add" onsubmit={submit} onkeydown={keydown}>
		{#if choices && choices.length === 0}
			<p class="note">{m.review.allChosen}</p>
		{:else if choices}
			<select
				bind:this={input}
				bind:value={draft}
				aria-label={m.review.addChoice(label)}
				aria-describedby={hintId}
			>
				{#each choices as c (c)}<option value={c}>{choiceText(c)}</option>{/each}
			</select>
		{:else}
			<input
				bind:this={input}
				bind:value={draft}
				type="text"
				spellcheck="false"
				aria-label={m.review.addName(label)}
				aria-describedby={hintId}
				placeholder={m.review.addPlaceholder}
				onpaste={paste}
			/>
		{/if}
		{#if !choices || choices.length > 0}
			<button type="submit" class="btn primary sm" disabled={busy}>{m.review.addItem}</button>
		{/if}
		<button bind:this={closer} type="button" class="btn ghost sm" onclick={close}>
			{m.common.close}
		</button>
		<p class="note" id={hintId}>{m.review.addHint}</p>
		<p class="note said" role="status">{message}</p>
	</form>
{:else}
	<button
		bind:this={opener}
		type="button"
		class="btn ghost sm opener"
		data-add={field}
		aria-label={m.review.addTo(label)}
		onclick={() => (open = true)}
	>
		<Icon name="plus" size={14} />{m.review.addItem}
	</button>
{/if}

<style>
	.opener {
		margin-top: var(--space-1);
		font-family: var(--font-ui);
	}
	.add {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		margin-top: var(--space-2);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
	}
	.add input,
	.add select {
		flex: 1 1 14rem;
		min-width: 0;
	}
	.note {
		flex-basis: 100%;
		margin: 0;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.said {
		color: var(--text-secondary);
	}
</style>
