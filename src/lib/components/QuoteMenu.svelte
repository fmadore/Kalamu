<script lang="ts">
	// A passage selected in the transcript beside an interview's answers, added as the
	// researcher's own quotation: the selection's menu (a right click on it, the menu key or
	// Shift+F10) holds one action, Add as a quotation, whose items are the entries it can go to,
	// each field's under its name. Opened only where the selection is in the transcript shown;
	// elsewhere the system's own menu stays. The engine finds the passage in the text, within one
	// turn, and gives it the turn's speaker and time (`CitedEdit::AddQuotation`).
	import ContextMenu from './ContextMenu.svelte';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import { cardsOf, charsAt, citedOf, quoteStart } from '#lib/cited.js';
	import { messages } from '#lib/i18n/index.js';
	import { selectionBox } from '#lib/passage.js';
	import { shortcutsHeld } from '#lib/state.svelte.js';

	let {
		text,
		fields,
		answers,
		offered,
		onadd
	}: {
		/** The transcript shown, as the engine counts it. */
		text: string | null;
		/** The fields of entries that quote the text. */
		fields: Field[];
		/** The lead set-up's answers as the researcher left them. */
		answers: FieldAnswer[];
		/** Whether a passage can be added now (answers read, nothing being edited). */
		offered: boolean;
		/** Adds the passage, in characters of the text, to entry `entry` of field `key`. */
		onadd: (key: string, entry: number, title: string, start: number, end: number) => void;
	} = $props();
	const m = messages();

	let menu = $state<{ x: number; y: number; start: number; end: number } | null>(null);
	let box = $state<ContextMenu | null>(null);

	/** The entries a passage can go to, each field's in order. */
	const groups = $derived(
		fields.map((f) => ({
			field: f,
			cards: cardsOf(citedOf(answers, f.key), f.kind.type === 'cited' ? f.kind.fields : []).map(
				(card, entry) => ({
					entry,
					title: card.title ?? card.notes[0]?.value ?? quoteStart(card.quotes[0]?.quote ?? '')
				})
			)
		}))
	);

	/** The transcript's element, and the passage selected in it, in characters of the text. */
	function chosen(): { start: number; end: number } | null {
		const sheet = document.querySelector<HTMLElement>('.source .desk.document');
		const selection = getSelection();
		if (!sheet || !text || !selection || selection.rangeCount === 0 || selection.isCollapsed)
			return null;
		const range = selection.getRangeAt(0);
		if (!sheet.contains(range.startContainer) || !sheet.contains(range.endContainer)) return null;
		const before = document.createRange();
		before.selectNodeContents(sheet);
		before.setEnd(range.startContainer, range.startOffset);
		const from = before.toString().length;
		const to = from + range.toString().length;
		if (to <= from || to > text.length) return null;
		return { start: charsAt(text, from), end: charsAt(text, to) };
	}

	function open(at: { start: number; end: number }, pointer: { x: number; y: number } | null) {
		const below = pointer ? null : selectionBox();
		menu = { x: pointer?.x ?? below?.left ?? 0, y: pointer?.y ?? below?.bottom ?? 0, ...at };
	}

	function onKey(e: KeyboardEvent) {
		if (shortcutsHeld() || !offered || menu) return;
		if (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) {
			const at = chosen();
			if (!at) return;
			e.preventDefault();
			open(at, null);
		}
	}

	function onContextMenu(e: MouseEvent) {
		if (menu) {
			e.preventDefault();
			return;
		}
		if (!offered) return;
		const at = chosen();
		if (!at) return;
		e.preventDefault();
		const pointer = e.clientX === 0 && e.clientY === 0 ? null : { x: e.clientX, y: e.clientY };
		// WebKit on macOS opens the menu as the button goes down: it waits for the release.
		if (e.buttons === 0) open(at, pointer);
		else addEventListener('pointerup', () => setTimeout(() => open(at, pointer)), { once: true });
	}
</script>

<svelte:window onkeydown={onKey} oncontextmenu={onContextMenu} />

{#if menu}
	{@const at = menu}
	<ContextMenu
		bind:this={box}
		x={at.x}
		y={at.y}
		label={m.cited.addQuotation}
		width="21rem"
		onclose={() => (menu = null)}
	>
		<p class="head">{m.cited.addQuotation}</p>
		{#each groups as group (group.field.key)}
			{#if group.cards.length > 0}
				<p class="group">{group.field.label}</p>
				{#each group.cards as card (card.entry)}
					<button
						type="button"
						role="menuitem"
						class="item"
						onclick={() => {
							box?.close();
							onadd(group.field.key, card.entry, card.title, at.start, at.end);
						}}>{card.title}</button
					>
				{/each}
			{/if}
		{/each}
	</ContextMenu>
{/if}

<style>
	.head {
		margin: 0;
		padding: var(--space-1) var(--space-2);
		font-weight: 600;
		color: var(--text-bright);
	}
	.group {
		margin: var(--space-2) 0 0;
		padding: 0 var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.item {
		display: block;
		width: 100%;
		min-height: 2rem;
		padding: var(--space-1) var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-bright);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.item:hover {
		background: var(--wash);
	}
	.item:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(-1 * var(--focus-width));
	}
</style>
